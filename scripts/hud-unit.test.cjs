const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../web/hud.js"), "utf8");
// Use production IDs so the fixture cannot keep removed UI alive by accident.
const markup = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");
const ids = [...markup.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);

function element(tag = "div") {
  const classes = new Set();
  return {
    tagName: tag, textContent: "", hidden: false, attributes: {}, children: [], className: "",
    style: { setProperty(name, value) { this[name] = value; } },
    setAttribute(name, value) { this.attributes[name] = value; },
    classList: {
      toggle(name, enabled) { if (enabled) classes.add(name); else classes.delete(name); },
      contains: name => classes.has(name),
    },
    replaceChildren(...nodes) { this.children = nodes; },
    append(...nodes) { this.children.push(...nodes); },
  };
}

function fixture(ready = false) {
  const nodes = new Map();
  const callbacks = new Map();
  const document = {
    readyState: ready ? "interactive" : "loading",
    getElementById: id => nodes.get(id) || null,
    addEventListener: (name, callback) => callbacks.set(name, callback),
    createElement: tag => element(tag),
  };
  const window = {};
  const parse = () => {
    for (const id of ids) nodes.set(id, element());
    document.readyState = "interactive";
    callbacks.get("DOMContentLoaded")?.();
  };
  const timer = () => { throw new Error("HUD cannot own a clock"); };
  vm.runInNewContext(source, { window, document, setInterval: timer, setTimeout: timer });
  if (ready) parse();
  return { nodes, hud: window.kivaHud, parse, at: id => nodes.get(id) };
}

function player(f, overrides = {}) {
  const data = { persona: "哥哥", weapon: "伏浪锚", fraction: 1, health: 1000, combo: 0, cooldown: 0, context: "准备就绪", ...overrides };
  f.hud.render(data.persona, data.weapon, data.fraction, data.health, data.combo, data.cooldown, data.context);
}

const lit = (f, id) => f.at(id).children.filter(child => child.classList.contains("on")).length;
const fills = (f, id) => f.at(id).children.map(block => block.children[1].style.width);

test("early snapshots replay safely once the page is parsed", () => {
  const f = fixture();
  f.hud.renderLoadout(2, 1000, true, false, true, true, true);
  f.hud.renderResources(60, 100, 3.24, 1.28, true, 3);
  f.hud.renderTarget("训练靶子", 999_980, 1_000_000);
  player(f, { health: 750, fraction: 0.75, combo: 0.8, context: "链斧离手" });
  f.parse();
  assert.equal(f.at("hud-status").textContent, "链斧离手");
  assert.equal(f.at("hud-health-text").textContent, "750 / 1,000");
  assert.equal(f.at("hud-stamina-fill").style.width, "60%");
  assert.equal(f.at("hud-key-e-time").textContent, "3.2");
  assert.equal(f.at("hud-key-q-time").textContent, "1.3");
  assert.equal(f.at("hud-key-e-glyph").textContent, "斧");
  assert.equal(f.at("hud-key-q-glyph").textContent, "弓");
  assert.equal(f.at("hud-brother-weapon").textContent, "谪星·摧山 · 斧 · 回收中");
  assert.equal(f.at("hud-sister-weapon").textContent, "羿之影 · 弓");
  assert.equal(f.at("hud-target-meter").attributes["aria-valuetext"], "999,980 / 1,000,000");
  assert.equal(f.at("hud-link-text").textContent, "拉近 · 妹妹三箭");
});

test("health is drawn in blocks of 100, with a continuous bar for huge pools", () => {
  const f = fixture(true);
  f.hud.renderLoadout(1, 1000, true, true, true, true, false);
  player(f, { health: 742, fraction: 0.742 });
  assert.equal(f.at("hud-health-blocks").children.length, 10);
  assert.deepEqual(fills(f, "hud-health-blocks"), ["100%", "100%", "100%", "100%", "100%", "100%", "100%", "42%", "0%", "0%"]);
  // The dummy's million health is one block, not ten thousand.
  f.hud.renderTarget("愁铃客 · 演武", 250_000, 1_000_000);
  assert.equal(f.at("hud-target-later").children.length, 1);
  assert.deepEqual(fills(f, "hud-target-later"), ["25%"]);
  assert.equal(f.at("hud-target-first").children.length, 0);
  assert.equal(f.at("hud-target-split").hidden, true);
  assert.equal(f.at("hud-target-phase").textContent, "");
});

test("a Boss's two phases sit side by side and the current one drains first", () => {
  const f = fixture(true);
  player(f);
  f.hud.renderTarget("愁铃客", 434, 700, "游走中", 1, 2);
  assert.equal(f.at("hud-target-phase").textContent, "一阶段");
  assert.equal(f.at("hud-target-split").hidden, false);
  // Phase one: the later phase is whole, the current one has 434 of 700.
  assert.deepEqual(fills(f, "hud-target-later"), Array(7).fill("100%"));
  assert.deepEqual(fills(f, "hud-target-first"), ["100%", "100%", "100%", "100%", "34%", "0%", "0%"]);
  assert.equal(f.at("hud-target-first").classList.contains("spent"), false);
  f.hud.renderTarget("愁铃客", 434, 700, "游走中", 2, 2);
  assert.equal(f.at("hud-target-phase").textContent, "二阶段");
  assert.deepEqual(fills(f, "hud-target-later"), ["100%", "100%", "100%", "100%", "34%", "0%", "0%"]);
  assert.deepEqual(fills(f, "hud-target-first"), Array(7).fill("0%"));
  assert.equal(f.at("hud-target-first").classList.contains("spent"), true);
  assert.equal(f.at("hud-target-health").textContent, "434 / 700");
});

