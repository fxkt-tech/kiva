const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

const source = fs.readFileSync(path.join(__dirname, "../web/audio.js"), "utf8");

function fixture({ slowDecode = false, brokenFile = false } = {}) {
  class Target {
    listeners = new Map();
    addEventListener(type, listener) {
      if (!this.listeners.has(type)) this.listeners.set(type, []);
      this.listeners.get(type).push(listener);
    }
    emit(type, event = {}) {
      for (const callback of this.listeners.get(type) || []) callback({ type, ...event });
    }
  }
  const started = [];
  const mutations = [];
  const decodes = [];
  let devices = 0;
  const controls = { "audio-muted": new Target(), "audio-volume": new Target() };
  const root = { dataset: { paused: "true" } };
  const canvas = { id: "game" };
  const document = Object.assign(new Target(), {
    documentElement: root, readyState: "loading", pointerLockElement: null,
    focused: true, hidden: false,
    hasFocus() { return this.focused; },
    getElementById: (id) => controls[id] || null,
  });
  class AudioNode {
    disconnected = false;
    gain = { value: 1 };
    pan = { value: 0 };
    connect() {}
    disconnect() { this.disconnected = true; }
  }
  class Device {
    state = "suspended";
    destination = new AudioNode();
    constructor() { devices++; }
    createGain() { return new AudioNode(); }
    createStereoPanner() { return new AudioNode(); }
    createAnalyser() {
      return Object.assign(new AudioNode(), {
        fftSize: 256,
        getFloatTimeDomainData(values) { values.fill(0.25); },
      });
    }
    resume() { this.state = "running"; return Promise.resolve(); }
    decodeAudioData() {
      if (slowDecode) return new Promise((resolve) => decodes.push(resolve));
      return Promise.resolve({ duration: 1 });
    }
    createBufferSource() {
      const node = new AudioNode();
      node.stopped = false;
      node.start = () => started.push(node);
      node.stop = () => { node.stopped = true; };
      return node;
    }
  }
  const window = Object.assign(new Target(), { AudioContext: Device });
  const storage = new Map();
  vm.runInNewContext(source, {
    window, document, Float32Array,
    localStorage: { getItem: (key) => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, value) },
    fetch: async () => ({ ok: !brokenFile, status: 404, arrayBuffer: async () => new ArrayBuffer(16) }),
    MutationObserver: class { constructor(callback) { mutations.push(callback); } observe() {} },
  }, { filename: "web/audio.js" });
  let number = 0;
  let sequence = 0;
  const catalog = { cues: {
    attack: { files: ["attack.ogg"], volume: 0.7, priority: 1 },
    death: { files: ["death.ogg"], volume: 0.9, priority: 3 },
  } };
  function frame(cues = [], fields = {}) {
    const packet = { frame: ++number, tick: 100, session: "1:player", paused: false, cues, ...fields };
    window.emit("kiva-audio", { detail: JSON.stringify(packet) });
    return packet;
  }
  function cue(fields = {}) {
    return { sequence: ++sequence, tick: 100, source: "player", life: "0", cue: "attack", variant: 0, gain: 0.7, pan: 0, ...fields };
  }
  function capture() {
    root.dataset.paused = "false";
    document.pointerLockElement = canvas;
    document.emit("pointerlockchange");
  }
  function gesture() { document.emit("pointerdown", { isTrusted: true }); }
  const flush = () => new Promise((resolve) => setImmediate(resolve));
  return { root, document, window, controls, started, decodes, capture, gesture, frame, cue, flush,
    devices: () => devices,
    diagnostics: () => window.__kivaAudio,
    async start() {
      frame([], { catalog, session: null, paused: true });
      gesture(); await flush(); capture(); frame();
      document.readyState = "complete";
      document.emit("DOMContentLoaded");
    },
    mutate() { mutations.forEach((callback) => callback()); },
  };
}

test("trusted activation creates one device, preloads before entry and plays one accepted cue", async () => {
  const game = fixture();
  assert.equal(game.devices(), 0);
  game.document.emit("pointerdown", { isTrusted: false });
  assert.equal(game.devices(), 0);
  await game.start();
  assert.equal(game.diagnostics().ready, true);
  assert.equal(game.diagnostics().volume, 0.65);
  game.gesture(); game.gesture();
  assert.equal(game.devices(), 1);
  game.frame([game.cue()]);
  assert.equal(game.started.length, 1);
  assert.equal(game.diagnostics().played, 1);
  assert(game.diagnostics().peak > 0);
});

