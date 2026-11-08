// Drive the production HUD and settings lifecycle with platform events, no WASM.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const read = (file) => fs.readFileSync(path.join(__dirname, "../web", file), "utf8");

class Element {
  constructor() {
    this.dataset = {}; this.attributes = new Map(); this.listeners = new Map();
    this.children = []; this.selectors = new Map(); this.hidden = false; this.disabled = false;
    this.style = {}; this.textContent = ""; this.tabIndex = 0; this.parentElement = null;
    const classes = new Set();
    this.classList = { add: (...values) => values.forEach((value) => classes.add(value)),
      remove: (...values) => values.forEach((value) => classes.delete(value)),
      toggle: (value, enabled) => enabled ? classes.add(value) : classes.delete(value) };
  }
  addEventListener(type, listener) { if (!this.listeners.has(type)) this.listeners.set(type, []); this.listeners.get(type).push(listener); }
  removeEventListener(type, listener) { this.listeners.set(type, (this.listeners.get(type) || []).filter((value) => value !== listener)); }
  emit(type, fields = {}) {
    const event = { target: this, preventDefault() {}, ...fields };
    for (const callback of [...(this.listeners.get(type) || [])]) callback(event);
  }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  getAttribute(name) { return this.attributes.get(name) ?? null; }
  hasAttribute(name) { return name.startsWith("data-") ? Object.hasOwn(this.dataset, name.slice(5).replace(/-([a-z])/g, (_, value) => value.toUpperCase())) : this.attributes.has(name); }
  removeAttribute(name) { this.attributes.delete(name); }
  querySelector(selector) { if (!this.selectors.has(selector)) this.selectors.set(selector, new Element()); return this.selectors.get(selector); }
  querySelectorAll(selector) {
    const key = selector.match(/^\[data-([a-z-]+)\]$/)?.[1]?.replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
    return this.children.flatMap((child) => [...(key && Object.hasOwn(child.dataset, key) ? [child] : []), ...child.querySelectorAll(selector)]);
  }
  append(...children) {
    for (const child of children) {
      if (child.parentElement) child.parentElement.children = child.parentElement.children.filter((entry) => entry !== child);
      child.parentElement = this; this.children.push(child);
    }
  }
  appendChild(child) { this.append(child); return child; }
  prepend(...children) { this.children.unshift(...children); }
  replaceChildren(...children) { this.children = []; this.append(...children); }
  remove() { if (this.parentElement) this.parentElement.children = this.parentElement.children.filter((child) => child !== this); this.parentElement = null; }
  closest() { return this; }
  focus() { this.focused = true; }
  click() { if (!this.disabled) this.emit("click"); }
  get nextElementSibling() { return this.querySelector("next-sibling"); }
}

const characters = [1, 2].map((id) => ({ id, name: id === 1 ? "皇子" : "寒冰", description: `角色${id}`,
  max_health: id === 1 ? 640 : 600, max_mana: 300, display_attributes: { attack_damage: id === 1 ? 64 : 59 },
  passives: [`被动${id}`], abilities: [{ slot: 0, name: `技能${id}`, target_hint: "方向", description: "说明", cost: 10, total_cooldown: 5, display_range: 600 }] }));

function state(selecting = false, finished = false, overrides = {}) {
  return { player_id: selecting ? null : "hero-1", paused: selecting || finished,
    session: selecting ? { phase: "Selecting", selection: { choices: characters, selected: characters[0],
      selection_revision: 1, selection_ack: { request_id: 0, accepted: true }, can_start: true } }
      : { phase: "Active", selection: null },
    match: { id: selecting ? null : "match-1", phase: selecting ? "Waiting" : finished ? "Finished" : "Running", error: null,
      bases: { ally: null, enemy: null }, elapsed_seconds: 0, wave_number: 1, next_wave_seconds: 60 },
    character_catalog: characters, units: selecting ? [] : [{ id: "hero-1", character_id: 1 }],
    abilities: [], selected: null, aim: null, equipment_aim: null,
    attack_move: false, notice: "", camera_locked: false, ...overrides };
}

