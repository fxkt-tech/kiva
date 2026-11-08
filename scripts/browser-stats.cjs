const { resetToSelection } = require("./browser-support.cjs");
// Exercise the real simulation snapshot and captured-pointer HUD together.
// Build first; the test owns and closes its ephemeral static HTTP fixture.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { launchBrowser, enterGame, resumeGame, selectCharacter, movePointer, clickPointer, clickUi,
  clickWorld, clickUnit, reportFailure, serveBuild, installFrameClock, state, nearHome, pause } = require("./browser-lane-support.cjs");

const expected = {
  attack_damage: 64, ability_power: 0, armor: 36, magic_resistance: 32,
  move_speed: 340, attack_speed: 0.7896, attack_range: 175,
};

async function observeHud(page) {
  return page.evaluate((fields) => {
    const state = window.__kivaState;
    const player = state.units.find((unit) => unit.id === state.player_id);
    function box(element) {
      if (!element || getComputedStyle(element).display === "none") return null;
      const { x, y, width, height } = element.getBoundingClientRect();
      return width && height ? { x, y, width, height } : null;
    }
    const project = (x, y, z) => {
      const m = state.view_projection;
      const w = m[3] * x + m[7] * y + m[11] * z + m[15];
      return [(m[0] * x + m[4] * y + m[8] * z + m[12]) / w * state.viewport[0] / 2 + state.viewport[0] / 2,
        state.viewport[1] / 2 - (m[1] * x + m[5] * y + m[9] * z + m[13]) / w * state.viewport[1] / 2];
    };
    const [x, z] = player.position;
    const bottom = project(x, player.visual_height, z);
    const top = project(x, player.visual_height + 2, z);
    const minimap = document.getElementById("minimap");
    return {
      attributes: player.display_attributes,
      values: Object.fromEntries(fields.map((field) => [field,
        document.querySelector(`[data-stat="${field}"]`)?.textContent])),
      entries: fields.map((field) => box(document.querySelector(`[data-stat="${field}"]`))),
      count: document.querySelectorAll("#hero-stats [data-stat]").length,
      fields: [...document.querySelectorAll("#hero-stats [data-stat]")].map((element) => element.dataset.stat),
      labels: [...document.querySelectorAll("#hero-stats dt")].map((element) => element.textContent.trim()),
      text: document.getElementById("hero-stats").textContent,
      panel: box(document.getElementById("hero-stats")),
      card: box(document.getElementById("hero-hud")),
      abilities: [...document.querySelectorAll(".ability")].map(box),
      skillNames: [...document.querySelectorAll(".ability small")].map(box),
      skillsRegion: box(document.getElementById("abilities")),
      health: box(document.getElementById("health-fill").parentElement),
      mana: box(document.getElementById("mana-fill").parentElement),
      equipment: [...document.querySelectorAll("[data-equipment-slot]")].map(box),
      match: box(document.getElementById("match-hud")),
      minimap: box(document.querySelector(".map-wrap")),
      mapCanvas: { cssWidth: minimap.clientWidth, cssHeight: minimap.clientHeight,
        width: minimap.width, height: minimap.height, dpr: devicePixelRatio },
      footprint: state.camera_footprint?.map(([x, z]) => project(x, 0, z)),
      heroReferenceHeight: Math.abs(top[1] - bottom[1]),
      hint: box(document.querySelector(".camera-hint")),
      viewport: { width: innerWidth, height: innerHeight },
      documentWidth: document.documentElement.scrollWidth,
    };
  }, Object.keys(expected));
}

function inside(inner, outer) {
  return inner && outer && inner.x >= outer.x - 1 && inner.y >= outer.y - 1
    && inner.x + inner.width <= outer.x + outer.width + 1
    && inner.y + inner.height <= outer.y + outer.height + 1;
}

function overlaps(a, b) {
  return a && b && Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x) > 1
    && Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y) > 1;
}

