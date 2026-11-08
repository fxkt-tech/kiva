const { resetToSelection } = require("./browser-support.cjs");
// Real equipment management and combat through the production input adapters.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { launchBrowser, serveBuild, installFrameClock, selectAndEnter, resumeGame, pause, clickUi,
  clickWorld, clickUnit, moveTo, nearHome, nearMiddle, state, hero, enemyHero, base, observed, waitTicks,
  reportFailure } = require("./browser-lane-support.cjs");

function progress(stage, snapshot) {
  const player = snapshot && hero(snapshot);
  const target = snapshot && enemyHero(snapshot);
  console.log(`EQUIPMENT ${stage}`, JSON.stringify({ tick: snapshot?.tick, aim: snapshot?.equipment_aim,
    player: player && { position: player.position, health: player.health, order: player.order, action: player.action },
    target: target && { id: target.id, position: target.position, health: target.health } }));
}

async function openEditor(page) {
  // The long combat fixture pauses explicitly; T itself never pauses.
  if (!(await state(page)).paused) await pause(page);
  await page.keyboard.press("t");
  await page.waitForFunction(() => !document.getElementById("equipment-editor").hidden
    && window.__kivaState.equipment.can_manage && !document.pointerLockElement
    && document.getElementById("entry").hidden);
  // Advance publication only while this test's explicitly paused setup is frozen.
  await page.evaluate(() => window.__laneSetFrameStep(250));
}

async function equip(page, id) {
  const before = await state(page);
  const slot = before.equipment.first_empty_slot;
  assert.notEqual(slot, null);
  const previous = before.equipment.ack.request_id;
  await clickUi(page, `[data-equip-id="${id}"]`);
  return observed(page, ({ slot, id, previous }) => {
    const snapshot = window.__kivaState;
    return snapshot.equipment.slots[slot]?.item?.id === id
      && snapshot.equipment.ack.accepted && snapshot.equipment.ack.request_id !== previous ? snapshot : false;
  }, { slot, id, previous });
}

async function unequip(page, slot) {
  await page.locator(`[data-equipment-slot="${slot}"]`).click({ button: "right" });
  return observed(page, (slot) => {
    const snapshot = window.__kivaState;
    return snapshot.equipment.slots[slot].item === null && snapshot.equipment.ack.accepted ? snapshot : false;
  }, slot);
}

async function continueMatch(page) {
  await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
  await page.locator("#equipment-close").click();
  assert.equal(await page.evaluate(() => window.__kivaState.paused), true);
  await resumeGame(page);
}

async function screenshot(page, label) {
  if (process.env.KIVA_BROWSER_HEADED !== "1") return;
  const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
  await fs.mkdir(directory, { recursive: true });
  const { width, height } = page.viewportSize();
  const scale = await page.evaluate(() => devicePixelRatio);
  await page.screenshot({ path: path.join(directory, `${label}-${width}x${height}-dpr${scale}.png`) });
}

async function attackOnceAndPause(page, id, expectSlow) {
  const before = (await state(page)).units.find((unit) => unit.id === id);
  assert(before?.alive, "the observed combat target must survive until the attack");
  progress(`basic attack requested; expect Slow=${expectSlow}`);
  await page.evaluate(({ id, health, expectSlow, initialTick }) => {
    function pauseAfterHit(event) {
      const snapshot = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
      const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
      const target = snapshot.units.find((unit) => unit.id === id);
      if (!target?.alive || !player?.alive || snapshot.tick > initialTick + 600) {
        window.__gearAttackFailure = `No observed basic hit: ${JSON.stringify({ player, target, expectSlow })}`;
        window.removeEventListener("kiva-state", pauseAfterHit);
        return;
      }
      // Lane units can deal damage at the same time. Require our actual basic
      // attack release as well as damage and the authored on-hit status.
      if (!target?.alive || target.health >= health || player.action !== "AttackBackswing"
        || target.statuses.some((effect) => effect.kind === "Slow") !== expectSlow) return;
      window.removeEventListener("kiva-state", pauseAfterHit);
      document.documentElement.dataset.paused = "true";
      document.exitPointerLock();
    }
    window.__gearAttackFailure = null;
    window.addEventListener("kiva-state", pauseAfterHit);
  }, { id, health: before.health, expectSlow, initialTick: (await state(page)).tick });
  await clickUnit(page, id, "right");
  const paused = await observed(page, () => {
    if (window.__gearAttackFailure) throw new Error(window.__gearAttackFailure);
    return window.__kivaState.paused && !document.pointerLockElement ? window.__kivaState : false;
  });
  const target = paused.units.find((unit) => unit.id === id);
  assert(target.health < before.health, "a released basic attack must damage its chosen target");
  assert.equal(target.statuses.some((effect) => effect.kind === "Slow"), expectSlow);
  assert.equal(paused.selected, id);
  progress(`basic hit paused; Slow=${expectSlow}`, paused);
  return paused;
}

