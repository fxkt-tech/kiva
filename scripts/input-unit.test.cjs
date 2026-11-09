const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = (name) => fs.readFileSync(path.join(__dirname, "../web", name), "utf8");

class Target {
  listeners = new Map();
  hidden = false;
  textContent = "";
  addEventListener(type, callback, options) {
    const entries = this.listeners.get(type) || [];
    entries.push({ callback, once: options?.once });
    this.listeners.set(type, entries);
  }
  dispatchEvent(event) {
    const entries = [...(this.listeners.get(event.type) || [])];
    for (const entry of entries) {
      if (entry.once) this.listeners.set(event.type, this.listeners.get(event.type).filter((item) => item !== entry));
      entry.callback(event);
    }
  }
}
const emit = (target, type, fields = {}) => {
  const event = { type, defaultPrevented: false, preventDefault() { this.defaultPrevented = true; }, ...fields };
  target.dispatchEvent(event);
  return event;
};

function fixture(gpu = {}) {
  const window = Object.assign(new Target(), { devicePixelRatio: 2, innerWidth: 1200, location: { reload() {} } });
  const document = Object.assign(new Target(), { readyState: "loading", pointerLockElement: null });
  const nodes = new Map();
  let focused = true;
  let rejectRequest;
  let resolveRequest;
  let requests = 0;
  document.hasFocus = () => focused;
  document.getElementById = (id) => nodes.get(id) || null;
  document.exitPointerLock = () => { document.pointerLockElement = null; };
  const context = vm.createContext({ window, document, navigator: { gpu }, CustomEvent: class { constructor(type) { this.type = type; } } });
  vm.runInContext(source("startup.js"), context);
  vm.runInContext(source("input.js"), context);
  const input = window.kivaInput;
  const parse = () => {
    for (const id of ["game", "entry", "enter", "retry", "entry-message"]) nodes.set(id, new Target());
    nodes.get("game").focus = () => { focused = true; };
    nodes.get("game").requestPointerLock = () => {
      requests++;
      return new Promise((resolve, reject) => { resolveRequest = resolve; rejectRequest = reject; });
    };
    document.readyState = "interactive";
    emit(document, "DOMContentLoaded");
  };
  const click = () => emit(nodes.get("enter"), "click");
  const confirm = () => {
    document.pointerLockElement = nodes.get("game");
    emit(document, "pointerlockchange");
  };
  const capture = () => { input.ready(); click(); confirm(); };
  const move = (x, clientX = 500, extras = {}) => emit(document, "pointermove", {
    isTrusted: true, pointerType: "mouse", movementX: x, movementY: 80, clientX, ...extras,
  });
  return {
    window, document, nodes, input, parse, click, confirm, capture, move,
    reject: () => rejectRequest(new Error("denied")), resolve: () => resolveRequest(),
    requests: () => requests,
    blur: () => { focused = false; emit(window, "blur"); },
    focus: () => { focused = true; emit(window, "focus"); },
  };
}

test("head scripts retain readiness and failures before DOM exists", () => {
  const f = fixture();
  f.input.ready();
  emit(f.window, "error");
  f.parse();
  assert.equal(f.window.__kivaStartup, "ready");
  assert.equal(f.nodes.get("enter").hidden, false);
  assert.equal(f.nodes.get("retry").hidden, true);
  for (const failure of ["error", "unhandledrejection", "TrunkApplicationError"]) {
    const broken = fixture();
    emit(broken.window, failure);
    broken.parse();
    assert.equal(broken.nodes.get("retry").hidden, false);
  }
  const unsupported = fixture(null);
  unsupported.parse();
  assert.equal(unsupported.nodes.get("retry").hidden, false);
});

test("input waits for actual pointerlockchange, not just promise resolution", async () => {
  const f = fixture(); f.parse(); f.input.ready(); f.click();
  f.resolve(); await Promise.resolve();
  f.document.pointerLockElement = f.nodes.get("game");
  f.move(90);
  assert.equal(f.input.isActive(), false);
  assert.equal(f.input.takeLook(), 0);
  f.confirm();
  assert.equal(f.input.isActive(), true);
  assert.equal(f.nodes.get("entry").hidden, true);
  assert.equal(f.input.takeLook(), 0);
  f.move(40);
  assert.equal(f.input.takeLook(), 40);
  assert.equal(f.input.takeLook(), 0);
});

test("lock loss and blur discard old motion and require a fresh click", () => {
  const f = fixture(); f.parse(); f.capture(); f.move(100);
  f.blur();
  assert.equal(f.input.isActive(), false);
  assert.equal(f.input.takeLook(), 0);
  f.focus();
  f.confirm(); // A stale lock notification after blur cannot resume control.
  assert.equal(f.input.isActive(), false);
  f.click(); f.confirm(); f.move(20);
  assert.equal(f.input.takeLook(), 20);
  f.document.pointerLockElement = null;
  emit(f.document, "pointerlockchange");
  f.move(80);
  assert.equal(f.input.takeLook(), 0);
  assert.equal(f.nodes.get("entry").hidden, false);
});

