const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = (name) => fs.readFileSync(path.join(__dirname, "../web", name), "utf8");

class Target {
  listeners = new Map();
  hidden = false;
  disabled = false;
  textContent = "";
  attributes = new Map();
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  getAttribute(name) { return this.attributes.get(name) ?? null; }
  removeAttribute(name) { this.attributes.delete(name); }
  set value(value) { this.setAttribute("value", value); }
  get value() { return Number(this.getAttribute("value")); }
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
    for (const id of ["game", "entry", "enter", "enter-label", "enter-progress", "retry", "entry-message"]) nodes.set(id, new Target());
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
  const capture = () => { input.ready(); click(); confirm(); input.takeDefenseEdges(); };
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

test("first entry is visible and disabled until real engine readiness", () => {
  const f = fixture(); f.parse();
  assert.equal(f.nodes.get("enter").hidden, false);
  assert.equal(f.nodes.get("enter").disabled, true);
  assert.match(f.nodes.get("enter-label").textContent, /正在下载/);
  f.click(); emit(f.nodes.get("game"), "click"); f.confirm();
  assert.equal(f.requests(), 0);
  assert.equal(f.input.isActive(), false);
  f.input.ready();
  assert.equal(f.nodes.get("enter").disabled, false);
  assert.equal(f.nodes.get("enter-label").textContent, "点击进入");
  assert.equal(f.requests(), 0);
});

test("capture pending has immediate feedback and becomes retryable after failure or blur", async () => {
  const f = fixture(); f.parse(); f.input.ready(); f.click();
  assert.equal(f.nodes.get("enter").disabled, true);
  assert.match(f.nodes.get("enter-label").textContent, /正在进入/);
  assert.equal(f.nodes.get("enter").getAttribute("aria-busy"), "true");
  f.click(); emit(f.nodes.get("game"), "click");
  assert.equal(f.requests(), 1);
  f.reject(); await Promise.resolve();
  assert.equal(f.nodes.get("enter").disabled, false);
  assert.equal(f.nodes.get("enter-label").textContent, "点击进入");
  f.click(); f.blur(); f.focus();
  assert.equal(f.nodes.get("enter").disabled, false);
  f.click(); f.confirm();
  assert.equal(f.input.isActive(), true);
});

test("real download progress replays before DOM and never enables entry at 100 percent", () => {
  const f = fixture();
  f.window.kivaStartup.progress("download", 512, 1024);
  f.parse();
  assert.equal(f.nodes.get("enter-label").textContent, "正在下载 50%");
  assert.equal(f.nodes.get("enter-progress").value, 512);
  assert.equal(f.nodes.get("enter-progress").max, 1024);
  f.window.kivaStartup.progress("download", 1024, 1024);
  assert.equal(f.nodes.get("enter-label").textContent, "正在下载 100%");
  assert.equal(f.nodes.get("enter").disabled, true);
  f.click(); emit(f.nodes.get("game"), "click");
  assert.equal(f.requests(), 0);
  f.window.kivaStartup.progress("initialize", 0, 0);
  assert.match(f.nodes.get("enter-label").textContent, /正在准备游戏/);
  assert.equal(f.nodes.get("enter-progress").getAttribute("value"), null);
  f.window.kivaStartup.progress("render", 0, 0);
  assert.match(f.nodes.get("enter-label").textContent, /正在准备画面/);
  assert.equal(f.nodes.get("enter").disabled, true);
  f.input.ready();
  assert.equal(f.nodes.get("enter").disabled, false);
  assert.equal(f.nodes.get("enter-progress").hidden, true);
  assert.equal(f.nodes.get("enter").getAttribute("aria-busy"), "false");
});

test("unknown and invalid totals show a stage without a fabricated percentage", () => {
  const f = fixture(); f.parse();
  for (const [loaded, total] of [[2560, 0], [2560, 2048], [2560, Infinity], [NaN, -1]]) {
    f.window.kivaStartup.progress("download", loaded, total);
    assert.equal(f.nodes.get("enter-label").textContent, "正在下载…");
    assert.equal(f.nodes.get("enter-progress").getAttribute("value"), null);
    assert.equal(f.nodes.get("enter").disabled, true);
  }
  f.window.kivaStartup.progress("download", 2560, 0);
  assert.match(f.nodes.get("entry-message").textContent, /已接收 2 KB/);
});

test("startup phases never regress, failed progress cannot reset failure, and real ready wins", () => {
  const f = fixture();
  const snapshots = [];
  const unsubscribe = f.window.kivaStartup.subscribe((state, detail) => snapshots.push({ state, ...detail }));
  f.window.kivaStartup.progress("render", 0, 0);
  f.window.kivaStartup.progress("download", 10, 10);
  f.window.kivaStartup.progress("initialize", 0, 0);
  f.window.kivaStartup.progress("unrecognized", 0, 0);
  assert.equal(snapshots.length, 2);
  f.window.kivaStartup.fail("渲染准备失败，请重新加载后重试。");
  f.window.kivaStartup.progress("render", 1, 1);
  f.parse();
  assert.equal(f.nodes.get("enter-label").textContent, "加载失败");
  assert.equal(f.nodes.get("enter").disabled, true);
  assert.equal(f.nodes.get("enter").getAttribute("aria-busy"), "false");
  assert.equal(f.nodes.get("enter-progress").hidden, true);
  assert.equal(f.nodes.get("retry").hidden, false);
  assert.match(f.nodes.get("entry-message").textContent, /渲染准备失败/);
  unsubscribe();
  const count = snapshots.length;
  f.input.ready();
  for (const failure of ["error", "unhandledrejection", "TrunkApplicationError"]) emit(f.window, failure);
  f.window.kivaStartup.fail("迟到错误");
  f.window.kivaStartup.progress("download", 0, 0);
  assert.equal(snapshots.length, count);
  assert.equal(f.window.__kivaStartup, "ready");
  assert.equal(f.nodes.get("enter-label").textContent, "点击进入");
  assert.equal(f.nodes.get("enter").disabled, false);
  assert.equal(f.nodes.get("retry").hidden, true);
});

test("pointer-lock event errors and synchronous rejection restore the entry button", () => {
  const f = fixture(); f.parse(); f.input.ready(); f.click();
  emit(f.document, "pointerlockerror");
  assert.equal(f.nodes.get("enter").disabled, false);
  assert.match(f.nodes.get("entry-message").textContent, /未能锁定/);
  f.nodes.get("game").requestPointerLock = () => { throw new Error("denied"); };
  f.click();
  assert.equal(f.nodes.get("enter").disabled, false);
  assert.match(f.nodes.get("entry-message").textContent, /未能锁定/);
});

test("resume after Escape keeps readiness and ignores delayed first-load notifications", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "Escape", isTrusted: true });
  f.window.kivaStartup.progress("download", 0, 0);
  f.window.kivaStartup.fail("迟到错误");
  assert.equal(f.nodes.get("enter-label").textContent, "点击进入");
  assert.equal(f.nodes.get("enter").disabled, false);
  assert.equal(f.nodes.get("enter-progress").hidden, true);
  f.click(); f.confirm();
  assert.equal(f.input.isActive(), true);
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

