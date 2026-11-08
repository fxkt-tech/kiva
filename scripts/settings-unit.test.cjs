// The settings component owns presentation; capture and simulation stay outside it.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const source = fs.readFileSync(path.join(__dirname, "../web/settings.js"), "utf8");

class Element {
  constructor(dataset = {}) {
    this.dataset = dataset; this.hidden = false; this.disabled = false;
    this.attributes = new Map(); this.listeners = new Map(); this.children = [];
    this.textContent = ""; this.tabIndex = 0;
  }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  addEventListener(type, listener) { this.listeners.set(type, listener); }
  emit(type, fields = {}) {
    const event = { preventDefault() { this.defaultPrevented = true; }, ...fields };
    this.listeners.get(type)?.(event); return event;
  }
  click() { if (!this.disabled) this.emit("click"); }
  focus() { this.focused = true; }
}

function fixture() {
  const categories = ["controls", "audio", "characters"];
  const tabs = categories.map((name) => new Element({ settingsTab: name }));
  const panels = categories.map((name) => new Element({ settingsPanel: name }));
  const controls = { mode: { value: "quick" }, muted: { checked: false }, volume: { value: "65" } };
  panels[0].children.push(controls.mode); panels[1].children.push(controls.muted, controls.volume);
  const elements = new Map(["settings-dialog", "settings-toggle", "settings-close", "settings-error"].map((id) => [id, new Element()]));
  const dialog = elements.get("settings-dialog"); dialog.hidden = true;
  dialog.querySelectorAll = (selector) => selector === "[data-settings-tab]" ? tabs : panels;
  const root = new Element(); root.dataset.paused = "false";
  const captured = { id: "game" };
  const document = { documentElement: root, pointerLockElement: captured,
    getElementById(id) { assert(elements.has(id), `unexpected DOM dependency ${id}`); return elements.get(id); } };
  const window = {}; let closeRequests = 0; const toggles = [];
  vm.runInNewContext(source, { document, window }, { filename: "web/settings.js" });
  const ui = window.createSettingsUi({ requestClose() { closeRequests++; },
    onToggle() { toggles.push({ opened: ui.isOpen(), flag: root.dataset.settingsOpen, hidden: dialog.hidden }); } });
  return { ui, document, root, captured, controls, tabs, panels, dialog, toggles,
    element: (id) => elements.get(id), closeRequests: () => closeRequests };
}

test("opening and closing publish settings state without changing pause or capture", () => {
  const f = fixture();
  assert.equal(f.root.dataset.settingsOpen, "false"); assert.equal(f.dialog.hidden, true);
  f.ui.open(); f.ui.open();
  assert.equal(f.ui.isOpen(), true); assert.equal(f.root.dataset.settingsOpen, "true");
  assert.equal(f.dialog.hidden, false); assert.equal(f.element("settings-toggle").attributes.get("aria-expanded"), "true");
  assert.equal(f.root.dataset.paused, "false"); assert.equal(f.document.pointerLockElement, f.captured);
  assert.deepEqual(f.toggles, [{ opened: true, flag: "true", hidden: false }]);
  f.ui.close(); f.ui.close();
  assert.equal(f.ui.isOpen(), false); assert.equal(f.root.dataset.settingsOpen, "false");
  assert.equal(f.dialog.hidden, true); assert.equal(f.toggles.length, 2);
  assert.equal(f.root.dataset.paused, "false"); assert.equal(f.document.pointerLockElement, f.captured);
});

test("categories show only their own panel and preserve independent control values", () => {
  const f = fixture(); f.ui.open();
  assert.deepEqual(f.panels.map((panel) => panel.hidden), [false, true, true]);
  f.controls.mode.value = "normal"; f.ui.select("audio");
  f.controls.muted.checked = true; f.controls.volume.value = "24";
  f.ui.select("characters"); f.ui.select("controls");
  assert.deepEqual(f.panels.map((panel) => panel.hidden), [false, true, true]);
  assert.equal(f.controls.mode.value, "normal");
  assert.equal(f.panels[1].children[0].checked, true); assert.equal(f.panels[1].children[1].value, "24");
  assert.equal(f.ui.select("missing"), false); assert.equal(f.panels[0].hidden, false);
  f.ui.select("audio"); f.ui.close(); f.ui.open();
  assert.deepEqual(f.panels.map((panel) => panel.hidden), [true, false, true]);
  assert.equal(f.tabs[1].attributes.get("aria-selected"), "true");
  assert.deepEqual(f.tabs.map((tab) => tab.tabIndex), [-1, 0, -1]);
});

