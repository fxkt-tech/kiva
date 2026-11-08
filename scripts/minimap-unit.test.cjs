// The real minimap module, with DOM/canvas effects recorded at its public seam.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

const source = fs.readFileSync(path.join(__dirname, "../web/minimap.js"), "utf8");

function fixture(dpr = 1) {
  class Target {
    listeners = new Map();
    addEventListener(type, listener) {
      if (!this.listeners.has(type)) this.listeners.set(type, []);
      this.listeners.get(type).push(listener);
    }
    emit(type, properties = {}) {
      const event = { type, preventDefault() {}, ...properties };
      for (const listener of this.listeners.get(type) || []) listener(event);
    }
  }
  const mutations = [];
  const calls = [];
  const context = new Proxy({}, { get(target, name) {
    if (name in target) return target[name];
    return (...args) => calls.push([name, ...args]);
  } });
  const bounds = { left: 600, top: 400, x: 600, y: 400, width: 200, height: 200 };
  const root = { dataset: { paused: "false", pointerLocked: "true", sessionPhase: "Active" } };
  const frame = { style: {}, dataset: {}, hidden: true };
  const game = {};
  const canvas = Object.assign(new Target(), {
    clientWidth: 200, clientHeight: 200, width: 180, height: 180, dataset: {},
    getContext: () => context,
    getBoundingClientRect: () => ({ ...bounds }),
  });
  const stage = { contains: (node) => node === canvas || node === frame,
    getBoundingClientRect: () => ({ ...bounds }) };
  const elements = { game, minimap: canvas, "minimap-stage": stage, "minimap-viewport": frame };
  canvas.parentElement = stage;
  let point = [0, 0];
  let obscured = false;
  let uiCaptured = false;
  const document = Object.assign(new Target(), {
    documentElement: root, pointerLockElement: game, focused: true,
    hasFocus() { return this.focused; },
    getElementById: (id) => elements[id],
    elementFromPoint: (x, y) => !obscured && x >= bounds.left && x <= bounds.left + bounds.width
      && y >= bounds.top && y <= bounds.top + bounds.height ? canvas : null,
  });
  const window = Object.assign(new Target(), {
    devicePixelRatio: dpr,
    kivaPointer: { position: () => [...point], setUiCapture(value) { uiCaptured = value; } },
  });
  vm.runInNewContext(source, { window, document, MutationObserver: class {
    constructor(callback) { mutations.push(callback); }
    observe() {}
  } }, { filename: "web/minimap.js" });
  const minimap = window.createMinimap(canvas);
  const state = {
    player_id: "4294966763", session: { phase: "Active" }, paused: false,
    map: { half_size: 32, lane_width: 7, ally_base: [-8, 24], enemy_base: [8, -24] },
    camera_center: [0, 0], camera_footprint: [[-4, -6], [6, -6], [6, 4], [-4, 4]],
    obstacles: [[12, 10, 2, 3]],
    units: [{ id: "4294966763", alive: true, team: "ally", role: "Hero", position: [0, 0] }],
  };
  minimap.render(state);
  const event = (properties = {}) => ({
    isTrusted: true, pointerType: "mouse", button: -1, buttons: 1,
    // Pointer Lock's native client coordinates are intentionally unrelated.
    clientX: 12, clientY: 13, ...properties,
  });
  const pixel = ([x, z]) => [bounds.left + 8 + (x + 32) * (bounds.width - 16) / 64,
    bounds.top + 8 + (z + 32) * (bounds.height - 16) / 64];
  return {
    root, document, window, frame, canvas, calls, state, minimap, bounds, pixel,
    place(position) { point = pixel(position); },
    placeCss(position) { point = position; },
    down(properties = {}) { document.emit("pointerdown", event({ button: 0, ...properties })); },
    move(properties = {}) { document.emit("pointermove", event(properties)); },
    up(properties = {}) { document.emit("pointerup", event({ button: 0, buttons: 0, ...properties })); },
    camera(value) { window.emit("kiva-camera", { detail: JSON.stringify(value) }); },
    request: () => root.dataset.cameraRequest ? JSON.parse(root.dataset.cameraRequest) : null,
    uiCaptured: () => uiCaptured,
    clearRequest() { delete root.dataset.cameraRequest; },
    flushMutations() { mutations.forEach((callback) => callback()); },
    obscure() { obscured = true; },
  };
}

