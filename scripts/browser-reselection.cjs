// Exercise the production pause-menu intents and complete lane-match lifecycle.
// The fixture observes projections and sends ordinary UI/input intents only.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, installFrameClock, openSelection, selectCharacter, resumeGame,
  state, hero, minions, nearHome, observed, firstWaveStart, waitForFirstWave, pause: pauseGame, clickWorld,
  reportFailure, serveBuild,
} = require("./browser-lane-support.cjs");

async function pause(page) {
  await pauseGame(page);
  await page.waitForFunction(() => !document.getElementById("reselect").disabled);
}

async function screenshot(page, name) {
  if (process.env.KIVA_BROWSER_HEADED !== "1") return;
  const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
  await fs.mkdir(directory, { recursive: true });
  const { width, height } = page.viewportSize();
  const scale = await page.evaluate(() => devicePixelRatio);
  await page.screenshot({ path: path.join(directory, `${name}-${width}x${height}-dpr${scale}.png`) });
}

function assertFreshMatch(snapshot, oldIds, characterId) {
  assert.equal(snapshot.session.phase, "Active");
  assert.equal(snapshot.match.phase, "Running");
  assert.equal(snapshot.match.wave_number, 1);
  assert.equal(snapshot.units.length, 8, "our visible base, hero and complete first wave must be fresh");
  assert.equal(snapshot.units.filter(({ role }) => role === "Hero").length, 1);
  assert.equal(snapshot.units.filter(({ role }) => role === "Base").length, 1);
  for (const team of ["ally"]) {
    assert.equal(minions(snapshot, team).filter(({ role }) => role === "MeleeMinion").length, 3);
    assert.equal(minions(snapshot, team).filter(({ role }) => role === "RangedMinion").length, 3);
  }
  assert(snapshot.units.every(({ id }) => !oldIds.includes(id)), "no root from the old match may survive");
  assert.equal(hero(snapshot).character_id, characterId);
  assert.equal(hero(snapshot).health, hero(snapshot).max_health);
  assert.equal(hero(snapshot).mana, hero(snapshot).max_mana);
  assert(snapshot.abilities.every((ability) => ability.cooldown === 0));
  assert.equal(snapshot.aim, null);
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
    && !document.getElementById("enter").disabled && document.querySelectorAll(".unit-label").length === 0);
  const selecting = await state(page);
  assert.equal(selecting.paused, true);
  assert.equal(selecting.match.phase, "Waiting");
  assert.equal(selecting.match.id, null);
  assert.equal(selecting.match.wave_number, 0);
  assert.equal(selecting.match.next_wave_seconds, null);
  assert.deepEqual(selecting.match.bases, { ally: null, enemy: null });
  assert.equal(selecting.player_id, null);
  assert.equal(selecting.aim, null);
  assert.equal(selecting.selected, null);
  assert.equal(selecting.notice, "");
  assert.deepEqual(selecting.abilities, []);
  assert.deepEqual(selecting.camera_center, [0, 3]);
  assert.equal(selecting.camera_locked, false);
  assert.equal(selecting.session.selection.selection_ack.request_id, 0);
  assert.equal(selecting.session.selection.selection_ack.accepted, true);
  assert.deepEqual(selecting.units, [], "Waiting must have no hidden default player or old NPCs");
  assert(await page.locator("#hero-hud").isHidden());
  assert(await page.locator("#reselection-actions").isHidden());
  assert(await page.locator("#match-hud").isHidden());
  assert.equal(await page.locator(".unit-label").count(), 0, `labels from player ${oldPlayer} and every old root must be removed`);
  await page.waitForTimeout(250);
  const still = await state(page);
  assert.equal(still.tick, 0);
  assert.deepEqual(still.units, []);
  return selecting;
}

