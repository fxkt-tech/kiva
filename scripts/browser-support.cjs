const { chromium } = require("playwright");
const fs = require("node:fs/promises");
const { createReadStream } = require("node:fs");
const http = require("node:http");
const path = require("node:path");
// CDP coordinates have the same CSS gain as the browser normal pointer.
const mousePositions = new WeakMap();

async function serveBuild(directory) {
  const root = path.resolve(directory);
  const types = {
    ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm",
    ".json": "application/json", ".glb": "model/gltf-binary", ".png": "image/png",
    ".jpg": "image/jpeg", ".ogg": "audio/ogg", ".mp3": "audio/mpeg",
  };
  const server = http.createServer(async (request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    const file = path.resolve(root, `.${pathname === "/" ? "/index.html" : pathname}`);
    if (!file.startsWith(root + path.sep)) { response.writeHead(403); response.end(); return; }
    try {
      const stat = await fs.stat(file);
      response.writeHead(200, { "Content-Type": types[path.extname(file)] || "application/octet-stream", "Content-Length": stat.size });
      createReadStream(file).pipe(response);
    } catch {
      response.writeHead(404); response.end();
    }
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  return { url: `http://127.0.0.1:${server.address().port}`, close: () => new Promise((resolve) => server.close(resolve)) };
}

async function launchBrowser() {
  const args = ["--no-sandbox", "--force-device-scale-factor=1", "--no-proxy-server"];
  if (process.env.KIVA_BROWSER_SOFTWARE === "1") {
    args.push("--enable-unsafe-webgpu", "--use-angle=swiftshader",
      "--enable-features=Vulkan", "--use-vulkan=swiftshader",
      "--disable-vulkan-surface", "--disable-gpu-watchdog");
  }
  return chromium.launch({
    headless: process.env.KIVA_BROWSER_HEADED !== "1",
    channel: process.env.KIVA_BROWSER_CHANNEL || "chromium",
    args,
  });
}

async function openSelection(page, url = process.env.KIVA_TEST_URL || "http://127.0.0.1:3221") {
  await page.goto(url, { timeout: 120000 });
  await page.locator("#enter").waitFor({ state: "visible", timeout: 120000 });
  await page.bringToFront();
  await page.waitForFunction(() => window.__kivaState?.session?.phase === "Selecting"
    && window.__kivaState.session.selection?.can_start, undefined, { timeout: 120000 });
}

async function selectCharacter(page, characterId) {
  const previousAck = await page.evaluate(() => window.__kivaState.session.selection.selection_ack.request_id);
  await page.locator(`[data-character-id="${characterId}"]`).click();
  await page.waitForFunction(({ characterId, previousAck }) => {
    const selection = window.__kivaState?.session?.selection;
    return selection?.selected?.id === characterId && selection.selection_ack.accepted
      && selection.selection_ack.request_id !== previousAck && selection.can_start
      && !document.getElementById("enter").disabled;
  }, { characterId, previousAck });
}

async function enterGame(page, url = process.env.KIVA_TEST_URL || "http://127.0.0.1:3221", characterId = 1) {
  await openSelection(page, url);
  await selectCharacter(page, characterId);
  await resumeGame(page);
}

async function resumeGame(page, activation = "mouse") {
  return captureWithButton(page, "#enter", activation, false);
}

async function restartGame(page, activation = "mouse") {
  return captureWithButton(page, "#restart", activation, true);
}

async function captureWithButton(page, selector, activation, replacement) {
  // Pointer Lock checks the native browser view as well as document focus;
  // Playwright's focus emulation alone does not make that view foreground.
  await page.bringToFront();
  const previousPlayer = await page.evaluate(() => window.__kivaState.player_id);
  const button = page.locator(selector);
  await page.waitForFunction((selector) => !document.querySelector(selector).disabled, selector);
  const box = await button.boundingBox();
  if (activation === "keyboard") {
    await button.focus();
    await page.keyboard.press("Enter");
  } else {
    await button.click();
  }
  mousePositions.set(page, { x: box.x + box.width / 2, y: box.y + box.height / 2 });
  await page.waitForFunction(() => document.pointerLockElement?.id === "game"
    && window.__kivaState?.session?.phase === "Active" && window.__kivaState.player_id
    && !window.__kivaState.paused && !document.getElementById("game-cursor").hidden);
  const currentPlayer = await page.evaluate(() => window.__kivaState.player_id);
  if (replacement && (!previousPlayer || currentPlayer === previousPlayer)) {
    throw new Error("Restarting must bind a new player instance");
  }
  if (!replacement && previousPlayer !== null && previousPlayer !== currentPlayer) {
    throw new Error("Resuming must preserve the existing player instance");
  }
  const center = await page.evaluate(() => {
    const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    const canvas = document.getElementById("game");
    const expected = [canvas.clientWidth / 2, canvas.clientHeight / 2];
    if (Math.abs(matrix.m41 - expected[0]) > 1 || Math.abs(matrix.m42 - expected[1]) > 1) {
      throw new Error(`Capture must center the game cursor before test calibration: ${matrix.m41},${matrix.m42}`);
    }
    return expected;
  });
  // Headed Chromium resets its synthetic movement anchor on capture; headless
  // keeps the entry-click anchor. Normalize it, then restore the visible center.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.evaluate(() => new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(resolve));
  }));
  await movePointer(page, ...center);
}

