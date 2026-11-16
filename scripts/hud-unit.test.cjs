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

// Rust writes these (client/hud.rs, `the_page_contract_fixtures_match_what_rust_sends`), so a
// renamed or removed field fails here instead of silently drawing nothing.
const fixtures = path.join(__dirname, "../tests/fixtures/hud");
const read = name => JSON.parse(fs.readFileSync(path.join(fixtures, `${name}.json`), "utf8"));
const LOADOUTS = read("loadouts");
const FRAMES = { idle: read("idle"), telegraph: read("boss-telegraph"), behind: read("boss-behind") };

const isObject = value => value !== null && typeof value === "object" && !Array.isArray(value);
// Every key any fixture uses at each path, so a test can only override a field Rust sends.
function shapeOf(values) {
  const shape = {};
  for (const value of values) {
    for (const [key, child] of Object.entries(value)) {
      shape[key] ??= [];
      if (isObject(child)) shape[key].push(child);
    }
  }
  return Object.fromEntries(Object.entries(shape).map(([key, children]) => [key, children.length ? shapeOf(children) : null]));
}
const SHAPE = shapeOf(Object.values(FRAMES));
function merge(base, patch, shape, at = "frame") {
  const result = isObject(base) ? { ...base } : {};
  for (const [key, value] of Object.entries(patch)) {
    assert.ok(shape && key in shape, `${at}.${key} is not a field Rust sends`);
    result[key] = isObject(value) && shape[key] ? merge(result[key], value, shape[key], `${at}.${key}`) : value;
  }
  return result;
}

function fixture(ready = false, defined = true) {
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
  if (defined) window.kivaHud.defineLoadouts(JSON.stringify(LOADOUTS));
  if (ready) parse();
  const hud = window.kivaHud;
  // One update: a fixture frame with some fields overridden.
  const send = (patch = {}, base = FRAMES.idle) => hud.frame(JSON.stringify(merge(base, patch, SHAPE)));
  return { nodes, hud, parse, send, at: id => nodes.get(id) };
}

const lit = (f, id) => f.at(id).children.filter(child => child.classList.contains("on")).length;
const fills = (f, id) => f.at(id).children.map(block => block.children[1].style.width);

test("every contract fixture paints, and early frames replay once the page is parsed", () => {
  const f = fixture();
  f.send({}, FRAMES.telegraph);
  f.parse();
  assert.equal(f.at("hud-status").textContent, "联动中 · 妹妹协同攻击");
  assert.equal(f.at("hud-health-text").textContent, "1,000 / 1,000");
  assert.equal(f.at("hud-key-e-time").textContent, "1.5");
  assert.equal(f.at("hud-key-e-glyph").textContent, "锤");
  assert.equal(f.at("hud-sister-weapon").textContent, "引冰落 · 铳");
  assert.equal(f.at("hud-target-name").textContent, "愁铃客");
  assert.equal(f.at("hud-boss-word").textContent, "蓄势");
  assert.equal(f.at("hud-reticle").style.left, "56.3%");
  assert.equal(f.at("hud-reticle-distance").textContent, "7.5m");
  // The link drains against the window's opening length, not one the page remembers.
  assert.equal(f.at("hud-link").style["--left"], "66.7%");
  assert.equal(f.at("hud-link-text").textContent, "妹妹三枪");

  f.send({}, FRAMES.behind);
  assert.equal(f.at("hud-key-e-glyph").textContent, "斧");
  assert.equal(f.at("hud-key-q-glyph").textContent, "弓");
  assert.equal(f.at("hud-brother-weapon").textContent, "谪星·摧山 · 斧 · 回收中");
  assert.equal(f.at("hud-sister-weapon").textContent, "羿之影 · 弓");
  assert.equal(f.at("hud-target-phase").textContent, "二阶段");
  assert.equal(f.at("hud-boss-word").textContent, "出招");
  assert.equal(f.at("hud-offscreen-label").textContent, "身后");
  assert.equal(f.at("hud-reticle").hidden, true);

  f.send();
  assert.equal(f.at("hud-target").hidden, true);
  assert.equal(f.at("hud-link-text").textContent, "锤 E 击飞原目标后可用");
});

