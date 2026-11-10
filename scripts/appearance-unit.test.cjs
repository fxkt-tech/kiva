const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = fs.readFileSync(path.join(__dirname, "../web/appearance.js"), "utf8");

function fixture(ready = false) {
  const nodes = new Map();
  const events = new Map();
  let active = false;
  const document = {
    readyState: ready ? "interactive" : "loading",
    getElementById: id => nodes.get(id) || null,
    addEventListener: (type, callback) => events.set(type, callback),
  };
  const parse = () => {
    for (const id of ["appearance-choulingke", "appearance-lihuowang", "appearance-hint", "hud-appearance"]) {
      nodes.set(id, {
        textContent: "",
        attributes: {},
        setAttribute(name, value) { this.attributes[name] = value; },
        addEventListener(type, callback) { this[type] = callback; },
      });
    }
    document.readyState = "interactive";
    events.get("DOMContentLoaded")?.();
  };
  if (ready) parse();
  const window = { kivaInput: { isActive: () => active } };
  const timer = () => { throw new Error("Appearance selection cannot own a clock"); };
  vm.runInNewContext(source, { window, document, setInterval: timer, setTimeout: timer });
  const key = (extra = {}) => {
    const event = { code: "F3", isTrusted: true, repeat: false, preventDefault() { this.prevented = true; }, ...extra };
    events.get("keydown")(event);
    return event;
  };
  return {
    nodes, parse, key,
    at: id => nodes.get(id),
    bridge: window.kivaAppearance,
    activate: () => { active = true; },
    pause: () => { active = false; },
    removeInput: () => { delete window.kivaInput; },
  };
}

test("early Rust selections replay when the DOM becomes ready", () => {
  const f = fixture();
  f.bridge.render(2);
  f.parse();
  assert.equal(f.at("appearance-choulingke").attributes["aria-pressed"], "false");
  assert.equal(f.at("appearance-lihuowang").attributes["aria-pressed"], "true");
  assert.match(f.at("appearance-hint").textContent, /身体与服饰/);
  assert.equal(f.at("hud-appearance").textContent, "外观 · 李火旺 · F3 切换");
  assert.equal(f.bridge.takeAppearance(), 0);
});

test("loading and paused button selections survive old snapshots until one consumption", () => {
  const f = fixture();
  f.parse();
  assert.equal(f.at("appearance-lihuowang").attributes["aria-pressed"], "true");
  f.at("appearance-lihuowang").click();
  f.bridge.render(1);
  assert.equal(f.at("appearance-lihuowang").attributes["aria-pressed"], "true");
  assert.match(f.at("appearance-hint").textContent, /李火旺/);
  assert.equal(f.at("hud-appearance").textContent, "外观 · 愁铃客 · F3 切换");
  assert.equal(f.bridge.takeAppearance(), 2);
  assert.equal(f.bridge.takeAppearance(), 0);
  f.bridge.render(2);
  assert.equal(f.at("hud-appearance").textContent, "外观 · 李火旺 · F3 切换");
  f.at("appearance-choulingke").click();
  assert.equal(f.bridge.takeAppearance(), 1);
  f.bridge.render(1);
  assert.match(f.at("appearance-hint").textContent, /身体与服饰/);
});

test("F3 requires active trusted non-repeat input and does not consume F2", () => {
  const f = fixture(true);
  assert.equal(f.key().prevented, undefined);
  assert.equal(f.bridge.takeAppearance(), 0);
  f.activate();
  for (const ignored of [{ repeat: true }, { isTrusted: false }, { code: "F2" }]) {
    assert.equal(f.key(ignored).prevented, undefined);
  }
  assert.equal(f.bridge.takeAppearance(), 0);
  assert.equal(f.key().prevented, true);
  assert.equal(f.bridge.takeAppearance(), 1);
  f.bridge.render(1);
  f.pause();
  assert.equal(f.key().prevented, undefined);
  assert.equal(f.bridge.takeAppearance(), 0);
  f.activate();
  f.removeInput();
  assert.equal(f.key().prevented, undefined);
  assert.equal(f.bridge.takeAppearance(), 0);
});

test("successive presses toggle from the pending choice before Rust samples", () => {
  const f = fixture(true);
  f.activate();
  f.key();
  f.key();
  assert.equal(f.bridge.takeAppearance(), 2);
  f.bridge.render(1);
  f.at("appearance-lihuowang").click();
  f.key();
  assert.equal(f.bridge.takeAppearance(), 1);
  f.bridge.render(1);
  f.key();
  f.key();
  f.key();
  assert.equal(f.bridge.takeAppearance(), 2);
  assert.equal(f.bridge.takeAppearance(), 0);
});

test("invalid snapshots cannot change or inject the displayed appearance", () => {
  const f = fixture(true);
  f.bridge.render(2);
  for (const invalid of [0, 3, -1, NaN, Infinity, null, "1", "<script>"]) f.bridge.render(invalid);
  assert.equal(f.at("appearance-lihuowang").attributes["aria-pressed"], "true");
  assert.equal(f.at("hud-appearance").textContent, "外观 · 李火旺 · F3 切换");
  f.nodes.clear();
  assert.doesNotThrow(() => f.bridge.render(1));
  assert.equal(Object.isFrozen(f.bridge), true);
});

test("brother defaults to Li Huowang before any Rust snapshot or request", () => {
  const f = fixture(true);
  assert.equal(f.at("appearance-lihuowang").attributes["aria-pressed"], "true");
  assert.equal(f.at("appearance-choulingke").attributes["aria-pressed"], "false");
  assert.equal(f.at("hud-appearance").textContent, "外观 · 李火旺 · F3 切换");
  assert.equal(f.bridge.takeAppearance(), 0);
});
