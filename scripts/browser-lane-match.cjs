// End-to-end checks of the real lane mode. Only normal input and public UI
// intents are sent; the fixture never writes to the simulation World.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, serveBuild, installFrameClock, selectAndEnter, state, hero, base,
  movePointer, moveTo, nearHome, clickUnit, clickWorld, waitTicks, pause,
  resumeGame, restartGame, firstWaveStart, waitForFirstWave, worldPoint, reportFailure,
} = require("./browser-lane-support.cjs");

async function screenshot(page, label) {
  if (process.env.KIVA_BROWSER_HEADED !== "1") return;
  const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts/lane";
  await fs.mkdir(directory, { recursive: true });
  const { width, height } = page.viewportSize();
  await page.screenshot({ path: path.join(directory, `${label}-${width}x${height}-dpr${await page.evaluate(() => devicePixelRatio)}.png`) });
}

async function assertLayout(page) {
  const boxes = await page.evaluate(() => {
    const selectors = ["#hero-hud", "#match-clock", "#ally-base-health", "#enemy-base-health"];
    return { width: innerWidth, height: innerHeight, scrollWidth: document.documentElement.scrollWidth,
      items: selectors.map((selector) => {
        const element = document.querySelector(selector);
        if (!element || !element.getClientRects().length) return { selector, missing: true };
        const { x, y, width, height } = element.getBoundingClientRect();
        return { selector, x, y, width, height };
      }) };
  });
  for (const box of boxes.items) {
    assert(!box.missing, `${box.selector} must be visible`);
    assert(box.width > 0 && box.height > 0 && box.x >= -1 && box.y >= -1
      && box.x + box.width <= boxes.width + 1 && box.y + box.height <= boxes.height + 1,
    `${box.selector} must fit the viewport`);
  }
  assert(boxes.scrollWidth <= boxes.width + 1, "no horizontal page overflow");
}