test("health is drawn in blocks of 100, with a continuous bar for huge pools", () => {
  const f = fixture(true);
  f.send({ health: { current: 742 } });
  assert.equal(f.at("hud-health-blocks").children.length, 10);
  assert.deepEqual(fills(f, "hud-health-blocks"), ["100%", "100%", "100%", "100%", "100%", "100%", "100%", "42%", "0%", "0%"]);
  assert.equal(f.at("hud-health-blocks").attributes["aria-valuenow"], "0.742");
  // The dummy's million health is one block, not ten thousand.
  f.send({ target: { name: "愁铃客 · 演武", current: 250_000, maximum: 1_000_000, status: "锁定目标", phase: 1, phases: 1 } });
  assert.equal(f.at("hud-target-later").children.length, 1);
  assert.deepEqual(fills(f, "hud-target-later"), ["25%"]);
  assert.equal(f.at("hud-target-first").children.length, 0);
  assert.equal(f.at("hud-target-split").hidden, true);
  assert.equal(f.at("hud-target-phase").textContent, "");
  assert.equal(f.at("hud-target-meter").attributes["aria-valuetext"], "250,000 / 1,000,000");
});

test("a Boss's two phases sit side by side and the current one drains first", () => {
  const f = fixture(true);
  f.send({ cue: null }, FRAMES.telegraph);
  assert.equal(f.at("hud-target-phase").textContent, "一阶段");
  assert.equal(f.at("hud-target-split").hidden, false);
  // Phase one: the later phase is whole, the current one has 434 of 700.
  assert.deepEqual(fills(f, "hud-target-later"), Array(7).fill("100%"));
  assert.deepEqual(fills(f, "hud-target-first"), ["100%", "100%", "100%", "100%", "34%", "0%", "0%"]);
  assert.equal(f.at("hud-target-first").classList.contains("spent"), false);
  f.send({ cue: null, target: { phase: 2 } }, FRAMES.telegraph);
  assert.equal(f.at("hud-target-phase").textContent, "二阶段");
  assert.deepEqual(fills(f, "hud-target-later"), ["100%", "100%", "100%", "100%", "34%", "0%", "0%"]);
  assert.deepEqual(fills(f, "hud-target-first"), Array(7).fill("0%"));
  assert.equal(f.at("hud-target-first").classList.contains("spent"), true);
  assert.equal(f.at("hud-target-health").textContent, "434 / 700");
});

test("the Boss cue shows the telegraph, the combo step and every swing that landed", () => {
  const f = fixture(true);
  f.send({ cue: null, target: { status: "伺机而动" } }, FRAMES.telegraph);
  assert.equal(f.at("hud-boss-cue").hidden, true);
  assert.equal(f.at("hud-boss-state").textContent, "伺机而动");
  f.send({}, FRAMES.telegraph);
  assert.equal(f.at("hud-boss-cue").hidden, false);
  assert.equal(f.at("hud-boss-state").hidden, true);
  assert.equal(f.at("hud-boss-word").textContent, "蓄势");
  assert.equal(f.at("hud-boss-chain").textContent, "连招 · 第二式");
  assert.equal(f.at("hud-boss-time").textContent, "0.3");
  assert.equal(f.at("hud-boss-count-fill").style.width, "25%");
  assert.equal(f.at("hud-boss-swings").children.length, 4);
  f.send({}, FRAMES.behind);
  assert.equal(f.at("hud-boss-word").textContent, "出招");
  assert.equal(f.at("hud-boss-chain").textContent, "");
  assert.equal(f.at("hud-boss-time").textContent, "");
  assert.equal(f.at("hud-boss-cue").classList.contains("striking"), true);
  assert.deepEqual(f.at("hud-boss-swings").children.map(swing => swing.classList.contains("hit")), [true, false, true, false]);
  f.send({ cue: null }, FRAMES.behind);
  assert.equal(f.at("hud-boss-cue").hidden, true);
});