test("rapid blur and recapture clear held keys; repeated keydown cannot resurrect them", () => {
  const f = fixture(); f.parse(); f.capture();
  const key = (type, code, repeat = false) => emit(f.document, type, { code, repeat, isTrusted: true });
  key("keydown", "KeyW"); key("keydown", "KeyD");
  assert.equal(f.input.keys(), 9);
  f.blur(); f.focus(); f.click(); f.confirm();
  assert.equal(f.input.keys(), 0);
  key("keydown", "KeyW", true);
  assert.equal(f.input.keys(), 0);
  key("keyup", "KeyW"); key("keydown", "KeyW");
  assert.equal(f.input.keys(), 1);
  key("keyup", "KeyW");
  assert.equal(f.input.keys(), 0);
  emit(f.document, "keydown", { code: "KeyA", isTrusted: false });
  assert.equal(f.input.keys(), 0);
});

test("capture failures allow retry; delayed rejection cannot cancel confirmed capture", async () => {
  const f = fixture(); f.parse(); f.input.ready(); f.click();
  f.reject(); await Promise.resolve();
  assert.match(f.nodes.get("entry-message").textContent, /未能锁定/);
  f.click(); f.confirm(); f.reject(); await Promise.resolve();
  assert.equal(f.requests(), 2);
  assert.equal(f.input.isActive(), true);
});

test("horizontal coalesced samples are consumed once and vertical motion is ignored", () => {
  const f = fixture(); f.parse(); f.capture();
  f.move(99, 500, { getCoalescedEvents: () => [{ movementX: 3 }, { movementX: 7 }] });
  assert.equal(f.input.takeLook(), 10);
  f.move(0, 500, { movementY: 500 });
  assert.equal(f.input.takeLook(), 0);
  f.move(8, 500, { isTrusted: false });
  f.move(8, 500, { pointerType: "touch" });
  assert.equal(f.input.takeLook(), 0);
});

test("150% system pixel units calibrate independently of DPR and reject clipped edges", () => {
  const f = fixture(); f.parse();
  f.move(0, 100); f.move(150, 200); // 150 raw pixels = 100 CSS pixels, DPR is 2.
  f.move(1500, 1199); f.move(-1500, 900); // Edge transitions must not bias the ratio.
  f.capture(); f.move(150);
  assert.equal(f.input.takeLook(), 100);
  f.window.devicePixelRatio = 1;
  f.move(90);
  assert.equal(f.input.takeLook(), 90);
});

test("startup and input bridges load before the generated WASM module", () => {
  const html = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");
  assert(html.indexOf('<script src="startup.js"></script>') < html.indexOf('<script src="input.js"></script>'));
  assert(html.indexOf('<script src="input.js"></script>') < html.indexOf('data-trunk rel="rust"'));
});

test("action presses are consumed once and repeat cannot replay an action", () => {
  const f = fixture(); f.parse(); f.capture();
  const press = (code, repeat = false) => emit(f.document, "keydown", { code, repeat, isTrusted: true });
  for (const code of ["Space", "ShiftLeft", "KeyE", "Tab", "Digit1", "KeyQ"]) press(code);
  emit(f.document, "pointerdown", { isTrusted: true, pointerType: "mouse", button: 0 });
  assert.equal(f.input.takeActions(), 1 | 2 | 4 | 8 | 16 | 64 | 256);
  assert.equal(f.input.takeActions(), 0);
  press("Space", true); press("KeyE");
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keyup", { code: "Space", isTrusted: true });
  press("Space");
  assert.equal(f.input.takeActions(), 1);
  emit(f.document, "keydown", { code: "Digit2", isTrusted: true });
  assert.equal(f.input.takeActions(), 128);
});

test("held sprint and block require fresh trusted presses and release independently", () => {
  const f = fixture(); f.parse(); f.capture();
  const press = (code) => emit(f.document, "keydown", { code, isTrusted: true });
  const release = (code) => emit(f.document, "keyup", { code, isTrusted: true });
  press("ShiftLeft"); press("ShiftRight"); press("KeyW");
  emit(f.document, "pointerdown", { isTrusted: true, pointerType: "mouse", button: 2 });
  assert.equal(f.input.keys(), 1 | 16 | 32);
  release("ShiftLeft");
  assert.equal(f.input.keys(), 1 | 16 | 32);
  release("ShiftRight");
  emit(f.document, "pointerup", { isTrusted: true, pointerType: "mouse", button: 2 });
  assert.equal(f.input.keys(), 1);
  emit(f.document, "pointerdown", { isTrusted: false, pointerType: "mouse", button: 2 });
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: false });
  assert.equal(f.input.keys(), 1);
});

