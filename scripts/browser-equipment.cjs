// Real equipment management and combat through the production input adapters.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { launchBrowser, serveBuild, enterGame, resumeGame, clickUi, clickWorld, reportFailure } = require("./browser-support.cjs");
const state = (page) => page.evaluate(() => window.__kivaState);
const hero = (snapshot) => snapshot.units.find((unit) => unit.id === snapshot.player_id);

async function observed(page, predicate, argument) {
  const result = await page.waitForFunction(predicate, argument);
  try { return await result.jsonValue(); }
  finally { await result.dispose(); }
}

async function openEditor(page) {
  await page.keyboard.press("p");
  await page.waitForFunction(() => !document.getElementById("equipment-editor").hidden
    && window.__kivaState.equipment.can_manage && !document.pointerLockElement);
}

async function equip(page, id) {
  const before = await state(page);
  const slot = before.equipment.first_empty_slot;
  assert.notEqual(slot, null);
  const previous = before.equipment.ack.request_id;
  await page.locator(`[data-equip-id="${id}"]`).click();
  return observed(page, ({ slot, id, previous }) => {
    const snapshot = window.__kivaState;
    return snapshot.equipment.slots[slot]?.item?.id === id
      && snapshot.equipment.ack.accepted && snapshot.equipment.ack.request_id !== previous ? snapshot : false;
  }, { slot, id, previous });
}

async function unequip(page, slot) {
  await page.locator(`[data-unequip-slot="${slot}"]`).click();
  return observed(page, (slot) => {
    const snapshot = window.__kivaState;
    return snapshot.equipment.slots[slot].item === null && snapshot.equipment.ack.accepted ? snapshot : false;
  }, slot);
}

