const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = (name) => fs.readFileSync(path.join(__dirname, "../web", name), "utf8");
const html = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");

class Target {
  constructor(tag = "div") { this.tagName = tag.toUpperCase(); }
  listeners = new Map();
  attributes = new Map();
  dataset = {};
  children = [];
  hidden = false;
  disabled = false;
  checked = false;
  value = "";
  textContent = "";
  clientHeight = 600;
  set innerHTML(_) { throw new Error("No HTML injection allowed"); }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  getAttribute(name) { return this.attributes.get(name) ?? null; }
  removeAttribute(name) { this.attributes.delete(name); }
  addEventListener(type, callback, options) {
    const entries = this.listeners.get(type) || [];
    entries.push({ callback, once: options?.once }); this.listeners.set(type, entries);
  }
  dispatchEvent(event) {
    event.target ||= this;
    for (const entry of [...(this.listeners.get(event.type) || [])]) {
      if (entry.once) this.listeners.set(event.type, this.listeners.get(event.type).filter((item) => item !== entry));
      entry.callback(event);
    }
  }
  appendChild(child) { this.children.push(child); child.parentNode = this; return child; }
  replaceChildren(...children) { this.children = []; children.forEach((child) => this.appendChild(child)); }
  querySelectorAll(selector) {
    const result = [];
    const match = (child) => selector === "input[data-part]" && child.tagName === "INPUT" && child.dataset.part;
    const visit = (parent) => parent.children.forEach((child) => { if (match(child)) result.push(child); visit(child); });
    visit(this); return result;
  }
  setPointerCapture(id) { this.captured = id; }
  releasePointerCapture(id) { if (this.captured === id) this.captured = null; }
  focus() { this.focused = true; }
}
function emit(target, type, fields = {}) {
  const event = { type, defaultPrevented: false, preventDefault() { this.defaultPrevented = true; }, ...fields };
  target.dispatchEvent(event); return event;
}
function fixture(search = "?view=model") {
  let reloads = 0;
  let copied = "";
  let clipboardFails = false;
  const window = Object.assign(new Target(), { location: { search, reload() { reloads++; } } });
  const document = Object.assign(new Target(), { readyState: "loading", documentElement: new Target("html") });
  const nodes = new Map();
  const controls = [];
  document.getElementById = (id) => nodes.get(id) || null;
  document.createElement = (tag) => new Target(tag);
  document.querySelectorAll = (selector) => controls.filter((control) =>
    selector === "[data-debug]" && control.dataset.debug || selector === "[data-view]" && control.dataset.view);
  const navigator = { gpu: {}, clipboard: { async writeText(text) { if (clipboardFails) throw new Error("denied"); copied = text; } } };
  const context = vm.createContext({ window, document, navigator, URLSearchParams,
    CustomEvent: class { constructor(type) { this.type = type; } } });
  const run = (name) => vm.runInContext(source(name), context);
  run("mode.js"); run("startup.js"); run("viewer.js");
  function parse() {
    for (const match of html.matchAll(/<([a-z]+)\b([^>]*\bid="([^"]+)"[^>]*)>/g)) {
      const item = new Target(match[1]); item.id = match[3];
      if (/\bhidden\b/.test(match[2])) item.hidden = true;
      nodes.set(item.id, item);
    }
    for (const match of html.matchAll(/<([a-z]+)\b[^>]*data-(debug|view)="([^"]+)"[^>]*>/g)) {
      const item = new Target(match[1]); item.dataset[match[2]] = match[3]; controls.push(item);
    }
    document.readyState = "interactive"; emit(document, "DOMContentLoaded");
  }
  const render = (changes = {}) => window.kivaViewer.render(JSON.stringify({ ...snapshot(), ...changes }));
  return { window, document, nodes, controls, navigator, parse, render, run, reloads: () => reloads,
    copy: () => copied, denyClipboard: () => { clipboardFails = true; },
    click: (name) => emit(nodes.get(`viewer-${name}`), "click"),
    change: (name, value) => { const item = nodes.get(`viewer-${name}`); item.value = value; emit(item, "change"); },
    take: () => window.kivaViewer.takeCommands() };
}
function snapshot() {
  return { version: 1, revision: 1, ready: true, appearance: "lihuowang", weapon: "fulangmao", preset: "brother_hammer",
    mode: "game", action: "game:idle", time: 0.25, duration: 2, frame: 15, playing: false, speed: 1, looping: true,
    trace: "actual sampler: Idle01\nsource mix=0", warning: "", status: "模型已就绪",
    camera: { yaw: 0, pitch: 0.2, distance: 3, target: [0, 1, 0] },
    debug: { grid: true, axes: true, bones: false, grips: false, bounds: false },
    parts: [{ id: "body:0", label: "衣装 <script>", visible: true }],
    catalog: {
      appearances: [{ id: "lihuowang", label: "李火旺" }, { id: "bailingmiao", label: "白灵淼" }],
      weapons: [{ id: "fulangmao", label: "伏浪锚" }, { id: "none", label: "无手持" }],
      presets: [{ id: "brother_hammer", label: "哥哥 / 伏浪锚" }],
      actions: [{ id: "game:idle", label: "待机", mode: "game", group: "游戏单体", duration: 2 },
        { id: "cc0:Idle01", label: "Idle01", mode: "source", group: "CC0", duration: 1.5 }],
    }, inspection: "角色：李火旺\n武器：伏浪锚\n帧：15" };
}

