// Exercise the real simulation snapshot and captured-pointer HUD together.
// Build first; the test owns and closes its ephemeral static HTTP fixture.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { launchBrowser, enterGame, resumeGame, movePointer, clickPointer, clickUi,
  clickWorld, reportFailure, serveBuild } = require("./browser-support.cjs");

const expected = {
  attack_damage: 40, ability_power: 0, armor: 0, magic_resistance: 0,
  move_speed: 5.5, attack_speed: 1.25, attack_range: 5.5,
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
    return {
      attributes: player.attributes,
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
      minimap: box(document.querySelector(".map-wrap")),
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
    assert.equal(Number(hud.values[field]), hud.attributes[field], `${field}: DOM must match the simulation snapshot`);
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
  await clickWorld(page, [-2, 6], "right");
  await page.waitForFunction(() => {
    const state = window.__kivaState;
    const player = state.units.find((unit) => unit.id === state.player_id);
    return Math.hypot(player.position[0] + 2, player.position[1] - 6) < 0.12;
  });
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  assertHud(await observeHud(page), "paused");
  await resumeGame(page);
  assertHud(await observeHud(page), "resumed");
}

async function main() {
  const browser = await launchBrowser();
  let site;
  let page;
  const errors = [];
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    for (const { width, height, scale } of [
      { width: 1280, height: 800, scale: 1 },
      { width: 800, height: 600, scale: 1 },
      { width: 1000, height: 720, scale: 2 },
    ]) {
      const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: scale });
      page = await context.newPage();
      page.setDefaultTimeout(15000);
      page.on("pageerror", (error) => errors.push(error.message));
      await enterGame(page, site.url);
      await page.locator('#hero-stats [data-stat="attack_speed"]').waitFor();
      await nextState(page);
      const scenario = `${width}x${height} DPR=${scale}`;
      assertHud(await observeHud(page), scenario);
      if (width === 1280) await assertUiIsolation(page);
      if (process.env.KIVA_BROWSER_HEADED === "1") {
        const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
        await fs.mkdir(directory, { recursive: true });
        await page.screenshot({ path: path.join(directory, `stats-${width}x${height}-dpr${scale}.png`) });
      }
      await context.close();
      page = null;
      console.log(`PASS effective attribute snapshot, readable HUD and layout at ${scenario}`);
    }
    assert.deepEqual(errors, [], "no uncaught browser errors");
    console.log("PASS captured-pointer HUD isolation, movement, pause and resume");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
