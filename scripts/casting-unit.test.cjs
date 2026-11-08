// Execute the real preference adapter without a browser, WASM or game rules.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const test = require("node:test");
const vm = require("node:vm");
const source = fs.readFileSync(path.join(__dirname, "../web/casting.js"), "utf8");

function fixture(saved, blocked = false) {
  const root = { dataset: {} };
  const select = { value: "", listeners: new Map(),
    addEventListener(type, listener) { this.listeners.set(type, listener); },
  };
  const callbacks = new Map();
  const storage = new Map(saved === undefined ? [] : [["kiva-cast-mode", saved]]);
  const context = {
    document: { documentElement: root,
      addEventListener(type, listener) { callbacks.set(type, listener); },
      getElementById(id) { assert.equal(id, "cast-mode"); return select; },
    },
    localStorage: {
      getItem(key) { if (blocked) throw new Error("storage unavailable"); return storage.get(key) ?? null; },
      setItem(key, value) { if (blocked) throw new Error("storage unavailable"); storage.set(key, value); },
    },
  };
  vm.runInNewContext(source, context, { filename: "web/casting.js" });
  return { root, select, storage,
    mount() { callbacks.get("DOMContentLoaded")(); },
    change(value) { select.value = value; select.listeners.get("change")(); },
  };
}

test("a valid saved preference is available before DOM controls mount", () => {
  for (const mode of ["normal", "quick"]) {
    const f = fixture(mode);
    assert.equal(f.root.dataset.castMode, mode);
    f.mount();
    assert.equal(f.select.value, mode);
  }
});

test("changing mode updates the input bridge and persists across reloads", () => {
  const f = fixture();
  assert.equal(f.root.dataset.castMode, "quick");
  f.mount();
  f.change("normal");
  assert.equal(f.root.dataset.castMode, "normal");
  const reloaded = fixture(f.storage.get("kiva-cast-mode"));
  assert.equal(reloaded.root.dataset.castMode, "normal");
  f.change("quick");
  assert.equal(f.root.dataset.castMode, "quick");
});

test("unknown preferences are rejected, and unavailable storage still permits switching", () => {
  const f = fixture("unexpected");
  f.mount();
  assert.equal(f.select.value, "quick");
  f.change("unexpected");
  assert.equal(f.select.value, "quick");
  assert.equal(f.root.dataset.castMode, "quick");
  const blocked = fixture("normal", true);
  blocked.mount();
  blocked.change("normal");
  assert.equal(blocked.root.dataset.castMode, "normal");
});