test("loss of capture discards queued actions, block and sprint without replay after recapture", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "Tab", isTrusted: true });
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  emit(f.document, "pointerdown", { isTrusted: true, pointerType: "mouse", button: 2 });
  f.blur(); f.focus(); f.click(); f.confirm();
  assert.equal(f.input.keys(), 0);
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keydown", { code: "ShiftLeft", repeat: true, isTrusted: true });
  emit(f.document, "keydown", { code: "Tab", repeat: true, isTrusted: true });
  assert.equal(f.input.keys(), 0);
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keydown", { code: "KeyE", isTrusted: true });
  f.document.pointerLockElement = null;
  emit(f.document, "pointerlockchange");
  assert.equal(f.input.takeActions(), 0);
  f.click(); f.confirm();
  assert.equal(f.input.takeActions(), 0);
});

test("wheel inertia is one switch per gesture and starts fresh after capture loss", () => {
  const f = fixture(); f.parse(); f.capture();
  const wheel = (timeStamp, deltaY = 10) => emit(f.document, "wheel", { isTrusted: true, timeStamp, deltaY });
  wheel(0);
  assert.equal(f.input.takeActions(), 32);
  for (const at of [20, 60, 120, 200]) wheel(at);
  assert.equal(f.input.takeActions(), 0);
  wheel(400, -100);
  assert.equal(f.input.takeActions(), 32);
  wheel(700, Number.NaN); wheel(710, 0);
  assert.equal(f.input.takeActions(), 0);
  f.blur(); f.focus(); f.click(); f.confirm();
  wheel(420);
  assert.equal(f.input.takeActions(), 32);
});

test("game keys and browser gestures prevent defaults only during confirmed capture", () => {
  const f = fixture(); f.parse();
  const key = (code) => emit(f.document, "keydown", { code, isTrusted: true });
  assert.equal(key("Tab").defaultPrevented, false);
  assert.equal(emit(f.document, "contextmenu", { isTrusted: true }).defaultPrevented, false);
  assert.equal(emit(f.document, "wheel", { isTrusted: true, deltaY: 10 }).defaultPrevented, false);
  assert.equal(f.input.takeActions(), 0);
  f.capture();
  for (const code of ["Tab", "Space", "ShiftLeft", "KeyW", "KeyC"]) assert.equal(key(code).defaultPrevented, true);
  assert.equal(key("ControlLeft").defaultPrevented, false);
  assert.equal(key("F2").defaultPrevented, false);
  assert.equal(emit(f.document, "contextmenu", { isTrusted: true }).defaultPrevented, true);
  assert.equal(emit(f.document, "wheel", { isTrusted: true, deltaY: 10 }).defaultPrevented, true);
  f.blur();
  assert.equal(key("Tab").defaultPrevented, false);
});

test("untrusted or nonmouse events cannot queue attacks or wheel switches", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "Space", isTrusted: false });
  emit(f.document, "pointerdown", { isTrusted: true, pointerType: "touch", button: 0 });
  emit(f.document, "pointerdown", { isTrusted: false, pointerType: "mouse", button: 0 });
  emit(f.document, "wheel", { isTrusted: false, deltaY: 100 });
  assert.equal(f.input.takeActions(), 0);
  assert.equal(f.input.keys(), 0);
});

test("vertical aiming consumes coalesced samples once independently from horizontal look", () => {
  const f = fixture(); f.parse(); f.capture();
  f.move(99, 500, { movementY: 100, getCoalescedEvents: () => [
    { movementX: 3, movementY: -5 },
    { movementX: 7, movementY: 15 },
    { movementX: Number.NaN, movementY: Number.POSITIVE_INFINITY },
  ] });
  assert.equal(f.input.takeAim(), 10);
  assert.equal(f.input.takeAim(), 0);
  assert.equal(f.input.takeLook(), 10);
  assert.equal(f.input.takeLook(), 0);
  f.move(5, 500, { movementY: -7 });
  assert.equal(f.input.takeLook(), 5);
  assert.equal(f.input.takeAim(), -7);
});

test("vertical aiming uses CSS calibration and discards inactive or untrusted samples", () => {
  const f = fixture(); f.parse();
  f.move(0, 100); f.move(150, 200);
  assert.equal(f.input.takeAim(), 0);
  f.capture();
  f.move(30, 500, { movementY: 60 });
  assert.equal(f.input.takeAim(), 40);
  assert.equal(f.input.takeLook(), 20);
  f.move(0, 500, { movementY: 100, isTrusted: false });
  f.move(0, 500, { movementY: 100, pointerType: "touch" });
  assert.equal(f.input.takeAim(), 0);
});