function assertHud(hud, scenario) {
  assert.equal(hud.count, 7, `${scenario}: all seven attributes must remain visible`);
  for (const [field, value] of Object.entries(expected)) {
    assert(Number.isFinite(hud.attributes[field]), `${field} must come from a finite simulation value`);
    assert(Math.abs(hud.attributes[field] - value) < 0.0001, `${field} must preserve its default behavior`);
    assert(Math.abs(Number(hud.values[field])-hud.attributes[field])<0.051, `${field}: DOM must match rounded display attributes`);
  }
  assert.deepEqual(hud.labels, ["攻击", "法强", "护甲", "魔抗", "移速", "攻速", "攻距"], "labels must match the requested order");
  assert.deepEqual(hud.fields, Object.keys(expected), "short labels must stay paired with the correct attributes");
  assert(!/单位|次\/秒/.test(hud.text), "attribute values must not display units");
  const viewport = { x: 0, y: 0, ...hud.viewport };
  assert(inside(hud.card, viewport), `${scenario}: the card must fit the viewport`);
  assert(inside(hud.panel, hud.card), `${scenario}: attributes belong inside the hero card`);
  assert(hud.panel.width <= 145, `${scenario}: the attribute panel must keep its compact 144px width`);
  assert(hud.panel.x < Math.min(...hud.abilities.map((box) => box.x)), "attributes must be on the left of the skill controls");
  for (const entry of hud.entries) assert(inside(entry, hud.panel), `${scenario}: each attribute must fit its panel`);
  for (const button of hud.abilities) {
    assert(inside(button, hud.card), `${scenario}: skill controls must fit the card`);
    assert(!overlaps(hud.panel, button), `${scenario}: attributes must not cover skill controls`);
  }
  assert(!overlaps(hud.card, hud.minimap), `${scenario}: the card must not overlap the minimap`);
  assert(!overlaps(hud.card, hud.hint), `${scenario}: the camera hint must remain readable`);
  assert(hud.documentWidth <= hud.viewport.width + 1, `${scenario}: no horizontal page overflow`);
  assert(hud.card.height <= 122, `${scenario}: the bottom action bar must stay compact`);
  assert(hud.viewport.height - hud.card.y - hud.card.height <= 10,
    `${scenario}: the action bar belongs against the bottom edge`);
  assert(inside(hud.minimap, viewport), `${scenario}: the minimap must remain visible inside the viewport`);
  assert(hud.viewport.width - hud.minimap.x - hud.minimap.width <= 12,
    `${scenario}: the minimap belongs against the right edge`);
  assert(hud.viewport.height - hud.minimap.y - hud.minimap.height <= 12,
    `${scenario}: the minimap must stay at the bottom, rather than above the action bar`);
  assert(hud.match.height <= 56, `${scenario}: the top match display must stay compact`);
  assert(hud.health.y >= hud.skillsRegion.y + hud.skillsRegion.height - 1,
    `${scenario}: health must be below the skill row`);
  assert(hud.mana.y >= hud.health.y + hud.health.height - 1,
    `${scenario}: mana must be below health`);
  for (const name of hud.skillNames) {
    assert(inside(name, hud.card), `${scenario}: skill names must remain readable inside the action bar`);
    assert(!overlaps(name, hud.health), `${scenario}: resource bars must not cover skill names`);
  }
  assert.equal(hud.equipment.length, 6);
  for (const slot of hud.equipment) {
    assert(inside(slot, hud.card), `${scenario}: all equipment slots must stay inside the action bar`);
    assert(slot.width >= 24 && slot.height >= 24, `${scenario}: equipment controls must remain usable`);
    assert(!overlaps(slot, hud.health) && !overlaps(slot, hud.mana), `${scenario}: equipment must not cover resources`);
  }
  assert.equal(new Set(hud.equipment.map((slot) => Math.round(slot.x))).size, 3,
    `${scenario}: equipment must have three columns`);
  assert.equal(new Set(hud.equipment.map((slot) => Math.round(slot.y))).size, 2,
    `${scenario}: equipment must have two rows`);
  const fraction = hud.heroReferenceHeight / hud.viewport.height;
  assert(fraction >= 0.075 && fraction <= 0.13,
    `${scenario}: default two-meter hero framing must be close to the reference (${fraction})`);
  assert.equal(hud.footprint?.length, 4, `${scenario}: actual camera footprint is required`);
  const corners = [[0, 0], [hud.viewport.width, 0], [hud.viewport.width, hud.viewport.height], [0, hud.viewport.height]];
  for (let index = 0; index < 4; index++) {
    assert(Math.hypot(...hud.footprint[index].map((value, axis) => value - corners[index][axis])) < 2,
      `${scenario}: camera footprint must project back to the viewport corner`);
  }
  assert(Math.abs(hud.mapCanvas.width / hud.mapCanvas.cssWidth - hud.mapCanvas.dpr) < 0.05,
    `${scenario}: minimap backing resolution must follow display density`);
}

