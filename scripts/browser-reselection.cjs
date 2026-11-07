// Exercise the production pause-menu intent and complete trial lifecycle.
// The fixture observes projections and sends ordinary UI/input intents only.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, openSelection, selectCharacter, resumeGame, clickWorld,
  reportFailure, serveBuild,
} = require("./browser-support.cjs");

const state = (page) => page.evaluate(() => window.__kivaState);

async function pause(page) {
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement
    && !document.getElementById("reselect").disabled);
}

async function screenshot(page, name) {
  if (process.env.KIVA_BROWSER_HEADED !== "1") return;
  const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
  await fs.mkdir(directory, { recursive: true });
  const { width, height } = page.viewportSize();
  const scale = await page.evaluate(() => devicePixelRatio);
  await page.screenshot({ path: path.join(directory, `${name}-${width}x${height}-dpr${scale}.png`) });
}

async function returnToSelection(page, oldPlayer, keyboard = false) {
  const button = page.locator("#reselect");
  assert(await button.isVisible());
  assert((await page.locator("#reselection-note").textContent()).includes("重开"));
  const box = await button.boundingBox();
  const viewport = page.viewportSize();
  assert(box && box.y >= 0 && box.y + box.height <= viewport.height, "reselection must fit the pause menu");
  await screenshot(page, "reselection-pause");
  if (keyboard) {
    await button.focus();
    await page.keyboard.press("Enter");
  } else {
    await button.click();
  }
  await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting"
    && window.__kivaState.player_id === null && window.__kivaState.tick === 0
    && !document.getElementById("enter").disabled);
  const selecting = await state(page);
  assert.equal(selecting.paused, true);
  assert.equal(selecting.aim, null);
  assert.equal(selecting.selected, null);
  assert.equal(selecting.notice, "");
  assert.deepEqual(selecting.abilities, []);
  assert.deepEqual(selecting.camera_center, [0, 3]);
  assert.equal(selecting.camera_locked, false);
  assert.equal(selecting.session.selection.selection_ack.request_id, 0);
  assert.equal(selecting.session.selection.selection_ack.accepted, true);
  assert.equal(selecting.units.length, 2);
  assert(selecting.units.every((unit) => unit.id !== oldPlayer && unit.character_id === null));
  for (const [name, position] of [["训练哨兵", [3, -2]], ["巡游哨兵", [-5, -6]]]) {
    const unit = selecting.units.find((unit) => unit.name === name);
    assert.deepEqual(unit.position, position);
    assert.equal(unit.health, 400);
    assert.deepEqual(unit.statuses, []);
    assert.equal(unit.order, "Idle");
  }
  assert(await page.locator("#hero-hud").isHidden());
  assert(await page.locator("#reselection-actions").isHidden());
  assert.equal(await page.locator(".unit-label").count(), 2, "old character labels must be removed");
  await page.waitForTimeout(250);
  assert.equal((await state(page)).tick, 0);
  return selecting;
}

async function rejectOldLaunch(page, oldLaunch) {
  await page.evaluate((oldLaunch) => {
    window.__testOldLaunchObserved = false;
    function replaceLaunch() {
      if (document.pointerLockElement?.id !== "game") return;
      document.removeEventListener("pointerlockchange", replaceLaunch);
      window.__testOldLaunchObserved = true;
      // Replace only the public intents after the production listener queues
      // this round's Launch. The actual native capture is still held.
      document.documentElement.dataset.sessionRequests = JSON.stringify([oldLaunch]);
    }
    document.addEventListener("pointerlockchange", replaceLaunch);
  }, oldLaunch);
  await page.locator("#enter").click();
  await page.waitForFunction(() => window.__testOldLaunchObserved
    && window.__kivaState.session.phase === "Selecting" && !document.pointerLockElement
    && !document.getElementById("enter").disabled);
  const rejected = await state(page);
  assert.equal(rejected.player_id, null);
  assert.equal(rejected.tick, 0);
  assert(rejected.session.selection.error);
}

