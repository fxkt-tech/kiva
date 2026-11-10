const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../web/hud.js"), "utf8");
const ids = [
  "hud-persona", "hud-weapon", "hud-health", "hud-health-max", "hud-health-bar", "hud-status", "hud-cooldown",
  "hud-combo", "hud-combo-name", "hud-combo-time", "hud-target", "hud-target-name", "hud-target-status", "hud-target-health", "hud-target-max", "hud-target-bar",
  "hud-skill-start", "hud-skill-attack", "hud-skill-guard", "hud-skill-combo", "hud-skill-caption", "hud-start-icon", "hud-combo-icon",
  "hud-melee", "hud-ranged", "hud-melee-name", "hud-ranged-name", "hud-melee-icon", "hud-ranged-icon", "hud-pair1", "hud-pair2",
];

function fixture(ready = false) {
  const nodes = new Map();
  const callbacks = new Map();
  const document = {
    readyState: ready ? "interactive" : "loading",
    getElementById: id => nodes.get(id) || null,
    addEventListener: (name, callback) => callbacks.set(name, callback),
  };
  const window = {};
  const parse = () => {
    for (const id of ids) {
      const classes = new Set();
      nodes.set(id, {
        textContent: "", hidden: false, attributes: {},
        setAttribute(name, value) { this.attributes[name] = value; },
        classList: {
          toggle(name, enabled) { if (enabled) classes.add(name); else classes.delete(name); },
          contains: name => classes.has(name),
        },
      });
    }
    document.readyState = "interactive";
    callbacks.get("DOMContentLoaded")?.();
  };
  const timer = () => { throw new Error("HUD cannot own a clock"); };
  vm.runInNewContext(source, { window, document, setInterval: timer, setTimeout: timer });
  if (ready) parse();
  return { nodes, hud: window.kivaHud, parse, at: id => nodes.get(id) };
}

function player(f, overrides = {}) {
  const data = { persona: "哥哥", weapon: "愁铃长刀", fraction: 1, health: 100, combo: 0, cooldown: 0, context: "准备就绪", ...overrides };
  f.hud.render(data.persona, data.weapon, data.fraction, data.health, data.combo, data.cooldown, data.context);
}

test("melee appearance label survives early replay and gameplay loadout switches", () => {
  const f = fixture();
  f.hud.renderMelee("烛泪");
  player(f, { weapon: "烛泪" });
  f.parse();
  assert.equal(f.at("hud-melee-name").textContent, "烛泪");
  assert.equal(f.at("hud-weapon").textContent, "烛泪");
  for (const set of [1, 2]) {
    f.hud.renderLoadout(set, 1, 100, true, false, true, false);
    player(f, { weapon: set === 1 ? "狙击枪" : "火炮" });
    assert.equal(f.at("hud-melee-name").textContent, "烛泪");
    assert.equal(f.at("hud-ranged-name").textContent, set === 1 ? "狙击枪" : "火炮");
    assert.equal(f.at("hud-weapon").textContent, set === 1 ? "狙击枪" : "火炮");
  }
  for (const invalid of [undefined, null, NaN, 2, ""]) f.hud.renderMelee(invalid);
  assert.equal(f.at("hud-melee-name").textContent, "烛泪");
  f.hud.renderMelee("愁铃长刀");
  assert.equal(f.at("hud-melee-name").textContent, "愁铃长刀");
});

test("player, loadout and selected target snapshots before DOM readiness replay together", () => {
  const f = fixture();
  player(f);
  f.hud.renderLoadout(2, 1, 150, true, false, true, true);
  f.hud.renderTarget("训练靶子", 999_980, 1_000_000);
  player(f, { persona: "妹妹", weapon: "火炮", fraction: 0.7, health: 104.5, combo: 0.8, cooldown: 3.24, context: "跃进" });
  f.parse();
  assert.equal(f.at("hud-persona").textContent, "妹妹");
  assert.equal(f.at("hud-weapon").textContent, "火炮");
  assert.equal(f.at("hud-health").textContent, "105");
  assert.equal(f.at("hud-health-max").textContent, "150");
  assert.equal(f.at("hud-health-bar").value, 0.7);
  assert.equal(f.at("hud-health-bar").max, 1);
  assert.equal(f.at("hud-target-health").textContent, "999,980");
  assert.equal(f.at("hud-target-bar").value, 0.99998);
  assert.equal(f.at("hud-target").hidden, false);
  assert.equal(f.at("hud-cooldown").textContent, "3.2");
  assert.equal(f.at("hud-cooldown").hidden, false);
  assert.equal(f.at("hud-combo-name").textContent, "空中旋刀");
  assert.equal(f.at("hud-combo-time").textContent, "0.8s");
  assert.equal(f.at("hud-status").hidden, true);
});

test("health values remain finite and progress values stay within their limits", () => {
  const f = fixture(true);
  player(f, { fraction: 20, health: 100.1, cooldown: -5 });
  assert.equal(f.at("hud-health-bar").value, 1);
  assert.equal(f.at("hud-health").textContent, "101");
  assert.equal(f.at("hud-cooldown").hidden, true);
  f.hud.renderTarget("敌人", 200, 120);
  assert.equal(f.at("hud-target-bar").value, 1);
  assert.equal(f.at("hud-target-health").textContent, "120");
  for (const invalid of [NaN, Infinity, -1]) {
    f.hud.renderLoadout(1, 0, invalid, false, false, false, false);
    f.hud.renderTarget("敌人", invalid, invalid);
    player(f, { persona: null, weapon: null, fraction: invalid, health: invalid, combo: invalid, cooldown: invalid, context: null });
    assert.equal(f.at("hud-health-bar").value, 0);
    assert.equal(f.at("hud-health").textContent, "0");
    assert.equal(f.at("hud-health-max").textContent, "0");
    assert.equal(f.at("hud-persona").textContent, "");
    assert.equal(f.at("hud-target").hidden, true);
    assert.equal(f.at("hud-combo").hidden, true);
  }
});

