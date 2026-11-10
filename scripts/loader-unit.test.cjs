const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = (name) => fs.readFileSync(path.join(__dirname, "../web", name), "utf8");

const emptyWasm = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]);

function fixture(fetch) {
  const window = new EventTarget();
  const snapshots = [];
  const context = vm.createContext({ window, navigator: { gpu: {} }, fetch, Response, ReadableStream, CustomEvent });
  vm.runInContext(source("startup.js"), context);
  vm.runInContext(source("loader.js"), context);
  window.kivaStartup.subscribe((state, detail) => snapshots.push({ state, ...detail }));
  return { window, snapshots, load: window.kivaLoader.load };
}

test("WASM streaming uses actual response bytes and keeps download completion separate from ready", async () => {
  const fetched = [];
  const f = fixture(async (source) => {
    fetched.push(source);
    return new Response(new ReadableStream({
      start(controller) {
        controller.enqueue(emptyWasm.slice(0, 4));
        controller.enqueue(emptyWasm.slice(4));
        controller.close();
      },
    }), { headers: { "content-type": "application/wasm", "content-length": "8" } });
  });
  const wasm = await f.load(async ({ module_or_path }) => WebAssembly.instantiateStreaming(module_or_path), "/game.wasm");
  assert(wasm.instance instanceof WebAssembly.Instance);
  assert.deepEqual(fetched, ["/game.wasm"]);
  assert.deepEqual(f.snapshots.filter((detail) => detail.phase === "download").map(({ loaded, total }) => [loaded, total]),
    [[0, 0], [0, 8], [4, 8], [8, 8]]);
  assert.equal(f.snapshots.at(-1).phase, "initialize");
  assert.equal(f.window.__kivaStartup, "loading");
});

test("missing, invalid and compressed response lengths stay indeterminate while retaining actual bytes", async () => {
  for (const headers of [{}, { "content-length": "garbage" }, { "content-length": "9007199254740992" },
    { "content-length": "4", "content-encoding": "gzip" }, { "content-length": "4", "content-encoding": "br" }]) {
    const f = fixture(async () => new Response(emptyWasm, { headers }));
    await f.load(async ({ module_or_path }) => module_or_path.arrayBuffer(), "/game.wasm");
    const downloaded = f.snapshots.filter((detail) => detail.phase === "download");
    assert(downloaded.every(({ total }) => total === 0));
    assert.equal(downloaded.at(-1).loaded, emptyWasm.length);
    assert.equal(f.window.__kivaStartup, "loading");
  }
});

test("CORS responses cannot treat a hidden encoding header as proof of an unencoded length", async () => {
  const f = fixture(async () => {
    const response = new Response(new ReadableStream({
      start(controller) {
        controller.enqueue(emptyWasm.slice(0, 2));
        controller.enqueue(emptyWasm.slice(2));
        controller.close();
      },
    }), { headers: { "content-length": "4" } });
    // A CORS-filtered fetch can expose Content-Length but hide Content-Encoding.
    Object.defineProperty(response, "type", { value: "cors" });
    return response;
  });
  await f.load(async ({ module_or_path }) => module_or_path.arrayBuffer(), "/game.wasm");
  const downloaded = f.snapshots.filter(({ phase }) => phase === "download");
  assert(downloaded.every(({ total }) => total === 0));
  assert.equal(downloaded.at(-1).loaded, emptyWasm.length);
});

test("a cached immediate response still waits for engine and render readiness", async () => {
  const f = fixture(async () => new Response(emptyWasm, { headers: { "content-length": "8" } }));
  await f.load(async ({ module_or_path }) => {
    await module_or_path.arrayBuffer();
    f.window.kivaStartup.progress("render", 0, 0);
    return "engine started";
  }, "/game.wasm");
  assert.equal(f.snapshots.at(-1).phase, "render");
  assert.equal(f.window.__kivaStartup, "loading");
  f.window.dispatchEvent(new Event("kiva-ready"));
  f.window.kivaStartup.progress("download", 8, 8);
  assert.equal(f.window.__kivaStartup, "ready");
});

