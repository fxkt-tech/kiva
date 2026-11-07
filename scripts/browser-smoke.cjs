// Set KIVA_SITE_DIR for an isolated temporary fixture, otherwise KIVA_TEST_URL.
// Exact skill damage/control, attack backswing and item interactions live in
// browser-skills/browser-equipment and the native simulation/client regressions.
// This smoke suite exercises the real renderer, navigation, UI and camera.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { PNG } = require("pngjs");
const {
  launchBrowser, installFrameClock, selectAndEnter, resumeGame, state, hero,
  observed, waitTicks, moveTo, nearHome, movePointer, clickUi, clickWorld,
  pause, reportFailure, serveBuild,
} = require("./browser-lane-support.cjs");

const distance = (left, right) => Math.hypot(...left.map((value, index) => value - right[index]));
const waitIdle = (page) => page.waitForFunction(() => {
  const snapshot = window.__kivaState;
  return snapshot.units.find((unit) => unit.id === snapshot.player_id).order === "Idle";
});
const waitCentered = (page) => page.waitForFunction(() => {
  const snapshot = window.__kivaState;
  const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
  return Math.hypot(...snapshot.camera_center.map((value, index) => value - player.position[index])) < 0.1;
});

async function assertRenderedScene(page, artifactDir) {
  const screenshot = await page.screenshot({ path: path.join(artifactDir, "single-lane-smoke.png") });
  const image = PNG.sync.read(screenshot);
  const colors = new Set();
  // Sample visible game terrain between the top HUD and bottom controls.
  for (let y = 200; y < 650; y += 5) {
    for (let x = 300; x < 1100; x += 5) {
      const index = (y * image.width + x) * 4;
      colors.add(`${image.data[index] >> 3},${image.data[index + 1] >> 3},${image.data[index + 2] >> 3}`);
    }
  }
  assert(colors.size > 30, "the real WebGPU scene must render visible terrain and unit geometry");
  assert(await page.locator("#hero-hud").isVisible());
  assert(await page.locator("#match-hud").isVisible());
  assert(await page.locator("#game-cursor").isVisible());
  assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game");
}

async function exerciseMovement(page, initial) {
  const first = nearHome(initial, 3, -3);
  const far = nearHome(initial, 11, -5);
  const replacement = nearHome(initial, 4.5, -3);
  await moveTo(page, first);
  await waitIdle(page);
  await clickWorld(page, far, "right");
  await observed(page, (from) => {
    const snapshot = window.__kivaState;
    const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
    return player.order === "Moving" && Math.hypot(...player.position.map((value, i) => value - from[i])) > 0.7
      ? snapshot : false;
  }, first);
  await moveTo(page, replacement);
  await waitIdle(page);
  assert(distance(hero(await state(page)).position, far) > 2, "the replacement click must cancel the old destination");

  await clickWorld(page, far, "right");
  await observed(page, () => {
    const snapshot = window.__kivaState;
    return snapshot.units.find((unit) => unit.id === snapshot.player_id).order === "Moving" ? snapshot : false;
  });
  await page.keyboard.press("s");
  await waitIdle(page);
  const stopped = await state(page);
  assert.equal(hero(stopped).action, "Ready");
  assert(distance(hero(stopped).position, far) > 1, "Stop must happen before reaching the queued destination");
  await waitTicks(page, 20);
  assert(distance(hero(await state(page)).position, hero(stopped).position) < 0.1,
    "Stop must cancel an active walking route and keep the hero still");
  console.log("PASS real ground picking, click-to-move, replacement orders and active Stop");
}

async function exerciseUiAndCamera(page) {
  const stationary = await state(page);
  const beforeHud = hero(stationary).position;
  await clickUi(page, '[data-slot="0"]');
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  const aiming = await state(page);
  assert(distance(hero(aiming).position, beforeHud) < 0.1, "a HUD click must not leak into a walking order");
  assert.equal(hero(aiming).order, "Idle");
  assert.equal(aiming.abilities[0].cooldown, 0, "selecting a targeted skill must not commit it");
  await clickWorld(page, beforeHud, "right");
  await page.waitForFunction(() => window.__kivaState.aim === null);
  assert.equal((await state(page)).abilities[0].cooldown, 0, "right click cancels aim without spending the ability");

  await page.keyboard.press("y");
  await page.waitForFunction(() => window.__kivaState.camera_locked);
  await waitCentered(page);
  await page.keyboard.press("y");
  await page.waitForFunction(() => !window.__kivaState.camera_locked);
  const beforePan = (await state(page)).camera_center;
  await movePointer(page, 2, 450);
  await page.waitForFunction((before) => Math.hypot(...window.__kivaState.camera_center.map((value, i) => value - before[i])) > 0.8,
    beforePan);
  assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game", "edge panning must retain native capture");
  await movePointer(page, 720, 450);
  await page.keyboard.down("Space");
  try { await waitCentered(page); }
  finally { await page.keyboard.up("Space"); }
  assert.equal((await state(page)).camera_locked, false, "temporary Space follow must not change Y mode");

  const beforeZoom = await state(page);
  await page.mouse.wheel(0, 100);
  await page.waitForFunction((before) => window.__kivaState.view_projection.some((value, i) => Math.abs(value - before[i]) > 0.0001),
    beforeZoom.view_projection);
  assert(distance((await state(page)).camera_center, beforeZoom.camera_center) < 0.1,
    "zoom changes the projection while preserving camera center");

  await page.setViewportSize({ width: 1000, height: 720 });
  await page.waitForFunction(() => window.__kivaState.viewport[0] === 1000 && window.__kivaState.viewport[1] === 720);
  const resized = await state(page);
  const destination = nearHome(resized, 4, -2);
  await moveTo(page, destination);
  await waitIdle(page);
  assert(distance(hero(await state(page)).position, destination) < 0.15, "resized CSS picking must reach its world destination");
  console.log("PASS HUD isolation, aim cancellation, Y/Space follow, captured edge pan, measured zoom and resized picking");
}

