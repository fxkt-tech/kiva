const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const bundledCues = JSON.parse(fs.readFileSync(path.join(__dirname, "../assets/audio/naraka-cues.json"), "utf8"));
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
      this.startAttempts = 0;
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
      const audio = this;
      const node = {
        connections: [], disconnects: 0, starts: 0, stops: 0,
        connect(target) { this.connections.push(target); },
        disconnect() { this.disconnects++; },
        start(...args) { audio.startAttempts++; if (options.startFailure || audio.failAtStart === audio.startAttempts) throw new Error("source unavailable"); this.starts++; this.startArgs = args; },
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
  if (options.viewer) window.kivaPage = { isViewer: () => true };
  if (!options.unsupported) window[options.webkit ? "webkitAudioContext" : "AudioContext"] = AudioContext;
  const document = Object.assign(new Target(), { readyState: options.parsed ? "complete" : "loading" });
  const nodes = new Map();
  document.getElementById = (id) => nodes.get(id) || null;
  const addNodes = () => {
    for (const id of ["enter", "game"]) nodes.set(id, new Target());
  };
  if (options.parsed) addNodes();
  const requests = [];
  const metadataResolvers = [];
  const fetch = options.sampleFiles ? async (url) => {
    requests.push(url);
    if (options.networkFailure) throw new Error("offline");
    if (url === "audio/naraka-cues.json" && options.deferredCues) {
      await new Promise((resolve) => metadataResolvers.push(resolve));
    }
    return { ok: !options.failedFiles?.includes(url), arrayBuffer: async () => url,
      json: async () => ({ file: options.noMusic ? null : "choulingke-bgm.ogg" }),
      text: async () => options.cuesText ?? JSON.stringify(options.cues ?? bundledCues),
    };
  } : undefined;
  const math = Object.create(Math);
  if (options.random) math.random = options.random;
  vm.runInContext(source, vm.createContext({ window, document, fetch, Math: math }));
  const parse = () => { addNodes(); document.readyState = "interactive"; document.emit("DOMContentLoaded"); };
  const gesture = (id = "enter", trusted = true) => {
    const event = nodes.get(id).emit(id === "enter" ? "click" : "pointerdown", { isTrusted: trusted });
    if (trusted) window.kivaAudio.setActive(true);
    return event;
  };
  return { audio: window.kivaAudio, window, document, nodes, contexts, requests, metadataResolvers, parse, gesture };
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
  assert.equal(f.requests.length, 54);
  const context = f.contexts[0];
  const music = context.sources[0];
  assert.equal(music.loop, true);
  assert.equal(music.buffer.name, "audio/choulingke-bgm.ogg");
  for (let i = 0; i < 6; i++) { f.audio.setActive(true); f.gesture("game"); }
  assert.equal(f.requests.length, 54);
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
  assert.equal(f.requests.length, 53);
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

function smallCues(cues) {
  return { version: 1, cues };
}
function layer(file = "naraka-1.ogg", gain = 1, delay = 0) {
  return { delay, variants: [{ file, weight: 1, gain }] };
}
function cue(priority = 2, layers = [layer()], offset = 0) {
  return { priority, offset, layers };
}

test("all sixteen original cues share one decode per media and keep natural pitch", async () => {
  const f = fixture({ parsed: true, sampleFiles: true }); f.gesture();
  assert.equal(f.audio.playEffect("naraka_attack_1"), false);
  await loaded();
  const media = f.requests.filter((url) => /^audio\/naraka-[0-9]+\.ogg$/.test(url));
  assert.equal(media.length, 48);
  assert.equal(new Set(media).size, 48);
  for (const name of Object.keys(bundledCues.cues)) {
    assert.equal(f.audio.playEffect(name), true, name);
    for (const node of f.contexts[0].sources.filter((source) => !source.loop)) {
      assert.equal(node.playbackRate, undefined);
      node.finish();
    }
  }
  assert.equal(f.audio.playEffect(null), false);
  assert.equal(f.audio.playEffect("toString"), false);
  assert.equal(f.audio.cueOffset("naraka_attack_1"), 0);
});

test("weighted independent variants apply each imported gain exactly once", async () => {
  const data = smallCues({ naraka_attack_1: cue(2, [{ delay: 0, variants: [
    { file: "naraka-1.ogg", weight: 1, gain: 0.5 },
    { file: "naraka-2.ogg", weight: 3, gain: 0.8 },
  ] }], 0.25) });
  for (const [random, name, gain] of [[0, "naraka-1.ogg", 0.5], [0.99, "naraka-2.ogg", 0.8]]) {
    const f = fixture({ parsed: true, sampleFiles: true, cues: data, random: () => random });
    f.gesture(); await loaded();
    assert.equal(f.audio.cueOffset("naraka_attack_1"), 0.25);
    assert.equal(f.audio.playEffect("naraka_attack_1"), true);
    const source = f.contexts[0].sources.at(-1);
    assert.equal(source.buffer.name, `audio/${name}`);
    assert.equal(source.connections[0].gain.value, 0.18 * gain);
  }
});

test("landing schedules two independent original layers, including future voices in pause cleanup", async () => {
  let randomCalls = 0;
  const f = fixture({ parsed: true, sampleFiles: true, random: () => (randomCalls++ % 2 ? 0.5 : 0) });
  f.gesture(); await loaded();
  const context = f.contexts[0]; context.currentTime = 7;
  assert.equal(f.audio.playEffect("naraka_land"), true);
  const layers = context.sources.slice(-2);
  assert.deepEqual(Array.from(layers[0].startArgs), [7.1, 0]);
  assert.deepEqual(Array.from(layers[1].startArgs), [7, 0]);
  assert.notEqual(layers[0].buffer, layers[1].buffer);
  assert.equal(randomCalls, 2);
  f.audio.setActive(false);
  assert.ok(layers.every((node) => node.stops === 1 && node.disconnects === 1));
  f.audio.setActive(true);
  assert.equal(context.sources.filter((node) => !node.loop).length, 2);
});

test("four effect slots protect critical cues and reserve all layers before stealing", async () => {
  const f = fixture({ parsed: true, sampleFiles: true }); f.gesture(); await loaded();
  const context = f.contexts[0];
  for (let i = 0; i < 4; i++) assert.equal(f.audio.playEffect("naraka_parry"), true);
  const critical = context.sources.slice(-4);
  const count = context.sources.length;
  assert.equal(f.audio.playEffect("naraka_step"), false);
  assert.equal(f.audio.playEffect("naraka_land"), false);
  assert.equal(context.sources.length, count);
  assert.ok(critical.every((node) => node.stops === 0));
  critical[0].finish();
  assert.equal(f.audio.playEffect("naraka_step"), true);
  const step = context.sources.at(-1);
  // One expendable slot is insufficient for the two-layer landing.
  assert.equal(f.audio.playEffect("naraka_land"), false);
  assert.equal(step.stops, 0);
  assert.equal(f.audio.playEffect("naraka_parry"), true);
  assert.equal(step.stops, 1);
  assert.ok(critical.slice(1).every((node) => node.stops === 0));
  assert.equal(context.sources.filter((node) => !node.loop && node.disconnects === 0).length, 4);
  assert.equal(context.sources.filter((node) => node.loop && node.disconnects === 0).length, 1);
});

test("a second-layer start failure cleans the entire cue without stopping BGM", async () => {
  const f = fixture({ parsed: true, sampleFiles: true }); f.gesture(); await loaded();
  const context = f.contexts[0];
  context.failAtStart = context.startAttempts + 2;
  assert.equal(f.audio.playEffect("naraka_land"), false);
  const layers = context.sources.slice(-2);
  assert.equal(layers[0].stops, 1);
  assert.ok(layers.every((node) => node.disconnects === 1));
  assert.equal(context.sources.find((node) => node.loop).stops, 0);
  assert.equal(f.audio.playSniperShot(), true);
});

test("invalid bounded metadata never requests arbitrary files or disables old audio", async () => {
  const valid = smallCues({ naraka_attack_1: cue() });
  const invalid = [
    { ...valid, version: 2 }, { version: 1, cues: [] },
    smallCues({ unknown: cue() }),
    smallCues({ naraka_attack_1: cue(4) }),
    smallCues({ naraka_attack_1: cue(2, [layer("../secret.ogg")]) }),
    smallCues({ naraka_attack_1: cue(2, [layer("https://example.com/a.ogg")]) }),
    smallCues({ naraka_attack_1: cue(2, [layer("naraka-1.ogg", 1.1)]) }),
    smallCues({ naraka_attack_1: cue(2, [layer("naraka-1.ogg", 1, -1)]) }),
    smallCues({ naraka_attack_1: cue(2, Array.from({ length: 5 }, () => layer())) }),
    smallCues({ naraka_attack_1: cue(2, [{ delay: 0, variants: [{ file: "naraka-1.ogg", weight: 0, gain: 1 }] }]) }),
    smallCues({ naraka_attack_1: cue(2, [{ delay: 0, variants: Array.from({ length: 17 }, () => layer().variants[0]) }]) }),
    smallCues({ naraka_attack_1: cue(2, [layer()], -0.1) }),
  ];
  for (const data of invalid) {
    const f = fixture({ parsed: true, sampleFiles: true, cues: data }); f.gesture(); await loaded();
    assert.equal(f.audio.playEffect("naraka_attack_1"), false);
    assert.equal(f.audio.playEffect("attack"), true);
    assert.equal(f.audio.playSniperShot(), true);
    assert.equal(f.requests.some((url) => /^audio\/naraka-[0-9]/.test(url)), false);
  }
  for (const cuesText of ['{"version":1,"cues":{"naraka_attack_1":{"priority":2,"offset":1e999,"layers":[]}}}', " ".repeat(65537), "{"]) {
    const f = fixture({ parsed: true, sampleFiles: true, cuesText }); f.gesture(); await loaded();
    assert.equal(f.audio.playEffect("naraka_attack_1"), false);
    assert.equal(f.audio.playSniperShot(), true);
  }
});

test("metadata arriving from a closed context cannot populate the replacement or replay requests", async () => {
  const f = fixture({ parsed: true, sampleFiles: true, deferredCues: true }); f.gesture(); await loaded();
  const old = f.contexts[0]; old.state = "closed";
  f.gesture("game"); await loaded();
  assert.equal(f.metadataResolvers.length, 2);
  assert.equal(f.audio.playEffect("naraka_attack_1"), false);
  f.metadataResolvers[0](); await loaded();
  assert.equal(f.requests.filter((url) => /^audio\/naraka-[0-9]/.test(url)).length, 0);
  f.metadataResolvers[1](); await loaded();
  assert.equal(f.requests.filter((url) => /^audio\/naraka-[0-9]/.test(url)).length, 48);
  const current = f.contexts[1];
  assert.equal(current.sources.filter((node) => !node.loop).length, 0);
  assert.equal(f.audio.playEffect("naraka_attack_1"), true);
  const playing = current.sources.at(-1);
  old.emit("statechange");
  assert.equal(playing.stops, 0);
  assert.equal(f.audio.playEffect("naraka_attack_2"), true);
});

test("missing selected media is discarded without reducing a layered cue or replaying on resume", async () => {
  const data = smallCues({ naraka_land: cue(1, [layer("naraka-1.ogg"), layer("naraka-2.ogg", 1, 0.1)]) });
  const f = fixture({ parsed: true, sampleFiles: true, cues: data, failedFiles: ["audio/naraka-2.ogg"] });
  f.gesture(); await loaded();
  assert.equal(f.audio.playEffect("naraka_land"), false);
  assert.equal(f.contexts[0].sources.filter((node) => !node.loop).length, 0);
  f.audio.setActive(false); f.audio.setActive(true);
  assert.equal(f.contexts[0].sources.filter((node) => !node.loop).length, 0);
});

test("viewer never installs an audio bridge, gesture listeners, or fetches", () => {
  const f = fixture({ parsed: true, sampleFiles: true, viewer: true });
  assert.equal(f.audio, undefined);
  assert.equal(f.nodes.get("enter").listeners.size, 0);
  assert.equal(f.nodes.get("game").listeners.size, 0);
  assert.equal(f.contexts.length, 0);
  assert.equal(f.requests.length, 0);
});

test("original ranged family cues preserve all six variants, gain, zero delay, and legacy gun access", async () => {
  let random = 0;
  const f = fixture({ parsed: true, sampleFiles: true, random: () => random });
  f.gesture();
  assert.equal(f.audio.playEffect("naraka_gun"), false);
  assert.equal(f.audio.playEffect("naraka_bow"), false);
  await loaded();
  const context = f.contexts[0]; context.currentTime = 3;
  assert.equal(context.sources.filter((source) => !source.loop).length, 0);
  const used = new Set();
  for (const name of ["naraka_gun", "naraka_bow"]) {
    const configured = bundledCues.cues[name];
    assert.equal(configured.priority, 2);
    assert.equal(configured.offset, 0);
    assert.equal(configured.layers.length, 1);
    assert.equal(configured.layers[0].delay, 0);
    for (const [index, value] of [0, 0.5, 0.99].entries()) {
      random = value;
      const count = context.sources.length;
      assert.equal(f.audio.playEffect(name), true);
      assert.equal(context.sources.length, count + 1);
      const source = context.sources.at(-1);
      const variant = configured.layers[0].variants[index];
      assert.equal(source.buffer.name, `audio/${variant.file}`);
      assert.equal(source.connections[0].gain.value, 0.18 * variant.gain);
      assert.deepEqual(Array.from(source.startArgs), [3, 0]);
      assert.equal(source.playbackRate, undefined);
      used.add(variant.file);
      source.finish();
    }
  }
  assert.equal(used.size, 6);
  assert.equal(f.audio.playSniperShot(), true);
  assert.equal(context.sources.at(-1).buffer, context.buffers[0]);
});

test("original ranged undecoded media never invokes procedural fallback or queues an old shot", async () => {
  const first = bundledCues.cues.naraka_gun.layers[0].variants[0].file;
  const f = fixture({ parsed: true, sampleFiles: true, random: () => 0, failedFiles: [`audio/${first}`] });
  f.gesture(); await loaded();
  assert.equal(f.audio.playEffect("naraka_gun"), false);
  assert.equal(f.contexts[0].sources.filter((source) => !source.loop).length, 0);
  f.audio.setActive(false); f.audio.setActive(true);
  assert.equal(f.contexts[0].sources.filter((source) => !source.loop).length, 0);
  assert.equal(f.audio.playEffect("naraka_bow"), true);
  assert.equal(f.contexts[0].sources.filter((source) => !source.loop).length, 1);
});
