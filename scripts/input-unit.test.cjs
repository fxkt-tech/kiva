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
const emit = (target, type, fields = {}) => target.dispatchEvent({ type, ...fields });

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
