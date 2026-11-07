// Production lane combat, captured input and read-only Rust projections.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, installFrameClock, selectAndEnter, resumeGame, pause, state, hero, enemyHero, base,
  observed, waitTicks, moveTo, nearHome, nearMiddle, clickUnit, movePointer, clickPointer, clickAndMovePointer,
  clickUi, clickWorld, worldPoint, reportFailure, serveBuild,
} = require("./browser-lane-support.cjs");

function progress(stage, snapshot) {
  const player = snapshot && hero(snapshot);
  const enemy = snapshot && enemyHero(snapshot);
  console.log(`SKILLS ${stage}`, JSON.stringify({ tick: snapshot?.tick, aim: snapshot?.aim,
    player: player && { position: player.position, health: player.health, order: player.order },
    enemy: enemy && { id: enemy.id, position: enemy.position, health: enemy.health } }));
}

async function status(page, id, kind) {
  const initialTick = (await state(page)).tick;
  return observed(page, ({ id, kind, initialTick }) => {
    const snapshot = window.__kivaState;
    const unit = snapshot.units.find((unit) => unit.id === id);
    if (!unit?.alive) throw new Error(`Target ${id} died while waiting for ${kind}`);
    if (unit.statuses.some((effect) => effect.kind === kind)) return snapshot;
    if (snapshot.tick > initialTick + 180) {
      throw new Error(`No ${kind} after 180 actual ticks: ${JSON.stringify({ unit, aim: snapshot.aim, abilities: snapshot.abilities })}`);
    }
    return false;
  }, { id, kind, initialTick });
}

async function freshMatch(page) {
  const old = await state(page);
  if (!old.paused) await pause(page);
  await page.locator("#reselect").click();
  await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting"
    && !document.getElementById("enter").disabled);
  assert.equal((await state(page)).units.length, 0);
  await resumeGame(page);
  const fresh = await state(page);
  assert.notEqual(fresh.match.id, old.match.id);
  assert.notEqual(fresh.player_id, old.player_id);
  assert.equal(fresh.match.phase, "Running");
  return fresh;
}

async function approach(page, forwardOffset = -0.65, lateral = -3) {
  const before = await state(page);
  progress(`walk to ${JSON.stringify(nearMiddle(before, forwardOffset, lateral))}`, before);
  await moveTo(page, nearMiddle(before, forwardOffset, lateral));
  await page.keyboard.press("s");
  const initialTick = (await state(page)).tick;
  const ready = await observed(page, (initialTick) => {
    const snapshot = window.__kivaState;
    const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
    const enemy = snapshot.units.find((unit) => unit.role === "Hero" && unit.team === "enemy");
    if (!player?.alive || !enemy?.alive) throw new Error("A hero died while preparing skill range");
    if (snapshot.tick > initialTick + 900) {
      throw new Error(`No combat range after 900 actual ticks: ${JSON.stringify({ player, enemy })}`);
    }
    return player.alive && enemy?.alive && player.order === "Idle"
      && Math.hypot(...enemy.position.map((value, i) => value - player.position[i])) < 7.5 ? snapshot : false;
  }, initialTick);
  progress("combat range ready", ready);
  return ready;
}