test("category mouse and keyboard navigation share the same selected state", () => {
  const f = fixture(); f.ui.open();
  f.tabs[2].click(); assert.equal(f.panels[2].hidden, false);
  const event = f.tabs[2].emit("keydown", { key: "ArrowDown" });
  assert.equal(event.defaultPrevented, true); assert.equal(f.panels[0].hidden, false); assert.equal(f.tabs[0].focused, true);
  f.tabs[0].emit("keydown", { key: "ArrowUp" }); assert.equal(f.panels[2].hidden, false);
  f.tabs[2].emit("keydown", { key: "Home" }); assert.equal(f.panels[0].hidden, false);
  f.tabs[0].emit("keydown", { key: "End" }); assert.equal(f.panels[2].hidden, false);
  assert.equal(f.tabs[2].emit("keydown", { key: "Escape" }).defaultPrevented, undefined, "the parent owns Escape");
});

test("close requests await parent capture confirmation and busy requests cannot repeat", () => {
  const f = fixture(); f.ui.open();
  f.element("settings-close").click(); assert.equal(f.closeRequests(), 1);
  assert.equal(f.ui.isOpen(), true); assert.equal(f.root.dataset.settingsOpen, "true");
  f.ui.setBusy(true); f.element("settings-close").click();
  assert.equal(f.closeRequests(), 1); assert.equal(f.element("settings-close").disabled, true);
  assert.equal(f.dialog.attributes.get("aria-busy"), "true");
  f.ui.setBusy(false); f.element("settings-close").click(); assert.equal(f.closeRequests(), 2);
  f.ui.close(); assert.equal(f.ui.isOpen(), false); assert.equal(f.element("settings-close").disabled, false);
});

test("capture errors stay visible until cleared or the dialog closes", () => {
  const f = fixture(); f.ui.open();
  f.ui.setError("捕获失败，请重试。");
  assert.equal(f.element("settings-error").hidden, false);
  assert.equal(f.element("settings-error").textContent, "捕获失败，请重试。");
  f.ui.select("audio"); assert.equal(f.element("settings-error").hidden, false);
  f.ui.setError(""); assert.equal(f.element("settings-error").hidden, true);
  f.ui.setError("旧错误"); f.ui.close(); f.ui.open();
  assert.equal(f.element("settings-error").hidden, true);
});

test("preferences and a single shared character widget keep their owners", () => {
  const html = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");
  const dialog = html.slice(html.indexOf('<div id="settings-dialog"'), html.indexOf('<div id="match-result"'));
  const category = (name) => dialog.match(new RegExp(`<section[^>]*data-settings-panel="${name}"[^>]*>([\\s\\S]*?)</section>`))[1];
  for (const id of ["cast-mode", "audio-muted", "audio-volume", "character-select", "character-choices", "character-preview", "character-apply"]) {
    assert.equal((html.match(new RegExp(`id="${id}"`, "g")) || []).length, 1, `${id} must keep a single owner`);
  }
  assert(category("controls").includes('id="cast-mode"'));
  assert(category("audio").includes('id="audio-muted"')); assert(category("audio").includes('id="audio-volume"'));
  assert(category("characters").includes('id="settings-character-host"'));
  assert(category("characters").includes('id="character-apply"'));
  assert(html.includes('id="entry-character-host"'));
  assert(!html.includes('id="equipment-manage"')); assert(!html.includes('id="reselect"'));
  assert(html.indexOf('<script src="characters.js">') < html.indexOf('data-bin="kiva"'));
  assert(html.indexOf('<script src="settings.js" defer>') < html.indexOf('<script src="hud.js" defer>'));
  assert(html.includes('href="web/settings.css"')); assert(!html.includes("Esc 释放并暂停"));
});
