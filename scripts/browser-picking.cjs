// Exercise the common unit picker through real Pointer Lock input and public
// gameplay state. No World writes, target teleports or extra combat rules.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, enterGame, resumeGame, selectCharacter, movePointer, clickPointer,
  installFrameClock, state, hero, moveTo, nearHome, panTo, waitTicks, pause,
  reportFailure, serveBuild, clickWorld,
} = require("./browser-lane-support.cjs");

const distance = (a, b) => Math.hypot(...a.map((value, index) => value - b[index]));

async function hover(page, id, point) {
  await movePointer(page, ...point);
  await page.waitForFunction((id) => window.__kivaState.hovered === id, id);
  await page.waitForFunction(() => document.getElementById("game-cursor").dataset.target === "enemy");
  assert(await page.locator("#game-cursor .cursor-sword").isVisible(), "enemy models must show a sword cursor");
  assert(await page.locator("#game-cursor .cursor-arrow").isHidden());
}

async function bodyPoint(page, id) {
  return page.evaluate((id) => window.__laneLabels.get(id)?.body_screen
    || window.__kivaState.units.find((unit) => unit.id === id)?.body_screen, id);
}

async function labelPoint(page, id, name) {
  return page.evaluate(({ id, name }) => {
    const unit = window.__kivaState.units.find((unit) => unit.id === id);
    const label = [...document.querySelectorAll(`#unit-labels .unit-label.${unit.team}`)]
      .find((candidate) => !candidate.hidden && candidate.querySelector("strong").textContent === unit.name);
    const region = name ? label?.querySelector("strong") : label?.querySelector(".bar");
    if (!region || label.hidden) throw new Error("fixture requires a visible real unit label");
    const box = region.getBoundingClientRect();
    // The edge of the actual label is deliberately away from the torso.
    return [box.left + 3, box.top + box.height / 2];
  }, { id, name });
}

async function selectAt(page, id, point, message) {
  await hover(page, id, point);
  await clickPointer(page, "left", 0);
  await page.waitForFunction((id) => window.__kivaState.selected === id, id);
  assert.equal((await state(page)).selected, id, message);
}

async function clearHover(page) {
  const { width, height } = page.viewportSize();
  await movePointer(page, width * 0.1, height * 0.4);
  await page.waitForFunction(() => window.__kivaState.hovered === null);
  await page.waitForFunction(() => document.getElementById("game-cursor").dataset.target === "none");
}

async function stop(page) {
  await page.keyboard.press("s");
  await page.waitForFunction(() => {
    const s = window.__kivaState;
    return s.units.find((unit) => unit.id === s.player_id).order === "Idle";
  });
}

async function assertUiIsolation(page) {
  const snapshot = await state(page);
  const before = hero(snapshot).position;
  const box = await page.locator("#hero-stats").boundingBox();
  await movePointer(page, box.x + box.width / 2, box.y + box.height / 2);
  await page.waitForFunction(() => window.__kivaState.hovered === null);
  assert.equal(await page.locator("#game-cursor").getAttribute("data-target"), "none");
  await clickPointer(page, "right", 0);
  await waitTicks(page, 6);
  const after = await state(page);
  assert.equal(hero(after).order, "Idle", "HUD panels must still own their input");
  assert(distance(hero(after).position, before) < 0.02);
}

async function exerciseInitialCursor(page) {
  const initial = await state(page);
  await movePointer(page, ...await bodyPoint(page, initial.player_id));
  await waitTicks(page, 6);
  assert.equal(await page.locator("#game-cursor").getAttribute("data-target"), "none",
    "friendly geometry must not advertise an enemy attack cursor");
  assert.equal((await state(page)).selected, null, "hovering does not commit a persistent selection");
  await clearHover(page);
  await assertUiIsolation(page);
  console.log("PASS friendly/ground normal cursor, no hover auto-selection and HUD input isolation");
}

async function reachEnemyBase(page) {
  const snapshot = await state(page);
  // Travel along a real flank outside the opposing hero's lane pursuit band.
  // The enemy base is stationary; its health is unknown until team vision
  // reaches it, so camera movement cannot manufacture a selectable enemy.
  assert.equal(snapshot.match.bases.enemy, null);
  await moveTo(page, nearHome(snapshot, 1, 10));
  await moveTo(page, nearHome(snapshot, 45, 10));
  await page.waitForFunction(() => window.__kivaState.units.some((unit) =>
    unit.team === "enemy" && unit.role === "Base" && unit.alive));
  const enemy = (await state(page)).units.find((unit) => unit.team === "enemy" && unit.role === "Base");
  await panTo(page, enemy.position);
  await waitTicks(page, 6);
  return enemy.id;
}