async function continueTrial(page) {
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

async function attackOnceAndPause(page, expectedHealth) {
  await page.evaluate(() => {
    const before = window.__kivaState.units.find((unit) => unit.name === "训练哨兵").health;
    function pauseAfterHit(event) {
      const snapshot = JSON.parse(event.detail);
      if (snapshot.units.find((unit) => unit.name === "训练哨兵").health >= before) return;
      window.removeEventListener("kiva-state", pauseAfterHit);
      document.documentElement.dataset.paused = "true";
      document.exitPointerLock();
    }
    window.addEventListener("kiva-state", pauseAfterHit);
  });
  await clickWorld(page, [3, -2, 1.1], "right");
  const paused = await observed(page, () => window.__kivaState.paused && !document.pointerLockElement ? window.__kivaState : false);
  const target = paused.units.find((unit) => unit.name === "训练哨兵");
  assert(Math.abs(target.health - expectedHealth) < 0.002, `one attack must leave ${expectedHealth} HP, got ${target.health}`);
  return paused;
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
  await enterGame(page, url);
  const original = await state(page);
  assert.equal(original.equipment.slots.length, 6);
  assert(original.equipment.slots.every(({ item }) => item === null));
  assert.equal((await page.evaluate(() => window.__kivaEquipmentCatalog)).length, 9);
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
    await page.locator('[data-equip-id="1"]').click();
    await page.waitForFunction(() => window.__kivaState.equipment.ack.error
      && !document.querySelector('[data-equip-id="1"]').disabled);
    assert((await state(page)).equipment.slots.every(({ item }) => item === null));
  }
  await equip(page, 1);
  const duplicate = await equip(page, 1);
  assert.equal(hero(duplicate).attributes.attack_damage, 80);
  assert.notEqual(duplicate.equipment.slots[0].item.instance_id, duplicate.equipment.slots[1].item.instance_id);
  assert.equal(hero(await unequip(page, 0)).attributes.attack_damage, 60);
  await equip(page, 2);
  for (const id of [4, 5, 6, 7]) await equip(page, id);
  const filled = await state(page);
  assert.equal(filled.equipment.first_empty_slot, null);
  assert.equal(await page.locator('[data-equip-id]:enabled').count(), 0);
  assert.deepEqual(hero(filled).attributes, { attack_damage: 60, ability_power: 25, armor: 25,
    magic_resistance: 20, move_speed: 6.25, attack_range: 6, attack_speed: 1.5625 });
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
  await continueTrial(page);
  assert.equal((await state(page)).equipment_aim, null, "a valid stale equipment click must be discarded at capture");
  assert.equal((await state(page)).equipment.slots[5].item.active.cooldown, 0);
  await layout(page);
  await screenshot(page, "equipment-combat");
  if (deep) await page.keyboard.press("6");
  else await clickUi(page, '[data-equipment-slot="5"]');
  await page.waitForFunction(() => window.__kivaState.equipment_aim === 5);
  await clickWorld(page, [0, 2]);
  assert.equal((await state(page)).equipment_aim, 5, "enemy equipment must keep aim on invalid ground");
  await clickWorld(page, [3, -2, 1.1]);
  const hit = await observed(page, () => {
    const snapshot = window.__kivaState;
    return snapshot.units.some((unit) => unit.name === "训练哨兵" && unit.health === 310)
      && snapshot.equipment.slots[5].item.active.cooldown > 0 ? snapshot : false;
  });
  assert.equal(hit.abilities[0].cooldown, 0, "equipment must not consume the matching hero ability cooldown");
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  const cooldown = (await state(page)).equipment.slots[5].item.active.cooldown;
  await page.waitForTimeout(250);
  assert.equal((await state(page)).equipment.slots[5].item.active.cooldown, cooldown);
  await resumeGame(page);
  assert.equal((await state(page)).player_id, original.player_id);

  if (deep) {
    await page.keyboard.press("q");
    await page.waitForFunction(() => window.__kivaState.aim === 0);
    await clickWorld(page, [3, -2, 1.1]);
    await page.waitForFunction(() => window.__kivaState.units.some((unit) => unit.name === "训练哨兵" && unit.health === 220));
    const struck = await attackOnceAndPause(page, 145.5);
    assert(struck.units.find((unit) => unit.name === "训练哨兵").statuses.some((effect) => effect.kind === "Slow"));
    await openEditor(page);
    await unequip(page, 4);
    await continueTrial(page);
    await attackOnceAndPause(page, 85.5);
    await openEditor(page);
    const oldBow = (await state(page)).equipment.slots[0].item;
    await unequip(page, 0);
    await equip(page, 8);
    await unequip(page, 1);
    await equip(page, 9);
    await continueTrial(page);
    const beforeDash = hero(await state(page)).position;
    const destination = [beforeDash[0] - 1.5, beforeDash[1]];
    await clickUi(page, '[data-equipment-slot="0"]');
    await page.waitForFunction(() => window.__kivaState.equipment_aim === 0);
    await clickWorld(page, destination);
    const dashed = await observed(page, (destination) => {
      const snapshot = window.__kivaState;
      const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
      return Math.hypot(player.position[0] - destination[0], player.position[1] - destination[1]) < 0.12
        && snapshot.equipment.slots[0].item.active.cooldown > 0 ? snapshot : false;
    }, destination);
    assert.equal(dashed.abilities[2].cooldown, 0);
    await page.evaluate(() => {
      function pauseChannel(event) {
        const snapshot = JSON.parse(event.detail);
        if (snapshot.units.find((unit) => unit.id === snapshot.player_id)?.action !== "Channeling") return;
        window.removeEventListener("kiva-state", pauseChannel);
        document.documentElement.dataset.paused = "true";
        document.exitPointerLock();
      }
      window.addEventListener("kiva-state", pauseChannel);
    });
    await page.keyboard.press("2");
    await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
    assert.equal((await state(page)).equipment_aim, null, "self equipment must cast without an aim click");
    await openEditor(page);
    await unequip(page, 1);
    assert.equal(hero(await state(page)).action, "Ready", "unequip must cancel future item channel pulses");
    await page.evaluate((instance) => {
      document.documentElement.dataset.equipmentRequests = JSON.stringify([{ type: "Unequip",
        player_id: window.__kivaState.player_id, slot: 0, instance_id: instance, request_id: 900002 }]);
    }, oldBow.instance_id);
    await page.waitForFunction(() => window.__kivaState.equipment.ack.request_id === 900002);
    assert.equal((await state(page)).equipment.ack.accepted, false);
    assert.equal((await state(page)).equipment.slots[0].item.id, 8);
    await page.locator("#equipment-close").click();
  } else {
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  }
  await page.locator("#reselect").click();
  await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting" && !document.getElementById("enter").disabled);
  assert((await state(page)).equipment.slots.every(({ item }) => item === null));
  assert(await page.locator("#equipment-editor").isHidden());
  await resumeGame(page);
  const fresh = await state(page);
  assert.notEqual(fresh.player_id, original.player_id);
  assert(fresh.equipment.slots.every(({ item }) => item === null));
  assert.equal(hero(fresh).attributes.attack_damage, 40);
  assert.equal(hero(fresh).attributes.ability_power, 0);
  console.log(`PASS six slots, stacking/removal, full inventory, real active input, independent cooldown, pause/restart${deep ? ", passive damage and shared dash/channel" : ""}`);
}

async function main() {
  const browser = await launchBrowser();
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
    for (const [index, config] of cases.entries()) {
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(20000);
      page.on("pageerror", (error) => errors.push(error.message));
      await exercise(page, site.url, index === 0);
      await context.close();
      page = null;
      console.log(`PASS equipment viewport ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
    }
    assert.deepEqual(errors, []);
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