test("the lock reticle follows the target on screen and becomes an edge arrow off screen", () => {
  const f = fixture(true);
  f.send({}, FRAMES.telegraph);
  assert.equal(f.at("hud-reticle").hidden, false);
  assert.equal(f.at("hud-reticle").style.top, "37.5%");
  assert.equal(f.at("hud-offscreen").hidden, true);
  f.send({}, FRAMES.behind);
  assert.equal(f.at("hud-reticle").hidden, true);
  assert.equal(f.at("hud-offscreen").hidden, false);
  assert.equal(f.at("hud-offscreen").classList.contains("right"), false);
  f.send({ lock: { kind: "aside", right: true, distance: 4 } }, FRAMES.behind);
  assert.equal(f.at("hud-offscreen-label").textContent, "侧方");
  assert.equal(f.at("hud-offscreen").classList.contains("right"), true);
  for (const lock of [null, { kind: "unknown", distance: 1 }]) {
    f.send({ lock }, FRAMES.behind);
    assert.equal(f.at("hud-offscreen").hidden, true);
    assert.equal(f.at("hud-reticle").hidden, true);
  }
  // No target: neither shows, whatever the lock says.
  f.send({ target: null }, FRAMES.telegraph);
  assert.equal(f.at("hud-reticle").hidden, true);
});

test("keys follow availability, cooldown water and empowerment", () => {
  const f = fixture(true);
  for (const [set, melee, ranged] of [[1, "锤", "铳"], [2, "斧", "弓"]]) {
    f.send({ loadout: { set } });
    assert.equal(f.at("hud-key-e-glyph").textContent, melee);
    assert.equal(f.at("hud-key-q-glyph").textContent, ranged);
    assert.equal(f.at("hud-key-e").classList.contains("ready"), true);
    assert.equal(f.at("hud-key-q").classList.contains("ready"), true);
  }
  // The water level is measured against the total each skill actually started.
  f.send({ melee: { left: 1.5, total: 3 }, ranged: { left: 3, total: 6 } });
  for (const id of ["hud-key-e", "hud-key-q"]) {
    assert.equal(f.at(id).style["--cooldown-fill"], "50%");
    assert.equal(f.at(id).classList.contains("cooling"), true);
    assert.equal(f.at(id).classList.contains("ready"), false);
  }
  f.send({ loadout: { meleeReady: false } });
  assert.equal(f.at("hud-key-e").classList.contains("ready"), false);
  assert.match(f.at("hud-key-e").attributes["aria-label"], /当前条件不可用/);
  f.send({ energy: { sun: 10, moon: 5 } });
  assert.equal(f.at("hud-key-e").classList.contains("empowered"), true);
  assert.equal(f.at("hud-key-q").classList.contains("empowered"), false);
  assert.match(f.at("hud-key-e").attributes["aria-label"], /强化/);
  f.send({}, FRAMES.behind);
  for (const id of ["hud-key-e", "hud-key-q"]) assert.equal(f.at(id).classList.contains("empowered"), true);
});

test("sun, moon, rage and the current pair's durability are drawn as segments", () => {
  const f = fixture(true);
  f.send({ energy: { sun: 7, moon: 10, rage: 13 }, durability: { values: [31, 50] } });
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
  f.send({ durability: { values: [12, 50] } });
  assert.equal(f.at("hud-durability-row").classList.contains("low"), true);
  f.send({ loadout: { set: 2 }, durability: { values: [12, 50] } });
  assert.equal(f.at("hud-durability-value").textContent, "50");
  assert.equal(lit(f, "hud-durability"), 20);

  f.send({ energy: { rage: 20, rageReady: true } });
  assert.equal(f.at("hud-rage-row").classList.contains("ready"), true);
  assert.match(f.at("hud-rage-row").attributes["aria-label"], /可用/);
  f.send({}, FRAMES.behind);
  assert.equal(f.at("hud-rage-row").classList.contains("enraged"), true);
  assert.match(f.at("hud-rage-row").attributes["aria-label"], /暴怒中 · 剩余 13s/);
});

test("the link waits with its condition, then shows the finisher and its window", () => {
  const f = fixture(true);
  f.send();
  assert.equal(f.at("hud-link").classList.contains("wait"), true);
  assert.equal(f.at("hud-link-text").textContent, "锤 E 击飞原目标后可用");
  assert.equal(f.at("hud-link").attributes["aria-disabled"], "true");
  f.send({ loadout: { comboReady: true }, combo: { left: 2.1, total: 3 } });
  assert.equal(f.at("hud-link").classList.contains("ready"), true);
  assert.equal(f.at("hud-link-text").textContent, "妹妹三枪");
  assert.equal(f.at("hud-link-time").textContent, "2.1");
  assert.equal(f.at("hud-link").style["--left"], "70%");
  assert.match(f.at("hud-link").attributes["aria-label"], /剩余 2.1s/);
  f.send({ loadout: { comboReady: true }, combo: { left: 0, total: 0 } });
  assert.equal(f.at("hud-link").classList.contains("ready"), false);
  assert.equal(f.at("hud-link-time").textContent, "");
});