async function nextState(page) {
  return page.evaluate(() => new Promise((resolve) => {
    window.addEventListener("kiva-state", (event) => resolve(JSON.parse(event.detail)), { once: true });
  }));
}

async function assertUiIsolation(page) {
  await page.keyboard.press("s");
  await page.waitForFunction(() => {
    const state = window.__kivaState;
    return state.units.find((unit) => unit.id === state.player_id).order === "Idle";
  });
  const before = await page.evaluate(() => {
    const state = window.__kivaState;
    return state.units.find((unit) => unit.id === state.player_id).position;
  });
  await clickUi(page, '[data-slot="0"]');
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  const box = await page.locator('#hero-stats [data-stat="attack_damage"]').boundingBox();
  await movePointer(page, box.x + box.width / 2, box.y + box.height / 2);
  await clickPointer(page);
  await clickPointer(page, "right");
  await nextState(page);
  const after = await nextState(page);
  const player = after.units.find((unit) => unit.id === after.player_id);
  assert.equal(player.order, "Idle", "stat clicks must not issue a world command");
  assert.equal(after.aim, 0, "stat clicks must not cast or cancel an aimed skill");
  assert.equal(after.abilities[0].cooldown, 0, "stat clicks must not commit the skill");
  assert(Math.hypot(...player.position.map((value, i) => value - before[i])) < 0.01,
    "stat clicks must not move the hero");
  assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game");
  await page.keyboard.press("s");
  const destination = nearHome(await state(page), 2.8, -1.8);
  await clickWorld(page, destination, "right");
  await page.waitForFunction(([x, z]) => {
    const snapshot = window.__kivaState;
    const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
    return Math.hypot(player.position[0] - x, player.position[1] - z) < 0.12;
  }, destination);
  await pause(page);
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  assertHud(await observeHud(page), "paused");
  await resumeGame(page);
  assertHud(await observeHud(page), "resumed");
}

async function exerciseCompactControls(page) {
  const original = await state(page);
  const passive = await page.locator("#hero-passive").boundingBox();
  await movePointer(page, passive.x + passive.width / 2, passive.y + passive.height / 2);
  await page.waitForFunction(() => {
    const s = window.__kivaState;
    const p = s.units.find((u) => u.id === s.player_id);
    const detail = document.getElementById("ability-detail");
    return !detail.hidden && p.passives.every((text) => detail.textContent.includes(text));
  });
  await clickUi(page, "#hero-passive");
  assert.equal((await state(page)).aim, null, "passive controls must not select an active ability");
  await clickUi(page, "#equipment-toggle");
  await page.waitForFunction(() => window.__kivaState.equipment.can_manage
    && !document.getElementById("equipment-editor").hidden && document.pointerLockElement?.id === "game"
    && !window.__kivaState.paused);
  await clickUi(page, '[data-equip-id="8"]');
  await page.waitForFunction(() => window.__kivaState.equipment.slots[0].item?.id === 8
    && document.querySelector('[data-stat="attack_damage"]').textContent === "74");
  await clickUi(page, "#equipment-close");
  assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game");
  assert.equal((await state(page)).player_id, original.player_id);
  await clickUi(page, '[data-equipment-slot="0"]');
  await page.waitForFunction(() => window.__kivaState.equipment_aim === 0);
  const start = await state(page);
  const player = start.units.find((u) => u.id === start.player_id);
  const delta = start.map.enemy_base.map((v, i) => v - start.map.ally_base[i]);
  const length = Math.hypot(...delta);
  const goal = player.position.map((v, i) => v + delta[i] / length * 2.5);
  await clickWorld(page, goal);
  await page.waitForFunction((goal) => {
    const s = window.__kivaState;
    const p = s.units.find((u) => u.id === s.player_id);
    return s.equipment.slots[0].item.active.cooldown > 0
      && Math.hypot(...p.position.map((v, i) => v - goal[i])) < 0.15;
  }, goal);
  assert.equal((await state(page)).abilities[0].cooldown, 0, "item controls retain an independent cast source");
  console.log("PASS passive hover, compact equipment control and real item cast");
}