test("capture loss, blur and confirmation clear pending vertical aim", () => {
  const f = fixture(); f.parse(); f.capture();
  f.move(0, 500, { movementY: 80 });
  f.blur();
  assert.equal(f.input.takeAim(), 0);
  f.focus(); f.click(); f.confirm();
  assert.equal(f.input.takeAim(), 0);
  f.move(0, 500, { movementY: 50 });
  f.document.pointerLockElement = null;
  emit(f.document, "pointerlockchange");
  assert.equal(f.input.takeAim(), 0);
  f.click(); f.confirm();
  assert.equal(f.input.takeAim(), 0);
  f.move(0, 500, { movementY: 20 });
  f.confirm();
  assert.equal(f.input.takeAim(), 0);
});

test("short Shift tap retains one queued dodge after release", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.keys(), 16);
  emit(f.document, "keyup", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.keys(), 0);
  assert.equal(f.input.takeActions(), 2);
  assert.equal(f.input.takeActions(), 0);
});

test("left and right Shift form one physical hold without duplicate dodge", () => {
  const f = fixture(); f.parse(); f.capture();
  const press = (code, repeat = false) => emit(f.document, "keydown", { code, repeat, isTrusted: true });
  const release = (code) => emit(f.document, "keyup", { code, isTrusted: true });
  press("ShiftLeft");
  assert.equal(f.input.takeActions(), 2);
  press("ShiftLeft", true); press("ShiftRight");
  assert.equal(f.input.takeActions(), 0);
  release("ShiftLeft");
  assert.equal(f.input.keys(), 16);
  assert.equal(f.input.takeActions(), 0);
  release("ShiftRight");
  assert.equal(f.input.keys(), 0);
  press("ShiftRight");
  assert.equal(f.input.takeActions(), 2);
});

test("inactive, untrusted and repeated Shift cannot dodge; Ctrl is unbound", () => {
  const f = fixture(); f.parse();
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
  f.capture();
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: false });
  emit(f.document, "keydown", { code: "ShiftLeft", repeat: true, isTrusted: true });
  emit(f.document, "keydown", { code: "ControlLeft", isTrusted: true });
  emit(f.document, "keydown", { code: "ControlRight", isTrusted: true });
  assert.equal(f.input.keys(), 0);
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  f.blur(); f.focus(); f.click(); f.confirm();
  assert.equal(f.input.keys(), 0);
  assert.equal(f.input.takeActions(), 0);
});

test("C tap queues one crouch toggle and keyup does not clear it", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
  assert.equal(f.input.keys(), 0);
  emit(f.document, "keyup", { code: "KeyC", isTrusted: true });
  assert.equal(f.input.takeActions(), 512);
  assert.equal(f.input.takeActions(), 0);
  assert.equal(f.input.keys(), 0);
});

test("two C taps before sampling cancel toggle parity without losing other actions", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "Space", isTrusted: true });
  for (let index = 0; index < 2; index++) {
    emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
    emit(f.document, "keyup", { code: "KeyC", isTrusted: true });
  }
  assert.equal(f.input.takeActions(), 1);
  for (let index = 0; index < 3; index++) {
    emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
    emit(f.document, "keyup", { code: "KeyC", isTrusted: true });
  }
  assert.equal(f.input.takeActions(), 512);
});

test("holding C or keyboard repeat cannot keep toggling stance", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
  assert.equal(f.input.takeActions(), 512);
  emit(f.document, "keydown", { code: "KeyC", repeat: true, isTrusted: true });
  emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keyup", { code: "KeyC", isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
  assert.equal(f.input.takeActions(), 512);
});

test("queued crouch toggles clear on blur, lock loss and capture confirmation", () => {
  const f = fixture(); f.parse(); f.capture();
  const press = () => emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
  press(); f.blur(); f.focus(); f.click(); f.confirm();
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keydown", { code: "KeyC", repeat: true, isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
  press(); f.document.pointerLockElement = null;
  emit(f.document, "pointerlockchange");
  assert.equal(f.input.takeActions(), 0);
  f.click(); f.confirm();
  press(); f.confirm();
  assert.equal(f.input.takeActions(), 0);
  press();
  assert.equal(f.input.takeActions(), 512);
});

test("inactive and untrusted C presses do not toggle or prevent browser input", () => {
  const f = fixture(); f.parse();
  const inactive = emit(f.document, "keydown", { code: "KeyC", isTrusted: true });
  assert.equal(inactive.defaultPrevented, false);
  assert.equal(f.input.takeActions(), 0);
  f.capture();
  const untrusted = emit(f.document, "keydown", { code: "KeyC", isTrusted: false });
  assert.equal(untrusted.defaultPrevented, false);
  assert.equal(f.input.takeActions(), 0);
});
