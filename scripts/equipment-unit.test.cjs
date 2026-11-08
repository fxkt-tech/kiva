// Exercise the production equipment popup with DOM controls and real intents.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const source = fs.readFileSync(path.join(__dirname, "../web/equipment.js"), "utf8");

class Element {
  constructor(tag = "div") {
    this.tagName = tag.toUpperCase(); this.dataset = {}; this.attributes = new Map();
    this.children = []; this.listeners = new Map(); this.hidden = false; this.disabled = false;
    this.textContent = ""; this.scrollTop = 0; this.clientHeight = 120;
    const classes = new Set();
    this.classList = { add: (value) => classes.add(value), contains: (value) => classes.has(value),
      toggle(value, enabled) { if (enabled) classes.add(value); else classes.delete(value); } };
  }
  append(...children) { this.children.push(...children); }
  replaceChildren(...children) { this.children = children; }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  hasAttribute(name) { return name.startsWith("data-")
    ? Object.hasOwn(this.dataset, name.slice(5).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase()))
    : this.attributes.has(name); }
  addEventListener(type, listener) { if (!this.listeners.has(type)) this.listeners.set(type, []); this.listeners.get(type).push(listener); }
  emit(type, fields = {}) {
    const event = { target: this, preventDefault() { this.defaultPrevented = true; }, ...fields };
    for (const listener of this.listeners.get(type) || []) listener(event);
    return event;
  }
  click() { if (!this.disabled) this.emit("click"); }
  focus() { this.focused = true; }
  contains(target) { return this === target || this.children.some((child) => child.contains(target)); }
  querySelectorAll(selector) {
    const matches = (element) => selector.startsWith("[data-")
      ? element.hasAttribute(selector.slice(1, -1)) : element.className === selector.slice(1);
    return this.children.flatMap((child) => [...(matches(child) ? [child] : []), ...child.querySelectorAll(selector)]);
  }
  querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
}

const definitions = [{ id: 1, name: "锋刃", description: "攻击装备", bonuses: ["攻击 +20"], passives: [], active: null },
  { id: 7, name: "法珠", description: "主动装备", bonuses: [], passives: [],
    active: { name: "法珠", description: "选择目标", target_hint: "瞄准敌人", cost: 10, total_cooldown: 5 } }];
function snapshot(items = [], overrides = {}) {
  const slots = Array.from({ length: 6 }, (_, slot) => ({ slot, item: items[slot] || null }));
  return { player_id: "player-1", paused: false, session: { phase: "Active" }, match: { phase: "Running" },
    equipment_aim: null, equipment_catalog: definitions,
    equipment: { slots, first_empty_slot: slots.find(({ item }) => !item)?.slot ?? null,
      can_manage: true, ack: { request_id: 0, accepted: true, error: null } }, ...overrides };
}
function fixture(earlyCatalog = false, onToggle = () => {}) {
  const elements = new Map(["game", "equipment-editor", "equipment-toggle", "equipment-close",
    "equipment-status", "equipment-catalog", "equipment-count", "equipment-catalog-scroll"].map((id) => [id, new Element()]));
  elements.get("equipment-editor").hidden = true;
  elements.get("equipment-catalog-scroll").append(elements.get("equipment-catalog"));
  const slots = Array.from({ length: 6 }, (_, index) => {
    const button = new Element("button"); button.dataset.equipmentSlot = String(index);
    for (const name of ["equipment-symbol", "equipment-cooldown"]) { const child = new Element("span"); child.className = name; button.append(child); }
    return button;
  });
  const root = new Element(); root.dataset = { paused: "false", pointerLocked: "true" };
  const events = new Element(); let blocked = false;
  const document = { documentElement: root, pointerLockElement: elements.get("game"),
    getElementById(id) { assert(elements.has(id), `unexpected DOM dependency: ${id}`); return elements.get(id); },
    createElement(tag) { return new Element(tag); },
    querySelectorAll(selector) { assert.equal(selector, "[data-equipment-slot]"); return slots; },
    querySelector() { return null; }, elementFromPoint() { return document.hit; }, hit: null };
  const window = { addEventListener: events.addEventListener.bind(events),
    removeEventListener(type, listener) { events.listeners.set(type, (events.listeners.get(type) || []).filter((entry) => entry !== listener)); },
    kivaPointer: { position: () => [100, 150] } };
  vm.runInNewContext(source, { document, window }, { filename: "web/equipment.js" });
  if (earlyCatalog) events.emit("kiva-state", { detail: JSON.stringify({ equipment_catalog: definitions }) });
  const ui = window.createEquipmentUi({ blocked: () => blocked, onToggle, pause() { throw new Error("popup must never pause"); } });
  return { ui, document, root, slots, window, element: (id) => elements.get(id),
    block(value) { blocked = value; },
    key(code, extra = {}) { return events.emit("keydown", { code, key: code === "Escape" ? "Escape" : code.slice(3), ...extra }); },
    wheel(fields) { return events.emit("wheel", fields); },
    request() { return JSON.parse(root.dataset.equipmentRequests).at(-1); },
    ack(state, requestId, error = null) {
      delete root.dataset.equipmentRequests;
      state.equipment.ack = { request_id: requestId, accepted: !error, error };
      ui.render(state);
    } };
}