async function exercisePauseAndFocus(page, context) {
  await pause(page);
  const frozen = await state(page);
  await page.waitForTimeout(500);
  const still = await state(page);
  assert.equal(still.tick, frozen.tick);
  assert.equal(still.match.elapsed_seconds, frozen.match.elapsed_seconds);
  assert.deepEqual(hero(still).position, hero(frozen).position);
  await resumeGame(page);
  assert.equal((await state(page)).player_id, frozen.player_id);
  assert.equal((await state(page)).match.id, frozen.match.id);

  // Native browser focus, rather than Playwright's emulated DOM focus, must
  // drive the blur/capture gate. The other tab is owned by this fixture only.
  const cdp = await context.newCDPSession(page);
  await cdp.send("Emulation.setFocusEmulationEnabled", { enabled: false });
  const otherPage = await context.newPage();
  try {
    await otherPage.bringToFront();
    await page.waitForFunction(() => document.documentElement.dataset.paused === "true", undefined, { polling: 100 });
    assert.equal(await page.evaluate(() => document.hasFocus()), false);
    await page.bringToFront();
    await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  } finally { await otherPage.close(); }
  await resumeGame(page);
  await waitIdle(page);
  const resumed = await state(page);
  assert.equal(resumed.player_id, frozen.player_id);
  assert.equal(resumed.match.id, frozen.match.id);
  console.log("PASS frozen simulation, native focus loss and same-instance clean resume");
}

async function main() {
  const browser = await launchBrowser();
  let page;
  let site;
  const errors = [];
  let webgpu = false;
  const observeErrors = (target) => {
    target.on("pageerror", (error) => errors.push(error.message));
    target.on("console", (message) => {
      if (message.text().includes("BrowserWebGpu")) webgpu = true;
      if (message.type() === "error" || message.text().includes("%cERROR%c")) errors.push(message.text());
    });
  };
  try {
    if (process.env.KIVA_SITE_DIR) site = await serveBuild(process.env.KIVA_SITE_DIR);
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 });
    page = await context.newPage();
    page.setDefaultTimeout(120000);
    observeErrors(page);
    await installFrameClock(page);
    const initial = await selectAndEnter(page, site?.url);
    assert(webgpu, "must use the real WebGPU renderer");
    const artifactDir = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
    await fs.mkdir(artifactDir, { recursive: true });
    await assertRenderedScene(page, artifactDir);
    console.log("PASS real WebGPU renderer, single-lane geometry, match HUD and captured cursor");
    await exerciseMovement(page, initial);
    await exerciseUiAndCamera(page);
    await exercisePauseAndFocus(page, context);
    await context.close();
    page = null;

    const highDpiContext = await browser.newContext({ viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 });
    page = await highDpiContext.newPage();
    page.setDefaultTimeout(120000);
    observeErrors(page);
    await installFrameClock(page);
    const scaled = await selectAndEnter(page, site?.url);
    assert.equal(await page.evaluate(() => devicePixelRatio), 2);
    assert.deepEqual(scaled.viewport, [1000, 720], "HUD projection remains in CSS pixels at DPR2");
    const destination = nearHome(scaled, 4, -2);
    await moveTo(page, destination);
    await waitIdle(page);
    assert(distance(hero(await state(page)).position, destination) < 0.15);
    await highDpiContext.close();
    page = null;
    console.log("PASS high-DPI CSS projection and world picking");
    assert.deepEqual(errors, [], "no browser or renderer errors");
    console.log(`Browser suite passed. Screenshot: ${artifactDir}/single-lane-smoke.png`);
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
