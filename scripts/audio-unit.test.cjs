const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../web/audio.js"), "utf8");

class Target {
  listeners = new Map();
  addEventListener(type, callback, options) {
    const entries = this.listeners.get(type) || [];
    entries.push({ callback, once: options?.once });
    this.listeners.set(type, entries);
  }
  emit(type, fields = {}) {
    const event = { type, ...fields };
    for (const entry of [...(this.listeners.get(type) || [])]) {
      if (entry.once) this.listeners.set(type, this.listeners.get(type).filter((item) => item !== entry));
      entry.callback(event);
    }
    return event;
  }
}

function fixture(options = {}) {
  const contexts = [];
  class AudioContext extends Target {
    constructor() {
      super();
      if (options.denied) throw new Error("device denied");
      this.state = options.state || "suspended";
      this.sampleRate = options.sampleRate || 48000;
      this.currentTime = 0;
      this.destination = {};
      this.buffers = [];
      this.sources = [];
      this.gains = [];
      this.resumeCalls = 0;
      this.resumeFailure = options.resumeFailure;
      contexts.push(this);
    }
    setState(state) {
      this.state = state;
      this.emit("statechange");
    }
    resume() {
      this.resumeCalls++;
      if (this.resumeFailure) return Promise.reject(new Error("resume denied"));
      if (options.deferred) return new Promise((resolve) => {
        this.finishResume = () => { this.setState("running"); resolve(); };
      });
      this.setState("running");
      return Promise.resolve();
    }
    createBuffer(channels, length, rate) {
      const data = new Float32Array(length);
      const buffer = { channels, length, sampleRate: rate, getChannelData: () => data };
      this.buffers.push(buffer);
      return buffer;
    }
    createBufferSource() {
      const node = {
        connections: [], disconnects: 0, starts: 0, stops: 0,
        connect(target) { this.connections.push(target); },
        disconnect() { this.disconnects++; },
        start(...args) { if (options.startFailure) throw new Error("source unavailable"); this.starts++; this.startArgs = args; },
        stop() { this.stops++; this.onended?.(); },
        finish() { this.onended?.(); },
      };
      this.sources.push(node);
      return node;
    }
    async decodeAudioData(data) {
      if (options.decodeFailure) throw new Error("decode failed");
      return { duration: 120, name: data };
    }
    createGain() {
      const node = {
        gain: { value: 1 }, connections: [], disconnects: 0,
        connect(target) { this.connections.push(target); },
        disconnect() { this.disconnects++; },
      };
      this.gains.push(node);
      return node;
    }
  }
  const window = new Target();
  if (!options.unsupported) window[options.webkit ? "webkitAudioContext" : "AudioContext"] = AudioContext;
  const document = Object.assign(new Target(), { readyState: options.parsed ? "complete" : "loading" });
  const nodes = new Map();
  document.getElementById = (id) => nodes.get(id) || null;
  const addNodes = () => {
    for (const id of ["enter", "game"]) nodes.set(id, new Target());
  };
  if (options.parsed) addNodes();
  const requests = [];
  const fetch = options.sampleFiles ? async (url) => {
    requests.push(url);
    if (options.networkFailure) throw new Error("offline");
    return { ok: true, arrayBuffer: async () => url,
      json: async () => ({ file: options.noMusic ? null : "choulingke-bgm.ogg" }) };
  } : undefined;
  vm.runInContext(source, vm.createContext({ window, document, fetch }));
  const parse = () => { addNodes(); document.readyState = "interactive"; document.emit("DOMContentLoaded"); };
  const gesture = (id = "enter", trusted = true) => {
    const event = nodes.get(id).emit(id === "enter" ? "click" : "pointerdown", { isTrusted: trusted });
    if (trusted) window.kivaAudio.setActive(true);
    return event;
  };
  return { audio: window.kivaAudio, window, document, nodes, contexts, requests, parse, gesture };
}

test("bridge is available before DOM and only trusted entry/canvas gestures unlock audio", () => {
  const f = fixture();
  assert.equal(f.audio.playSniperShot(), false);
  assert.equal(f.contexts.length, 0);
  f.parse();
  f.gesture("enter", false);
  f.gesture("game", false);
  assert.equal(f.contexts.length, 0);
  f.gesture();
  assert.equal(f.contexts.length, 1);
  assert.equal(f.contexts[0].resumeCalls, 1);
  assert.equal(f.contexts[0].sources.length, 0);
  assert.equal(f.audio.playSniperShot(), true);
  assert.equal(f.contexts[0].sources[0].starts, 1);

  const mounted = fixture({ parsed: true, state: "running", webkit: true });
  mounted.gesture("game");
  assert.equal(mounted.contexts.length, 1);
  assert.equal(mounted.contexts[0].resumeCalls, 0);
  assert.equal(mounted.audio.playSniperShot(), true);
});