const pointer = (f, type, button = 2, extras = {}) => emit(f.document, type, {
  isTrusted: true, pointerType: "mouse", button, ...extras,
});
const defense = (f) => Array.from(f.input.takeDefenseEdges());

test("right edges preserve complete taps and repress ordering without action bits", () => {
  const f = fixture(); f.parse(); f.capture();
  pointer(f, "pointerdown"); pointer(f, "pointerdown");
  pointer(f, "pointerup"); pointer(f, "pointerup");
  pointer(f, "pointerdown"); pointer(f, "pointerup");
  assert.deepEqual(defense(f), [1, 2, 1, 2]);
  assert.deepEqual(defense(f), []);
  assert.equal(f.input.takeActions(), 0);
  assert.equal(f.input.keys(), 0);
});

test("right edges remain queued across unsampled frames and are consumed once", () => {
  const f = fixture(); f.parse(); f.capture();
  pointer(f, "pointerdown");
  for (let frame = 0; frame < 5; frame++) {
    f.move(5);
    f.input.takeLook(); f.input.takeActions(); f.input.keys();
  }
  pointer(f, "pointerup");
  assert.deepEqual(defense(f), [1, 2]);
  for (let step = 0; step < 5; step++) assert.deepEqual(defense(f), []);
});

test("pointercancel cancels instead of releasing and needs a new physical press", () => {
  const f = fixture(); f.parse(); f.capture();
  pointer(f, "pointerdown"); assert.deepEqual(defense(f), [1]);
  pointer(f, "pointercancel"); pointer(f, "pointerup");
  assert.deepEqual(defense(f), [3]);
  pointer(f, "pointerdown"); pointer(f, "pointerup");
  assert.deepEqual(defense(f), [1, 2]);
  pointer(f, "pointerdown"); pointer(f, "pointerup"); pointer(f, "pointercancel");
  assert.deepEqual(defense(f), [3]);
});

