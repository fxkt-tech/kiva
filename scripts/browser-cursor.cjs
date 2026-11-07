const assert = require("node:assert/strict");
const { launchBrowser, enterGame, resumeGame, movePointer, moveRelative, clickUi, clickWorld, reportFailure, serveBuild, installFrameClock, state, hero, observed, nearHome, waitTicks } = require("./browser-lane-support.cjs");

async function main() {
  const browser = await launchBrowser();
  let site;
  let page;
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
    page.setDefaultTimeout(120000);
    await installFrameClock(page);
    await enterGame(page, site.url);
    const captured = await page.evaluate(() => document.pointerLockElement?.id || null);
    console.log("Captured game canvas:", captured);
    assert.equal(captured, "game", "gameplay must capture the mouse so it cannot leave the game window");
    assert(await page.locator("#game-cursor").isVisible());
    const centered = await page.evaluate(() => {
      const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
      return [matrix.m41, matrix.m42];
    });
    assert.deepEqual(centered, [640, 400], "entry must center the cursor without replaying old movement");
    await movePointer(page, 1279, 400);
    const center = await page.evaluate(() => window.__kivaState.camera_center);
    await page.waitForFunction((before) => Math.hypot(...window.__kivaState.camera_center.map((value, i) => value - before[i])) > 1, center);
    await moveRelative(page, 10000, 0);
    await moveRelative(page, -60, 0);
    const nextCenter = () => page.evaluate(() => new Promise((resolve) => {
      window.addEventListener("kiva-state", (event) => resolve(JSON.parse(event.detail).camera_center), { once: true });
    }));
    const stopped = await nextCenter();
    assert.deepEqual(await nextCenter(), stopped, "moving back from the edge must stop camera scrolling");
    assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game");
    console.log("PASS bounded visible cursor, edge scrolling and immediate recovery after overshoot");

    await page.keyboard.press("Escape");
    await page.waitForFunction(() => !document.pointerLockElement && window.__kivaState.paused);
    assert(await page.locator("#game-cursor").isHidden());
    await resumeGame(page);
    await page.keyboard.down("Space");
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      const p = s.units.find((unit) => unit.id === s.player_id).position;
      return Math.hypot(...s.camera_center.map((value, i) => value - p[i])) < 0.1;
    });
    await page.keyboard.up("Space");
    const home = await state(page);
    const destination = nearHome(home, 2.8, -1.8);
    await clickWorld(page, destination, "right");
    await page.waitForFunction(([x, z]) => {
      const s = window.__kivaState;
      const p = s.units.find((unit) => unit.id === s.player_id).position;
      return Math.hypot(p[0] - x, p[1] - z) < 0.1;
    }, destination);
    await page.waitForFunction(() => {
      const snapshot = window.__kivaState;
      return snapshot.units.find((unit) => unit.id === snapshot.player_id).order === "Idle";
    });
    const beforeStandard = await state(page);
    const origin = hero(beforeStandard).position;
    const lane = beforeStandard.map.enemy_base.map((value, index) => value - beforeStandard.map.ally_base[index]);
    const length = Math.hypot(...lane);
    const direction = lane.map((value) => value / length);
    const range = beforeStandard.abilities[2].range;
    const landing = origin.map((value, index) => value + direction[index] * range);
    await clickUi(page, '[data-slot="2"]');
    await page.waitForFunction(() => window.__kivaState.aim === 2);
    // Jarvan E places a flag. A far ground click must clamp its actual landing
    // to the authored range without turning the player's body into a dash.
    await clickWorld(page, origin.map((value, index) => value + direction[index] * range * 2));
    const placed = await observed(page, (landing) => {
      const snapshot = window.__kivaState;
      return snapshot.abilities[2].cooldown > 0 && snapshot.objects.some((object) =>
        object.kind === "Standard" && object.owner === snapshot.player_id && object.team === "ally" && object.active
          && Math.hypot(...object.position.map((value, index) => value - landing[index])) < 0.1)
        ? snapshot : false;
    }, landing);
    assert(Math.hypot(...hero(placed).position.map((value, index) => value - origin[index])) < 0.1,
      "the captured flag-placement click must keep the hero stationary");
    assert.equal(hero(placed).order, "Idle");
    assert.equal(placed.aim, null);
    console.log("PASS Escape/re-entry, click-to-move and max-range flag placement with the captured cursor");

    await page.context().close();
    const highDpi = await browser.newContext({ viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 });
    page = await highDpi.newPage();
    page.setDefaultTimeout(120000);
    await installFrameClock(page);
    await enterGame(page, site.url);
    await movePointer(page, 560, 400);
    const highHome = await state(page);
    const highDestination = nearHome(highHome, 2.8, -1.8);
    await clickWorld(page, highDestination, "right");
    await page.waitForFunction(([x, z]) => {
      const s = window.__kivaState;
      const p = s.units.find((unit) => unit.id === s.player_id).position;
      return Math.hypot(p[0] - x, p[1] - z) < 0.1;
    }, highDestination);
    await page.setViewportSize({ width: 800, height: 600 });
    await page.waitForFunction(() => window.__kivaState.viewport[0] === 800);
    await movePointer(page, 799, 300);
    await moveRelative(page, 1000, 0);
    await moveRelative(page, -100, 0);
    await nextCenter();
    await nextCenter();
    await page.keyboard.down("Space");
    await waitTicks(page, 3);
    await page.keyboard.up("Space");
    const resizedDestination = nearHome(highHome, 3.8, -0.5);
    await clickWorld(page, resizedDestination, "right");
    await page.waitForFunction(([x, z]) => {
      const s = window.__kivaState;
      const p = s.units.find((unit) => unit.id === s.player_id).position;
      return Math.hypot(p[0] - x, p[1] - z) < 0.1;
    }, resizedDestination);
    await highDpi.close();
    console.log("PASS captured cursor, resize bounds and world picking at high DPI");

    page = await browser.newPage();
    await page.addInitScript(() => {
      HTMLCanvasElement.prototype.requestPointerLock = () => Promise.reject(new DOMException("Denied", "NotAllowedError"));
    });
    await page.goto(site.url);
    await page.locator("#enter").waitFor({ state: "visible", timeout: 120000 });
    await page.locator("#enter").click();
    await page.locator("#pointer-error").waitFor({ state: "visible" });
    assert.equal(await page.evaluate(() => document.pointerLockElement), null);
    assert.equal(await page.evaluate(() => document.documentElement.dataset.paused), "true");
    console.log("PASS capture refusal keeps the game paused with a retry message");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
