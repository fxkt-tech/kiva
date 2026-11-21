const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = fs.readFileSync(path.join(__dirname, "../web/render_style.js"), "utf8");

function fixture({ ready = false, viewer = false } = {}) {
  const nodes = new Map();
  const events = new Map();
  let active = false;
  let writes = 0;
  const document = {
    readyState: ready ? "interactive" : "loading",
    getElementById: id => nodes.get(id) || null,
    addEventListener: (type, callback) => events.set(type, callback),
  };
  const parse = () => {
    let text = "";
    nodes.set("hud-render-style", {
      get textContent() { return text; },
      set textContent(value) { text = value; writes += 1; },
    });
    document.readyState = "interactive";
    events.get("DOMContentLoaded")?.();
  };
  if (ready) parse();
  const window = { kivaInput: { isActive: () => active }, kivaPage: { isViewer: () => viewer } };
  const timer = () => { throw new Error("Render style selection cannot own a clock"); };
  vm.runInNewContext(source, { window, document, setInterval: timer, setTimeout: timer });
  const key = (extra = {}) => {
    const event = { code: "F8", isTrusted: true, repeat: false, preventDefault() { this.prevented = true; }, ...extra };
    events.get("keydown")(event);
    return event;
  };
  return {
    parse, key,
    status: () => nodes.get("hud-render-style").textContent,
    writes: () => writes,
    bridge: window.kivaRenderStyle,
    events,
    activate: () => { active = true; },
    pause: () => { active = false; },
  };
}

test("the viewer installs no bridge and no key handler", () => {
  const f = fixture({ ready: true, viewer: true });
  assert.equal(f.bridge, undefined);
  assert.equal(f.events.has("keydown"), false);
});

test("early Rust state replays when the DOM becomes ready", () => {
  const f = fixture();
  f.bridge.render(2, 2);
  f.parse();
  assert.equal(f.status(), "渲染 · 岩彩 · F8 切换");
  assert.equal(f.bridge.takeStyle(), 0);
});

test("F8 needs active, trusted, non-repeat input and is consumed once", () => {
  const f = fixture({ ready: true });
  f.bridge.render(1, 2);
  assert.equal(f.key().prevented, undefined, "inactive input stays with the browser");
  assert.equal(f.bridge.takeStyle(), 0);
  f.activate();
  assert.equal(f.key({ isTrusted: false }).prevented, undefined);
  assert.equal(f.key({ repeat: true }).prevented, undefined);
  assert.equal(f.key({ code: "F4" }).prevented, undefined, "F4 stays the hit overlay");
  assert.equal(f.bridge.takeStyle(), 0);
  assert.equal(f.key().prevented, true);
  assert.equal(f.key().prevented, true);
  assert.equal(f.bridge.takeStyle(), 1, "presses before Rust samples collapse into one request");
  assert.equal(f.bridge.takeStyle(), 0);
  f.pause();
  assert.equal(f.key().prevented, undefined);
  assert.equal(f.bridge.takeStyle(), 0);
});

test("the HUD follows Rust, names single-style levels and ignores unknown values", () => {
  const f = fixture({ ready: true });
  f.bridge.render(1, 1);
  assert.equal(f.status(), "渲染 · 写实 · 本关仅此一种");
  f.bridge.render(2, 2);
  assert.equal(f.status(), "渲染 · 岩彩 · F8 切换");
  f.bridge.render(9, 0);
  assert.equal(f.status(), "渲染 · 岩彩 · F8 切换");
});

test("per-frame renders touch the DOM only on change", () => {
  const f = fixture({ ready: true });
  f.bridge.render(1, 2);
  const settled = f.writes();
  for (let i = 0; i < 10; i += 1) f.bridge.render(1, 2);
  assert.equal(f.writes(), settled);
  f.bridge.render(2, 2);
  assert.equal(f.writes(), settled + 1);
});
