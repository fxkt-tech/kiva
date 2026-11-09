(() => {
  "use strict";

  const MAX_VOICES = 2;
  const voices = new Set();
  let context = null;
  let shot = null;

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
        for (const voice of [...voices]) stop(voice);
        const AudioContext = window.AudioContext || window.webkitAudioContext;
        if (!AudioContext) return;
        context = new AudioContext();
        shot = synthesize(context);
        const current = context;
        current.addEventListener("statechange", () => {
          // A suspended old tail must not wait to sound on a later gesture.
          if (context === current && current.state !== "running") for (const voice of [...voices]) stop(voice);
        });
      }
      if (context.state !== "running") {
        Promise.resolve(context.resume()).catch(() => {});
      }
    } catch {
      // Browser policy or a missing audio device must not interrupt the game.
    }
  }

  window.kivaAudio = {
    playSniperShot() {
      // Never create/resume a context or queue a sound outside a trusted gesture.
      if (!context || context.state !== "running" || !shot) return false;
      let source;
      let gain;
      let voice;
      try {
        while (voices.size >= MAX_VOICES) stop(voices.values().next().value);
        source = context.createBufferSource();
        gain = context.createGain();
        source.buffer = shot;
        // Even the maximum two concurrent normalized voices stay below full scale.
        gain.gain.value = 0.5;
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
        voices.add(voice);
        source.start();
        return true;
      } catch {
        if (voice) voice.release();
        else {
          if (source) source.disconnect();
          if (gain) gain.disconnect();
        }
        return false;
      }
    },
  };

  function mount() {
    document.getElementById("enter")?.addEventListener("click", unlock);
    document.getElementById("game")?.addEventListener("pointerdown", unlock);
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", mount, { once: true });
  else mount();
})();