test("blur, Escape and lock loss retain cancellation across immediate recapture", () => {
  for (const lose of [
    (f) => { f.blur(); f.focus(); },
    (f) => emit(f.document, "keydown", { code: "Escape", isTrusted: true }),
    (f) => { f.document.pointerLockElement = null; emit(f.document, "pointerlockchange"); },
  ]) {
    const f = fixture(); f.parse(); f.capture();
    pointer(f, "pointerdown"); assert.deepEqual(defense(f), [1]);
    lose(f); pointer(f, "pointerup");
    f.click(); f.confirm();
    assert.deepEqual(defense(f), [3]);
    pointer(f, "pointerup"); assert.deepEqual(defense(f), []);
    pointer(f, "pointerdown"); pointer(f, "pointerup");
    assert.deepEqual(defense(f), [1, 2]);
  }
});

test("defense cancellation precedes a new gesture without losing it", () => {
  const f = fixture(); f.parse(); f.capture();
  pointer(f, "pointerdown"); f.blur(); f.focus(); f.click(); f.confirm();
  pointer(f, "pointerdown"); pointer(f, "pointerup");
  assert.deepEqual(defense(f), [3, 1, 2]);
  assert.deepEqual(defense(f), []);
});

test("right queue is bounded; overflow cancels the current gesture until a fresh press", () => {
  const f = fixture(); f.parse(); f.capture();
  for (let tap = 0; tap < 32; tap++) { pointer(f, "pointerdown"); pointer(f, "pointerup"); }
  assert.equal(defense(f).length, 64);
  for (let tap = 0; tap < 32; tap++) { pointer(f, "pointerdown"); pointer(f, "pointerup"); }
  pointer(f, "pointerdown"); // 65th edge collapses to cancellation.
  pointer(f, "pointerdown");
  assert.deepEqual(defense(f), [3]);
  pointer(f, "pointerup"); assert.deepEqual(defense(f), []);
  pointer(f, "pointerdown"); pointer(f, "pointerup");
  assert.deepEqual(defense(f), [1, 2]);
});

test("untrusted, touch and inactive right edges cannot alter an accepted mouse gesture", () => {
  const f = fixture(); f.parse();
  pointer(f, "pointerdown"); pointer(f, "pointerup");
  assert.deepEqual(defense(f), []);
  f.capture(); pointer(f, "pointerdown");
  pointer(f, "pointerup", 2, { isTrusted: false });
  pointer(f, "pointercancel", 2, { pointerType: "touch" });
  pointer(f, "pointerdown", 2, { pointerType: "touch" });
  pointer(f, "pointercancel", 2, { isTrusted: false });
  pointer(f, "pointerup");
  assert.deepEqual(defense(f), [1, 2]);
});

test("loss of capture discards queued actions, parry and sprint without replay after recapture", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "Tab", isTrusted: true });
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  emit(f.document, "pointerdown", { isTrusted: true, pointerType: "mouse", button: 2 });
  f.blur(); f.focus(); f.click(); f.confirm();
  assert.equal(f.input.keys(), 0);
  assert.equal(f.input.takeActions(), 2048);
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

test("wheel has no weapon or attack mapping", () => {
  const f = fixture(); f.parse(); f.capture();
  for (const deltaY of [10, -10, 0, Number.NaN]) {
    emit(f.document, "wheel", { isTrusted: true, deltaY });
    assert.equal(f.input.takeActions(), 0);
    assert.deepEqual(defense(f), []);
  }
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

test("vertical pointer samples have no aim interface and never alter horizontal look", () => {
  const f = fixture(); f.parse(); f.capture();
  assert.equal(f.input.takeAim, undefined);
  f.move(99, 500, { movementY: 100, getCoalescedEvents: () => [
    { movementX: 3, movementY: -5 }, { movementX: 7, movementY: 15 },
  ] });
  assert.equal(f.input.takeLook(), 10);
  f.move(0, 500, { movementY: 1000 });
  assert.equal(f.input.takeLook(), 0);
});

test("short Shift tap retains one queued dodge after release", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.keys(), 16);
  emit(f.document, "keyup", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.keys(), 0);
  assert.equal(f.input.takeActions(), 2 | 2048);
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
  assert.equal(f.input.takeActions(), 2 | 2048);
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
  assert.equal(f.input.takeActions(), 2048);
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

test("E and Q queue distinct ordinary skills on keydown, never on hold or release", () => {
  const f = fixture(); f.parse(); f.capture();
  for (const [code, bit] of [["KeyE", 8], ["KeyQ", 256]]) {
    emit(f.document, "keydown", { code, isTrusted: true });
    assert.equal(f.input.takeActions(), bit);
    emit(f.document, "keydown", { code, isTrusted: true, repeat: true });
    emit(f.document, "keydown", { code, isTrusted: true });
    assert.equal(f.input.takeActions(), 0);
    emit(f.document, "keyup", { code, isTrusted: true });
    assert.equal(f.input.takeActions(), 0);
    emit(f.document, "keydown", { code, isTrusted: true });
    assert.equal(f.input.takeActions(), bit);
  }
});

test("left attacks and middle lock toggles require fresh button edges", () => {
  const f = fixture(); f.parse(); f.capture();
  pointer(f, "pointerdown", 0); pointer(f, "pointerdown", 0);
  assert.equal(f.input.takeActions(), 4);
  pointer(f, "pointerdown", 0); assert.equal(f.input.takeActions(), 0);
  pointer(f, "pointerup", 0); pointer(f, "pointerdown", 0);
  assert.equal(f.input.takeActions(), 4);
  pointer(f, "pointerdown", 1); pointer(f, "pointerdown", 1);
  assert.equal(f.input.takeActions(), 32);
  pointer(f, "pointerup", 1); pointer(f, "pointerdown", 1);
  pointer(f, "pointerup", 1); pointer(f, "pointerdown", 1);
  assert.equal(f.input.takeActions(), 0); // Two toggles retain parity.
});

test("capture click never attacks; confirmation discards previous device actions", () => {
  const f = fixture(); f.parse(); f.input.ready();
  pointer(f, "pointerdown", 0); f.click(); f.confirm();
  pointer(f, "pointerup", 0);
  assert.equal(f.input.takeActions(), 0);
  pointer(f, "pointerdown", 1);
  emit(f.document, "keydown", { code: "KeyE", isTrusted: true });
  f.blur(); f.focus(); f.click(); f.confirm();
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keydown", { code: "KeyE", isTrusted: true, repeat: true });
  assert.equal(f.input.takeActions(), 0);
  assert.deepEqual(defense(f), [3]);
});

test("the last requested pair wins within one browser batch", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "Digit2", isTrusted: true });
  emit(f.document, "keydown", { code: "Digit1", isTrusted: true });
  assert.equal(f.input.takeActions(), 64);
});