async function exercise(page, url, config) {
  await openSelection(page, url);
  await selectCharacter(page, 2);
  const initial = await state(page);
  const timeOrigin = await page.evaluate(() => performance.timeOrigin);
  const oldLaunch = { type: "Launch", character_id: 2,
    selection_revision: initial.session.selection.selection_revision };
  await resumeGame(page);
  const first = await state(page);
  const firstPlayer = first.player_id;
  const oldIds = first.units.map((unit) => unit.id);
  await page.keyboard.press("q");
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  await clickWorld(page, [3, -2]);
  await page.waitForFunction(() => window.__kivaState.units.some((unit) => unit.name === "训练哨兵" && unit.health === 301));
  await page.keyboard.press("y");
  await page.waitForFunction(() => window.__kivaState.camera_locked);

  // Pause a real delayed ability at commit, with no host round trip between the
  // observed cooldown and pause intent. Simulation reset must remove its area.
  await page.evaluate(() => {
    function pauseOnArea(event) {
      const snapshot = JSON.parse(event.detail);
      if (!(snapshot.abilities[1]?.cooldown > 0)) return;
      window.removeEventListener("kiva-state", pauseOnArea);
      document.documentElement.dataset.paused = "true";
      document.exitPointerLock();
    }
    window.addEventListener("kiva-state", pauseOnArea);
  });
  await page.keyboard.press("w");
  await page.waitForFunction(() => window.__kivaState.aim === 1);
  await clickWorld(page, [3, -2]);
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement
    && !document.getElementById("reselect").disabled);
  assert(await page.locator("#character-select").isHidden());
  const selecting = await returnToSelection(page, firstPlayer, config.deviceScaleFactor === 2);
  assert.equal(selecting.session.selection.selected.id, 2, "keep the previous role preselected");
  assert(selecting.session.selection.selection_revision > oldLaunch.selection_revision);
  assert(selecting.units.every((unit) => !oldIds.includes(unit.id)), "NPCs must also be fresh instances");
  assert.deepEqual(selecting.obstacles, first.obstacles, "reuse the same static arena");
  await rejectOldLaunch(page, oldLaunch);
  await selectCharacter(page, 1);
  await resumeGame(page);
  const second = await state(page);
  const secondPlayer = second.player_id;
  const hero = second.units.find((unit) => unit.id === secondPlayer);
  assert.notEqual(secondPlayer, firstPlayer);
  assert.equal(hero.character_id, 1);
  assert.equal(hero.health, 600);
  assert.equal(hero.mana, 300);
  assert.equal(second.units.length, 3);
  assert(second.abilities.every((ability) => ability.cooldown === 0));
  assert.equal(second.aim, null);
  await page.waitForTimeout(1000);
  assert.equal((await state(page)).units.find((unit) => unit.name === "训练哨兵").health, 400,
    "old delayed impacts cannot damage new trial actors");

  await page.evaluate((oldPlayer) => {
    document.documentElement.dataset.sessionRequests = JSON.stringify([
      { type: "ReturnToSelection", player_id: oldPlayer },
    ]);
  }, firstPlayer);
  await page.waitForFunction(() => !document.documentElement.hasAttribute("data-session-requests"));
  assert.equal((await state(page)).player_id, secondPlayer, "old return requests cannot reset the new trial");
  await pause(page);
  await resumeGame(page);
  assert.equal((await state(page)).player_id, secondPlayer, "ordinary resume still preserves the player");
  await pause(page);
  await returnToSelection(page, secondPlayer);
  await resumeGame(page); // Same role, without another card click or page reload.
  const third = await state(page);
  assert.notEqual(third.player_id, secondPlayer);
  assert.equal(third.units.find((unit) => unit.id === third.player_id).character_id, 1);
  assert.equal(third.units.length, 3);
  assert.equal(await page.evaluate(() => performance.timeOrigin), timeOrigin, "reselection must not reload the page");
  console.log(`PASS full trial reset, clean NPCs/effects, old request isolation, changed/same role and ordinary resume ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
}

async function main() {
  const browser = await launchBrowser();
  let site;
  let page;
  const errors = [];
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    for (const config of [
      { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 },
      { viewport: { width: 800, height: 600 }, deviceScaleFactor: 1 },
      { viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 },
    ]) {
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(20000);
      page.on("pageerror", (error) => errors.push(error.message));
      await exercise(page, site.url, config);
      await context.close();
      page = null;
    }
    assert.deepEqual(errors, [], "no uncaught browser errors");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