test("all numerical values are bounded, strings stay inert, and death clears the target", () => {
  const f = fixture(true);
  const hostile = '<img src=x onerror="throw 1">';
  // Rust writes non-finite numbers as null; strings and wrong types arrive from nowhere else.
  for (const invalid of [null, -1, "7", true]) {
    const meter = { current: invalid, maximum: invalid };
    f.send({
      context: hostile, health: meter, stamina: meter,
      loadout: { set: 1, meleeReady: invalid, rangedReady: invalid, comboReady: invalid },
      melee: { left: invalid, total: invalid }, combo: { left: invalid, total: invalid },
      energy: { sun: invalid, sunMaximum: invalid, rage: invalid },
      durability: { values: [invalid], maximum: invalid },
      target: { name: hostile, current: invalid, maximum: invalid, status: hostile, phase: invalid, phases: invalid },
      cue: { striking: invalid, left: invalid, total: invalid, chain: invalid, swings: invalid, resolved: invalid },
      lock: { kind: "onScreen", x: invalid, y: invalid, distance: invalid },
    }, FRAMES.telegraph);
    assert.equal(f.at("hud-health-blocks").children.length, 0);
    assert.equal(f.at("hud-stamina-fill").style.width, "0%");
    assert.equal(f.at("hud-target").hidden, true);
    assert.equal(f.at("hud-status").textContent, hostile);
    assert.equal(f.at("hud-status").innerHTML, undefined);
  }
  // A target can never read as above full.
  f.send({ target: { current: 900 } }, FRAMES.telegraph);
  assert.equal(f.at("hud-target-meter").attributes["aria-valuenow"], "1.000");
  f.send({ health: { current: 0 } }, FRAMES.telegraph);
  assert.equal(f.at("hud-target").hidden, true);
  for (const id of ["hud-key-e", "hud-key-q", "hud-link", "hud-rage-row"]) assert.equal(f.at(id).attributes["aria-disabled"], "true");

  // Unreadable frames are ignored and the last good one kept.
  for (const broken of ["{not json", "null", "3", ""]) f.hud.frame(broken);
  assert.equal(f.at("hud-status").textContent, "联动中 · 妹妹协同攻击");
  f.nodes.clear();
  assert.doesNotThrow(() => f.send());
});

test("pair wording comes only from defineLoadouts; nothing is drawn for an undefined pair", () => {
  const f = fixture(true, false);
  assert.doesNotThrow(() => f.send({ context: "未定义" }));
  assert.equal(f.at("hud-status").textContent, "");
  assert.equal(f.at("hud-key-e-glyph").textContent, "");

  // Malformed definitions, and rows missing a field, are ignored.
  f.hud.defineLoadouts("{not json");
  f.hud.defineLoadouts(JSON.stringify({ 1: { melee: "残缺" }, x: LOADOUTS[1] }));
  assert.equal(f.at("hud-status").textContent, "");

  // A third pair is just another row; no slot number is hard-coded in the page.
  const rows = { ...LOADOUTS, 3: { ...LOADOUTS[1], melee: "新近战", meleeGlyph: "新", comboCondition: "新条件" } };
  f.hud.defineLoadouts(JSON.stringify(rows));
  f.send({ loadout: { set: 3 } });
  assert.equal(f.at("hud-key-e-glyph").textContent, "新");
  assert.equal(f.at("hud-brother-weapon").textContent, "新近战 · 新");
  assert.equal(f.at("hud-link-text").textContent, "新条件");
  assert.equal(f.at("hud-status").textContent, "准备就绪");
  f.send({ loadout: { set: 9 }, context: "未知组" });
  assert.equal(f.at("hud-status").textContent, "准备就绪");
});

test("a test can only override fields Rust actually sends", () => {
  const f = fixture(true);
  assert.throws(() => f.send({ healthFraction: 1 }), /frame.healthFraction is not a field Rust sends/);
  assert.throws(() => f.send({ loadout: { attackReady: true } }), /frame.loadout.attackReady/);
});