async function approachEnemy(page) {
  const before = await state(page);
  progress(`walk to ${JSON.stringify(nearMiddle(before, -0.65, -3))}`, before);
  await moveTo(page, nearMiddle(before, -0.65, -3));
  await page.keyboard.press("s");
  const initialTick = (await state(page)).tick;
  const ready = await observed(page, (initialTick) => {
    const snapshot = window.__kivaState;
    const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
    const target = snapshot.units.find((unit) => unit.role === "Hero" && unit.team === "enemy");
    if (!player?.alive || !target?.alive || snapshot.tick > initialTick + 900) {
      throw new Error(`No equipment combat range: ${JSON.stringify({ player, target })}`);
    }
    return player.alive && target?.alive && player.order === "Idle"
      && Math.hypot(...target.position.map((value, i) => value - player.position[i])) < 7.0 ? snapshot : false;
  }, initialTick);
  progress("combat range ready", ready);
  return ready;
}

async function layout(page) {
  const boxes = await page.evaluate(() => {
    const rect = (selector) => {
      const { x, y, width, height } = document.querySelector(selector).getBoundingClientRect();
      return { x, y, width, height };
    };
    return { viewport: [innerWidth, innerHeight], card: rect("#hero-hud"), gear: rect("#equipment-hud"),
      stats: rect("#hero-stats"), scrollWidth: document.documentElement.scrollWidth,
      labels: [...document.querySelectorAll("#hero-stats dt")].map((element) => element.textContent),
      slots: document.querySelectorAll("[data-equipment-slot]").length };
  });
  assert.equal(boxes.slots, 6);
  assert(boxes.stats.width <= 145);
  assert.deepEqual(boxes.labels, ["攻击", "法强", "护甲", "魔抗", "移速", "攻速", "攻距"]);
  for (const box of [boxes.card, boxes.gear, boxes.stats]) {
    assert(box.x >= -1 && box.y >= -1 && box.x + box.width <= boxes.viewport[0] + 1
      && box.y + box.height <= boxes.viewport[1] + 1, "combat HUD must fit the viewport");
  }
  assert(boxes.scrollWidth <= boxes.viewport[0] + 1);
}