async function rejectOldLaunch(page, oldLaunch) {
  const revision = (await state(page)).session.selection.selection_revision;
  await page.evaluate((oldLaunch) => {
    window.__testOldLaunchObserved = false;
    function replaceLaunch() {
      if (document.pointerLockElement?.id !== "game") return;
      document.removeEventListener("pointerlockchange", replaceLaunch);
      window.__testOldLaunchObserved = true;
      // Replace the public intents after the production listener has queued
      // this round's Launch. Native capture is still genuinely held.
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
  assert.deepEqual(rejected.units, []);
  assert(rejected.session.selection.error);
  assert.equal(rejected.session.selection.selection_revision, revision);
  assert.equal(rejected.session.selection.can_start, true, "stale Launch cannot revoke a valid newer choice");
}

async function exercise(page, url, config) {
  await openSelection(page, url);
  await selectCharacter(page, 2);
  const initial = await state(page);
  const timeOrigin = await page.evaluate(() => {
    window.__reselectionCanvas = document.getElementById("game");
    return performance.timeOrigin;
  });
  const oldLaunch = { type: "Launch", character_id: 2,
    selection_revision: initial.session.selection.selection_revision };
  await resumeGame(page);
  const firstStart = await firstWaveStart(page);
  assert.equal(firstStart.units.length, 3);
  const first = await waitForFirstWave(page);
  const firstPlayer = first.player_id;
  const oldIds = first.units.map((unit) => unit.id);
  assertFreshMatch(first, [], 2);
  await page.keyboard.press("y");
  await page.waitForFunction(() => window.__kivaState.camera_locked);

  // Commit a real actual volley on a safe flank, then pause in the same browser
  // callback that first observes its cooldown. Native reset/match tests prove
  // exact old-area/old-life delivery cleanup; this UI test verifies a committed
  // cast can cross reset with fresh roots, full new HP and no inherited state.
  await page.evaluate(() => {
    window.__reselectionCommitted = null;
    function pauseOnArea(event) {
      const snapshot = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
      if (!(snapshot.abilities[1]?.cooldown > 0)) return;
      window.removeEventListener("kiva-state", pauseOnArea);
      const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
      window.__reselectionCommitted = { tick: snapshot.tick, cooldown: snapshot.abilities[1].cooldown, mana: player.mana };
      document.documentElement.dataset.paused = "true";
      document.exitPointerLock();
    }
    window.addEventListener("kiva-state", pauseOnArea);
  });
  await page.keyboard.press("w");
  await page.waitForFunction(() => window.__kivaState.aim === 1);
  await clickWorld(page, nearHome(first, 4, -3));
  await page.waitForFunction(() => window.__reselectionCommitted && window.__kivaState.paused
    && !document.pointerLockElement && !document.getElementById("reselect").disabled);
  const committed = await page.evaluate(() => window.__reselectionCommitted);
  assert(committed.cooldown > first.abilities[1].total_cooldown - 0.3,
    "pause must happen during the actual real launched volley, shortly after commit");
  assert(committed.mana < hero(first).max_mana, "the old match must have really spent its ability cost");
  assert(await page.locator("#character-select").isHidden());
  const selecting = await returnToSelection(page, firstPlayer, config.deviceScaleFactor === 2);
  assert.equal(selecting.session.selection.selected.id, 2, "keep the previous role preselected");
  assert(selecting.session.selection.selection_revision > oldLaunch.selection_revision);
  assert.deepEqual(selecting.obstacles, first.obstacles, "reuse static navigation");
  assert.deepEqual(selecting.map, first.map, "reuse the same single-lane definition");
  await rejectOldLaunch(page, oldLaunch);
  await selectCharacter(page, 1);
  await resumeGame(page);
  const secondStart = await firstWaveStart(page);
  assert.equal(secondStart.units.length, 3);
  const second = await waitForFirstWave(page);
  const secondPlayer = second.player_id;
  assertFreshMatch(second, oldIds, 1);
  assert.notEqual(second.match.id, first.match.id);
  const clean = await observed(page, (minimum) => window.__kivaState.tick >= minimum ? window.__kivaState : false,
    second.tick + 45);
  assert(clean.tick < 360, "check the clean first wave before armies cross the enlarged lane");
  assert.equal(clean.units.length, 8);
  assert(clean.units.every((unit)=>unit.team==="ally" && unit.health===unit.max_health && unit.statuses.length===0),
    "the newly observed roster has no inherited wounds/status; distant enemies remain private");
  assert.equal(hero(clean).health, hero(clean).max_health);
  assert.deepEqual(hero(clean).statuses, []);
  // The new enemy hero is allowed to attack our minions already; their wounds
  // are legitimate new AI effects and must not be mistaken for leaked old W.
  assert(clean.abilities.every((ability) => ability.cooldown === 0));

  await page.evaluate((oldPlayer) => {
    document.documentElement.dataset.sessionRequests = JSON.stringify([
      { type: "ReturnToSelection", player_id: oldPlayer },
    ]);
  }, firstPlayer);
  await page.waitForFunction(() => !document.documentElement.hasAttribute("data-session-requests"));
  assert.equal((await state(page)).player_id, secondPlayer, "old return cannot reset the new player");
  assert.equal((await state(page)).match.id, second.match.id, "old return cannot reset the new match");
  await pause(page);
  await resumeGame(page);
  const resumed = await state(page);
  assert.equal(resumed.player_id, secondPlayer, "ordinary resume preserves the player");
  assert.equal(resumed.match.id, second.match.id, "ordinary resume preserves the match");
  await pause(page);
  const nextSelection = await returnToSelection(page, secondPlayer);
  assert.equal(nextSelection.session.selection.selected.id, 1);
  assert(nextSelection.session.selection.selection_revision > selecting.session.selection.selection_revision);
  await resumeGame(page); // Same role, without another card click or page reload.
  const thirdStart = await firstWaveStart(page);
  assert.equal(thirdStart.units.length, 3);
  const third = await waitForFirstWave(page);
  assertFreshMatch(third, second.units.map((unit) => unit.id), 1);
  assert.notEqual(third.match.id, second.match.id);
  assert.equal(await page.evaluate(() => performance.timeOrigin), timeOrigin, "reselection must not reload the page");
  assert(await page.evaluate(() => document.getElementById("game") === window.__reselectionCanvas),
    "reselection reuses the real canvas");
  console.log(`PASS full lane reset, committed delayed cast, empty Waiting, scheduled 3-to-8 visible rosters, old request isolation, changed/same role and resume ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
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
      page.setDefaultTimeout(120000);
      page.on("pageerror", (error) => errors.push(error.message));
      await installFrameClock(page);
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