async function movePointer(page, x, y) {
  const cursor = await page.evaluate(() => {
    if (document.pointerLockElement?.id !== "game") return null;
    const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    return { x: matrix.m41, y: matrix.m42 };
  });
  if (!cursor) {
    await page.mouse.move(x, y);
    mousePositions.set(page, { x, y });
    return;
  }
  const last = mousePositions.get(page);
  assertMousePosition(last);
  const next = { x: last.x + x - cursor.x, y: last.y + y - cursor.y };
  await page.mouse.move(next.x, next.y);
  mousePositions.set(page, next);
  await page.waitForFunction(([x, y]) => {
    const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    return Math.abs(matrix.m41 - x) < 2 && Math.abs(matrix.m42 - y) < 2;
  }, [x, y]);
}

function assertMousePosition(position) {
  if (!position) throw new Error("enterGame/resumeGame must initialize the captured pointer");
}

async function moveRelative(page, dx, dy) {
  const before = await page.evaluate(() => {
    const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    return { x: matrix.m41, y: matrix.m42, width: innerWidth, height: innerHeight };
  });
  const last = mousePositions.get(page);
  assertMousePosition(last);
  const next = { x: last.x + dx, y: last.y + dy };
  await page.mouse.move(next.x, next.y);
  mousePositions.set(page, next);
  const expected = [Math.max(0, Math.min(before.width - 1, before.x + dx)), Math.max(0, Math.min(before.height - 1, before.y + dy))];
  await page.waitForFunction(([x, y]) => {
    const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    return Math.abs(matrix.m41 - x) < 2 && Math.abs(matrix.m42 - y) < 2;
  }, expected);
}

async function clickPointer(page, button = "left", delay = 90) {
  if (!delay) {
    const position = mousePositions.get(page);
    assertMousePosition(position);
    // Playwright batches an immediate click's native press/release events.
    await page.mouse.click(position.x, position.y, { button });
    return;
  }
  await page.mouse.down({ button });
  await page.waitForTimeout(delay);
  await page.mouse.up({ button });
}

async function clickAndMovePointer(page, x, y) {
  const cursor = await page.evaluate(() => {
    if (document.pointerLockElement?.id !== "game") throw new Error("Captured pointer required");
    const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    return { x: matrix.m41, y: matrix.m42 };
  });
  const last = mousePositions.get(page);
  assertMousePosition(last);
  const next = { x: last.x + x - cursor.x, y: last.y + y - cursor.y };
  // Plan the follow-up motion before committing the click. Short animations
  // must not spend their input window on host-side cursor queries.
  await clickPointer(page, "left", 0);
  await page.mouse.move(next.x, next.y);
  mousePositions.set(page, next);
  // The caller observes motion together with its animation condition, avoiding
  // a separate feedback round trip in the short interaction window.
}

async function clickUi(page, selector, delay = 90) {
  if (!await page.evaluate(() => document.pointerLockElement?.id === "game")) {
    await page.locator(selector).click({ delay });
    return;
  }
  const box = await page.locator(selector).boundingBox();
  await movePointer(page, box.x + box.width / 2, box.y + box.height / 2);
  await clickPointer(page, "left", delay);
}

async function worldPoint(page, position) {
  return page.evaluate(([x, z, y = 0]) => {
    const { view_projection: m, viewport: [width, height] } = window.__kivaState;
    const w = m[3] * x + m[7] * y + m[11] * z + m[15];
    return [(m[0] * x + m[4] * y + m[8] * z + m[12]) / w * width / 2 + width / 2,
      height / 2 - (m[1] * x + m[5] * y + m[9] * z + m[13]) / w * height / 2];
  }, position);
}

async function clickWorld(page, position, button = "left") {
  const [x, y] = await worldPoint(page, position);
  await movePointer(page, x, y);
  await clickPointer(page, button);
}

async function reportFailure(page) {
  const state = await page.evaluate(() => ({
    startup: window.__kivaStartup,
    paused: window.__kivaState?.paused,
    tick: window.__kivaState?.tick,
    aim: window.__kivaState?.aim,
    selected: window.__kivaState?.selected,
    player_id: window.__kivaState?.player_id,
    match: window.__kivaState?.match,
    map: window.__kivaState?.map,
    session: window.__kivaState?.session,
    camera_center: window.__kivaState?.camera_center,
    camera_locked: window.__kivaState?.camera_locked,
    fixture_clock: window.__laneDiagnostics,
    date_now: Date.now(),
    performance_now: performance.now(),
    captured: document.pointerLockElement?.id || null,
    domPaused: document.documentElement.dataset.paused,
    pointerError: document.getElementById("pointer-error")?.textContent,
    selectionError: document.getElementById("selection-error")?.textContent,
    units: window.__kivaState?.units,
    abilities: window.__kivaState?.abilities,
    notice: window.__kivaState?.notice,
    focused: document.hasFocus(),
    activeElement: document.activeElement?.id,
  })).catch(() => "page unavailable");
  console.error("Browser state:", JSON.stringify(state, null, 2));
}

module.exports = { launchBrowser, openSelection, selectCharacter, enterGame, resumeGame, restartGame, movePointer, moveRelative, clickPointer, clickAndMovePointer, clickUi, worldPoint, clickWorld, reportFailure, serveBuild };
