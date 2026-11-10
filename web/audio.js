(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;

  const MAX_VOICES = 4;
  const voices = new Set();
  let context = null;
  let shot = null;
  let active = false;
  let samples = new Map();
  let loading = null;
  let music = null;
  let musicOffset = 0;
  let musicStart = 0;
  const files = Object.freeze({ attack: "attack.ogg", hit: "hit.ogg", deflect: "deflect.ogg" });

  function stopMusic(preserve = true) {
    if (!music) return;
    const buffer = samples.get("bgm");
    if (preserve && buffer?.duration > 0) {
      musicOffset = (musicOffset + Math.max(0, context.currentTime - musicStart)) % buffer.duration;
    } else musicOffset = 0;
    stop(music);
    music = null;
  }

  function startMusic() {
    const buffer = samples.get("bgm");
    if (music || !active || document.hidden || !buffer || context?.state !== "running") return;
    musicStart = context.currentTime;
    music = playBuffer(buffer, 0.18, true, musicOffset);
  }

  function loadSamples(audio) {
    if (loading || typeof fetch !== "function" || typeof audio.decodeAudioData !== "function") return;
    async function decode(name, file) {
      try {
        const response = await fetch(`audio/${file}`);
        if (!response.ok) return;
        const buffer = await audio.decodeAudioData(await response.arrayBuffer());
        if (context !== audio) return;
        samples.set(name, buffer);
        if (name === "bgm") startMusic();
      } catch { /* Missing/unsupported audio never queues an old effect. */ }
    }
    loading = Promise.all([
      ...Object.entries(files).map(([name, file]) => decode(name, file)),
      (async () => {
        try {
          const response = await fetch("audio/music.json");
          if (!response.ok) return;
          const { file } = await response.json();
          // Optional, verified local music only. No guessed song or missing-file request.
          if (typeof file === "string" && /^[a-z0-9][a-z0-9_-]*\.ogg$/.test(file)) {
            await decode("bgm", file);
          }
        } catch { /* Optional music does not gate startup. */ }
      })(),
    ]);
  }

  function playBuffer(buffer, volume, loop = false, offset = 0) {
    if (!active || !context || context.state !== "running" || !buffer) return null;
    let source;
    let gain;
    let voice;
    try {
      if (!loop) while (voices.size >= MAX_VOICES) stop(voices.values().next().value);
      source = context.createBufferSource();
      gain = context.createGain();
      source.buffer = buffer;
      source.loop = loop;
      // Four effect voices plus music have a total peak budget below one.
      gain.gain.value = volume;
      source.connect(gain);
      gain.connect(context.destination);
      let released = false;
      voice = { source, release() {
        if (released) return;
        released = true;
        source.onended = null;
        source.disconnect();
        gain.disconnect();
        voices.delete(voice);
      } };
      source.onended = voice.release;
      if (!loop) voices.add(voice);
      source.start(0, offset);
      return voice;
    } catch {
      if (voice) voice.release();
      else {
        source?.disconnect();
        gain?.disconnect();
      }
      return null;
    }
  }

  function synthesize(audio) {
    const rate = audio.sampleRate;
    const buffer = audio.createBuffer(1, Math.ceil(rate * 0.72), rate);
    const samples = buffer.getChannelData(0);
    const dry = new Float32Array(Math.ceil(rate * 0.42));
    let random = 0x4b495641;
    let previous = 0;
    let low = 0;
    for (let i = 0; i < dry.length; i++) {
      const t = i / rate;
      // A fixed noise seed keeps this original procedural shot consistent.
      random = (Math.imul(random, 1664525) + 1013904223) >>> 0;
      const noise = random / 0x80000000 - 1;
      low += 0.16 * (noise - low);
      const attack = 1 - Math.exp(-t / 0.00045);
      const phase = 2 * Math.PI * (44 * t + 105 * 0.035 * (1 - Math.exp(-t / 0.035)));
      const boom = Math.sin(phase) * Math.exp(-t / 0.085) * 0.85;
      const crack = (noise - previous) * Math.exp(-t / 0.009) * 0.58;
      const blast = low * Math.exp(-t / 0.047) * 0.65;
      const hitTime = Math.max(0, t - 0.012);
      const hit = t < 0.012 ? 0 : (
        Math.sin(2 * Math.PI * 1350 * hitTime) * 0.12 + noise * 0.18
      ) * (1 - Math.exp(-hitTime / 0.0003)) * Math.exp(-hitTime / 0.023);
      const fade = Math.min(1, (dry.length - 1 - i) / (rate * 0.025));
      dry[i] = (attack * (boom + crack + blast) + hit) * fade;
      previous = noise;
    }
    // Short reflections leave room for the next action instead of a long tail.
    for (const [delay, gain] of [[0, 1], [0.105, 0.17], [0.19, 0.075], [0.29, 0.035]]) {
      const offset = Math.round(delay * rate);
      for (let i = 0; i < dry.length && i + offset < samples.length; i++) {
        samples[i + offset] += dry[i] * gain;
      }
    }
    let peak = 0;
    for (const value of samples) peak = Math.max(peak, Math.abs(value));
    if (peak > 0) {
      const scale = 0.9 / peak;
      for (let i = 0; i < samples.length; i++) samples[i] *= scale;
    }
    return buffer;
  }

  function stop(voice) {
    try { voice.source.stop(); } catch { /* Already ended or unavailable. */ }
    voice.release();
  }

  function unlock(event) {
    if (event.isTrusted !== true) return;
    try {
      if (!context || context.state === "closed" || !shot) {
        // A new gesture can observe "closed" before the old statechange event
        // arrives. Release that graph before replacing the context reference.
        stopMusic(false);
        for (const voice of [...voices]) stop(voice);
        samples = new Map();
        loading = null;
        const AudioContext = window.AudioContext || window.webkitAudioContext;
        if (!AudioContext) return;
        context = new AudioContext();
        shot = synthesize(context);
        const current = context;
        current.addEventListener("statechange", () => {
          // A suspended old tail must not wait to sound on a later gesture.
          if (context !== current) return;
          if (current.state !== "running") {
            stopMusic(current.state !== "closed");
            for (const voice of [...voices]) stop(voice);
          } else startMusic();
        });
      }
      loadSamples(context);
      if (context.state !== "running") {
        Promise.resolve(context.resume()).catch(() => {});
      }
    } catch {
      // Browser policy or a missing audio device must not interrupt the game.
    }
  }

  window.kivaAudio = Object.freeze({
    playSniperShot() {
      return playBuffer(shot, 0.18) !== null;
    },
    playEffect(name) {
      if (!Object.hasOwn(files, name) || name === "bgm") return false;
      return playBuffer(samples.get(name), 0.18) !== null;
    },
    setActive(value) {
      active = value === true && !document.hidden;
      if (active) startMusic();
      else {
        stopMusic();
        for (const voice of [...voices]) stop(voice);
      }
    },
  });
  // Capture loss can occur between two engine frames.
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) window.kivaAudio.setActive(false);
  });
  document.addEventListener("pointerlockchange", () => {
    if (document.pointerLockElement !== document.getElementById("game")) window.kivaAudio.setActive(false);
  });
  window.addEventListener?.("blur", () => window.kivaAudio.setActive(false));

  function mount() {
    document.getElementById("enter")?.addEventListener("click", unlock);
    document.getElementById("game")?.addEventListener("pointerdown", unlock);
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", mount, { once: true });
  else mount();
})();
