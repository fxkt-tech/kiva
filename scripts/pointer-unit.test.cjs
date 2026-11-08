// Exercise the real browser pointer owner without a renderer or a second model.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");

const source = fs.readFileSync(path.join(__dirname, "../web/pointer.js"), "utf8");

function fixture(dpr = 1) {
  class Target {
    listeners = new Map();
    addEventListener(type, listener) {
      if (!this.listeners.has(type)) this.listeners.set(type, []);
      this.listeners.get(type).push(listener);
    }
    emit(type, properties = {}) {
      const event = { type, ...properties };
      for (const listener of this.listeners.get(type) || []) listener(event);
    }
  }

  const mutations = [];
  const root = { dataset: { paused: "false", pointerLocked: "true" } };
  const canvas = { clientWidth: 1000, clientHeight: 720 };
  let transformWrites = 0;
  let transform = "";
  const cursor = { hidden: true, dataset: {}, style: {
    set transform(value) { transform = value; transformWrites++; },
    get transform() { return transform; },
    setProperty() {},
  } };
  const document = Object.assign(new Target(), {
    documentElement: root,
    pointerLockElement: null,
    focused: true,
    hasFocus() { return this.focused; },
    getElementById: (id) => id === "game" ? canvas : cursor,
    elementFromPoint: () => null,
  });
  const window = new Target();
  const context = { window, document, devicePixelRatio: dpr,
    MutationObserver: class {
      constructor(callback) { mutations.push(callback); }
      observe() {}
    },
  };
  vm.runInNewContext(source, context, { filename: "web/pointer.js" });
  const api = window.kivaPointer;
  const pointerEvent = (properties = {}) => ({
    isTrusted: true, pointerType: "mouse", button: -1, buttons: 0,
    clientX: 0, clientY: 0, movementX: 0, movementY: 0, ...properties,
  });
  const move = (properties) => document.emit("pointermove", pointerEvent(properties));
  return {
    api, root, document, window, canvas, cursor, move,
    position: () => Array.from(api.position()),
    take: (button) => Array.from(api.takePress(button)),
    down(button = 0) { document.emit("pointerdown", pointerEvent({ button, buttons: 1 })); },
    capture() {
      document.pointerLockElement = canvas;
      document.emit("pointerlockchange");
      api.sync(true);
    },
    flushMutations() { mutations.forEach((callback) => callback()); },
    changeDpr(value) { context.devicePixelRatio = value; window.emit("resize"); },
    writes: () => transformWrites,
  };
}

test("coalesced motion discards overshoot before reversing and draws once", () => {
  const f = fixture();
  f.capture();
  f.move({ movementX: 498 });
  assert.deepEqual(f.position(), [998, 360]);
  const before = f.writes();
  f.move({ movementX: 5, getCoalescedEvents: () => [
    { movementX: 10, movementY: 0 },
    { movementX: -5, movementY: 0 },
  ] });
  assert.deepEqual(f.position(), [994, 360], "return motion must not repay discarded overshoot");
  assert.equal(f.writes() - before, 1, "one delivered event updates the DOM only once");
});

test("a mismatched press discards the remaining batch rather than replaying it", () => {
  const f = fixture();
  f.capture();
  f.down(2);
  f.move({ movementX: 10 });
  f.down(0);
  assert.deepEqual(f.take(0), []);
  f.move({ movementX: 10 });
  f.down(0);
  assert.deepEqual(f.take(0), [520, 360, 0], "a future press must not consume the stale left snapshot");
  assert.deepEqual(f.take(0), []);
});

for (const [label, dpr, units] of [["CSS", 2, 1], ["physical", 1.5, 1.5]]) {
  test(`${label} movement calibrates against the actual unlocked CSS displacement`, () => {
    const f = fixture(dpr);
    f.move({ clientX: 200, clientY: 300, movementX: 900, movementY: 100 });
    f.move({ clientX: 248, clientY: 300, movementX: 48 * units });
    f.capture();
    f.move({ movementX: 24 * units, movementY: -12 * units });
    assert.deepEqual(f.position(), [524, 348]);
  });
}