test("target damage updates exact large values and clearing the target removes old health", () => {
  const f = fixture(true);
  player(f);
  f.hud.renderTarget("训练靶子", 1_000_000, 1_000_000);
  f.hud.renderTarget("训练靶子", 999_980, 1_000_000);
  assert.equal(f.at("hud-target-health").textContent, "999,980");
  for (const [name, current, max] of [["", 0, 0], ["敌人", 0, 120], ["敌人", 100, 0]]) {
    f.hud.renderTarget(name, current, max);
    assert.equal(f.at("hud-target").hidden, true);
    assert.equal(f.at("hud-target-name").textContent, "");
    assert.equal(f.at("hud-target-bar").value, 0);
  }
  f.hud.renderTarget("敌人", 24, 120);
  assert.equal(f.at("hud-target").hidden, false);
  assert.equal(f.at("hud-target-max").textContent, "120");
  assert.equal(f.at("hud-target-bar").value, 0.2);
});

test("weapon-set and persona identity use explicit enum fields and readiness snapshots", () => {
  const f = fixture(true);
  player(f, { weapon: "可更名的武器" });
  f.hud.renderLoadout(1, 0, 100, true, true, true, false);
  assert.equal(f.at("hud-melee").classList.contains("active"), true);
  assert.equal(f.at("hud-start-icon").attributes.href, "#hud-icon-blade");
  assert.equal(f.at("hud-combo-icon").attributes.href, "#hud-icon-sniper");
  assert.equal(f.at("hud-skill-start").attributes["aria-disabled"], "false");
  f.hud.renderLoadout(2, 0, 100, true, false, true, false);
  assert.equal(f.at("hud-melee-name").textContent, "愁铃长刀");
  assert.equal(f.at("hud-ranged-name").textContent, "火炮");
  assert.equal(f.at("hud-start-icon").attributes.href, "#hud-icon-cannon");
  assert.equal(f.at("hud-combo-icon").attributes.href, "#hud-icon-blade");
  assert.match(f.at("hud-skill-start").attributes.title, /切换远程/);
  assert.equal(f.at("hud-pair2").attributes["aria-current"], "true");
  f.hud.renderLoadout(2, 1, 100, false, true, false, false);
  assert.equal(f.at("hud-melee").classList.contains("active"), false);
  assert.equal(f.at("hud-ranged").classList.contains("active"), true);
  assert.equal(f.at("hud-skill-start").classList.contains("ready"), true);
  assert.equal(f.at("hud-skill-attack").classList.contains("ready"), false);
  assert.equal(f.at("hud-skill-guard").classList.contains("ready"), false);
});

test("cooldown and combo consume snapshots only and expire without stale highlighting", () => {
  const f = fixture(true);
  f.hud.renderLoadout(1, 0, 100, false, false, true, true);
  player(f, { combo: 0.6, cooldown: 2.5 });
  assert.equal(f.at("hud-skill-start").classList.contains("cooling"), true);
  assert.equal(f.at("hud-skill-combo").classList.contains("ready"), true);
  assert.equal(f.at("hud-combo").hidden, false);
  f.hud.renderLoadout(1, 0, 100, false, false, true, false);
  assert.equal(f.at("hud-skill-combo").classList.contains("ready"), false);
  assert.equal(f.at("hud-combo-time").textContent, "0.6s");
  player(f, { cooldown: 1 });
  assert.equal(f.at("hud-combo").hidden, true);
  assert.equal(f.at("hud-status").hidden, false);
  assert.equal(f.at("hud-combo-time").textContent, "0.0s");
  f.hud.renderLoadout(1, 0, 100, true, true, true, false);
  player(f);
  assert.equal(f.at("hud-cooldown").hidden, true);
  assert.equal(f.at("hud-skill-start").classList.contains("cooling"), false);
  assert.equal(f.at("hud-skill-start").classList.contains("ready"), true);
});

test("defeat hides target and combo while disabling stale available skills", () => {
  const f = fixture(true);
  f.hud.renderLoadout(1, 0, 100, true, true, true, true);
  f.hud.renderTarget("敌人", 120, 120);
  player(f, { health: 0, fraction: 0, combo: 0.5 });
  for (const id of ["hud-target", "hud-combo"]) assert.equal(f.at(id).hidden, true);
  for (const id of ["hud-skill-start", "hud-skill-attack", "hud-skill-guard", "hud-skill-combo"]) {
    assert.equal(f.at(id).classList.contains("ready"), false);
    assert.equal(f.at(id).attributes["aria-disabled"], "true");
  }
  f.hud.renderTarget("", 0, 0);
  player(f);
  assert.equal(f.at("hud-target").hidden, true);
});

test("HUD strings remain inert text and absent elements are harmless", () => {
  const f = fixture(true);
  const hostile = '<img src=x onerror="throw 1">';
  f.nodes.delete("hud-health");
  player(f, { persona: hostile, weapon: hostile, context: hostile });
  f.hud.renderTarget(hostile, 120, 120, hostile);
  for (const id of ["hud-persona", "hud-weapon", "hud-status", "hud-target-name", "hud-target-status"]) {
    assert.equal(f.at(id).textContent, hostile);
    assert.equal(f.at(id).innerHTML, undefined);
  }
  f.nodes.clear();
  assert.doesNotThrow(() => {
    player(f); f.hud.renderLoadout(1, 0, 100, true, true, true, false); f.hud.renderTarget("敌人", 10, 120);
  });
});
