// No browser/GPU: execute the actual early bootstrap without any DOM objects.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = fs.readFileSync(path.join(__dirname, "../web/startup.js"), "utf8");
const equipmentSource = fs.readFileSync(path.join(__dirname, "../web/equipment.js"), "utf8");

function bootstrap(gpu = {}) {
  const window = new EventTarget();
  const context = vm.createContext({ window, navigator: { gpu } });
  vm.runInContext(source, context);
  // Both synchronous head scripts must be safe before any DOM exists.
  vm.runInContext(equipmentSource, context);
  return window;
}

for (const error of ["TrunkApplicationError", "error", "unhandledrejection"]) {
  test(`${error} before the HUD is retained and replayed`, () => {
    const window = bootstrap();
    window.dispatchEvent(new Event(error));
    const observed = [];
    window.kivaStartup.subscribe((state) => observed.push(state));
    assert.deepEqual(observed, ["failed"]);
    assert.equal(window.__kivaStartup, "failed");
  });
}

test("readiness before HUD subscription is retained; old errors cannot override it", () => {
  const window = bootstrap();
  window.dispatchEvent(new Event("kiva-ready"));
  const observed = [];
  window.kivaStartup.subscribe((state) => observed.push(state));
  for (const error of ["TrunkApplicationError", "error", "unhandledrejection"]) window.dispatchEvent(new Event(error));
  assert.deepEqual(observed, ["ready"]);
  assert.equal(window.__kivaStartup, "ready");
});

test("actual engine readiness supersedes an earlier nonfatal startup notification", () => {
  const window = bootstrap();
  const observed = [];
  window.kivaStartup.subscribe((state) => observed.push(state));
  window.dispatchEvent(new Event("error"));
  window.dispatchEvent(new Event("kiva-ready"));
  window.dispatchEvent(new Event("unhandledrejection"));
  assert.deepEqual(observed, ["loading", "failed", "ready"]);
  assert.equal(window.__kivaStartup, "ready");
});

test("unsupported WebGPU is reported even before DOM/HUD initialization", () => {
  assert.equal(bootstrap(null).__kivaStartup, "failed");
});

test("the synchronous observer is installed before Trunk can evaluate its module", () => {
  const html = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");
  const observer = html.indexOf('<script src="startup.js"></script>');
  assert(observer > 0);
  assert(observer < html.indexOf('data-trunk rel="rust"'), "observer must precede the generated module");
  assert(observer < html.indexOf('<script src="hud.js" defer>'));
  const equipment = html.indexOf('<script src="equipment.js"></script>');
  assert(equipment > observer && equipment < html.indexOf('data-trunk rel="rust"'),
    "the static catalog observer must precede the generated module too");
});
