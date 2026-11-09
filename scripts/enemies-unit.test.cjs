const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = fs.readFileSync(path.join(__dirname, "../web/enemies.js"), "utf8");

function fixture(ready = false) {
  const nodes = new Map();
  const events = new Map();
  let active = false;
  const document = {
    readyState: ready ? "interactive" : "loading",
    getElementById: (id) => nodes.get(id),
    addEventListener: (type, callback) => events.set(type, callback),
  };
  const parse = () => {
    for (const id of ["enemy-mode-target", "enemy-mode-combat", "enemy-mode-hint", "hud-enemy-mode"]) {
      nodes.set(id, {
        textContent: "",
        attributes: {},
        setAttribute(name, value) { this.attributes[name] = value; },
        addEventListener(type, callback) { this[type] = callback; },
      });
    }
    events.get("DOMContentLoaded")?.();
  };
  if (ready) parse();
  const window = { kivaInput: { isActive: () => active } };
  vm.runInNewContext(source, { window, document });
  const key = (extra = {}) => {
    const event = { code: "F2", isTrusted: true, repeat: false, preventDefault() { this.prevented = true; }, ...extra };
    events.get("keydown")(event);
    return event;
  };
  return { nodes, parse, key, modes: window.kivaEnemies, activate: () => { active = true; } };
}

test("mode requests survive loading and are consumed once even while paused", () => {
  const f = fixture();
  f.parse();
  assert.equal(f.nodes.get("enemy-mode-target").attributes["aria-pressed"], "true");
  assert.equal(f.nodes.get("hud-enemy-mode").textContent, "靶子模式 · F2 切换");
  f.nodes.get("enemy-mode-combat").click();
  f.modes.render(1); // An old Rust snapshot cannot discard the pending request.
  assert.equal(f.nodes.get("enemy-mode-combat").attributes["aria-pressed"], "true");
  assert.equal(f.modes.takeMode(), 2);
  assert.equal(f.modes.takeMode(), 0);
  f.modes.render(2);
  assert.equal(f.nodes.get("hud-enemy-mode").textContent, "实战模式 · F2 切换");
  f.nodes.get("enemy-mode-target").click();
  assert.equal(f.modes.takeMode(), 1);
  f.modes.render(1);
  assert.match(f.nodes.get("enemy-mode-hint").textContent, /100 万生命/);
});

test("F2 switches only during active play and ignores repeat and untrusted input", () => {
  const f = fixture(true);
  assert.equal(f.key().prevented, undefined);
  assert.equal(f.modes.takeMode(), 0);
  f.activate();
  assert.equal(f.key().prevented, true);
  assert.equal(f.modes.takeMode(), 2);
  f.modes.render(2);
  for (const ignored of [{ repeat: true }, { isTrusted: false }, { code: "F3" }]) f.key(ignored);
  assert.equal(f.modes.takeMode(), 0);
  f.key();
  f.key(); // Two full presses before sampling cancel out.
  assert.equal(f.modes.takeMode(), 2);
});

test("mode snapshots ignore invalid values and tolerate absent DOM nodes", () => {
  const f = fixture(true);
  f.modes.render(1);
  for (const invalid of [0, 3, NaN, null, "<script>"]) f.modes.render(invalid);
  assert.equal(f.nodes.get("hud-enemy-mode").textContent, "靶子模式 · F2 切换");
  f.nodes.clear();
  assert.doesNotThrow(() => f.modes.render(2));
});