test("shots while locked or resuming are discarded instead of replayed after unlock", async () => {
  const f = fixture({ deferred: true }); f.parse();
  assert.equal(f.audio.playSniperShot(), false);
  f.gesture();
  for (let i = 0; i < 6; i++) assert.equal(f.audio.playSniperShot(), false);
  const context = f.contexts[0];
  assert.equal(context.resumeCalls, 1);
  context.finishResume(); await Promise.resolve();
  assert.equal(context.sources.length, 0);
  assert.equal(f.audio.playSniperShot(), true);
  assert.equal(context.sources.length, 1);
});

test("unsupported, denied, and rejected audio do not throw or leak rejected promises", async () => {
  for (const options of [{ unsupported: true }, { denied: true }]) {
    const f = fixture(options); f.parse();
    assert.doesNotThrow(() => f.gesture());
    assert.equal(f.audio.playSniperShot(), false);
    assert.equal(f.contexts.length, 0);
  }
  const f = fixture({ resumeFailure: true }); f.parse(); f.gesture();
  await Promise.resolve(); await Promise.resolve();
  assert.equal(f.audio.playSniperShot(), false);
  assert.equal(f.contexts[0].sources.length, 0);
  f.contexts[0].resumeFailure = false;
  f.gesture("game");
  assert.equal(f.contexts[0].sources.length, 0);
  assert.equal(f.audio.playSniperShot(), true);
});

test("cached original synthesis is finite, normalized, and includes a short decaying tail", () => {
  const f = fixture({ parsed: true }); f.gesture();
  f.audio.playSniperShot(); f.audio.playSniperShot();
  const context = f.contexts[0];
  assert.equal(context.buffers.length, 1);
  assert.equal(context.sources[0].buffer, context.sources[1].buffer);
  const buffer = context.buffers[0];
  assert.equal(buffer.channels, 1);
  assert.equal(buffer.sampleRate, 48000);
  assert.equal(buffer.length, Math.ceil(48000 * 0.72));
  const samples = buffer.getChannelData(0);
  let peak = 0;
  for (const sample of samples) {
    assert.equal(Number.isFinite(sample), true);
    peak = Math.max(peak, Math.abs(sample));
  }
  assert.ok(peak > 0.89 && peak < 0.901);
  const energy = (start, end) => {
    let sum = 0;
    for (let i = Math.floor(start * buffer.sampleRate); i < Math.floor(end * buffer.sampleRate); i++) sum += samples[i] ** 2;
    return sum / ((end - start) * buffer.sampleRate);
  };
  assert.ok(energy(0, 0.08) > energy(0.32, 0.4) * 15);
  assert.ok(energy(0.32, 0.4) > energy(0.61, 0.69) * 15);
  assert.ok(energy(0.61, 0.69) > 0);
  assert.equal(samples[0], 0);
  assert.equal(samples[samples.length - 1], 0);
  const repeat = fixture({ parsed: true }); repeat.gesture();
  assert.deepEqual(Array.from(repeat.contexts[0].buffers[0].getChannelData(0)), Array.from(samples));
});

test("voices are bounded, connected correctly, and disconnected on completion", () => {
  const f = fixture({ parsed: true }); f.gesture();
  const context = f.contexts[0];
  for (let i = 0; i < 7; i++) assert.equal(f.audio.playSniperShot(), true);
  assert.equal(context.sources.filter((node) => node.disconnects === 0).length, 4);
  assert.equal(context.sources[0].stops, 1);
  for (let i = 0; i < context.sources.length; i++) {
    const source = context.sources[i];
    const gain = context.gains[i];
    assert.equal(source.connections[0], gain);
    assert.equal(gain.connections[0], context.destination);
    assert.equal(gain.gain.value, 0.18);
    source.finish(); source.finish();
    assert.equal(source.onended, null);
    assert.equal(source.disconnects, 1);
    assert.equal(gain.disconnects, 1);
  }
  assert.equal(f.audio.playSniperShot(), true);
  assert.equal(context.sources[7].disconnects, 0);
});

test("suspension drops active tails and a closed context can be unlocked afresh", () => {
  const f = fixture({ parsed: true }); f.gesture(); f.audio.playSniperShot();
  const old = f.contexts[0];
  old.setState("suspended");
  assert.equal(old.sources[0].stops, 1);
  assert.equal(old.sources[0].disconnects, 1);
  assert.equal(f.audio.playSniperShot(), false);
  f.gesture();
  assert.equal(old.sources.length, 1);
  old.setState("closed"); f.gesture();
  assert.equal(f.contexts.length, 2);
  assert.equal(f.contexts[1].buffers.length, 1);
  assert.equal(f.audio.playSniperShot(), true);
  old.emit("statechange");
  assert.equal(f.contexts[1].sources[0].stops, 0);
});

test("closed context replacement releases old voices before a delayed statechange", () => {
  const f = fixture({ parsed: true }); f.gesture(); f.audio.playSniperShot();
  const old = f.contexts[0];
  // The state can be observed by a new gesture before its queued event arrives.
  old.state = "closed";
  f.gesture("game");
  assert.equal(f.contexts.length, 2);
  assert.equal(old.sources[0].stops, 1);
  assert.equal(old.sources[0].disconnects, 1);
  assert.equal(old.gains[0].disconnects, 1);
  assert.equal(f.audio.playSniperShot(), true);
  old.emit("statechange");
  assert.equal(f.contexts[1].sources[0].stops, 0);
  assert.equal(f.contexts[1].sources[0].disconnects, 0);
});