test("the Boss cue shows the telegraph, the combo step and every swing that landed", () => {
  const f = fixture(true);
  player(f);
  f.hud.renderTarget("愁铃客", 600, 700, "伺机而动", 2, 2);
  assert.equal(f.at("hud-boss-cue").hidden, true);
  assert.equal(f.at("hud-boss-state").textContent, "伺机而动");
  f.hud.renderBossCue(1, 0.3, 1.2, 2, 4, 0);
  assert.equal(f.at("hud-boss-cue").hidden, false);
  assert.equal(f.at("hud-boss-state").hidden, true);
  assert.equal(f.at("hud-boss-word").textContent, "蓄势");
  assert.equal(f.at("hud-boss-chain").textContent, "连招 · 第二式");
  assert.equal(f.at("hud-boss-time").textContent, "0.3");
  assert.equal(f.at("hud-boss-count-fill").style.width, "25%");
  assert.equal(f.at("hud-boss-swings").children.length, 4);
  f.hud.renderBossCue(2, 0, 0, 1, 4, 0b0101);
  assert.equal(f.at("hud-boss-word").textContent, "出招");
  assert.equal(f.at("hud-boss-chain").textContent, "");
  assert.equal(f.at("hud-boss-cue").classList.contains("striking"), true);
  assert.deepEqual(f.at("hud-boss-swings").children.map(swing => swing.classList.contains("hit")), [true, false, true, false]);
  f.hud.renderBossCue(0, 0, 0, 0, 0, 0);
  assert.equal(f.at("hud-boss-cue").hidden, true);
});

test("the lock reticle follows the target on screen and becomes an edge arrow off screen", () => {
  const f = fixture(true);
  player(f);
  f.hud.renderTarget("愁铃客", 600, 700, "", 1, 2);
  f.hud.renderLock(1, 0.56, 0.35, 7.42, 0);
  assert.equal(f.at("hud-reticle").hidden, false);
  assert.equal(f.at("hud-reticle").style.left, "56%");
  assert.equal(f.at("hud-reticle").style.top, "35%");
  assert.equal(f.at("hud-reticle-distance").textContent, "7.4m");
  assert.equal(f.at("hud-offscreen").hidden, true);
  f.hud.renderLock(3, 0, 0, 4, -1);
  assert.equal(f.at("hud-reticle").hidden, true);
  assert.equal(f.at("hud-offscreen").hidden, false);
  assert.equal(f.at("hud-offscreen-label").textContent, "身后");
  assert.equal(f.at("hud-offscreen").classList.contains("right"), false);
  f.hud.renderLock(2, 0, 0, 4, 1);
  assert.equal(f.at("hud-offscreen-label").textContent, "侧方");
  assert.equal(f.at("hud-offscreen").classList.contains("right"), true);
  // No target: neither shows, whatever the last lock said.
  f.hud.renderTarget("", 0, 0);
  assert.equal(f.at("hud-offscreen").hidden, true);
  assert.equal(f.at("hud-reticle").hidden, true);
});

test("keys follow availability, cooldown water and empowerment", () => {
  const f = fixture(true);
  player(f);
  for (const [pair, melee, ranged] of [[1, "锤", "铳"], [2, "斧", "弓"]]) {
    f.hud.renderLoadout(pair, 1000, true, true, true, true, false);
    f.hud.renderResources(100, 100, 0, 0, false, 0);
    assert.equal(f.at("hud-key-e-glyph").textContent, melee);
    assert.equal(f.at("hud-key-q-glyph").textContent, ranged);
    assert.equal(f.at("hud-key-e").classList.contains("ready"), true);
    assert.equal(f.at("hud-key-q").classList.contains("ready"), true);
  }
  f.hud.renderResources(80, 100, 1.5, 2, false, 0);
  for (const id of ["hud-key-e", "hud-key-q"]) {
    assert.equal(f.at(id).style["--cooldown-fill"], "50%");
    assert.equal(f.at(id).classList.contains("cooling"), true);
    assert.equal(f.at(id).classList.contains("ready"), false);
  }
  f.hud.renderResources(80, 100, 0, 0, false, 0);
  f.hud.renderEnergy(10, 10, 5, 10, 8, 20, false, false);
  assert.equal(f.at("hud-key-e").classList.contains("empowered"), true);
  assert.equal(f.at("hud-key-q").classList.contains("empowered"), false);
  assert.match(f.at("hud-key-e").attributes["aria-label"], /强化/);
  f.hud.renderEnergy(0, 10, 0, 10, 12.4, 20, true, false);
  for (const id of ["hud-key-e", "hud-key-q"]) assert.equal(f.at(id).classList.contains("empowered"), true);
});