async function exerciseControls(page, keyboardW) {
  const before = await state(page);
  const names = before.abilities.map((ability) => ability.name);
  assert.deepEqual(await page.locator("[data-slot] small").allTextContents(), names);
  for (let slot = 0; slot < names.length; slot++) {
    assert((await page.locator(`[data-slot="${slot}"]`).getAttribute("title")).includes(names[slot]));
  }

  // Held HUD input and invalid candidates must preserve aim without movement,
  // resource consumption or a cooldown. Structures are invalid skill targets.
  await clickUi(page, '[data-slot="0"]', 1200);
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  for (const candidate of [null, before.player_id, base(before, "ally").id, base(before, "enemy").id]) {
    if (candidate === null) await clickWorld(page, nearHome(before, 5, -4));
    else await clickUnit(page, candidate);
    await waitTicks(page);
    const rejected = await state(page);
    assert.equal(rejected.aim, 0, "invalid enemy aim must remain selected");
    assert.equal(rejected.abilities[0].cooldown, 0);
    assert.equal(hero(rejected).mana, hero(before).max_mana);
    assert(Math.hypot(...hero(rejected).position.map((value, i) => value - hero(before).position[i])) < 0.1,
      "HUD and invalid candidates must not issue walking orders");
    progress(`invalid aim preserved: ${candidate || "ground"}`, rejected);
  }

  const ready = await approach(page);
  assert.equal(ready.aim, null, "a movement order must cancel selected aim");
  const target = enemyHero(ready);
  const speed = target.attributes.move_speed;
  await page.keyboard.press("w");
  await page.waitForFunction(() => window.__kivaState.aim === 1);
  await page.keyboard.press("s");
  await page.waitForFunction(() => window.__kivaState.aim === null);
  await page.keyboard.press("q");
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  await clickUnit(page, target.id);
  progress("Q submitted");
  const afterQ = await status(page, target.id, "Stun");
  progress("Q stun observed", afterQ);
  const stunned = afterQ.units.find((unit) => unit.id === target.id);
  // Other lane units attack concurrently: bind the effect to this actual
  // target and check damage occurred, rather than assuming an idle 400HP NPC.
  assert(stunned.health < target.health);
  assert.equal(stunned.control.can_move, false);
  assert.equal(stunned.control.can_attack, false);
  assert.equal(stunned.control.can_cast, false);
  assert.equal(afterQ.aim, null);
  assert.equal(afterQ.paused, false);
  assert(afterQ.abilities[0].cooldown > 0);
  assert(hero(afterQ).mana < hero(ready).mana);

  if (keyboardW) await page.keyboard.press("w");
  else await clickUi(page, '[data-slot="1"]');
  await page.waitForFunction(() => window.__kivaState.aim === 1);
  await clickWorld(page, stunned.position);
  progress("W submitted");
  const afterW = await status(page, target.id, "Root");
  progress("W Root/Slow observed", afterW);
  const rooted = afterW.units.find((unit) => unit.id === target.id);
  assert(rooted.health < stunned.health);
  assert(rooted.statuses.some((effect) => effect.kind === "Slow"));
  assert.equal(rooted.control.can_move, false);
  assert.equal(rooted.control.can_attack, true);
  assert.equal(rooted.control.can_cast, true);
  assert(Math.abs(rooted.attributes.move_speed - speed * 0.7) < 0.001,
    "composed Root/Slow and final speed must agree in the same snapshot");
  assert(afterW.abilities[1].cooldown > 0);
  await observed(page, (id) => {
    const unit = window.__kivaState.units.find((unit) => unit.id === id);
    return unit?.alive && !unit.statuses.some((effect) => effect.kind === "Root")
      && unit.statuses.some((effect) => effect.kind === "Slow") && unit.control.can_move;
  }, target.id);
  // Only the long tail of Slow remains; short Root/Slow composition was already
  // observed at the fine frame step. Every simulation tick still runs.
  await page.evaluate(() => window.__laneSetFrameStep(250));
  try {
    const restored = await observed(page, ({ id, speed }) => {
      const unit = window.__kivaState.units.find((unit) => unit.id === id);
      if (!unit?.alive) throw new Error(`Target ${id} died before Slow restoration`);
      return !unit.statuses.some((effect) => ["Root", "Slow", "Stun"].includes(effect.kind))
        && Math.abs(unit.attributes.move_speed - speed) < 0.001 ? window.__kivaState : false;
    }, { id: target.id, speed });
    progress("Slow expired; speed restored", restored);
  } finally {
    await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
  }
}