test("the full map has a fixed boundary, common world projection and DPR-only backing", () => {
  const f = fixture(2);
  assert.equal(f.canvas.dataset.mapInset, "8");
  assert.equal(f.canvas.width, 400);
  assert.equal(f.canvas.height, 400);
  assert(f.calls.some(([name, x, y, width, height]) => name === "strokeRect"
    && x === 8 && y === 8 && width === 184 && height === 184), "outline the complete map");
  assert(f.calls.some(([name, x, y]) => name === "moveTo" && x === 77 && y === 169),
    "the allied lane end uses the full map projection");
  assert(f.calls.some(([name, x, y]) => name === "lineTo" && x === 123 && y === 31),
    "the enemy lane end stays on the same full map");
  f.place([16, -16]);
  f.down();
  assert.deepEqual(f.request(), { player_id: "4294966763", center: [16, -16] },
    "DPR must not multiply the CSS-to-world input");
});

test("a rectangular viewport uses the real four-corner bounds and per-frame camera channel", () => {
  const f = fixture();
  assert.equal(f.frame.hidden, false);
  assert.equal(f.frame.style.left, "88.5px");
  assert.equal(f.frame.style.top, "82.75px");
  assert.equal(f.frame.style.width, "28.75px");
  assert.equal(f.frame.style.height, "28.75px");
  const drawn = f.calls.length;
  f.camera({ center: [12, 10], footprint: [[2, 0], [22, 0], [20, 20], [4, 20]] });
  assert.equal(f.frame.style.left, "105.75px");
  assert.equal(f.frame.style.top, "100px");
  assert.equal(f.frame.style.width, "57.5px");
  assert.equal(f.frame.style.height, "57.5px");
  assert.equal(f.calls.length, drawn, "camera frames must not redraw terrain or units");
  f.camera({ center: [0, 0], footprint: null });
  assert.equal(f.frame.hidden, true, "invalid ground rays cannot invent a viewport");
});

test("clicking then dragging outside the viewport submits only the latest world target", () => {
  const f = fixture();
  f.place([20, 12]);
  f.down();
  assert.equal(f.root.dataset.minimapDragging, "true");
  assert.equal(f.uiCaptured(), true, "the pointer owner must retain UI capture in event-time presses");
  assert.deepEqual(f.request().center, [20, 12]);
  f.place([24, 16]);
  f.move();
  assert.deepEqual(f.request().center, [24, 16]);
  f.up();
  assert.equal(f.root.dataset.minimapDragging, undefined);
  assert.equal(f.uiCaptured(), false);
  assert.deepEqual(f.request().center, [24, 16], "release preserves the last valid target for Rust");
  f.place([0, 0]);
  f.move({ buttons: 0 });
  assert.deepEqual(f.request().center, [24, 16], "released movement cannot continue dragging");
});

test("a chorded left press starts the same gesture while ordinary held motion cannot", () => {
  const f = fixture();
  f.place([20, 20]);
  f.move({ button: -1, buttons: 1 });
  assert.equal(f.request(), null, "moving over the map with a previously held button is not a new press");
  f.down({ button: 2, buttons: 2 });
  assert.equal(f.request(), null);
  f.move({ button: 0, buttons: 3 });
  assert.deepEqual(f.request().center, [20, 20]);
  assert.equal(f.root.dataset.minimapDragging, "true");
  assert.equal(f.uiCaptured(), true);
  f.place([24, 22]);
  f.move({ button: -1, buttons: 3 });
  assert.deepEqual(f.request().center, [24, 22]);
  f.move({ button: 0, buttons: 2 });
  assert.deepEqual(f.request().center, [24, 22], "a chorded left release preserves the final valid target for Rust");
  assert.equal(f.uiCaptured(), false);
  assert.equal(f.root.dataset.minimapDragging, undefined);
});

