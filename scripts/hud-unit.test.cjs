const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../web/hud.js"), "utf8");
const ids = [
  "hud-stamina", "hud-stamina-max", "hud-stamina-bar", "hud-skill-ranged", "hud-ranged-cooldown", "hud-ranged-caption", "hud-ranged-skill-icon",
  "hud-persona", "hud-weapon", "hud-health", "hud-health-max", "hud-health-bar", "hud-status", "hud-cooldown",
  "hud-combo", "hud-combo-name", "hud-combo-time", "hud-target", "hud-target-name", "hud-target-status", "hud-target-health", "hud-target-max", "hud-target-bar",
  "hud-skill-start", "hud-skill-attack", "hud-skill-guard", "hud-skill-combo", "hud-skill-caption", "hud-start-icon", "hud-combo-icon",
  "hud-melee", "hud-ranged", "hud-melee-name", "hud-melee-caption", "hud-ranged-name", "hud-melee-icon", "hud-ranged-icon", "hud-pair1", "hud-pair2",
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

test("early player, resources, pair and target snapshots replay safely together", () => {
  const f = fixture();
  f.hud.renderLoadout(2, 1000, true, false, true, true, true);
  f.hud.renderResources(60, 100, 3.24, 1.28, true, 3);
  f.hud.renderTarget("训练靶子", 999_980, 1_000_000);
  player(f, { persona: "哥哥", weapon: "链斧", health: 750, fraction: 0.75, combo: 0.8, context: "链斧离手" });
  f.parse();
  assert.equal(f.at("hud-persona").textContent, "哥哥");
  assert.equal(f.at("hud-health-max").textContent, "1,000");
  assert.equal(f.at("hud-health-bar").value, 0.75);
  assert.equal(f.at("hud-stamina-bar").value, 0.6);
  assert.equal(f.at("hud-cooldown").textContent, "3.2");
  assert.equal(f.at("hud-ranged-cooldown").textContent, "1.3");
  assert.equal(f.at("hud-melee-name").textContent, "谪星·摧山");
  assert.equal(f.at("hud-melee-caption").textContent, "链斧 · 回收中");
  assert.match(f.at("hud-melee").attributes["aria-label"], /谪星·摧山 · 链斧 · 回收中/);
  assert.equal(f.at("hud-target-health").textContent, "999,980");
  assert.equal(f.at("hud-combo-name").textContent, "拉近 · 妹妹三箭");
  assert.equal(f.at("hud-skill-attack").attributes["aria-disabled"], "true");
  assert.match(f.at("hud-skill-guard").attributes["aria-label"], /防御中/);
});

test("pair identity uses explicit enums, with E and Q always separate", () => {
  const f = fixture(true); player(f, { weapon: "可更名的武器" });
  for (const [pair, melee, ranged, icon] of [[1,"伏浪锚","引冰落","hammer"],[2,"谪星·摧山","羿之影","axe"]]) {
    f.hud.renderLoadout(pair, 1000, true, true, true, true, false);
    f.hud.renderResources(100,100,0,0,false,0);
    assert.equal(f.at("hud-melee-name").textContent,melee);
    assert.equal(f.at("hud-melee-caption").textContent,"哥哥 · 左键 / E");
    assert.equal(f.at("hud-melee").attributes.title,`${melee} · ${pair === 1 ? "锤子" : "链斧"}`);
    assert.equal(f.at("hud-ranged-name").textContent,ranged);
    assert.equal(f.at("hud-start-icon").attributes.href,`#hud-icon-${icon}`);
    assert.equal(f.at("hud-skill-start").classList.contains("ready"),true);
    assert.equal(f.at("hud-skill-ranged").classList.contains("ready"),true);
  }
  f.hud.renderResources(80,100,2,0,false,0);
  assert.equal(f.at("hud-skill-start").classList.contains("ready"),false);
  assert.equal(f.at("hud-skill-ranged").classList.contains("ready"),true);
  f.hud.renderResources(80,100,0,3,false,0);
  assert.equal(f.at("hud-skill-start").classList.contains("ready"),true);
  assert.equal(f.at("hud-skill-ranged").classList.contains("ready"),false);
});

test("all numerical values are finite and bounded, with inert strings", () => {
  const f=fixture(true);const hostile='<img src=x onerror="throw 1">';
  for (const invalid of [NaN,Infinity,-1]) {
    f.hud.renderLoadout(1,invalid,true,true,true,true,true);
    f.hud.renderResources(invalid,invalid,invalid,invalid,true,99);
    f.hud.renderTarget(hostile,invalid,invalid,hostile);
    player(f,{persona:hostile,weapon:hostile,health:invalid,fraction:invalid,combo:invalid,context:hostile});
    assert.equal(f.at("hud-health-bar").value,0);
    assert.equal(f.at("hud-stamina-bar").value,0);
    assert.equal(f.at("hud-target").hidden,true);
    assert.equal(f.at("hud-combo").hidden,true);
    for(const id of ["hud-persona","hud-weapon","hud-status"]) {assert.equal(f.at(id).textContent,hostile);assert.equal(f.at(id).innerHTML,undefined);}
  }
  f.hud.renderTarget("敌人",200,120);player(f,{health:100});
  assert.equal(f.at("hud-target-health").textContent,"120");
  assert.equal(f.at("hud-target-bar").value,1);
  f.nodes.clear();assert.doesNotThrow(()=>{player(f);f.hud.renderResources(100,100,0,0,false,0);});
});

test("target clearing, death and expired combo remove stale state without a clock",()=>{
  const f=fixture(true);f.hud.renderLoadout(1,1000,true,true,true,true,true);
  player(f,{combo:0.6});f.hud.renderTarget("靶子",999980,1000000);
  assert.equal(f.at("hud-target").hidden,false);assert.equal(f.at("hud-combo").hidden,false);
  assert.match(f.at("hud-skill-combo").attributes.title,/锤 E 原目标浮空/);
  player(f,{combo:0});assert.equal(f.at("hud-combo").hidden,true);assert.equal(f.at("hud-skill-combo").classList.contains("ready"),false);
  f.hud.renderTarget("",0,0);assert.equal(f.at("hud-target").hidden,true);assert.equal(f.at("hud-target-name").textContent,"");
  f.hud.renderTarget("靶子",50,100);player(f,{health:0,combo:0.5});
  for(const id of ["hud-target","hud-combo"])assert.equal(f.at(id).hidden,true);
  for(const id of ["hud-skill-start","hud-skill-ranged","hud-skill-attack","hud-skill-guard","hud-skill-combo"])assert.equal(f.at(id).attributes["aria-disabled"],"true");
});