test("T and close toggle the live popup without changing pause or Pointer Lock", () => {
  const f = fixture(); f.ui.render(snapshot());
  const captured = f.document.pointerLockElement;
  f.key("KeyP"); assert.equal(f.ui.isOpen(), false);
  f.key("KeyT", { repeat: true }); f.key("KeyT", { ctrlKey: true }); assert.equal(f.ui.isOpen(), false);
  f.key("KeyT"); assert.equal(f.ui.isOpen(), true); assert.equal(f.element("equipment-editor").hidden, false);
  assert.equal(f.root.dataset.paused, "false"); assert.equal(f.document.pointerLockElement, captured);
  assert.equal(f.element("equipment-close").focused, undefined, "locked canvas focus stays with gameplay");
  f.key("KeyT"); assert.equal(f.ui.isOpen(), false);
  f.element("equipment-toggle").click(); f.element("equipment-close").click();
  assert.equal(f.ui.isOpen(), false); assert.equal(f.root.dataset.paused, "false"); assert.equal(f.document.pointerLockElement, captured);
});

test("paused-menu presentation is notified once per toggle and equipment never changes the pause flag", () => {
  const changes = []; let f;
  f = fixture(false, () => changes.push(f.ui.isOpen()));
  f.root.dataset.paused = "true"; f.document.pointerLockElement = null;
  f.ui.render(snapshot([], { paused: true }));
  f.element("equipment-toggle").click(); f.element("equipment-toggle").click();
  assert.equal(f.element("equipment-close").focused, true);
  f.ui.close(); f.ui.close();
  assert.deepEqual(changes, [true, false]); assert.equal(f.root.dataset.paused, "true");
});

test("catalog survives an early snapshot and equips into Rust's first empty slot with ack gating", () => {
  const f = fixture(true); const state = snapshot([], { equipment_catalog: null }); f.ui.render(state); f.key("KeyT");
  const buttons = f.element("equipment-catalog").querySelectorAll("[data-equip-id]");
  assert.equal(buttons.length, 2); buttons[0].click();
  assert.deepEqual(f.request(), { type: "Equip", equipment_id: 1, slot: 0, player_id: "player-1", request_id: 1 });
  assert.equal(f.ui.isPending(), true); assert(buttons.every((button) => button.disabled));
  buttons[1].click(); assert.equal(JSON.parse(f.root.dataset.equipmentRequests).length, 1);
  f.ack(state, 1); assert.equal(f.ui.isPending(), false); assert.equal(buttons[0].disabled, false);
  buttons[0].click(); f.ack(state, null, "装备请求格式无效"); assert.equal(f.ui.isPending(), false);
  assert.equal(f.element("equipment-status").classList.contains("error"), true);
  assert.equal(f.element("equipment-status").textContent, "装备请求格式无效");
});