test("a failing source start releases its graph and reports unavailable playback", () => {
  const f = fixture({ parsed: true, startFailure: true }); f.gesture();
  for (let i = 0; i < 4; i++) assert.equal(f.audio.playSniperShot(), false);
  assert.ok(f.contexts[0].sources.every((node) => node.disconnects === 1));
  assert.ok(f.contexts[0].gains.every((node) => node.disconnects === 1));
});


async function loaded() {
  await new Promise(setImmediate);
}

test("bundled effects load once after trusted activation and BGM has a single loop", async () => {
  const f = fixture({ parsed: true, sampleFiles: true });
  assert.equal(f.audio.playEffect("attack"), false);
  f.gesture("enter", false);
  assert.equal(f.requests.length, 0);
  f.gesture();
  await loaded();
  assert.equal(f.requests.length, 5);
  const context = f.contexts[0];
  const music = context.sources[0];
  assert.equal(music.loop, true);
  assert.equal(music.buffer.name, "audio/choulingke-bgm.ogg");
  for (let i = 0; i < 6; i++) { f.audio.setActive(true); f.gesture("game"); }
  assert.equal(f.requests.length, 5);
  assert.equal(context.sources.length, 1);
  for (const name of ["attack", "hit", "deflect"]) {
    assert.equal(f.audio.playEffect(name), true);
    assert.equal(context.sources.at(-1).buffer.name, `audio/${name}.ogg`);
    assert.equal(context.sources.at(-1).loop, false);
  }
  assert.equal(f.audio.playEffect("bgm"), false);
  assert.equal(f.audio.playEffect("__proto__"), false);
});

test("pause stops voices and resumes music from its offset without overlapping loops", async () => {
  const f = fixture({ parsed: true, sampleFiles: true });
  f.gesture(); await loaded();
  const context = f.contexts[0];
  f.audio.playEffect("deflect");
  context.currentTime = 11;
  f.audio.setActive(false);
  assert.equal(f.audio.playEffect("attack"), false);
  assert.ok(context.sources.every((source) => source.stops === 1 && source.disconnects === 1));
  f.audio.setActive(true);
  const resumed = context.sources.at(-1);
  assert.deepEqual(Array.from(resumed.startArgs), [0, 11]);
  assert.equal(resumed.loop, true);
  f.window.emit("blur");
  assert.equal(resumed.stops, 1);
  assert.equal(f.audio.playSniperShot(), false);
  f.document.hidden = true;
  f.audio.setActive(true);
  assert.equal(f.audio.playEffect("hit"), false);
});

test("asset loading cannot start paused BGM or replay an effect requested before decode", async () => {
  const f = fixture({ parsed: true, sampleFiles: true });
  f.gesture();
  assert.equal(f.audio.playEffect("attack"), false);
  f.audio.setActive(false);
  await loaded();
  assert.equal(f.contexts[0].sources.length, 0);
  f.audio.setActive(true);
  assert.equal(f.contexts[0].sources.length, 1);
  assert.equal(f.contexts[0].sources[0].loop, true);
});

test("failed asset loads and decoding leave gameplay and procedural gunshots available", async () => {
  for (const options of [{ networkFailure: true }, { decodeFailure: true }]) {
    const f = fixture({ parsed: true, sampleFiles: true, ...options });
    f.gesture(); await loaded();
    assert.equal(f.audio.playEffect("attack"), false);
    assert.equal(f.audio.playSniperShot(), true);
    assert.equal(f.contexts[0].sources.filter((source) => source.loop).length, 0);
  }
});


test("unverified music stays absent without requesting a missing or arbitrary track", async () => {
  const f = fixture({ parsed: true, sampleFiles: true, noMusic: true });
  f.gesture(); await loaded();
  assert.equal(f.requests.length, 4);
  assert.equal(f.contexts[0].sources.length, 0);
  assert.equal(f.audio.playEffect("attack"), true);
  assert.equal(f.requests.some((url) => url.includes("choulingke-bgm")), false);
});

test("all bundled effects are actual Ogg files and Trunk copies their directory", () => {
  const root = path.join(__dirname, "..");
  for (const name of ["attack", "hit", "deflect"]) {
    const bytes = fs.readFileSync(path.join(root, "assets/audio", `${name}.ogg`));
    assert.equal(bytes.subarray(0, 4).toString("ascii"), "OggS");
    assert.ok(bytes.length > 1000);
  }
  const music = JSON.parse(fs.readFileSync(path.join(root, "assets/audio/music.json")));
  if (music.file) assert.ok(fs.existsSync(path.join(root, "assets/audio", music.file)));
  const html = fs.readFileSync(path.join(root, "index.html"), "utf8");
  assert.match(html, /data-trunk rel="copy-dir" href="assets\/audio"/);
});