test("all-direction release and repress keeps the release fact with the final held snapshot", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "KeyW", isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
  emit(f.document, "keyup", { code: "KeyW", isTrusted: true });
  emit(f.document, "keydown", { code: "KeyW", isTrusted: true });
  for (let frame = 0; frame < 3; frame++) assert.equal(f.input.keys(), 1);
  assert.equal(f.input.takeActions(), 1024);
  assert.equal(f.input.takeActions(), 0);
});

test("opposing directions and overlapping Shift hands do not end a physical hold", () => {
  const f = fixture(); f.parse(); f.capture();
  for (const code of ["KeyW", "KeyS", "ShiftLeft", "ShiftRight"]) {
    emit(f.document, "keydown", { code, isTrusted: true });
  }
  assert.equal(f.input.takeActions(), 2);
  for (const code of ["KeyW", "ShiftLeft"]) emit(f.document, "keyup", { code, isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
  assert.equal(f.input.keys(), 2 | 16);
  for (const code of ["KeyS", "ShiftRight"]) emit(f.document, "keyup", { code, isTrusted: true });
  assert.equal(f.input.takeActions(), 1024 | 2048);
  for (const code of ["KeyS", "ShiftRight"]) emit(f.document, "keyup", { code, isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
});

test("Shift release and repress delivers both the release fact and a fresh dodge", () => {
  const f = fixture(); f.parse(); f.capture();
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.takeActions(), 2);
  emit(f.document, "keyup", { code: "ShiftLeft", isTrusted: true });
  emit(f.document, "keydown", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.keys(), 16);
  assert.equal(f.input.takeActions(), 2048 | 2);
  assert.equal(f.input.takeActions(), 0);
});

test("physical release facts survive blur and recapture between samples", () => {
  const f = fixture(); f.parse(); f.capture();
  for (const code of ["KeyW", "ShiftLeft", "KeyE"]) emit(f.document, "keydown", { code, isTrusted: true });
  assert.equal(f.input.takeActions(), 2 | 8);
  f.blur(); f.focus(); f.click(); f.confirm();
  for (const code of ["KeyW", "ShiftLeft"]) emit(f.document, "keydown", { code, isTrusted: true });
  assert.equal(f.input.keys(), 1 | 16);
  assert.equal(f.input.takeActions(), 1024 | 2048 | 2);
  assert.equal(f.input.takeActions(), 0);
});

test("untrusted or inactive key releases cannot publish release facts", () => {
  const f = fixture(); f.parse();
  emit(f.document, "keyup", { code: "KeyW", isTrusted: true });
  emit(f.document, "keyup", { code: "ShiftLeft", isTrusted: true });
  assert.equal(f.input.takeActions(), 0);
  f.capture();
  emit(f.document, "keydown", { code: "KeyW", isTrusted: true });
  emit(f.document, "keyup", { code: "KeyW", isTrusted: false });
  assert.equal(f.input.keys(), 1);
  assert.equal(f.input.takeActions(), 0);
});
