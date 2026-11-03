// Procedural WebAudio sound palette. No external samples or autoplay prompts.
(() => {
  let context, master, noise;
  let muted = false;
  try { muted = localStorage.getItem('kiva-muted') === 'true'; } catch {}
  const button = () => document.getElementById('sound-toggle');
  function refreshButton() {
    const element = button();
    if (element) {
      element.textContent = muted ? 'SOUND OFF' : 'SOUND ON';
      element.setAttribute('aria-pressed', String(muted));
    }
  }
  function init() {
    if (context) return;
    context = new AudioContext();
    master = context.createGain();
    master.gain.value = muted ? 0 : 0.35;
    master.connect(context.destination);
    noise = context.createBuffer(1, context.sampleRate * 0.6, context.sampleRate);
    const data = noise.getChannelData(0);
    for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1;
  }
  function envelope(node, duration, volume) {
    const gain = context.createGain();
    const now = context.currentTime;
    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.exponentialRampToValueAtTime(Math.max(volume, 0.001), now + 0.006);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);
    node.connect(gain);
    gain.connect(master);
    node.onended = () => { node.disconnect(); gain.disconnect(); };
    node.start(now);
    node.stop(now + duration + 0.015);
  }
  function tone(from, to, duration, volume, type = 'sine') {
    const oscillator = context.createOscillator();
    oscillator.type = type;
    oscillator.frequency.setValueAtTime(from, context.currentTime);
    oscillator.frequency.exponentialRampToValueAtTime(Math.max(20, to), context.currentTime + duration);
    envelope(oscillator, duration, volume);
  }
  function rush(from, to, duration, volume, type = 'bandpass') {
    const source = context.createBufferSource();
    source.buffer = noise;
    const filter = context.createBiquadFilter();
    filter.type = type;
    filter.Q.value = 0.8;
    filter.frequency.setValueAtTime(from, context.currentTime);
    filter.frequency.exponentialRampToValueAtTime(Math.max(30, to), context.currentTime + duration);
    const gain = context.createGain();
    const now = context.currentTime;
    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.exponentialRampToValueAtTime(volume, now + 0.01);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);
    source.connect(filter); filter.connect(gain); gain.connect(master);
    source.onended = () => { source.disconnect(); filter.disconnect(); gain.disconnect(); };
    source.start(now); source.stop(now + duration + 0.015);
  }
  function play(name) {
    if (!context || context.state !== 'running' || muted) return;
    switch (name) {
      case 'swing1': rush(2300, 450, 0.17, 0.40); tone(150, 75, 0.13, 0.10); break;
      case 'swing2': rush(1700, 700, 0.19, 0.45); tone(190, 90, 0.15, 0.09); break;
      case 'swing3': rush(3000, 180, 0.28, 0.58); tone(125, 42, 0.23, 0.18); break;
      case 'hit': rush(1800, 380, 0.11, 0.65); tone(125, 43, 0.14, 0.30, 'triangle'); break;
      case 'guard': tone(1450, 1350, 0.25, 0.16, 'triangle'); tone(2210, 2030, 0.18, 0.10); rush(5500, 1600, 0.08, 0.45, 'highpass'); break;
      case 'parry': tone(650, 830, 0.09, 0.08, 'triangle'); break;
      case 'charge-ready': tone(880, 1320, 0.18, 0.12, 'sine'); break;
      case 'sword-energy': rush(4500, 300, 0.35, 0.35); tone(220, 80, 0.25, 0.12); break;
      case 'jump': rush(300, 1600, 0.16, 0.22); tone(180, 360, 0.13, 0.10); break;
      case 'land': rush(800, 130, 0.13, 0.40, 'lowpass'); tone(95, 38, 0.14, 0.22); break;
      case 'dodge': rush(2400, 230, 0.23, 0.36); break;
      case 'evade': tone(700, 1100, 0.13, 0.08); break;
      case 'step': rush(600, 150, 0.065, 0.18, 'lowpass'); tone(105, 65, 0.055, 0.07); break;
      case 'windup': tone(280, 390, 0.22, 0.07, 'triangle'); break;
      case 'down': rush(600, 90, 0.3, 0.38, 'lowpass'); tone(100, 28, 0.35, 0.23); break;
      case 'revive': case 'reset': tone(520, 700, 0.32, 0.07); tone(780, 1050, 0.4, 0.05); break;
    }
  }
  function toggle() {
    muted = !muted;
    if (master) master.gain.setTargetAtTime(muted ? 0 : 0.35, context.currentTime, 0.03);
    try { localStorage.setItem('kiva-muted', String(muted)); } catch {}
    refreshButton();
  }
  window.KivaAudio = {
    unlock() { init(); context.resume().then(() => play('revive')).catch(() => {}); },
    toggle,
    get state() { return context?.state || 'idle'; },
    get muted() { return muted; },
  };
  window.addEventListener('kiva-sound', event => play(event.detail));
  document.addEventListener('pointerlockchange', () => {
    if (!context) return;
    if (document.pointerLockElement) context.resume().catch(() => {});
    else context.suspend().catch(() => {});
  });
  button()?.addEventListener('click', toggle);
  refreshButton();
})();