test("immutable page mode only accepts exact model query and precedes game bridges", () => {
  for (const search of ["", "?view=unknown", "?view=Model", "?foo=model"]) {
    const f = fixture(search); assert.equal(f.window.kivaPage.isViewer(), false); assert.equal(f.window.kivaViewer, undefined);
  }
  const f = fixture("?view=model&x=1");
  assert.equal(f.window.kivaPage.isViewer(), true);
  f.window.location.search = "";
  assert.equal(f.window.kivaPage.isViewer(), true);
  assert.equal(Object.isFrozen(f.window.kivaPage), true);
  for (const name of ["input.js", "hud.js", "enemies.js", "appearance.js", "audio.js"]) {
    assert.ok(html.indexOf('src="mode.js"') < html.indexOf(`src="${name}"`));
  }
});

test("viewer mode installs no game capture, HUD, selection or audio bridges/listeners", () => {
  const f = fixture();
  const beforeWindow = [...f.window.listeners.keys()];
  const beforeDocument = [...f.document.listeners.keys()];
  for (const name of ["input.js", "hud.js", "enemies.js", "appearance.js", "audio.js"]) f.run(name);
  for (const name of ["kivaInput", "kivaHud", "kivaEnemies", "kivaAppearance", "kivaAudio"]) assert.equal(f.window[name], undefined);
  assert.deepEqual([...f.window.listeners.keys()], beforeWindow);
  assert.deepEqual([...f.document.listeners.keys()], beforeDocument);
});

test("early snapshot/catalog replay waits for true engine ready and current model ready", () => {
  const f = fixture(); f.render({ ready: false }); f.parse();
  assert.equal(f.nodes.get("game").parentNode, f.nodes.get("viewer-viewport"));
  assert.equal(f.nodes.get("viewer-current").textContent, "李火旺 · 伏浪锚");
  assert.equal(f.nodes.get("viewer-loading").hidden, false);
  f.window.kivaViewer.ready();
  assert.equal(f.window.__kivaStartup, "ready");
  assert.equal(f.nodes.get("viewer-loading").hidden, false);
  f.render({ catalog: undefined });
  assert.equal(f.nodes.get("viewer-loading").hidden, true);
  assert.equal(f.nodes.get("viewer-appearance").children.length, 2);
  assert.equal(f.nodes.get("viewer-time").textContent, "0.250 / 2.000 s · 第 15 帧");
  assert.equal(f.nodes.get("viewer-parts").children[0].children[1].textContent, "衣装 <script>");
});