test("bottom slots remove the current passive or cooling instance only while the popup is open", () => {
  const f = fixture(); const passive = { id: 1, name: "锋刃", instance_id: "old", active: null };
  let state = snapshot([passive]); f.ui.render(state);
  f.slots[0].emit("kiva-contextmenu"); assert.equal(f.root.dataset.equipmentRequests, undefined);
  f.key("KeyT"); assert.equal(f.slots[0].disabled, false);
  f.slots[0].click(); assert.equal(f.root.dataset.equipmentSkill, undefined);
  state = snapshot([{ ...passive, instance_id: "current" }]); f.ui.render(state);
  const event = f.slots[0].emit("kiva-contextmenu"); assert.equal(event.defaultPrevented, true);
  assert.deepEqual(f.request(), { type: "Unequip", slot: 0, instance_id: "current", player_id: "player-1", request_id: 1 });
  f.slots[0].emit("contextmenu"); assert.equal(JSON.parse(f.root.dataset.equipmentRequests).length, 1, "pending prevents duplicate removal");
  f.ack(snapshot([{ id: 7, name: "法珠", instance_id: "cooling", active: { available: false, cooldown: 3 } }]), 1);
  assert.equal(f.slots[0].disabled, false); f.slots[0].emit("contextmenu"); assert.equal(f.request().instance_id, "cooling");
  f.ui.close(); assert.equal(f.slots[0].disabled, true);
});

test("active left clicks keep the existing skill intent while the catalog is open", () => {
  const f = fixture(); f.ui.render(snapshot([{ id: 7, name: "法珠", instance_id: "active-1", active: { available: true, cooldown: 0 } }]));
  f.key("KeyT"); f.slots[0].click();
  assert.deepEqual(JSON.parse(f.root.dataset.equipmentSkill), { slot: 0, instance_id: "active-1" });
  assert.equal(f.element("game").focused, true); assert.equal(f.root.dataset.paused, "false");
});

test("full inventory prevents equips and pause/selection/player changes/finish close stale popup state", () => {
  const f = fixture(); const item = { id: 1, name: "锋刃", instance_id: "one", active: null };
  f.ui.render(snapshot(Array(6).fill(item))); f.key("KeyT");
  assert(f.element("equipment-catalog").querySelectorAll("[data-equip-id]").every((button) => button.disabled));
  for (const next of [snapshot([], { paused: true }), snapshot([], { session: { phase: "Selecting" }, player_id: null }),
    snapshot([], { player_id: "player-2" }), snapshot([], { match: { phase: "Finished" } })]) {
    f.ui.close();
    f.ui.render(snapshot()); f.key("KeyT"); assert.equal(f.ui.isOpen(), true);
    f.ui.render(next); assert.equal(f.ui.isOpen(), false); assert.equal(f.element("equipment-editor").hidden, true);
  }
  f.key("KeyT"); assert.equal(f.ui.isOpen(), false, "finished matches cannot reopen the popup");
  f.ui.render(snapshot()); f.block(true); f.key("KeyT"); assert.equal(f.ui.isOpen(), false);
});

test("captured wheel scrolls only the popup catalog using the shared CSS pointer", () => {
  const f = fixture(); f.ui.render(snapshot()); f.key("KeyT");
  const scroll = f.element("equipment-catalog-scroll"); f.document.hit = f.element("equipment-catalog");
  assert.equal(f.wheel({ deltaY: 2, deltaMode: 1 }).defaultPrevented, true); assert.equal(scroll.scrollTop, 32);
  f.wheel({ deltaY: 1, deltaMode: 2 }); assert.equal(scroll.scrollTop, 152);
  f.document.hit = f.element("game"); assert.equal(f.wheel({ deltaY: 20, deltaMode: 0 }).defaultPrevented, undefined);
  assert.equal(scroll.scrollTop, 152);
  f.document.pointerLockElement = null; f.document.hit = f.element("equipment-catalog");
  assert.equal(f.wheel({ deltaY: 20, deltaMode: 0 }).defaultPrevented, undefined, "unlocked DOM scroll remains native");
});

test("popup markup contains the catalog only and isolates the actual card rather than the whole screen", () => {
  const html = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");
  const wrapper = html.match(/<div id="equipment-editor"[^>]*>/)[0];
  assert(!wrapper.includes("data-ui")); assert(!wrapper.includes('class="overlay"'));
  assert(wrapper.includes('aria-modal="false"')); assert(!html.includes('id="equipment-inventory"'));
  assert(html.includes('class="equipment-editor-card panel" data-ui'));
});