test("sun, moon, rage and the current pair's durability are drawn as segments", () => {
  const f = fixture(true);
  player(f);
  f.hud.renderLoadout(1, 1000, true, true, true, true, false);
  f.hud.renderEnergy(7, 10, 10, 10, 13, 20, false, false);
  f.hud.renderDurability(new Float32Array([31, 50]), 50);
  assert.equal(lit(f, "hud-sun"), 7);
  assert.equal(f.at("hud-sun-value").textContent, "7");
  assert.equal(lit(f, "hud-moon"), 10);
  assert.equal(f.at("hud-moon-row").classList.contains("full"), true);
  assert.equal(f.at("hud-moon-tag").hidden, false);
  assert.equal(f.at("hud-rage").children.length, 20);
  assert.equal(lit(f, "hud-rage"), 13);
  assert.equal(f.at("hud-rage-row").attributes["aria-disabled"], "true");
  // Durability: 20 segments of 2.5 for the current pair only; low at 30% or less.
  assert.equal(f.at("hud-durability").children.length, 20);
  assert.equal(lit(f, "hud-durability"), 12);
  assert.equal(f.at("hud-durability-value").textContent, "31");
  assert.equal(f.at("hud-durability-row").classList.contains("low"), false);
  f.hud.renderDurability([12, 50], 50);
  assert.equal(f.at("hud-durability-row").classList.contains("low"), true);
  f.hud.renderLoadout(2, 1000, true, true, true, true, false);
  assert.equal(f.at("hud-durability-value").textContent, "50");
  assert.equal(lit(f, "hud-durability"), 20);

  f.hud.renderEnergy(0, 10, 0, 10, 20, 20, false, true);
  assert.equal(f.at("hud-rage-row").classList.contains("ready"), true);
  assert.match(f.at("hud-rage-row").attributes["aria-label"], /可用/);
  f.hud.renderEnergy(0, 10, 0, 10, 12.4, 20, true, false);
  assert.equal(f.at("hud-rage-row").classList.contains("enraged"), true);
  assert.match(f.at("hud-rage-row").attributes["aria-label"], /暴怒中 · 剩余 13s/);
});

test("the link waits with its condition, then shows the finisher and its window", () => {
  const f = fixture(true);
  f.hud.renderLoadout(1, 1000, true, true, true, true, false);
  player(f);
  assert.equal(f.at("hud-link").classList.contains("wait"), true);
  assert.equal(f.at("hud-link-text").textContent, "锤 E 击飞原目标后可用");
  assert.equal(f.at("hud-link").attributes["aria-disabled"], "true");
  f.hud.renderLoadout(1, 1000, true, true, true, true, true);
  player(f, { combo: 3 });
  player(f, { combo: 2.1 });
  assert.equal(f.at("hud-link").classList.contains("ready"), true);
  assert.equal(f.at("hud-link-text").textContent, "妹妹三枪");
  assert.equal(f.at("hud-link-time").textContent, "2.1");
  assert.equal(f.at("hud-link").style["--left"], "70%");
  player(f, { combo: 0 });
  assert.equal(f.at("hud-link").classList.contains("ready"), false);
  assert.equal(f.at("hud-link-time").textContent, "");
});

test("all numerical values are bounded, strings stay inert, and death clears the target", () => {
  const f = fixture(true);
  const hostile = '<img src=x onerror="throw 1">';
  for (const invalid of [NaN, Infinity, -1]) {
    f.hud.renderLoadout(1, invalid, true, true, true, true, true);
    f.hud.renderResources(invalid, invalid, invalid, invalid, true, 99);
    f.hud.renderTarget(hostile, invalid, invalid, hostile, invalid, invalid);
    f.hud.renderBossCue(invalid, invalid, invalid, invalid, invalid, invalid);
    f.hud.renderLock(invalid, invalid, invalid, invalid, invalid);
    player(f, { health: invalid, fraction: invalid, combo: invalid, context: hostile });
    assert.equal(f.at("hud-health-blocks").children.length, 0);
    assert.equal(f.at("hud-stamina-fill").style.width, "0%");
    assert.equal(f.at("hud-target").hidden, true);
    assert.equal(f.at("hud-status").textContent, hostile);
    assert.equal(f.at("hud-status").innerHTML, undefined);
  }
  f.hud.renderLoadout(1, 1000, true, true, true, true, true);
  f.hud.renderTarget("敌人", 200, 120);
  player(f, { health: 100 });
  assert.equal(f.at("hud-target-meter").attributes["aria-valuenow"], "1.000");
  player(f, { health: 0, combo: 0.5 });
  assert.equal(f.at("hud-target").hidden, true);
  for (const id of ["hud-key-e", "hud-key-q", "hud-link", "hud-rage-row"]) assert.equal(f.at(id).attributes["aria-disabled"], "true");
  f.nodes.clear();
  assert.doesNotThrow(() => { player(f); f.hud.renderResources(100, 100, 0, 0, false, 0); });
});
