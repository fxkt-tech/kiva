const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");

const source = fs.readFileSync(path.join(__dirname, "../web/hud.js"), "utf8");
const ids = ["hud-persona", "hud-weapon", "hud-health", "hud-health-bar", "hud-status", "hud-cooldown", "hud-combo", "hud-combo-time"];

function fixture(ready = false) {
  const nodes = new Map();
  const callbacks = new Map();
  const document = {
    readyState: ready ? "interactive" : "loading",
    getElementById: (id) => nodes.get(id) || null,
    addEventListener: (name, callback) => callbacks.set(name, callback),
  };
  const window = {};
  const parse = () => {
    for (const id of ids) nodes.set(id, { textContent: "", hidden: false });
    document.readyState = "interactive";
    callbacks.get("DOMContentLoaded")?.();
  };
  vm.runInNewContext(source, { window, document });
  if (ready) parse();
  return { nodes, hud: window.kivaHud, parse };
}

test("HUD snapshots before DOM readiness replay the latest state", () => {
  const f = fixture();
  f.hud.render("哥哥", "锤子", 1, 100, 0, 0, "待机");
  f.hud.render("妹妹", "狙击枪", 0.7, 69.5, 1.25, 3.24, "联动");
  f.parse();
  assert.equal(f.nodes.get("hud-persona").textContent, "妹妹");
  assert.equal(f.nodes.get("hud-weapon").textContent, "狙击枪");
  assert.equal(f.nodes.get("hud-health").textContent, "70");
  assert.equal(f.nodes.get("hud-health-bar").value, 0.7);
  assert.equal(f.nodes.get("hud-health-bar").max, 1);
  assert.equal(f.nodes.get("hud-status").textContent, "联动");
  assert.equal(f.nodes.get("hud-cooldown").textContent, "E · 3.2s");
  assert.equal(f.nodes.get("hud-combo-time").textContent, "1.3s");
  assert.equal(f.nodes.get("hud-combo").hidden, false);
});

test("HUD strings remain inert text and absent nodes are harmless", () => {
  const f = fixture(true);
  const hostile = '<img src=x onerror="throw 1">';
  f.nodes.delete("hud-health");
  assert.doesNotThrow(() => f.hud.render(hostile, hostile, 1, 100, 0, 0, hostile));
  for (const id of ["hud-persona", "hud-weapon", "hud-status"]) {
    const node = f.nodes.get(id);
    assert.equal(node.textContent, hostile);
    assert.equal(node.innerHTML, undefined);
  }
});

test("health, cooldown and combo snapshots clamp invalid values", () => {
  const f = fixture(true);
  f.hud.render("哥哥", "锤子", 20, 100.1, 1, -5, "");
  assert.equal(f.nodes.get("hud-health-bar").value, 1);
  assert.equal(f.nodes.get("hud-health").textContent, "101");
  assert.equal(f.nodes.get("hud-cooldown").textContent, "就绪");
  for (const invalid of [Number.NaN, Number.POSITIVE_INFINITY, -1]) {
    f.hud.render(null, null, invalid, invalid, invalid, invalid, null);
    assert.equal(f.nodes.get("hud-health-bar").value, 0);
    assert.equal(f.nodes.get("hud-health").textContent, "0");
    assert.equal(f.nodes.get("hud-persona").textContent, "");
    assert.equal(f.nodes.get("hud-combo").hidden, true);
    assert.equal(f.nodes.get("hud-cooldown").textContent, "就绪");
  }
});

test("expired combo hides its prompt and later valid snapshots show it again", () => {
  const f = fixture(true);
  f.hud.render("妹妹", "火炮", 0.5, 50, 0.6, 2, "跳跃");
  assert.equal(f.nodes.get("hud-combo").hidden, false);
  f.hud.render("哥哥", "斧子", 0.5, 50, 0, 1, "旋转");
  assert.equal(f.nodes.get("hud-combo").hidden, true);
  assert.equal(f.nodes.get("hud-combo-time").textContent, "0.0s");
  f.hud.render("哥哥", "锤子", 0.5, 50, 0.8, 0, "击飞");
  assert.equal(f.nodes.get("hud-combo").hidden, false);
  assert.equal(f.nodes.get("hud-combo-time").textContent, "0.8s");
});