async function exerciseEnemy(page, id) {
  await selectAt(page, id, await bodyPoint(page, id), "an enemy body uses the same common picker");
  await clearHover(page);
  await clickPointer(page, "left", 0);
  await page.waitForFunction(() => window.__kivaState.selected === null);
  for (const name of [false, true]) {
    await movePointer(page, ...await labelPoint(page, id, name));
    await page.waitForFunction((id) => window.__kivaState.hovered !== id
      && document.getElementById("game-cursor").dataset.target === "none", id);
    await clickPointer(page, "left", 0);
    await waitTicks(page, 6);
    assert.equal((await state(page)).selected, null,
      `${name ? "names" : "health bars"} must not create a unit hit`);
  }

  await page.keyboard.press("a");
  await page.waitForFunction(() => window.__kivaState.attack_move);
  await hover(page, id, await bodyPoint(page, id));
  await clickPointer(page, "left", 0);
  await page.waitForFunction((id) => {
    const s = window.__kivaState;
    return !s.attack_move && s.selected === id
      && s.units.find((unit) => unit.id === s.player_id).order === "Attacking";
  }, id);
  await stop(page);

  // A ground click is still attack-move, even though the previous model is
  // the persistent inspected selection. Do not redirect to that old target.
  const ground = nearHome(await state(page), 45, 14);
  await panTo(page, ground);
  await page.keyboard.press("a");
  await page.waitForFunction(() => window.__kivaState.attack_move);
  await clickWorld(page, ground);
  await page.waitForFunction(() => {
    const s = window.__kivaState;
    return !s.attack_move && s.units.find((unit) => unit.id === s.player_id).order === "Attack move";
  });
  await stop(page);
  await panTo(page, (await state(page)).units.find((unit) => unit.id === id).position);
  await waitTicks(page, 6);
  console.log("PASS A+left on a model locks that target; A+left on ground keeps attack-move");

  const before = await state(page);
  const enemy = before.units.find((unit) => unit.id === id);
  const start = hero(before).position;
  assert(distance(start, enemy.position) > hero(before).attributes.attack_range + 3,
    "fixture must click from outside auto-attack range");
  await hover(page, id, await bodyPoint(page, id));
  await clickPointer(page, "right", 0);
  await page.waitForFunction((id) => {
    const s = window.__kivaState;
    return s.selected === id && s.units.find((unit) => unit.id === s.player_id).order === "Attacking";
  }, id);
  await page.evaluate(() => window.__laneSetFrameStep(250));
  try {
    await page.waitForFunction(({ id, start, health }) => {
      const s = window.__kivaState;
      const p = s.units.find((unit) => unit.id === s.player_id);
      const target = s.units.find((unit) => unit.id === id);
      if (!p.alive) throw new Error("fixture hero died on its real flank route");
      return s.selected === id && p.order === "Attacking" && target?.health < health
        && Math.hypot(p.position[0] - start[0], p.position[1] - start[1]) > 2
        && ["Attacking", "AttackBackswing"].includes(p.action);
    }, { id, start, health: enemy.health });
  } finally { await page.evaluate(() => window.__laneSetFrameStep(1000 / 60)); }
  await stop(page);
  await clearHover(page);
  assert.equal((await state(page)).selected, id, "leaving a model only clears hover, not persistent selection");
  await hover(page, id, await bodyPoint(page, id));
  console.log("PASS enemy model sword cursor, labels excluded, out-of-range pursuit and real auto-attack damage");
}

async function exerciseLifecycle(page, original) {
  await pause(page);
  assert.equal((await state(page)).hovered, null);
  assert.equal(await page.locator("#game-cursor").getAttribute("data-target"), "none");
  await resumeGame(page);
  assert.equal((await state(page)).player_id, original);
  await page.locator("#game-cursor").waitFor();
  await pause(page);
  await page.locator("#reselect").click();
  await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting"
    && !document.getElementById("enter").disabled);
  await selectCharacter(page, 2);
  await resumeGame(page);
  const replacement = (await state(page)).player_id;
  assert.notEqual(replacement, original);
  await movePointer(page, ...await bodyPoint(page, replacement));
  await waitTicks(page, 6);
  assert.equal(await page.locator("#game-cursor").getAttribute("data-target"), "none",
    "a new friendly hero must not retain the previous enemy sword cursor");
  assert.notEqual((await state(page)).hovered, original);
  console.log("PASS pause/resume and fresh hero selection without stale pick identities");
}

async function main() {
  let browser, site, page;
  const errors = [];
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    const cases = [[1280, 800, 1], [800, 600, 1], [1000, 720, 2]];
    const requested = process.env.KIVA_PICKING_CASE;
    const selected = cases.filter(([w, h, scale]) => !requested || `${w}x${h}@${scale}` === requested);
    assert(selected.length, "unknown KIVA_PICKING_CASE");
    for (const [width, height, scale] of selected) {
      browser = await launchBrowser();
      const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: scale });
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      page.on("pageerror", (error) => errors.push(error.message));
      await installFrameClock(page);
      await enterGame(page, site.url);
      await page.waitForFunction(() => Object.hasOwn(window.__kivaState, "hovered"));
      const original = (await state(page)).player_id;
      await exerciseInitialCursor(page);
      const enemy = await reachEnemyBase(page);
      await exerciseEnemy(page, enemy);
      if (width === 1280) await exerciseLifecycle(page, original);
      if (process.env.KIVA_BROWSER_HEADED === "1") {
        const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
        await fs.mkdir(directory, { recursive: true });
        await page.screenshot({ path: path.join(directory, `picking-${width}x${height}-dpr${scale}.png`) });
      }
      await context.close();
      page = null;
      await browser.close();
      browser = null;
      console.log(`PASS common unit picking ${width}x${height} DPR${scale}`);
    }
    assert.deepEqual(errors, [], "no uncaught browser errors");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { if (browser) await browser.close(); }
    finally { if (site) await site.close(); }
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