async function exerciseAirborne(page, mouseR) {
  progress("R fresh match");
  await freshMatch(page);
  const ready = await approach(page, 0.35, -5);
  const target = enemyHero(ready);
  // The real Q holds the intelligent enemy during R's windup/projectile flight;
  // the flank also keeps the projectile clear of the minion line.
  await clickUi(page, '[data-slot="3"]');
  await page.waitForFunction(() => window.__kivaState.aim === 3);
  assert.equal((await state(page)).abilities[3].cooldown, 0, "selecting directional R must not self-cast");
  await page.keyboard.press("s");
  await page.waitForFunction(() => window.__kivaState.aim === null);
  // Prepare listeners before Q starts the short control window.
  await page.evaluate((id) => {
    window.__testSkillTrace = [];
    window.__recordSkillTrace = (event) => {
      const snapshot = window.__kivaState;
      const target = snapshot.units.find((unit) => unit.id === id);
      window.__testSkillTrace.push({ event, time: performance.now(), tick: snapshot.tick,
        selected: snapshot.selected, height: target?.visual_height,
        remaining: target?.statuses.find((effect) => effect.kind === "Airborne")?.remaining,
        displaced: target?.control.is_displaced });
    };
    window.addEventListener("kiva-state", () => {
      window.__testLastSnapshotAt = performance.now();
      if (window.__kivaState.units.find((unit) => unit.id === id)?.statuses.some((effect) => effect.kind === "Airborne")) {
        window.__recordSkillTrace("airborne-state");
      }
    });
    let pauseAfterPick = false;
    document.addEventListener("pointerdown", (event) => {
      if (window.__testPauseOnPick && event.isTrusted && event.button === 0) {
        window.__testPauseOnPick = false;
        pauseAfterPick = true;
        window.__recordSkillTrace("pointerdown");
      }
    });
    window.addEventListener("kiva-labels", () => {
      // This display frame follows the actual pick. Pause through the existing
      // production intent before native unlock notification can cost a tick.
      if (pauseAfterPick) {
        pauseAfterPick = false;
        window.__recordSkillTrace("pause-after-pick");
        document.documentElement.dataset.paused = "true";
        document.exitPointerLock();
      }
    });
  }, target.id);
  await clickUi(page, '[data-slot="0"]');
  await clickUnit(page, target.id);
  const stunned = await status(page, target.id, "Stun");
  const origin = stunned.units.find((unit) => unit.id === target.id).position;
  const player = hero(stunned).position;
  const direction = origin.map((value, i) => value - player[i]);
  const distance = Math.hypot(...direction);
  const landing = origin.map((value, i) => value + direction[i] / distance * 4);
  assert(landing.every((value) => Math.abs(value) < stunned.map.half_size - 1),
    "the authored four-unit displacement needs a clear map endpoint");
  assert.notEqual(stunned.selected, target.id, "the airborne click must make a new selection");
  // Keyboard and HUD selection share the directional targeting path.
  if (mouseR) await clickUi(page, '[data-slot="3"]');
  else await page.keyboard.press("r");
  await page.waitForFunction(() => window.__kivaState.aim === 3);
  assert.equal((await state(page)).abilities[3].cooldown, 0, "selecting R must not self-cast");
  const castPoint = await worldPoint(page, origin);
  const airbornePoint = await worldPoint(page, [...landing, 2.6]);
  await movePointer(page, ...castPoint);
  await clickAndMovePointer(page, ...airbornePoint);
  progress("R submitted; waiting for displaced airborne body");
  await page.waitForFunction(({ id, point, initialTick }) => {
    const snapshot = window.__kivaState;
    const target = snapshot.units.find((unit) => unit.id === id);
    if (!target?.alive) throw new Error(`R target ${id} died before the airborne pick`);
    if (snapshot.tick > initialTick + 180) {
      throw new Error(`R did not produce a pickable airborne endpoint: ${JSON.stringify(target)}`);
    }
    const cursor = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    const ready = target?.statuses.some((effect) => effect.kind === "Airborne")
      && !target.control.is_displaced && target.visual_height > 0.3
      && Math.abs(cursor.m41 - point[0]) < 2 && Math.abs(cursor.m42 - point[1]) < 2
      && window.__laneLabels.get(id)?.body_screen;
    if (ready) { window.__recordSkillTrace("pick-ready"); window.__testPauseOnPick = true; }
    return ready;
  }, { id: target.id, point: airbornePoint, initialTick: stunned.tick });
  await clickPointer(page, "left", 0);
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  const paused = await page.evaluate((id) => ({ tick: window.__kivaState.tick, clock: performance.now(),
    selected: window.__kivaState.selected, target: window.__kivaState.units.find((unit) => unit.id === id) }), target.id);
  assert.equal(paused.selected, target.id);
  assert(paused.target.health <= stunned.units.find((unit) => unit.id === target.id).health,
    "control-only R must not restore health during the displacement");
  assert.equal(paused.target.control.can_cast, false);
  assert(paused.target.statuses.some((effect) => effect.kind === "Airborne"));
  assert(paused.target.visual_height > 0);
  assert(Math.abs(Math.hypot(...paused.target.position.map((value, i) => value - origin[i])) - 4) < 0.08,
    "measure displacement while controlled, before AI can move after landing");
  progress("airborne body selected and paused");
  await page.evaluate(() => window.__laneSetFrameStep(250));
  try {
    await page.waitForFunction((clock) => window.__testLastSnapshotAt >= clock + 400, paused.clock);
  } finally {
    await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
  }
  const still = await state(page);
  const frozen = still.units.find((unit) => unit.id === target.id);
  assert.equal(still.tick, paused.tick);
  assert.equal(frozen.visual_height, paused.target.visual_height);
  assert.deepEqual(frozen.statuses, paused.target.statuses);
  assert.deepEqual(frozen.screen, paused.target.screen);
  assert.deepEqual(frozen.position, paused.target.position);
  assert(await page.locator('#target-statuses [data-status="Airborne"]').count() > 0);
  progress("exact airborne pose remained frozen", still);
  if (process.env.KIVA_BROWSER_HEADED === "1") {
    const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
    await fs.mkdir(directory, { recursive: true });
    const size = page.viewportSize();
    const dpr = await page.evaluate(() => devicePixelRatio);
    const style = await page.addStyleTag({ content: "#entry { visibility: hidden !important; }" });
    try { await page.screenshot({ path: path.join(directory, `airborne-${size.width}x${size.height}-dpr${dpr}.png`) }); }
    finally { await style.evaluate((element) => element.remove()); }
  }
  await resumeGame(page);
  await observed(page, (id) => {
    const unit = window.__kivaState.units.find((unit) => unit.id === id);
    return unit?.alive && !unit.statuses.some((effect) => effect.kind === "Airborne")
      && unit.visual_height === 0 && !unit.control.is_displaced;
  }, target.id);
}