test("transport sends bounded commands once without optimistic playback or a JS clock", () => {
  const f = fixture(); f.parse(); f.render();
  f.click("play"); f.click("step-next"); f.change("speed", "0.5");
  const timeline = f.nodes.get("viewer-timeline"); timeline.value = "1.5"; emit(timeline, "input");
  assert.equal(f.take(), "play\t1\nstep\t1\nspeed\t0.5\nseek\t1.5");
  assert.equal(f.take(), "");
  assert.equal(f.nodes.get("viewer-play").textContent, "播放");
  assert.equal(f.nodes.get("viewer-time").textContent, "0.250 / 2.000 s · 第 15 帧");
  f.render({ playing: true, time: 1.5, frame: 90 });
  assert.equal(f.nodes.get("viewer-play").textContent, "暂停");
  assert.equal(f.nodes.get("viewer-time").textContent, "1.500 / 2.000 s · 第 90 帧");
});

test("latest selections coalesce, malformed IDs/nonfinite values never cross bridge", () => {
  const f = fixture(); f.parse(); f.render();
  f.change("appearance", "bailingmiao"); f.change("appearance", "lihuowang");
  f.change("weapon", "none\nplay\t1"); f.change("speed", "NaN");
  f.change("weapon", "x".repeat(161));
  assert.equal(f.take(), "appearance\tlihuowang");
  for (let index = 0; index < 500; index++) f.click("step-next");
  assert.equal(f.take().split("\n").length, 128);
});

test("stale/invalid snapshots cannot replace confirmed state or erase retained catalog", () => {
  const f = fixture(); f.parse(); f.render({ revision: 3 });
  f.render({ revision: 2, time: 1 });
  f.render({ revision: 4, speed: -1 });
  f.window.kivaViewer.render("not JSON");
  f.render({ revision: 3, catalog: undefined, time: 0.5, frame: 30 });
  assert.equal(f.nodes.get("viewer-time").textContent, "0.500 / 2.000 s · 第 30 帧");
  assert.equal(f.nodes.get("viewer-appearance").children.length, 2);
});

test("source catalog, search and reference are explicit and driven by Rust snapshot", () => {
  const f = fixture(); f.parse(); f.render();
  f.click("source-mode"); assert.equal(f.take(), "mode\tsource");
  assert.equal(f.nodes.get("viewer-source-mode").getAttribute("aria-pressed"), "false");
  f.render({ mode: "source", action: "cc0:Idle01" });
  assert.equal(f.nodes.get("viewer-action").children[0].textContent, "CC0 · Idle01");
  assert.match(f.nodes.get("viewer-mode-help").textContent, /不叠加游戏持握修正/);
  const search = f.nodes.get("viewer-search"); search.value = "absent"; emit(search, "input");
  assert.equal(f.nodes.get("viewer-action").disabled, true);
  f.click("reference"); assert.equal(f.take(), "mode\treference");
  f.render({ mode: "reference" });
  assert.match(f.nodes.get("viewer-mode-help").textContent, /不一定等于蒙皮绑定/);
});

