const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../web/hud.js"), "utf8");
// Use production IDs so the fixture cannot keep removed UI alive by accident.
const markup = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");
const ids = [...markup.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);

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
        style: { setProperty(name, value) { this[name] = value; } },
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
  assert.equal(f.at("hud-health-bar").value, 0.75);
  assert.equal(f.at("hud-stamina-bar").value, 0.6);
  assert.equal(f.at("hud-skill-start-time").textContent, "3.2s");
  assert.equal(f.at("hud-skill-ranged-time").textContent, "1.3s");
  assert.equal(f.at("hud-start-label").textContent, "斧");
  assert.equal(f.at("hud-ranged-skill-label").textContent, "弓");
  assert.match(f.at("hud-pair2").attributes["aria-label"], /谪星·摧山.*回收中/);
  assert.equal(f.at("hud-target-bar").attributes["aria-valuetext"], "999,980 / 1,000,000");
  assert.equal(f.at("hud-combo-name").textContent, "拉近 · 妹妹三箭");

});

test("pair identity uses explicit enums, with E and Q always separate", () => {
  const f = fixture(true); player(f, { weapon: "可更名的武器" });
  for (const [pair, melee, ranged] of [[1,"锤","铳"],[2,"斧","弓"]]) {
    f.hud.renderLoadout(pair, 1000, true, true, true, true, false);
    f.hud.renderResources(100,100,0,0,false,0);
    assert.equal(f.at("hud-start-label").textContent, melee);
    assert.equal(f.at("hud-ranged-skill-label").textContent, ranged);
    assert.equal(f.at("hud-combo-label").textContent, "联动");
    assert.equal(f.at(`hud-pair${pair}`).attributes["aria-current"], "true");
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
  for(const id of ["hud-skill-start","hud-skill-ranged","hud-skill-combo","hud-skill-rage"])assert.equal(f.at(id).attributes["aria-disabled"],"true");
});

test("cooldown water follows snapshots; reserved resources have no invented values", () => {
  const f = fixture(true);
  player(f);
  f.hud.renderLoadout(1, 1000, true, true, true, true, false);
  f.hud.renderResources(100, 100, 1.5, 2, false, 0);
  for (const id of ["hud-skill-start", "hud-skill-ranged"]) {
    assert.equal(f.at(id).style["--cooldown-fill"], "50%");
    assert.equal(f.at(id).classList.contains("cooling"), true);
  }
  f.hud.renderResources(100, 100, 0, 0, false, 0);
  assert.equal(f.at("hud-skill-start").classList.contains("cooling"), false);
  assert.equal(f.at("hud-skill-start-time").textContent, "");
  for (const id of ["hud-ranged-energy-bar", "hud-melee-energy-bar", "hud-rage-value-bar"]) {
    assert.equal(f.at(id).value, 0);
  }
  assert.equal(f.at("hud-rage-value").style["--rage-fraction"], "0");
  assert.equal(f.at("hud-skill-rage").attributes["aria-disabled"], "true");
  assert.match(markup, /id="hud-skill-rage"[^>]* disabled/);
});