test("running settings keep combat audio available without Pointer Lock", async () => {
  const game = fixture();
  await game.start();
  game.root.dataset.settingsOpen = "true";
  game.document.pointerLockElement = null;
  game.document.emit("pointerlockchange");
  game.frame([game.cue()]);
  assert.equal(game.diagnostics().active, 1);
  game.controls["audio-volume"].value = "30";
  game.controls["audio-volume"].emit("input", { target: game.controls["audio-volume"] });
  assert.equal(game.diagnostics().volume, 0.3);
  game.controls["audio-muted"].checked = true;
  game.controls["audio-muted"].emit("change", { target: game.controls["audio-muted"] });
  assert.equal(game.diagnostics().active, 0);
  assert.equal(game.diagnostics().muted, true);
  assert.equal(game.root.dataset.paused, "false");
});

test("missing decode readiness drops the cue permanently instead of playing it late", async () => {
  const game = fixture({ slowDecode: true });
  await game.start();
  game.frame([game.cue()]);
  assert.equal(game.diagnostics().dropped, 1);
  assert.equal(game.started.length, 0);
  for (const resolve of game.decodes) resolve({ duration: 1 });
  await game.flush();
  assert.equal(game.diagnostics().ready, true);
  assert.equal(game.started.length, 0, "decoding cannot replay a stale combat event");
  game.frame([game.cue()]);
  assert.equal(game.started.length, 1);
});

test("pause, focus loss, unlock and reselection stop every owned node", async () => {
  const game = fixture(); await game.start();
  for (const boundary of ["pause", "blur", "unlock", "reselect"]) {
    game.capture(); game.document.focused = true; game.frame();
    game.frame([game.cue()]);
    const node = game.started.at(-1);
    assert.equal(node.stopped, false);
    if (boundary === "pause") { game.root.dataset.paused = "true"; game.mutate(); }
    if (boundary === "blur") { game.document.focused = false; game.window.emit("blur"); }
    if (boundary === "unlock") { game.document.pointerLockElement = null; game.document.emit("pointerlockchange"); }
    if (boundary === "reselect") game.frame([], { session: null, paused: true });
    assert.equal(node.stopped, true, boundary);
    assert.equal(node.disconnected, true, boundary);
    assert.equal(game.diagnostics().active, 0, boundary);
  }
});

test("an older display packet cannot revive the previous match or duplicate a sound", async () => {
  const game = fixture(); await game.start();
  const old = game.frame([game.cue()]);
  game.frame([], { session: "2:new-player", paused: false });
  game.window.emit("kiva-audio", { detail: JSON.stringify(old) });
  assert.equal(game.diagnostics().session, "2:new-player");
  assert.equal(game.diagnostics().active, 0);
  assert.equal(game.diagnostics().played, 1);
});

test("a new life stops the same actor's old death voice without stopping other actors", async () => {
  const game = fixture(); await game.start();
  game.frame([game.cue({ cue: "death" }), game.cue({ source: "other" })]);
  game.frame([game.cue({ life: "1" })]);
  assert.equal(game.started[0].stopped, true);
  assert.equal(game.started[1].stopped, false);
  assert.equal(game.diagnostics().active, 2);
});

test("voice count is bounded and attack spam cannot evict death notification", async () => {
  const game = fixture(); await game.start();
  game.frame([game.cue({ cue: "death" }), ...Array.from({ length: 15 }, () => game.cue())]);
  assert.equal(game.diagnostics().active, 16);
  game.frame(Array.from({ length: 25 }, () => game.cue()));
  assert.equal(game.diagnostics().active, 16);
  assert.equal(game.started[0].stopped, false);
  game.started.at(-1).onended();
  assert.equal(game.diagnostics().active, 15, "natural ends disconnect and release the voice");
});

test("muting and volume controls stay cosmetic, bounded and silent while muted", async () => {
  const game = fixture(); await game.start();
  game.frame([game.cue()]);
  game.controls["audio-muted"].emit("change", { target: { checked: true } });
  assert.equal(game.diagnostics().muted, true);
  assert.equal(game.diagnostics().active, 0);
  game.frame([game.cue()]);
  assert.equal(game.started.length, 1);
  game.controls["audio-volume"].emit("input", { target: { value: "140" } });
  assert.equal(game.diagnostics().volume, 1);
  game.window.kivaAudio.setMuted(false);
  game.frame([game.cue()]);
  assert.equal(game.started.length, 2);
  assert(Object.isFrozen(game.diagnostics()));
});

test("real asset failure is observable without blocking input or creating a delayed queue", async () => {
  const game = fixture({ brokenFile: true }); await game.start();
  assert.equal(game.diagnostics().ready, false);
  assert.equal(game.diagnostics().errors.length, 2);
  assert(game.diagnostics().errors.every((message) => message.includes("HTTP 404")));
  game.frame([game.cue()]);
  assert.equal(game.started.length, 0);
  assert.equal(game.diagnostics().dropped, 1);
});