function fixture(selecting = false, { earlyCatalog = false } = {}) {
  const elements = new Map();
  const element = (id) => { if (!elements.has(id)) elements.set(id, new Element()); return elements.get(id); };
  const root = new Element(); const pauseWrites = [];
  root.dataset = new Proxy({ paused: String(selecting), pointerLocked: String(!selecting) }, {
    set(target, key, value) { if (key === "paused") pauseWrites.push(value); target[key] = value; return true; },
  });
  const canvas = element("game"); canvas.id = "game";
  const document = new Element(); Object.assign(document, {
    documentElement: root, pointerLockElement: selecting ? null : canvas, focused: true,
    getElementById: element, createElement: () => new Element(),
    querySelector: (selector) => selector.includes("data-virtual-hover") ? null : element(selector),
    hasFocus() { return this.focused; },
    exitPointerLock() { this.pointerLockElement = null; this.emit("pointerlockchange"); },
  });
  const tabs = ["controls", "audio", "characters"].map((name) => { const tab = new Element(); tab.dataset.settingsTab = name; return tab; });
  const panels = ["controls", "audio", "characters"].map((name) => { const panel = new Element(); panel.dataset.settingsPanel = name; return panel; });
  element("settings-dialog").querySelectorAll = (selector) => selector === "[data-settings-tab]" ? tabs : panels;
  element("settings-dialog").hidden = true;
  const window = new Element(); let startup; let captureRequests = 0; let reject = false;
  Object.assign(window, {
    createMinimap: () => ({ render() {} }), renderSkillState() {},
    kivaPointer: { sync() {} }, kivaStartup: { subscribe(callback) { startup = callback; } },
    createEquipmentUi: () => ({ close() {}, isOpen: () => false, isPending: () => false, render() {}, detail: () => null }),
  });
  canvas.requestPointerLock = () => { captureRequests++; return reject ? Promise.reject(new Error("denied")) : Promise.resolve(); };
  const context = { window, document, MutationObserver: class { observe() {} }, location: { reload() {} } };
  vm.runInNewContext(read("characters.js"), context, { filename: "web/characters.js" });
  if (earlyCatalog) window.emit("kiva-state", { detail: JSON.stringify({ character_catalog: characters }) });
  vm.runInNewContext(read("match.js"), context, { filename: "web/match.js" });
  vm.runInNewContext(read("settings.js"), context, { filename: "web/settings.js" });
  vm.runInNewContext(read("hud.js"), context, { filename: "web/hud.js" });
  const publish = (snapshot) => window.emit("kiva-state", { detail: JSON.stringify(snapshot) });
  element("entry-character-host").append(element("character-select"));
  startup("ready"); publish(state(selecting, false, { character_catalog: earlyCatalog ? null : characters })); pauseWrites.length = 0;
  return { root, document, window, element, pauseWrites, publish,
    choose(id, force = false) {
      const button = element("character-choices").children.find((node) => Number(node.dataset.characterId) === id);
      assert(button, `role ${id} is rendered`); if (force) button.emit("click"); else button.click();
    },
    consume() { delete root.dataset.sessionRequests; },
    escape() { window.emit("keydown", { key: "Escape" }); },
    unlock() { document.exitPointerLock(); },
    confirm() { document.pointerLockElement = canvas; document.emit("pointerlockchange"); },
    rejectCapture() { reject = true; }, requests: () => captureRequests,
    flush: () => new Promise((resolve) => setImmediate(resolve)) };
}

test("Escape opens settings and releases the pointer without requesting a pause", () => {
  const f = fixture(); f.escape();
  assert.equal(f.root.dataset.settingsOpen, "true"); assert.equal(f.document.pointerLockElement, null);
  assert.equal(f.element("settings-dialog").hidden, false); assert.equal(f.element("entry").hidden, true);
  assert.equal(f.root.dataset.paused, "false"); assert(!f.pauseWrites.includes("true"));
});

test("focused pointer loss without a key event opens the same non-pausing settings", () => {
  const f = fixture(); f.unlock();
  assert.equal(f.root.dataset.settingsOpen, "true"); assert.equal(f.root.dataset.paused, "false");
  assert(!f.pauseWrites.includes("true"));
});

test("closing waits for actual capture notification, not its fulfilled promise", async () => {
  const f = fixture(); f.escape(); f.element("settings-close").click(); await f.flush();
  assert.equal(f.requests(), 1); assert.equal(f.root.dataset.settingsOpen, "true");
  assert.equal(f.element("settings-close").disabled, true); assert.equal(f.root.dataset.paused, "false");
  f.confirm();
  assert.equal(f.root.dataset.settingsOpen, "false"); assert.equal(f.element("settings-dialog").hidden, true);
  assert.equal(f.root.dataset.pointerLocked, "true"); assert.equal(f.root.dataset.paused, "false");
});

test("capture rejection keeps a running game in settings with a retryable error", async () => {
  const f = fixture(); f.escape(); f.rejectCapture(); f.element("settings-close").click(); await f.flush();
  assert.equal(f.root.dataset.settingsOpen, "true"); assert.equal(f.element("settings-dialog").hidden, false);
  assert.equal(f.element("settings-error").hidden, false); assert(f.element("settings-error").textContent.includes("重试"));
  assert.equal(f.element("settings-close").disabled, false); assert.equal(f.root.dataset.paused, "false");
  assert(!f.pauseWrites.includes("true"));
});