async function exerciseReselection(page) {
  const before = await state(page);
  await pause(page);
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  await page.locator('[data-settings-tab="characters"]').click();
  await resetToSelection(page);
  await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting"
    && !document.getElementById("enter").disabled);
  assert(await page.locator("#hero-hud").isHidden(), "the new layout must hide the previous hero during selection");
  await selectCharacter(page, 2);
  await resumeGame(page);
  const after = await state(page);
  assert.notEqual(after.player_id, before.player_id);
  assert(after.equipment.slots.every(({item}) => item === null));
  assert.equal(after.units.find((u) => u.id === after.player_id).name, "寒冰射手");
  assert.equal(await page.locator("#hero-name").textContent(), "寒冰射手");
  assert.equal(await page.locator('[data-slot="0"] small').textContent(), "射手的专注");
  assert(await page.locator(".map-wrap").isVisible());
  console.log("PASS compact HUD and minimap through real hero reselection");
}

async function main() {
  let browser;
  let site;
  let page;
  const errors = [];
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    const cases = [
      { width: 1920, height: 1080, scale: 1 },
      { width: 1280, height: 800, scale: 1 },
      { width: 800, height: 600, scale: 1 },
      { width: 1000, height: 720, scale: 2 },
    ];
    const requested = process.env.KIVA_STATS_CASE;
    const selected = cases.filter(({width,height,scale}) => !requested || `${width}x${height}@${scale}` === requested);
    assert(selected.length, "unknown KIVA_STATS_CASE");
    for (const { width, height, scale } of selected) {
      browser = await launchBrowser();
      const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: scale });
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      await installFrameClock(page);
      page.on("pageerror", (error) => errors.push(error.message));
      await enterGame(page, site.url);
      await page.locator('#hero-stats [data-stat="attack_speed"]').waitFor();
      await nextState(page);
      await page.waitForFunction(() => window.__kivaState.camera_footprint?.length === 4);
      const scenario = `${width}x${height} DPR=${scale}`;
      assertHud(await observeHud(page), scenario);
      if (width === 1280) await assertUiIsolation(page);
      if (width === 1280) {
        await clickUnit(page, (await state(page)).player_id);
        await page.waitForFunction(() => !document.getElementById("target-panel").hidden);
        const target = await page.locator("#target-panel").boundingBox();
        assert(target.x <= 16 && target.y <= 16, "selected target belongs in the top-left corner");
        await exerciseCompactControls(page);
      }
      if (process.env.KIVA_BROWSER_HEADED === "1") {
        const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
        await fs.mkdir(directory, { recursive: true });
        await page.screenshot({ path: path.join(directory, `stats-${width}x${height}-dpr${scale}.png`) });
      }
      if (width === 1280) await exerciseReselection(page);
      await context.close();
      page = null;
      await browser.close();
      browser = null;
      console.log(`PASS effective attribute snapshot, readable HUD and layout at ${scenario}`);
    }
    assert.deepEqual(errors, [], "no uncaught browser errors");
    console.log("PASS captured-pointer HUD isolation, movement, pause and resume");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { if (browser) await browser.close(); }
    finally { if (site) await site.close(); }
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