async function exerciseMap(page, url, deep) {
  const initial = await selectAndEnter(page, url);
  assert.equal(typeof initial.match.id, "string", "match identity must preserve u64 precision");
  assert.equal(initial.map.half_size, 32);
  assert.equal(base(initial, "ally").max_health, 3000);
  assert.equal(initial.match.bases.enemy,null);
  assert.equal((await page.locator("#enemy-base-health").textContent()),"未知");
  assert.deepEqual(initial.map.ally_base, [-8, 24]);
  assert.deepEqual(initial.map.enemy_base, [8, -24]);
  const allyProjection = await worldPoint(page, [...base(initial, "ally").position, 1.4]);
  const enemyProjection = await worldPoint(page, [...initial.map.enemy_base, 1.4]);
  assert(allyProjection[0] < enemyProjection[0] && allyProjection[1] > enemyProjection[1],
    "the default camera orientation keeps allied/enemy bases southwest/northeast even when one is off screen");
  assert.equal(initial.camera_locked, false);
  assert(Math.hypot(...initial.camera_center.map((value,i) => value - hero(initial).position[i])) < 0.15);
  const playerScreen = hero(initial).body_screen;
  assert(playerScreen && playerScreen[0] > 0 && playerScreen[0] < page.viewportSize().width
    && playerScreen[1] > 0 && playerScreen[1] < page.viewportSize().height,
  "the large map starts at its visible hero instead of forcing both bases into one screen");
  await assertLayout(page);
  await screenshot(page, "single-lane-start");

  // Hero-only R cannot target an ally or an unobserved enemy location. Camera
  // movement is not a source of team vision and must not leak the enemy base.
  await page.keyboard.press("r");
  await page.waitForFunction(()=>window.__kivaState.aim===3);
  await clickUnit(page,base(initial,"ally").id);
  assert.equal((await state(page)).aim,3);
  await clickWorld(page,initial.map.enemy_base);
  const invalid=await state(page);
  assert.equal(invalid.aim,3);
  assert.equal(invalid.abilities[3].cooldown,0);
  assert.equal(invalid.match.bases.enemy,null,"panning cannot reveal enemy base health");
  assert.deepEqual(hero(invalid).position,hero(initial).position);
  await page.keyboard.press("s");
  await moveTo(page, nearHome(initial, 2.8, -1.8));
  await page.keyboard.press("s");

  await pause(page);
  const frozen = await state(page);
  await page.waitForTimeout(250);
  const afterPause = await state(page);
  assert.equal(afterPause.tick, frozen.tick);
  assert.equal(afterPause.match.id, frozen.match.id);
  assert.equal(afterPause.match.wave_number, frozen.match.wave_number);
  assert.equal(afterPause.match.next_wave_seconds, frozen.match.next_wave_seconds);
  assert.deepEqual(afterPause.units.map(({ id, position, visual_height }) => ({ id, position, visual_height })),
    frozen.units.map(({ id, position, visual_height }) => ({ id, position, visual_height })));
  await resumeGame(page);
  assert.equal((await state(page)).player_id, frozen.player_id);
  assert.equal((await state(page)).match.id, frozen.match.id);

  if (deep) {
    await page.keyboard.press("y");
    await page.waitForFunction(() => window.__kivaState.camera_locked);
    await page.keyboard.down("Space");
    await waitTicks(page, 3);
    await page.keyboard.up("Space");
    await page.keyboard.press("y");
    await page.waitForFunction(() => !window.__kivaState.camera_locked);
    const beforePan = (await state(page)).camera_center;
    await page.keyboard.down("ArrowLeft");
    try {
      await page.waitForFunction((before) => Math.hypot(...window.__kivaState.camera_center.map((v, i) => v - before[i])) > 0.5, beforePan);
    } finally { await page.keyboard.up("ArrowLeft"); }
    await movePointer(page, 1, page.viewportSize().height / 2);
    const edge = (await state(page)).camera_center;
    await page.waitForFunction((before) => Math.hypot(...window.__kivaState.camera_center.map((v, i) => v - before[i])) > 0.3, edge);
    await movePointer(page, page.viewportSize().width / 2, page.viewportSize().height / 2);
    const beforeZoom = (await state(page)).view_projection;
    await page.mouse.wheel(0, -240);
    await page.waitForFunction((before) => window.__kivaState.view_projection.some((v, i) => Math.abs(v - before[i]) > 0.0001), beforeZoom);
    await page.mouse.wheel(0, 240);
    await page.keyboard.down("Space");
    await waitTicks(page, 3);
    await page.keyboard.up("Space");
  }
  console.log(`PASS 64-square map, 0s first pair and scheduled 3+3 waves, structure aim, pause and preserved camera ${page.viewportSize().width}x${page.viewportSize().height}`);
}