test("calibration ignores a stationary axis and does not count coalesced parents twice", () => {
  const f = fixture();
  f.move({ clientX: 200, clientY: 300 });
  f.move({ clientX: 200, clientY: 330, movementX: 45, movementY: 45,
    getCoalescedEvents: () => [
      { movementX: 15, movementY: 15 },
      { movementX: 30, movementY: 30 },
    ],
  });
  f.capture();
  f.move({ movementX: 36, movementY: -18 });
  assert.deepEqual(f.position(), [524, 348]);
});

for (const edge of ["current", "previous"]) {
  test(`a partially clipped ${edge} coordinate cannot bias calibration on the other axis`, () => {
    const f = fixture();
    f.move({ clientX: edge === "current" ? 984 : 999, clientY: 300 });
    f.move({ clientX: edge === "current" ? 999 : 984, clientY: 330,
      movementX: edge === "current" ? 45 : -45, movementY: 45 });
    f.capture();
    f.move({ movementX: 36, movementY: -18 });
    assert.deepEqual(f.position(), [524, 348], "the unbounded y sample must retain the actual CSS gain");
  });
}

test("a clipped coalesced sample cannot bias calibration after returning inside", () => {
  const f = fixture();
  f.move({ clientX: 950, clientY: 300 });
  f.move({ clientX: 989, clientY: 330, movementX: 135, movementY: 45,
    getCoalescedEvents: () => [
      { clientX: 999, clientY: 310, movementX: 150, movementY: 15 },
      { clientX: 989, clientY: 330, movementX: -15, movementY: 30 },
    ],
  });
  f.capture();
  f.move({ movementX: 36, movementY: -18 });
  assert.deepEqual(f.position(), [524, 348], "the intermediate clipped x sample must not change the valid y gain");
});

test("missing or reset calibration uses CSS units without a DPR assumption", () => {
  const f = fixture(1.5);
  f.move({ clientX: 200, clientY: 300 });
  f.move({ clientX: 248, clientY: 300, movementX: 72 });
  f.changeDpr(2);
  f.capture();
  f.move({ movementX: 20, movementY: -10 });
  assert.deepEqual(f.position(), [520, 350]);
});

test("presses retain event-time coordinates across subsequent motion and chorded buttons", () => {
  const f = fixture();
  f.capture();
  f.down(0);
  f.move({ movementX: 30, button: 2, buttons: 3 });
  f.move({ movementX: 10, button: 2, buttons: 1 });
  f.move({ movementX: 10, button: -1, buttons: 1 });
  assert.deepEqual(f.take(0), [500, 360, 0]);
  assert.deepEqual(f.take(2), [530, 360, 0]);
  assert.deepEqual(f.take(0), [], "button release and ordinary motion are not new presses");
  assert.deepEqual(f.position(), [550, 360]);
});

test("pause, focus loss and capture loss clear presses and require a centered recapture", () => {
  const f = fixture();
  for (const boundary of ["pause", "focus", "capture"]) {
    f.capture();
    f.move({ movementX: 40 });
    f.down();
    if (boundary === "pause") {
      f.root.dataset.paused = "true";
      f.flushMutations();
    } else if (boundary === "focus") {
      f.document.focused = false;
      f.window.emit("blur");
    } else {
      f.document.pointerLockElement = null;
      f.document.emit("pointerlockchange");
    }
    assert.deepEqual(f.position(), []);
    assert.equal(f.cursor.hidden, true);
    f.root.dataset.paused = "false";
    f.document.focused = true;
    f.capture();
    assert.deepEqual(f.position(), [500, 360]);
    assert.deepEqual(f.take(0), []);
  }
});

