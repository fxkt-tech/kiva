// The sole browser audio device. Cue identity and audibility come from Rust;
// this module only loads local media, mixes voices and owns their lifecycle.
(() => {
  const root = document.documentElement;
  const MAX_VOICES = 16;
  const MAX_AGE_TICKS = 15;
  const DEFAULT_VOLUME = 0.65;
  const files = new Map();
  const active = new Set();
  const lives = new Map();
  const playedByCue = Object.create(null);
  const errors = [];
  let context = null;
  let master = null;
  let analyser = null;
  let samples = null;
  let catalog = null;
  let session = null;
  let paused = true;
  let lastFrame = 0;
  let lastSequence = 0;
  let played = 0;
  let dropped = 0;
  let volume = DEFAULT_VOLUME;
  let muted = false;

  try {
    const saved = Number(localStorage.getItem("kiva-audio-volume"));
    if (localStorage.getItem("kiva-audio-volume") !== null && Number.isFinite(saved)) {
      volume = Math.max(0, Math.min(1, saved));
    }
    muted = localStorage.getItem("kiva-audio-muted") === "true";
  } catch { /* Storage availability does not affect sound or entering a match. */ }

  function report(message) {
    errors.push(String(message));
    if (errors.length > 8) errors.shift();
    diagnostics();
  }

  function canPlay() {
    return session !== null && !paused && !muted && volume > 0
      && root.dataset.paused !== "true" && document.hasFocus()
      && document.pointerLockElement?.id === "game";
  }

  function diagnostics() {
    let peak = 0;
    if (analyser && active.size && context.state === "running") {
      analyser.getFloatTimeDomainData(samples);
      for (const value of samples) peak = Math.max(peak, Math.abs(value));
    }
    const loaded = [...files.values()].filter((file) => file.buffer).length;
    window.__kivaAudio = Object.freeze({
      session, paused, volume, muted, active: active.size, played, dropped,
      context_state: context?.state || "uninitialized",
      loaded_files: loaded, expected_files: files.size,
      ready: files.size > 0 && loaded === files.size,
      peak, played_by_cue: Object.freeze({ ...playedByCue }),
      errors: Object.freeze([...errors]),
    });
  }

  function dispose(voice, stop = true) {
    if (!active.delete(voice)) return;
    voice.node.onended = null;
    if (stop) {
      try { voice.node.stop(); } catch { /* A naturally completed source is stopped already. */ }
    }
    voice.node.disconnect();
    voice.gain.disconnect();
    voice.pan.disconnect();
  }

  function stopAll() {
    for (const voice of [...active]) dispose(voice);
    diagnostics();
  }

  function applyVolume() {
    if (master) master.gain.value = muted ? 0 : volume;
    if (muted || volume === 0) stopAll();
    const checkbox = document.getElementById("audio-muted");
    const slider = document.getElementById("audio-volume");
    if (checkbox) checkbox.checked = muted;
    if (slider) slider.value = String(Math.round(volume * 100));
    diagnostics();
  }

  function setVolume(value) {
    if (!Number.isFinite(value)) return;
    volume = Math.max(0, Math.min(1, value));
    try { localStorage.setItem("kiva-audio-volume", String(volume)); } catch {}
    applyVolume();
  }

  function setMuted(value) {
    muted = Boolean(value);
    try { localStorage.setItem("kiva-audio-muted", String(muted)); } catch {}
    applyVolume();
  }

  function loadFile(name) {
    if (!/^[a-z0-9-]+\.ogg$/.test(name)) {
      report(`Invalid audio filename: ${name}`);
      return;
    }
    if (files.has(name)) return;
    const file = { bytes: null, buffer: null, failed: false, decoding: false };
    files.set(name, file);
    fetch(`assets/audio/${name}`).then((response) => {
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      return response.arrayBuffer();
    }).then((bytes) => {
      file.bytes = bytes;
      decode(name, file);
    }).catch((error) => {
      file.failed = true;
      report(`Audio load failed (${name}): ${error.message}`);
    });
  }

  function decode(name, file) {
    if (!context || !file.bytes || file.buffer || file.decoding || file.failed) return;
    file.decoding = true;
    context.decodeAudioData(file.bytes.slice(0)).then((buffer) => {
      file.buffer = buffer;
      file.bytes = null;
      file.decoding = false;
      diagnostics();
    }).catch((error) => {
      file.failed = true;
      file.decoding = false;
      report(`Audio decode failed (${name}): ${error.message}`);
    });
  }

  function installCatalog(value) {
    if (!value?.cues || typeof value.cues !== "object") return;
    // The immutable builtin catalog is sent once before the user enters. Rust
    // remains the only owner of hero/ability -> cue selection.
    catalog = value.cues;
    for (const cue of Object.values(catalog)) {
      for (const name of cue.files || []) loadFile(name);
    }
    diagnostics();
  }

  function unlock() {
    try {
      if (!context) {
        const AudioDevice = window.AudioContext || window.webkitAudioContext;
        if (!AudioDevice) throw new Error("Web Audio is unavailable");
        context = new AudioDevice({ latencyHint: "interactive" });
        master = context.createGain();
        analyser = context.createAnalyser();
        analyser.fftSize = 256;
        samples = new Float32Array(analyser.fftSize);
        master.connect(analyser);
        analyser.connect(context.destination);
        applyVolume();
        for (const [name, file] of files) decode(name, file);
      }
      // Call resume synchronously inside the trusted gesture, before awaiting
      // anything. No sound is queued while permission/decoding is pending.
      const resume = context.resume();
      Promise.resolve(resume).then(diagnostics).catch((error) => {
        report(`Audio activation failed: ${error.message}`);
      });
    } catch (error) {
      report(`Audio activation failed: ${error.message}`);
    }
    diagnostics();
  }

  function play(cue, tick) {
    if (!Number.isSafeInteger(cue.sequence) || cue.sequence <= lastSequence) return;
    lastSequence = cue.sequence;
    const definition = catalog?.[cue.cue];
    const name = definition?.files?.[cue.variant];
    if (!name || !Number.isSafeInteger(cue.tick) || cue.tick > tick
      || tick - cue.tick > MAX_AGE_TICKS || !Number.isFinite(cue.gain)
      || !Number.isFinite(cue.pan) || typeof cue.source !== "string"
      || typeof cue.life !== "string") {
      dropped++;
      return;
    }
    const previousLife = lives.get(cue.source);
    if (previousLife !== undefined && previousLife !== cue.life) {
      for (const voice of [...active]) {
        if (voice.source === cue.source) dispose(voice);
      }
    }
    lives.set(cue.source, cue.life);
    const buffer = files.get(name)?.buffer;
    if (!canPlay() || context?.state !== "running" || !buffer) {
      dropped++;
      return;
    }
    const priority = definition.priority;
    if (active.size >= MAX_VOICES) {
      const weakest = [...active].reduce((a, b) => a.priority <= b.priority ? a : b);
      if (weakest.priority > priority) {
        dropped++;
        return;
      }
      dispose(weakest);
    }
    try {
      const node = context.createBufferSource();
      const gain = context.createGain();
      const pan = context.createStereoPanner();
      node.buffer = buffer;
      gain.gain.value = Math.max(0, Math.min(1, cue.gain));
      pan.pan.value = Math.max(-1, Math.min(1, cue.pan));
      node.connect(gain);
      gain.connect(pan);
      pan.connect(master);
      const voice = { node, gain, pan, priority, source: cue.source, life: cue.life };
      node.onended = () => { dispose(voice, false); diagnostics(); };
      active.add(voice);
      try { node.start(); } catch (error) { dispose(voice); throw error; }
      played++;
      playedByCue[cue.cue] = (playedByCue[cue.cue] || 0) + 1;
    } catch (error) {
      report(`Audio playback failed (${name}): ${error.message}`);
    }
  }

  window.addEventListener("kiva-audio", (event) => {
    let packet;
    try { packet = JSON.parse(event.detail); } catch { return; }
    if (!Number.isSafeInteger(packet.frame) || packet.frame <= lastFrame
      || !Number.isSafeInteger(packet.tick) || typeof packet.paused !== "boolean"
      || (packet.session !== null && typeof packet.session !== "string")) return;
    lastFrame = packet.frame;
    if (packet.catalog) installCatalog(packet.catalog);
    if (session !== packet.session) {
      stopAll();
      lives.clear();
      lastSequence = 0;
      session = packet.session;
    }
    paused = packet.paused;
    if (!canPlay()) stopAll();
    for (const cue of (packet.cues || []).slice(0, 48)) play(cue, packet.tick);
    diagnostics();
  });

  // Audio-only side effects: never consume, prevent or rewrite game input.
  const gesture = (event) => { if (event.isTrusted) unlock(); };
  document.addEventListener("pointerdown", gesture, { capture: true, passive: true });
  document.addEventListener("keydown", gesture, { capture: true, passive: true });
  document.addEventListener("pointerlockchange", () => {
    if (document.pointerLockElement?.id !== "game") stopAll();
  });
  window.addEventListener("blur", stopAll);
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) stopAll();
  });
  new MutationObserver(() => {
    if (root.dataset.paused === "true") stopAll();
  }).observe(root, { attributes: true, attributeFilter: ["data-paused"] });

  function bindControls() {
    document.getElementById("audio-muted")?.addEventListener("change", (event) => setMuted(event.target.checked));
    document.getElementById("audio-volume")?.addEventListener("input", (event) => setVolume(Number(event.target.value) / 100));
    applyVolume();
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bindControls, { once: true });
  else bindControls();
  window.kivaAudio = Object.freeze({ unlock, setMuted, setVolume });
  diagnostics();
})();