test("real blur still pauses and keeps settings available for returning", () => {
  const f = fixture(); f.document.focused = false; f.window.emit("blur");
  assert.equal(f.root.dataset.paused, "true"); assert.equal(f.root.dataset.settingsOpen, "true");
  assert.equal(f.document.pointerLockElement, null);
});

test("late capture confirmation after blur cannot resume an unfocused game", () => {
  const f = fixture(); f.escape(); f.element("settings-close").click();
  f.document.focused = false; f.window.emit("blur"); f.confirm();
  assert.equal(f.root.dataset.paused, "true"); assert.equal(f.root.dataset.settingsOpen, "true");
  assert.equal(f.document.pointerLockElement, null);
});

test("returning to selection clears settings and exposes the original character entry", () => {
  const f = fixture(); f.escape(); f.publish(state(true));
  assert.equal(f.root.dataset.settingsOpen, "false"); assert.equal(f.element("settings-dialog").hidden, true);
  assert.equal(f.element("entry").hidden, false); assert.equal(f.root.dataset.sessionPhase, "Selecting");
});

test("terminal match projection removes settings so the match result owns the screen", () => {
  const f = fixture(); f.escape(); f.publish(state(false, true));
  assert.equal(f.root.dataset.settingsOpen, "false"); assert.equal(f.element("settings-dialog").hidden, true);
});

test("settings can close back to character selection without asking for capture", () => {
  const f = fixture(true); f.escape(); assert.equal(f.root.dataset.settingsOpen, "true");
  f.element("settings-close").click();
  assert.equal(f.root.dataset.settingsOpen, "false"); assert.equal(f.requests(), 0); assert.equal(f.element("entry").hidden, false);
});

test("the early character catalog survives a late HUD and active browsing sends no session intent", () => {
  const f = fixture(false, { earlyCatalog: true });
  assert.equal(f.window.kivaCharacters.catalog().length, 2);
  const widget = f.element("character-select");
  assert.equal(widget.parentElement, f.element("entry-character-host"));
  f.escape(); f.choose(2);
  assert.equal(widget.parentElement, f.element("settings-character-host"));
  assert.equal(f.element("preview-name").textContent, "寒冰");
  assert.equal(f.element('[data-preview-stat="attack_damage"]').textContent, "59");
  assert.equal(f.root.dataset.sessionRequests, undefined);
  assert.equal(f.root.dataset.paused, "false");
  assert.equal(f.window.__kivaState.units[0].character_id, 1, "only the draft changed");
  f.publish(state(false, false, { character_catalog: null }));
  assert.equal(f.window.kivaCharacters.catalog().length, 2);
  assert.equal(f.element("preview-name").textContent, "寒冰");
});

test("initial selection reuses the same widget and retains the SelectCharacter/Launch protocol", () => {
  const f = fixture(true); const widget = f.element("character-select");
  f.escape(); f.choose(2);
  assert.deepEqual(JSON.parse(f.root.dataset.sessionRequests), [{ type: "SelectCharacter", character_id: 2, request_id: 1 }]);
  f.consume();
  const selected = state(true); selected.session.selection.selected = characters[1];
  selected.session.selection.selection_revision = 2; selected.session.selection.selection_ack.request_id = 1;
  f.publish(selected); assert.equal(f.element("preview-name").textContent, "寒冰");
  f.element("character-apply").click(); assert.equal(f.requests(), 1);
  assert.equal(f.root.dataset.sessionRequests, undefined);
  f.confirm();
  assert.deepEqual(JSON.parse(f.root.dataset.sessionRequests), [{ type: "Launch", character_id: 2, selection_revision: 2 }]);
  f.consume(); f.publish(state(false, false, { player_id: "hero-2", units: [] }));
  assert.equal(f.root.dataset.settingsOpen, "false");
  assert.equal(widget.parentElement, f.element("entry-character-host"));
  f.escape(); assert.equal(widget.parentElement, f.element("settings-character-host"));
  assert.equal(f.element("preview-name").textContent, "寒冰", "the confirmed launch choice covers a missing first model view");
});

test("initial launch and selection errors are visible on the inline settings surface", async () => {
  const f = fixture(true); f.escape();
  f.element("settings-dialog").querySelectorAll("[data-settings-tab]")[2].click();
  f.rejectCapture(); f.element("character-apply").click(); await f.flush();
  assert.equal(f.element("entry").hidden, true); assert.equal(f.root.dataset.settingsOpen, "true");
  assert.equal(f.element("settings-error").hidden, false); assert(f.element("settings-error").textContent.includes("重试"));
  assert.equal(f.element("character-apply").disabled, false); assert.equal(f.root.dataset.paused, "true");
  const rejected = state(true); rejected.session.selection.error = "角色配置暂不可用";
  f.publish(rejected); assert.equal(f.element("character-status").textContent, "角色配置暂不可用");
});