test("grabbing inside the viewport preserves its actual camera-center offset", () => {
  const f = fixture();
  f.place([3, 2]);
  f.down();
  assert.equal(f.request(), null, "grabbing the box cannot jump the camera");
  f.place([7, 5]);
  f.move();
  assert.deepEqual(f.request().center, [4, 3]);
  f.camera({ center: [4, 3], footprint: [[0, -3], [10, -3], [10, 7], [0, 7]] });
  f.place([8, 6]);
  f.move();
  assert.deepEqual(f.request().center, [5, 4], "published motion cannot reset the original grab offset");
});

test("dragging beyond the map clamps the final target before Rust applies camera bounds", () => {
  const f = fixture();
  f.place([20, 20]);
  f.down();
  f.placeCss([10000, -10000]);
  f.move();
  assert.deepEqual(f.request().center, [32, -32]);
  f.up();
  f.camera({ center: [32, 32], footprint: [[20, 22], [44, 22], [44, 48], [20, 48]] });
  assert.equal(f.frame.style.width, "34.5px");
  assert.equal(f.frame.style.height, "28.75px");
  assert.equal(f.frame.style.left, "157.5px");
  assert.equal(f.frame.style.top, "163.25px");
});

test("an off-center viewport grab can still reach both opposite map boundaries", () => {
  const f = fixture();
  f.place([-3, -4]);
  f.down();
  assert.equal(f.request(), null);
  f.placeCss([-10000, -10000]);
  f.move();
  assert.deepEqual(f.request().center, [-32, -32], "apply the original grab offset before clipping");
  f.placeCss([10000, 10000]);
  f.move();
  assert.deepEqual(f.request().center, [32, 32]);
});

test("untrusted, non-left, obscured and inactive input cannot navigate the minimap", () => {
  for (const boundary of ["untrusted", "right", "touch", "obscured", "pause", "focus", "lock", "confirmation", "selection"]) {
    const f = fixture();
    f.place([20, 20]);
    const properties = {};
    if (boundary === "untrusted") properties.isTrusted = false;
    if (boundary === "right") { properties.button = 2; properties.buttons = 2; }
    if (boundary === "touch") properties.pointerType = "touch";
    if (boundary === "obscured") f.obscure();
    if (boundary === "pause") f.root.dataset.paused = "true";
    if (boundary === "focus") f.document.focused = false;
    if (boundary === "lock") f.document.pointerLockElement = null;
    if (boundary === "confirmation") f.root.dataset.pointerLocked = "false";
    if (boundary === "selection") { f.state.session.phase = "Selecting"; f.minimap.render(f.state); }
    f.down(properties);
    assert.equal(f.request(), null, boundary);
    assert.equal(f.root.dataset.minimapDragging, undefined, boundary);
    assert.equal(f.uiCaptured(), false, boundary);
  }
});

test("pause, focus, capture, cancellation, resize and player changes discard gestures and pending input", () => {
  for (const boundary of ["pause", "focus", "lock", "cancel", "buttons", "resize", "player", "selection"]) {
    const f = fixture();
    f.place([20, 20]);
    f.down();
    assert(f.request());
    if (boundary === "pause") { f.root.dataset.paused = "true"; f.flushMutations(); }
    if (boundary === "focus") { f.document.focused = false; f.window.emit("blur"); }
    if (boundary === "lock") { f.document.pointerLockElement = null; f.document.emit("pointerlockchange"); }
    if (boundary === "cancel") f.document.emit("pointercancel");
    if (boundary === "buttons") f.move({ buttons: 0 });
    if (boundary === "resize") { f.bounds.width = 180; f.window.emit("resize"); }
    if (boundary === "player") { f.state.player_id = "replacement"; f.minimap.render(f.state); }
    if (boundary === "selection") { f.state.player_id = null; f.state.session.phase = "Selecting"; f.minimap.render(f.state); }
    assert.equal(f.request(), null, boundary);
    assert.equal(f.root.dataset.minimapDragging, undefined, boundary);
    assert.equal(f.uiCaptured(), false, boundary);
    f.root.dataset.paused = "false";
    f.document.focused = true;
    f.document.pointerLockElement = {};
    f.place([0, 0]);
    f.move();
    assert.equal(f.request(), null, `${boundary}: later motion cannot replay the old gesture`);
  }
});