async function exercise(page, url, deep) {
  progress(`load ${page.viewportSize().width}×${page.viewportSize().height}`);
  await selectAndEnter(page, url);
  const original = await state(page);
  progress("entered live lane", original);
  assert.equal(original.equipment.slots.length, 6);
  assert(original.equipment.slots.every(({ item }) => item === null));
  assert.equal((await page.evaluate(() => window.__kivaEquipmentCatalog)).length, 9);
  await page.keyboard.press("t");
  await page.waitForFunction(() => !document.getElementById("equipment-editor").hidden
    && !window.__kivaState.paused && document.pointerLockElement?.id === "game");
  const popupTick = (await state(page)).tick;
  await page.waitForFunction((tick) => window.__kivaState.tick > tick && !window.__kivaState.paused, popupTick);
  assert.equal(await page.locator("#equipment-inventory").count(), 0);
  await clickUi(page, "#equipment-close");
  assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game");
  await layout(page);
  await openEditor(page);
  if (deep) {
    // Keep the real UI pending state, then replace only its public request with
    // a malformed batch. Generic parse errors must allow an ordinary retry.
    await page.locator('[data-equip-id="1"]').evaluate((button) => button.addEventListener("click", () => {
      document.documentElement.dataset.equipmentRequests = JSON.stringify([
        { type: "Equip", player_id: window.__kivaState.player_id, slot: 0, equipment_id: 1, request_id: 900001 },
        { type: "WorldWrite", stats: { attack_damage: 999999 } },
      ]);
    }, { once: true }));
    await clickUi(page, '[data-equip-id="1"]');
    await page.waitForFunction(() => window.__kivaState.equipment.ack.error
      && !document.querySelector('[data-equip-id="1"]').disabled);
    assert((await state(page)).equipment.slots.every(({ item }) => item === null));
  }
  await equip(page, 1);
  const duplicate = await equip(page, 1);
  assert.equal(hero(duplicate).attributes.attack_damage, 104);
  assert.notEqual(duplicate.equipment.slots[0].item.instance_id, duplicate.equipment.slots[1].item.instance_id);
  assert.equal(hero(await unequip(page, 0)).attributes.attack_damage, 84);
  await equip(page, 2);
  for (const id of [4, 5, 6, 7]) await equip(page, id);
  const filled = await state(page);
  assert.equal(filled.equipment.first_empty_slot, null);
  assert.equal(await page.locator('[data-equip-id]:enabled').count(), 0);
  assert.deepEqual(hero(filled).attributes, { attack_damage: 84, ability_power: 25, armor: 61,
    magic_resistance: 52, move_speed: 4.15, attack_range: 2.25, attack_speed: 0.9541 });
  progress("six slots and exact attributes verified", filled);
  await screenshot(page, "equipment-editor");
  const orb = filled.equipment.slots[5].item;
  await page.evaluate((instance) => {
    function staleClick() {
      if (document.pointerLockElement?.id !== "game") return;
      document.removeEventListener("pointerlockchange", staleClick);
      document.documentElement.dataset.equipmentSkill = JSON.stringify({ slot: 5, instance_id: instance });
    }
    document.addEventListener("pointerlockchange", staleClick);
  }, orb.instance_id);
  await continueMatch(page);
  assert.equal((await state(page)).equipment_aim, null, "a valid stale equipment click must be discarded at capture");
  assert.equal((await state(page)).equipment.slots[5].item.active.cooldown, 0);
  await layout(page);
  await screenshot(page, "equipment-combat");
  const ready = await state(page);
  if (deep) await page.keyboard.press("6");
  else await clickUi(page, '[data-equipment-slot="5"]');
  await page.waitForFunction(() => window.__kivaState.equipment_aim === 5);
  for (const candidate of [null, ready.player_id, base(ready, "ally").id]) {
    if (candidate === null) await clickWorld(page, nearHome(ready, 7, -7));
    else await clickUnit(page, candidate);
    await waitTicks(page);
    const rejected = await state(page);
    assert.equal(rejected.equipment_aim, 5, "enemy equipment must preserve invalid/friendly/structure aim");
    assert.equal(rejected.equipment.slots[5].item.active.cooldown, 0);
    assert.equal(rejected.abilities[0].cooldown, 0);
    progress(`invalid item aim preserved: ${candidate || "ground"}`, rejected);
  }
  // Keep the long base-camera checks at home, then approach actual combat.
  // Movement cancels the old item aim; re-arm through its real input path.
  const combat = await approachEnemy(page);
  const target = enemyHero(combat);
  if (deep) await page.keyboard.press("6");
  else await clickUi(page, '[data-equipment-slot="5"]');
  await page.waitForFunction(() => window.__kivaState.equipment_aim === 5);
  const beforeHitSnapshot = await state(page);
  const beforeHit = beforeHitSnapshot.units.find((unit) => unit.id === target.id);
  await clickUnit(page, target.id);
  progress("item active submitted");
  const hit = await observed(page, ({ id, initialTick }) => {
    const snapshot = window.__kivaState;
    const target = snapshot.units.find((unit) => unit.id === id);
    if (!target?.alive || snapshot.tick > initialTick + 180) {
      throw new Error(`Item active did not Stun its target: ${JSON.stringify(target)}`);
    }
    return target.statuses.some((effect) => effect.kind === "Stun")
      && snapshot.equipment.slots[5].item.active.cooldown > 0 ? snapshot : false;
  }, { id: target.id, initialTick: beforeHitSnapshot.tick });
  assert(hit.units.find((unit) => unit.id === target.id).health < beforeHit.health,
    "the real item active must damage the captured target");
  assert.equal(hit.equipment_aim, null);
  assert(hero(hit).mana < hero(ready).mana);
  assert.equal(hit.abilities[0].cooldown, 0, "equipment must not consume the matching hero ability cooldown");
  progress("item damage/Stun and separate role cooldown verified", hit);
  await pause(page);
  const frozen = await page.evaluate(() => ({ snapshot: window.__kivaState, clock: performance.now() }));
  await page.evaluate(() => window.__laneSetFrameStep(250));
  try {
    await page.waitForFunction((clock) => window.__gearLastSnapshotAt >= clock + 400, frozen.clock);
  } finally {
    await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
  }
  const still = await state(page);
  assert.equal(still.tick, frozen.snapshot.tick);
  assert.equal(still.equipment.slots[5].item.active.cooldown, frozen.snapshot.equipment.slots[5].item.active.cooldown);
  assert.equal(still.abilities[0].cooldown, frozen.snapshot.abilities[0].cooldown);
  progress("paused cooldown remained exact", still);
  await resumeGame(page);
  assert.equal((await state(page)).player_id, original.player_id);

  if (deep) {
    const itemCooldown = (await state(page)).equipment.slots[5].item.active.cooldown;
    const beforeRole = (await state(page)).units.find((unit) => unit.id === target.id);
    await page.keyboard.press("q");
    await page.waitForFunction(() => window.__kivaState.aim === 0);
    await clickUnit(page, target.id);
    const roleHit = await observed(page, ({id,armor}) => {
      const snapshot = window.__kivaState;
      return snapshot.abilities[0].cooldown > 0
        && snapshot.units.find((unit) => unit.id === id)?.attributes.armor < armor ? snapshot : false;
    }, {id:target.id,armor:beforeRole.attributes.armor});
    assert(roleHit.units.find((unit) => unit.id === target.id).health < beforeRole.health);
    assert(roleHit.equipment.slots[5].item.active.cooldown > 0);
    assert(roleHit.equipment.slots[5].item.active.cooldown <= itemCooldown,
      "role Q must not refresh the separate item cooldown");
    progress("role Q hit while item cooldown remained independent", roleHit);
    const struck = await attackOnceAndPause(page, target.id, true);
    assert(struck.units.find((unit) => unit.id === target.id).statuses.some((effect) => effect.kind === "Slow"));
    await openEditor(page);
    await unequip(page, 4);
    await continueMatch(page);
    await page.keyboard.press("s");
    await page.waitForFunction(() => {
      const snapshot = window.__kivaState;
      return snapshot.units.find((unit) => unit.id === snapshot.player_id)?.order === "Idle";
    });
    await page.evaluate(() => window.__laneSetFrameStep(250));
    try {
      await observed(page, (id) => {
        const unit = window.__kivaState.units.find((unit) => unit.id === id);
        if (!unit?.alive) throw new Error(`Target ${id} died before checking passive removal`);
        return !unit.statuses.some((effect) => effect.kind === "Slow");
      }, target.id);
    } finally {
      await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
    }
    progress("old Slow expired after unequip");
    await attackOnceAndPause(page, target.id, false);
    await openEditor(page);
    const oldBow = (await state(page)).equipment.slots[0].item;
    await unequip(page, 0);
    await equip(page, 8);
    await unequip(page, 1);
    await equip(page, 9);
    await continueMatch(page);
    await page.keyboard.press("s");
    await page.keyboard.press("w"); // Real shield/slow supports the live disengage.
    // A live ranged opponent keeps attacking during a retreat. Use the actual
    // newly equipped dash to disengage before walking back; the old stationary
    // training target never required this survival step.
    const retreat = await state(page);
    const retreatOrigin = hero(retreat).position;
    const laneLength = Math.hypot(...retreat.map.enemy_base.map((value, i) => value - retreat.map.ally_base[i]));
    const laneDirection = retreat.map.enemy_base.map((value, i) => (value - retreat.map.ally_base[i]) / laneLength);
    const lateral = [-laneDirection[1], laneDirection[0]];
    const relative = retreatOrigin.map((value, i) => value - retreat.map.ally_base[i]);
    const forward = relative.reduce((sum, value, i) => sum + value * laneDirection[i], 0);
    const side = Math.sign(relative.reduce((sum, value, i) => sum + value * lateral[i], 0)) || -1;
    // Leave both the hero pursuit strip (5.5) and the minion acquisition
    // radius (8). Returning along the lane would invite another real fight.
    const safePoint = nearHome(retreat, forward, side * (retreat.map.lane_width / 2 + 6));
    const retreatRange = retreat.equipment.slots[0].item.active.range;
    const retreatLength = Math.hypot(...safePoint.map((value, i) => value - retreatOrigin[i]));
    const retreatPoint = retreatOrigin.map((value, i) => value
      + (safePoint[i] - value) / retreatLength * retreatRange);
    await clickUi(page, '[data-equipment-slot="0"]');
    await page.waitForFunction(() => window.__kivaState.equipment_aim === 0);
    await clickWorld(page, retreatPoint);
    await observed(page, (destination) => {
      const snapshot = window.__kivaState;
      const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
      if (!player.alive) throw new Error("Player died before the equipment disengage finished");
      return snapshot.equipment.slots[0].item.active.cooldown > 0
        && Math.hypot(...player.position.map((value, i) => value - destination[i])) < 0.15;
    }, retreatPoint);
    progress("real item dash disengaged from the live opponent", await state(page));
    // A production movement order chooses a safe open part of the map. The
    // max-range dash is measured from the resulting actual hero position.
    await moveTo(page, safePoint);
    await page.keyboard.press("s");
    await page.evaluate(() => window.__laneSetFrameStep(250));
    try {
      await observed(page, () => {
        const snapshot = window.__kivaState;
        const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
        if (!player.alive) throw new Error("Player died before the independent item dash check");
        return snapshot.equipment.slots[0].item.active.cooldown === 0
          && snapshot.equipment.slots[0].item.active.available;
      });
    } finally { await page.evaluate(() => window.__laneSetFrameStep(1000 / 60)); }
    const dashSnapshot = await state(page);
    const beforeDash = hero(dashSnapshot).position;
    const dashRange = dashSnapshot.equipment.slots[0].item.active.range;
    const direction = lateral.map((value) => value * side);
    const destination = beforeDash.map((value, i) => value + direction[i] * dashRange);
    await clickUi(page, '[data-equipment-slot="0"]');
    await page.waitForFunction(() => window.__kivaState.equipment_aim === 0);
    await clickWorld(page, beforeDash.map((value, i) => value + direction[i] * dashRange * 2));
    const dashed = await observed(page, (destination) => {
      const snapshot = window.__kivaState;
      const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
      return Math.hypot(player.position[0] - destination[0], player.position[1] - destination[1]) < 0.12
        && snapshot.equipment.slots[0].item.active.cooldown > 0 ? snapshot : false;
    }, destination);
    assert.equal(dashed.abilities[2].cooldown, 0);
    progress("item far dash reached maximum range; role E unchanged", dashed);
    await page.evaluate(() => {
      function pauseChannel(event) {
        const snapshot = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
        if (snapshot.units.find((unit) => unit.id === snapshot.player_id)?.action !== "Channeling") return;
        window.removeEventListener("kiva-state", pauseChannel);
        document.documentElement.dataset.paused = "true";
        document.exitPointerLock();
      }
      window.addEventListener("kiva-state", pauseChannel);
    });
    await page.keyboard.press("2");
    progress("self item channel submitted");
    await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
    assert.equal((await state(page)).equipment_aim, null, "self equipment must cast without an aim click");
    await openEditor(page);
    await unequip(page, 1);
    assert.equal(hero(await state(page)).action, "Ready", "unequip must cancel future item channel pulses");
    progress("self item channel cancelled by unequip");
    await page.evaluate((instance) => {
      document.documentElement.dataset.equipmentRequests = JSON.stringify([{ type: "Unequip",
        player_id: window.__kivaState.player_id, slot: 0, instance_id: instance, request_id: 900002 }]);
    }, oldBow.instance_id);
    await page.waitForFunction(() => window.__kivaState.equipment.ack.request_id === 900002);
    assert.equal((await state(page)).equipment.ack.accepted, false);
    assert.equal((await state(page)).equipment.slots[0].item.id, 8);
    await page.locator("#equipment-close").click();
  } else {
    await pause(page);
  }
  await resetToSelection(page);
  await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting" && !document.getElementById("enter").disabled);
  assert((await state(page)).equipment.slots.every(({ item }) => item === null));
  assert(await page.locator("#equipment-editor").isHidden());
  await resumeGame(page);
  const fresh = await state(page);
  assert.notEqual(fresh.player_id, original.player_id);
  assert.notEqual(fresh.match.id, original.match.id);
  assert.equal(fresh.match.wave_number, 1);
  assert(fresh.equipment.slots.every(({ item }) => item === null));
  assert.equal(hero(fresh).attributes.attack_damage, 64);
  assert.equal(hero(fresh).attributes.ability_power, 0);
  console.log(`PASS six slots, stacking/removal, full inventory, real lane item input, invalid targets, independent cooldown, pause/reselection${deep ? ", on-hit Slow/unequip, far dash/channel and stale instance" : ""}`);
}