async function exerciseDash(page, keyboardE) {
  progress("E fresh match");
  await freshMatch(page);
  await page.keyboard.press("s");
  await page.keyboard.press("e");
  await page.waitForFunction(() => window.__kivaState.aim === 2);
  await page.keyboard.press("s");
  await page.waitForFunction(() => window.__kivaState.aim === null);
  const snapshot = await state(page);
  const origin = hero(snapshot).position;
  const range = snapshot.abilities[2].range;
  const length = Math.hypot(...origin);
  const direction = origin.map((value) => -value / length);
  const destination = origin.map((value, i) => value + direction[i] * range);
  if (keyboardE) await page.keyboard.press("e");
  else await clickUi(page, '[data-slot="2"]');
  await clickWorld(page, origin.map((value, i) => value + direction[i] * range * 2));
  const dashed = await observed(page, (destination) => {
    const snapshot = window.__kivaState;
    const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
    return Math.hypot(...player.position.map((value, i) => value - destination[i])) < 0.1
      && snapshot.abilities[2].cooldown > 0 ? snapshot : false;
  }, destination);
  assert(hero(dashed).mana < hero(snapshot).mana);
  assert.equal(dashed.aim, null);
  progress("far E reached maximum-range endpoint", dashed);
}

async function main() {
  const browser = await launchBrowser();
  let site;
  let page;
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    const cases = [
      { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 },
      { viewport: { width: 800, height: 600 }, deviceScaleFactor: 1 },
      { viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 },
    ];
    const requested = process.env.KIVA_SKILLS_CASE;
    const selected = cases.filter(({ viewport, deviceScaleFactor }) => !requested
      || `${viewport.width}x${viewport.height}@${deviceScaleFactor}` === requested);
    assert(selected.length, `Unknown KIVA_SKILLS_CASE: ${requested}`);
    for (const config of selected) {
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      await installFrameClock(page);
      progress(`load ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
      progress("entered live lane", await selectAndEnter(page, site.url));
      const alternateInput = config.viewport.width === 800;
      await exerciseControls(page, alternateInput);
      await exerciseAirborne(page, alternateInput);
      await exerciseDash(page, alternateInput);
      console.log(`PASS lane Q/W/E/R keyboard and HUD, invalid/friendly/structure aim, composed controls, restoration, airborne picking/freeze and far dash ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
      await context.close();
      page = null;
    }
  } catch (error) {
    if (page) {
      console.error("Skill trace:", JSON.stringify(await page.evaluate(() => window.__testSkillTrace)));
      await reportFailure(page);
    }
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
