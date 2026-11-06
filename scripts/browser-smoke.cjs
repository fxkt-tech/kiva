// Optional integration suite. Connects to an already running Trunk/static server.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { PNG } = require("pngjs");
const { launchBrowser, enterGame, resumeGame, movePointer, clickUi, clickWorld, reportFailure } = require("./browser-support.cjs");

async function main() {
  const browser = await launchBrowser();
  let page;
  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 });
    page = await context.newPage();
    page.setDefaultTimeout(20000);
    const errors = [];
    let webgpu = false;
    page.on("pageerror", (error) => errors.push(error.message));
    page.on("console", (message) => {
      if (message.text().includes("BrowserWebGpu")) webgpu = true;
      if (message.type() === "error" || message.text().includes("%cERROR%c")) errors.push(message.text());
    });
    const state = () => page.evaluate(() => window.__kivaState);
    const playerPosition = () => page.evaluate(() => {
      const s = window.__kivaState;
      return s.units.find((u) => u.id === s.player_id).position;
    });
    const waitPosition = (position) => page.waitForFunction(([x, y]) => {
      const s = window.__kivaState;
      const p = s.units.find((u) => u.id === s.player_id).position;
      return Math.hypot(p[0] - x, p[1] - y) < 0.12;
    }, position);
    const waitIdle = () => page.waitForFunction(() => {
      const s = window.__kivaState;
      return s.units.find((u) => u.id === s.player_id).order === "Idle";
    });
    await enterGame(page);
    assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game");
    assert(webgpu, "must use the WebGPU renderer");

    const artifactDir = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
    await fs.mkdir(artifactDir, { recursive: true });
    const screenshot = await page.screenshot({ path: path.join(artifactDir, "proving-grounds.png") });
    const image = PNG.sync.read(screenshot);
    const colors = new Set();
    for (let y = 200; y < 650; y += 5) {
      for (let x = 300; x < 1100; x += 5) {
        const i = (y * image.width + x) * 4;
        colors.add(`${image.data[i] >> 3},${image.data[i + 1] >> 3},${image.data[i + 2] >> 3}`);
      }
    }
    assert(colors.size > 30, "scene must render visible geometry");
    console.log("PASS WebGPU scene, HUD and visible cursor");

    await clickWorld(page, [-4, 6], "right");
    await waitPosition([-4, 6]);
    await clickWorld(page, [7, 6], "right");
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      return s.units.find((u) => u.id === s.player_id).position[0] > -2;
    });
    await clickWorld(page, [0, 6], "right");
    await waitPosition([0, 6]);
    await page.keyboard.press("s");
    await waitIdle();
    console.log("PASS click-to-move and replacing orders");

    await page.keyboard.press("q");
    await page.waitForFunction(() => window.__kivaState.aim === 0);
    await clickWorld(page, [0, 6], "right");
    await page.waitForFunction(() => window.__kivaState.aim === null);
    assert.equal((await state()).abilities[0].cooldown, 0);
    await page.keyboard.press("q");
    await clickWorld(page, [3, -2]);
    await page.waitForFunction(() => window.__kivaState.units.some((u) => u.name === "训练哨兵" && u.health === 325));
    await page.keyboard.press("w");
    await clickWorld(page, [3, -2]);
    await page.waitForFunction(() => window.__kivaState.units.some((u) => u.name === "训练哨兵" && u.health === 205));
    console.log("PASS aim cancellation, projectile and delayed area damage");

    await page.keyboard.press("e");
    await clickWorld(page, [-3, 6]);
    await waitPosition([-3, 6]);
    assert((await state()).abilities[2].cooldown > 0);
    await page.keyboard.press("r");
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      return s.units.find((u) => u.id === s.player_id).action === "Channeling";
    });
    await clickWorld(page, [0, 6], "right");
    await waitPosition([0, 6]);
    assert((await state()).abilities[3].cooldown > 0);
    console.log("PASS dash, channel interruption and persistent cooldowns");

    await clickWorld(page, [3, -2, 1.3], "right");
    await page.waitForFunction(() => window.__kivaState.units.some((u) => u.name === "训练哨兵" && u.health < 205));
    await page.keyboard.press("s");
    await waitIdle();
    await page.keyboard.press("a");
    await page.waitForFunction(() => window.__kivaState.attack_move);
    await clickWorld(page, [3, -4]);
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      return s.units.find((u) => u.id === s.player_id).order === "Attack move";
    });
    await page.keyboard.press("s");
    await waitIdle();
    console.log("PASS chase attack, attack move and stop");

    const beforeHud = await playerPosition();
    await clickUi(page, '[data-slot="0"]');
    await page.waitForFunction(() => window.__kivaState.aim === 0);
    assert(Math.hypot(...(await playerPosition()).map((v, i) => v - beforeHud[i])) < 0.1);
    await clickWorld(page, [0, 6], "right");
    await page.keyboard.press("y");
    await page.waitForFunction(() => window.__kivaState.camera_locked);
    await page.keyboard.press("y");
    await page.waitForFunction(() => !window.__kivaState.camera_locked);
    const beforePan = (await state()).camera_center;
    await movePointer(page, 2, 450);
    await page.waitForFunction((before) => {
      const after = window.__kivaState.camera_center;
      return Math.hypot(after[0] - before[0], after[1] - before[1]) > 1;
    }, beforePan);
    await movePointer(page, 720, 450);
    await page.keyboard.down("Space");
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      const p = s.units.find((u) => u.id === s.player_id).position;
      return Math.hypot(s.camera_center[0] - p[0], s.camera_center[1] - p[1]) < 0.1;
    });
    await page.keyboard.up("Space");
    await page.mouse.wheel(0, 100);
    await page.setViewportSize({ width: 1000, height: 720 });
    await page.waitForFunction(() => window.__kivaState.viewport[0] === 1000);
    const start = await playerPosition();
    const destination = [start[0] - 2, start[1]];
    await clickWorld(page, destination, "right");
    await waitPosition(destination);
    console.log("PASS UI isolation, camera modes, edge pan, zoom and resized picking");

    await page.keyboard.press("Escape");
    await page.waitForFunction(() => window.__kivaState.paused);
    const tick = (await state()).tick;
    await page.waitForTimeout(500);
    assert.equal((await state()).tick, tick);
    await resumeGame(page);
    // Playwright normally emulates focus for every page. Disable that emulation
    // so the next tab exercises real DOM blur and the game's focus handling.
    const cdp = await context.newCDPSession(page);
    await cdp.send("Emulation.setFocusEmulationEnabled", { enabled: false });
    const otherPage = await page.context().newPage();
    await otherPage.bringToFront();
    await page.waitForFunction(() => document.documentElement.dataset.paused === "true");
    await page.bringToFront();
    await page.waitForFunction(() => window.__kivaState.paused);
    await otherPage.close();
    await resumeGame(page);
    await waitIdle();
    console.log("PASS pause, focus loss and clean resume");
    await context.close();

    const highDpiContext = await browser.newContext({ viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 });
    const highDpiPage = await highDpiContext.newPage();
    page = highDpiPage;
    highDpiPage.on("pageerror", (error) => errors.push(error.message));
    await enterGame(highDpiPage);
    await clickWorld(highDpiPage, [-2, 6], "right");
    await highDpiPage.waitForFunction(() => {
      const s = window.__kivaState;
      const p = s.units.find((u) => u.id === s.player_id).position;
      return Math.hypot(p[0] + 2, p[1] - 6) < 0.12;
    });
    await highDpiContext.close();
    console.log("PASS high-DPI picking");
    assert.deepEqual(errors, [], "no browser or renderer errors");
    console.log(`Browser suite passed. Screenshot: ${artifactDir}/proving-grounds.png`);
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    await browser.close();
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