async function main() {
  let browser;
  let site;
  let page;
  const errors = [];
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    const cases = [
      { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 },
      { viewport: { width: 800, height: 600 }, deviceScaleFactor: 1 },
      { viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 },
    ];
    const requested = process.env.KIVA_EQUIPMENT_CASE;
    const selected = cases.filter((config) => !requested
      || `${config.viewport.width}x${config.viewport.height}@${config.deviceScaleFactor}` === requested);
    assert(selected.length, "unknown KIVA_EQUIPMENT_CASE");
    for (const config of selected) {
      // Each WebGPU case owns its browser/device lifetime. A lost software
      // device must fail this case rather than poison the next viewport.
      browser = await launchBrowser();
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      let failBrowser;
      const browserFailure = new Promise((_, reject) => { failBrowser = reject; });
      await installFrameClock(page);
      await page.addInitScript(() => window.addEventListener("kiva-state", () => {
        window.__gearLastSnapshotAt = performance.now();
      }));
      page.on("pageerror", (error) => {
        errors.push(error.message);
        console.error("Browser pageerror:", error.stack || error.message);
        failBrowser(error);
      });
      page.on("console", (message) => {
        // tracing-wasm writes even Rust WARN/ERROR through console.log.
        if (process.env.KIVA_PAN_DIAGNOSTICS === "1" || message.type() === "error"
          || /%c(?:WARN|ERROR)%c/.test(message.text())) {
          console.log(`Browser console ${message.type()}:`, message.text());
        }
        if (message.text().includes("Quitting the application due to")) {
          failBrowser(new Error(message.text()));
        }
      });
      await Promise.race([exercise(page, site.url, config.viewport.width === 1280), browserFailure]);
      await context.close();
      page = null;
      await browser.close();
      browser = null;
      console.log(`PASS equipment viewport ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
    }
    assert.deepEqual(errors, []);
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { if (browser) await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