test("camera uses canvas-only pointer capture; form interaction, blur and cancellation do not leak deltas", () => {
  const f = fixture(); f.parse(); f.render(); const canvas = f.nodes.get("game");
  emit(f.document, "pointermove", { pointerId: 1, clientX: 30, clientY: 40 });
  emit(f.nodes.get("viewer-search"), "wheel", { deltaY: 50 });
  assert.equal(f.take(), "");
  emit(canvas, "pointerdown", { pointerId: 1, button: 0, clientX: 10, clientY: 20 });
  emit(canvas, "pointermove", { pointerId: 1, clientX: 22, clientY: 25 });
  assert.equal(canvas.captured, 1); assert.equal(f.take(), "orbit\t12\t5");
  emit(f.window, "blur");
  emit(canvas, "pointermove", { pointerId: 1, clientX: 40, clientY: 60 });
  assert.equal(f.take(), ""); assert.equal(canvas.captured, null);
  emit(canvas, "pointerdown", { pointerId: 2, button: 2, clientX: 30, clientY: 30 });
  emit(canvas, "pointermove", { pointerId: 2, clientX: 35, clientY: 27 });
  emit(canvas, "pointercancel", { pointerId: 2 });
  emit(canvas, "pointermove", { pointerId: 2, clientX: 40, clientY: 50 });
  assert.equal(f.take(), "pan\t5\t-3");
  const wheel = emit(canvas, "wheel", { deltaY: 2, deltaMode: 1 });
  assert.equal(f.take(), "zoom\t32"); assert.equal(wheel.defaultPrevented, true);
  assert.equal(f.document.listeners.has("pointerlockchange"), false);
});

test("part hiding is unavailable before selected render readiness and does not mutate source state", () => {
  const f = fixture(); f.parse(); f.render({ ready: false });
  const part = f.nodes.get("viewer-parts").children[0].children[0];
  assert.equal(part.disabled, true);
  f.render(); assert.equal(part.disabled, false);
  part.checked = false; emit(part, "change");
  assert.equal(f.take(), "part\tbody:0\t0");
  f.render(); assert.equal(part.checked, true);
});

test("copy uses confirmed inspection text, clipboard rejection is visible and retry works", async () => {
  const f = fixture(); f.parse(); f.render(); f.click("copy"); await new Promise(setImmediate);
  assert.equal(f.copy(), snapshot().inspection);
  assert.match(f.nodes.get("viewer-copy-status").textContent, /已复制/);
  f.denyClipboard(); f.click("copy"); await new Promise(setImmediate);
  assert.match(f.nodes.get("viewer-copy-status").textContent, /复制未成功/);
});

test("download/failure replay never fakes readiness; only retry requests a reload", () => {
  const f = fixture(); f.window.kivaStartup.progress("download", 512, 1024); f.parse();
  assert.equal(f.nodes.get("viewer-loading-label").textContent, "正在下载 50%");
  assert.equal(f.nodes.get("viewer-progress").value, 512);
  f.window.kivaStartup.progress("download", 1024, 1024);
  assert.equal(f.nodes.get("viewer-loading").hidden, false);
  f.window.kivaStartup.fail("test failure");
  assert.equal(f.nodes.get("viewer-retry").hidden, false);
  f.click("retry"); assert.equal(f.reloads(), 1);
  f.render(); f.window.kivaViewer.ready();
  assert.equal(f.nodes.get("viewer-loading").hidden, true);
  assert.equal(f.window.__kivaStartup, "ready");
});


test("render failure after global ready remains recoverable without resetting startup", () => {
  const f = fixture(); f.parse(); f.render(); f.window.kivaViewer.ready();
  assert.equal(f.nodes.get("viewer-loading").hidden, true);
  f.render({ failed: true, ready: false, status: "所选模型管线失败" });
  assert.equal(f.window.__kivaStartup, "ready");
  assert.equal(f.nodes.get("viewer-loading").hidden, false);
  assert.equal(f.nodes.get("viewer-retry").hidden, false);
  assert.equal(f.nodes.get("viewer-loading-detail").textContent, "所选模型管线失败");
  f.render(); assert.equal(f.nodes.get("viewer-loading").hidden, true);
  f.window.kivaViewer.fail("GPU device lost");
  assert.equal(f.nodes.get("viewer-retry").hidden, false);
  assert.equal(f.nodes.get("viewer-loading-detail").textContent, "GPU device lost");
  f.click("retry"); assert.equal(f.reloads(), 1);
  f.render(); assert.equal(f.nodes.get("viewer-loading").hidden, true);
});