test("unconfirmed capture and explicit press clearing cannot replay input", () => {
  const f = fixture();
  f.root.dataset.pointerLocked = "false";
  f.capture();
  f.down();
  assert.deepEqual(f.position(), []);
  assert.equal(f.cursor.hidden, true);
  f.root.dataset.pointerLocked = "true";
  f.api.sync(true);
  f.down();
  f.api.clearPresses();
  assert.deepEqual(f.take(0), []);
  assert.deepEqual(f.position(), [500, 360]);
});

test("UI gesture ownership survives chorded presses and release before the input frame", () => {
  const f = fixture();
  f.capture();
  f.down();
  f.api.setUiCapture(true);
  f.move({ movementX: 30, button: 2, buttons: 3 });
  f.api.setUiCapture(false);
  f.move({ movementX: 10 });
  f.down(2);
  assert.deepEqual(f.take(0), [500, 360, 0], "the first press retains its normal UI hit position");
  assert.deepEqual(f.take(2), [530, 360, 1], "a queued gesture press keeps ownership after release");
  assert.deepEqual(f.take(2), [540, 360, 0], "a later press must return to normal world input");
});

test("pause, loss of capture and focus reset UI gesture capture before recapture", () => {
  const f = fixture();
  for (const boundary of ["pause", "capture", "focus"]) {
    f.root.dataset.paused = "false";
    f.document.focused = true;
    f.capture();
    f.api.setUiCapture(true);
    if (boundary === "pause") {
      f.root.dataset.paused = "true";
      f.flushMutations();
    } else if (boundary === "capture") {
      f.document.pointerLockElement = null;
      f.document.emit("pointerlockchange");
    } else {
      f.document.focused = false;
      f.window.emit("blur");
    }
    f.root.dataset.paused = "false";
    f.document.focused = true;
    f.capture();
    f.down();
    assert.deepEqual(f.take(0), [500, 360, 0], `${boundary} cannot leave a captured UI gesture behind`);
  }
});

test("enemy model feedback changes only the cursor shape and respects UI gesture capture", () => {
  const f = fixture();
  f.capture();
  const before = f.position();
  f.window.emit("kiva-camera", { detail: JSON.stringify({ enemy_hovered: true }) });
  assert.equal(f.cursor.dataset.target, "enemy");
  assert.deepEqual(f.position(), before, "target feedback must never reposition the pointer");
  f.api.setUiCapture(true);
  assert.equal(f.cursor.dataset.target, "none");
  f.api.setUiCapture(false);
  assert.equal(f.cursor.dataset.target, "enemy");
  f.document.elementFromPoint = () => ({ closest: (selector) => selector === "[data-ui]" ? {} : null });
  f.move({ movementX: 2 });
  assert.equal(f.cursor.dataset.target, "none", "panels must immediately keep the normal cursor");
  f.document.elementFromPoint = () => null;
  f.window.emit("kiva-camera", { detail: JSON.stringify({ enemy_hovered: false }) });
  assert.equal(f.cursor.dataset.target, "none");
});

test("pause, unlock and blur discard an old sword cursor before a centered recapture", () => {
  const f = fixture();
  for (const boundary of ["pause", "capture", "focus"]) {
    f.root.dataset.paused = "false";
    f.document.focused = true;
    f.capture();
    f.window.emit("kiva-camera", { detail: JSON.stringify({ enemy_hovered: true }) });
    assert.equal(f.cursor.dataset.target, "enemy");
    if (boundary === "pause") {
      f.root.dataset.paused = "true";
      f.flushMutations();
    } else if (boundary === "capture") {
      f.document.pointerLockElement = null;
      f.document.emit("pointerlockchange");
    } else {
      f.document.focused = false;
      f.window.emit("blur");
    }
    assert.equal(f.cursor.dataset.target, "none");
    assert.equal(f.cursor.hidden, true);
    f.root.dataset.paused = "false";
    f.document.focused = true;
    f.capture();
    assert.equal(f.cursor.dataset.target, "none", `${boundary} cannot replay a stale target cursor`);
    assert.deepEqual(f.position(), [500, 360]);
  }
});