async function exerciseOutcome(page) {
  // Accelerate only the fixture's monotonic clock. Production tick ordering,
  // wave intervals, damage and AI all run unchanged, through real fixed steps.
  await page.evaluate(() => window.__laneSetFrameStep(250));
  // Software rendering can advance fewer Bevy frames than the fixture's RAF
  // clock: Time's max-delta cap still applies. Bound the real match as well as
  // host time instead of treating performance.now() as simulation progress.
  const startedAt = Date.now();
  const progress = setInterval(async () => {
    try {
      const snapshot = await page.evaluate(() => ({ tick: window.__kivaState.tick,
        match: window.__kivaState.match, fixture: window.__laneDiagnostics }));
      console.log("MATCH progress", JSON.stringify({ wall_seconds: (Date.now() - startedAt) / 1000, ...snapshot }));
    } catch (error) { console.error("MATCH diagnostic:", error.message); }
  }, 10000);
  try {
    await page.waitForFunction(() => {
      const snapshot = window.__kivaState;
      if (snapshot.tick > 60 * 60 * 10) throw new Error("The real lane match did not finish within ten simulated minutes");
      return snapshot.match.phase === "Finished";
    }, undefined, { timeout: 360000 });
  } finally { clearInterval(progress); }
  await page.locator("#match-result").waitFor({ state: "visible" });
  const finished = await state(page);
  assert(["Victory", "Defeat", "Draw"].includes(finished.match.outcome));
  if (finished.match.outcome!=="Victory") assert.equal(base(finished,"ally").health,0);
  assert.equal(await page.locator("#match-result").isVisible(),true);
  assert.equal(await page.evaluate(() => document.pointerLockElement), null);
  await screenshot(page, "single-lane-result");
  const fixed = finished.units.map(({ id, position, health, visual_height }) => ({ id, position, health, visual_height }));
  await page.waitForTimeout(500);
  const stable = await state(page);
  assert.equal(stable.tick, finished.tick);
  assert.equal(stable.match.wave_number, finished.match.wave_number);
  assert.equal(stable.match.next_wave_seconds, null);
  assert.deepEqual(stable.units.map(({ id, position, health, visual_height }) => ({ id, position, health, visual_height })), fixed);
  await page.evaluate(() => { document.documentElement.dataset.paused = "false"; });
  await page.waitForTimeout(180);
  assert.equal((await state(page)).tick, finished.tick, "a resume intent must not restart a finished match");

  await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
  await restartGame(page);
  const first = await firstWaveStart(page);
  assert.equal(first.units.length, 3);
  const restarted = await waitForFirstWave(page);
  assert.notEqual(restarted.match.id, finished.match.id);
  assert.notEqual(restarted.player_id, finished.player_id);
  assert.equal(restarted.match.wave_number, 1);
  assert.equal(restarted.match.phase, "Running");
  assert(restarted.tick >= 150 && restarted.tick < 300, "observe all scheduled first-wave pairs after restart");
  assert(restarted.units.every((unit) => !finished.units.some((old) => old.id === unit.id)),
    "restart must replace the bases, both heroes and minions");
  assert.equal(await page.locator("#match-result").isHidden(), true);
  await page.evaluate((old) => {
    document.documentElement.dataset.sessionRequests = JSON.stringify([
      { type: "RestartMatch", match_id: old.match.id, player_id: old.player_id },
    ]);
  }, finished);
  await page.waitForFunction(() => !document.documentElement.hasAttribute("data-session-requests"));
  assert.equal((await state(page)).match.id, restarted.match.id, "an old restart must not reset a newer match");
  console.log(`PASS actual ${finished.match.outcome}, terminal freeze, trusted restart and stale match isolation`);
}

async function main() {
  let browser;
  let site, page;
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    const cases = [
      { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 },
      { viewport: { width: 800, height: 600 }, deviceScaleFactor: 1 },
      { viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 },
    ];
    const requested = process.env.KIVA_LANE_CASE;
    const selected = cases.filter(({ viewport, deviceScaleFactor }) => !requested
      || `${viewport.width}x${viewport.height}@${deviceScaleFactor}` === requested);
    assert(selected.length, `Unknown KIVA_LANE_CASE: ${requested}`);
    for (const config of selected) {
      browser = await launchBrowser();
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      const errors = [];
      let failBrowser;
      const browserFailure = new Promise((_, reject) => { failBrowser = reject; });
      page.on("pageerror", (error) => {
        errors.push(error.message);
        console.error("Browser pageerror:", error.stack || error.message);
        failBrowser(error);
      });
      page.on("console", (message) => {
        if (message.type() === "error" || /%c(?:WARN|ERROR)%c/.test(message.text())) {
          console.log(`Browser console ${message.type()}:`, message.text());
        }
        if (message.text().includes("Quitting the application due to")) failBrowser(new Error(message.text()));
      });
      await installFrameClock(page);
      await Promise.race([exerciseMap(page, site.url, config.viewport.width === 1280), browserFailure]);
      if (config.viewport.width === 1280) await Promise.race([exerciseOutcome(page), browserFailure]);
      assert.deepEqual(errors, []);
      await context.close();
      page = null;
      await browser.close();
      browser = null;
    }
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { if (browser) await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