test("HTTP, fetch, stream and initialization failures reject and replay a retryable failure", async () => {
  const brokenStream = () => {
    let read = false;
    return new ReadableStream({
      pull(controller) {
        if (read) controller.error(new Error("connection lost"));
        else { read = true; controller.enqueue(emptyWasm.slice(0, 4)); }
      },
    });
  };
  for (const fetch of [
    async () => new Response("not found", { status: 404 }),
    async () => { throw new Error("offline"); },
    async () => new Response(brokenStream(), { headers: { "content-length": "8" } }),
  ]) {
    const f = fixture(fetch);
    await assert.rejects(f.load(async ({ module_or_path }) => module_or_path.arrayBuffer(), "/game.wasm"));
    assert.equal(f.window.__kivaStartup, "failed");
    assert.match(f.snapshots.at(-1).reason, /重新加载/);
    f.window.dispatchEvent(new Event("kiva-ready"));
    assert.equal(f.window.__kivaStartup, "ready");
  }
  const f = fixture(async () => new Response(emptyWasm));
  await assert.rejects(f.load(async () => { throw new Error("compile failed"); }, "/game.wasm"));
  assert.equal(f.window.__kivaStartup, "failed");
});

test("loader errors arriving after real ready cannot replace a working entry state", async () => {
  const f = fixture(async () => new Response(emptyWasm));
  await assert.rejects(f.load(async ({ module_or_path }) => {
    await module_or_path.arrayBuffer();
    f.window.dispatchEvent(new Event("kiva-ready"));
    throw new Error("late error");
  }, "/game.wasm"));
  assert.equal(f.window.__kivaStartup, "ready");
});

test("initialization failure cancels an unfinished download and preserves its original error", async () => {
  for (const cancellationFails of [false, true]) {
    const failure = new Error("initialization rejected before reading the response");
    let cancellationReason;
    const f = fixture(async () => new Response(new ReadableStream({
      pull(controller) { controller.enqueue(emptyWasm.slice(0, 4)); },
      cancel(reason) {
        cancellationReason = reason;
        if (cancellationFails) return Promise.reject(new Error("cancel failed"));
      },
    })));
    let reportedError;
    f.window.addEventListener("TrunkApplicationError", (event) => { reportedError = event.detail.reason; });
    await assert.rejects(f.load(async () => { throw failure; }, "/game.wasm"), (error) => error === failure);
    assert.equal(cancellationReason, failure);
    assert.equal(reportedError, failure);
    assert.equal(f.window.__kivaStartup, "failed");
    assert.equal(f.snapshots.some(({ phase }) => phase === "initialize"), false);
  }
});

test("truncated known-size responses fail instead of announcing initialization", async () => {
  const f = fixture(async () => new Response(emptyWasm, {
    headers: { "content-type": "application/wasm", "content-length": "16" },
  }));
  let errors = 0;
  f.window.addEventListener("TrunkApplicationError", () => { errors++; });
  await assert.rejects(f.load(async ({ module_or_path }) => WebAssembly.instantiateStreaming(module_or_path), "/game.wasm"));
  assert.equal(f.window.__kivaStartup, "failed");
  assert.equal(f.snapshots.some(({ phase }) => phase === "initialize"), false);
  assert.match(f.snapshots.at(-1).reason, /下载中断/);
  assert.equal(errors, 1);
});

test("canceling the tracked response cancels its original reader", async () => {
  let cancelled;
  const f = fixture(async () => new Response(new ReadableStream({ cancel(reason) { cancelled = reason; } })));
  await f.load(async ({ module_or_path }) => { await module_or_path.body.cancel("stopped"); }, "/game.wasm");
  assert.equal(cancelled, "stopped");
});

test("Trunk uses the dedicated loader while retaining its bindings and started event", async () => {
  const config = fs.readFileSync(path.join(__dirname, "../Trunk.toml"), "utf8");
  const html = fs.readFileSync(path.join(__dirname, "../index.html"), "utf8");
  assert.match(config, /kivaLoader\.load\(init, '\{base\}\{wasm\}'\)/);
  assert.match(config, /window\.wasmBindings = bindings/);
  assert.match(config, /TrunkApplicationStarted/);
  assert(html.indexOf('<script src="loader.js"></script>') < html.indexOf('data-trunk rel="rust"'));
  const pattern = config.match(/pattern_script = """([\s\S]*?)"""/)[1];
  const emitted = pattern.replaceAll("{base}", "/nested/")
    .replaceAll("{js}", "game-hashed.js").replaceAll("{wasm}", "game-hashed_bg.wasm")
    .replace(/<\/?script[^>]*>/g, "").replace(/^import .*;$/m, "");
  const wasm = {};
  const bindings = {};
  let started;
  const window = { kivaLoader: { async load(_init, url) {
    assert.equal(url, "/nested/game-hashed_bg.wasm");
    return wasm;
  } } };
  await vm.runInNewContext(`(async () => { ${emitted} })()`, {
    window, init() {}, bindings, CustomEvent,
    dispatchEvent(event) { started = event; },
  });
  assert.equal(window.wasmBindings, bindings);
  assert.equal(started.type, "TrunkApplicationStarted");
  assert.equal(started.detail.wasm, wasm);
});