test("role confirmation freezes identities and choice, then waits for a replacement match", () => {
  const f = fixture(); f.escape(); f.choose(2); f.element("character-apply").click();
  f.choose(1, true); assert.equal(f.element("preview-name").textContent, "寒冰");
  assert.equal(f.root.dataset.sessionRequests, undefined, "capture request alone cannot recreate the match");
  f.confirm();
  assert.deepEqual(JSON.parse(f.root.dataset.sessionRequests), [{ type: "ChangeCharacter", match_id: "match-1", player_id: "hero-1", character_id: 2 }]);
  assert.equal(f.root.dataset.settingsOpen, "true"); assert.equal(f.element("character-apply").disabled, true);
  f.consume(); f.publish(state(false, false, { player_id: "hero-2", units: [],
    match: { ...state().match, id: "match-2" } }));
  assert.equal(f.root.dataset.settingsOpen, "false"); assert.equal(f.root.dataset.paused, "false");
  f.escape(); assert.equal(f.element("preview-name").textContent, "寒冰", "a delayed model view must not default to the first catalog role");
});

test("a refused role change releases capture and preserves a retryable draft", () => {
  const f = fixture(); f.escape(); f.choose(2); f.element("character-apply").click(); f.confirm();
  f.consume(); f.publish(state());
  assert.equal(f.root.dataset.settingsOpen, "true"); assert.equal(f.document.pointerLockElement, null);
  assert.equal(f.element("settings-error").hidden, false); assert(f.element("settings-error").textContent.includes("重新确认"));
  assert.equal(f.element("preview-name").textContent, "寒冰");
  assert.equal(f.element("character-apply").disabled, false); assert.equal(f.root.dataset.paused, "false");
});

test("role capture denial sends no mutation and leaves the selected preview available", async () => {
  const f = fixture(); f.escape(); f.choose(2); f.rejectCapture(); f.element("character-apply").click(); await f.flush();
  assert.equal(f.root.dataset.sessionRequests, undefined); assert.equal(f.root.dataset.settingsOpen, "true");
  assert.equal(f.element("settings-dialog").dataset.category, "characters");
  assert.equal(f.element("preview-name").textContent, "寒冰"); assert.equal(f.element("character-apply").disabled, false);
  assert.equal(f.element("settings-error").hidden, false); assert.equal(f.root.dataset.paused, "false");
});

test("finishing during role capture keeps the frozen change eligible for confirmation", () => {
  const f = fixture(); f.escape(); f.choose(2); f.element("character-apply").click();
  f.publish(state(false, true));
  f.root.dataset.paused = "true"; // Rust has published the old match's terminal pause.
  assert.equal(f.root.dataset.settingsOpen, "true"); f.confirm();
  assert.deepEqual(JSON.parse(f.root.dataset.sessionRequests), [{ type: "ChangeCharacter", match_id: "match-1", player_id: "hero-1", character_id: 2 }]);
  assert.equal(f.root.dataset.settingsOpen, "true");
  assert.equal(f.root.dataset.paused, "false", "the new match must not inherit the old terminal pause request");
});

test("the result role action opens the inline chooser without a pending reset or a return-page intent", () => {
  const f = fixture(); f.publish(state(false, true)); f.element("result-reselect").click();
  assert.equal(f.root.dataset.settingsOpen, "true"); assert.equal(f.element("settings-dialog").dataset.category, "characters");
  assert.equal(f.element("result-reselect").disabled, false); assert.equal(f.root.dataset.sessionRequests, undefined);
  f.publish(state(false, true)); assert.equal(f.root.dataset.settingsOpen, "true", "repeated result projections do not close browsing");
  f.element("settings-close").click(); assert.equal(f.root.dataset.settingsOpen, "false");
  assert.equal(f.element("match-result").hidden, false); assert.equal(f.requests(), 0, "cancelling the result chooser needs no capture");
});

test("an unrelated replacement before capture cancels the old role attempt", () => {
  const f = fixture(); f.escape(); f.choose(2); f.element("character-apply").click();
  f.publish(state(false, false, { player_id: "hero-external", units: [{ id: "hero-external", character_id: 1 }],
    match: { ...state().match, id: "match-external" } }));
  assert.equal(f.element("settings-error").hidden, false); assert.equal(f.element("character-apply").disabled, false);
  f.confirm();
  assert.equal(f.root.dataset.sessionRequests, undefined); assert.equal(f.document.pointerLockElement, null);
  assert.equal(f.root.dataset.settingsOpen, "true");
});
