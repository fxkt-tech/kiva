// Real Pointer Lock interaction against a temporary static fixture. State is
// observed through Rust projections; tests never mutate the simulation world.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, enterGame, resumeGame, movePointer, clickPointer,
  clickUi, clickWorld, worldPoint, reportFailure, serveBuild,
} = require("./browser-support.cjs");

async function status(page, kind) {
  await page.waitForFunction((kind) => window.__kivaState.units.some((unit) =>
    unit.name === "训练哨兵" && unit.statuses.some((effect) => effect.kind === kind)), kind);
  return page.evaluate(() => window.__kivaState.units.find((unit) => unit.name === "训练哨兵"));
}

async function exerciseSkills(page) {
  const before = await page.evaluate(() => {
    const s = window.__kivaState;
    const hero = s.units.find((unit) => unit.id === s.player_id);
    return { position: hero.position, mana: hero.mana, names: s.abilities.map((ability) => ability.name) };
  });
  const buttons = await page.locator("[data-slot] small").allTextContents();
  for (let slot = 0; slot < before.names.length; slot++) {
    assert((await page.locator(`[data-slot="${slot}"]`).getAttribute("title")).includes(before.names[slot]));
  }
  assert.deepEqual(buttons, before.names, "button labels must follow the equipped definitions");

  // An enemy-target ability must remain selected for empty and friendly clicks.
  // Holding a HUD click across updates also checks document-focus handling.
  await clickUi(page, '[data-slot="0"]', 1200);
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  await clickWorld(page, [0, 2]);
  await page.waitForTimeout(180);
  assert.equal(await page.evaluate(() => window.__kivaState.aim), 0);
  await clickWorld(page, [...before.position, 1.1]);
  await page.waitForTimeout(180);
  assert.equal(await page.evaluate(() => window.__kivaState.aim), 0);
  assert.equal(await page.evaluate(() => window.__kivaState.abilities[0].cooldown), 0);
  await clickWorld(page, [3, -2, 1.1]);
  const stunned = await status(page, "Stun");
  assert.equal(stunned.health, 325);
  assert.equal(stunned.control.can_move, false);
  assert.equal(stunned.control.can_attack, false);
  assert.equal(stunned.control.can_cast, false);
  assert.equal(await page.evaluate(() => window.__kivaState.aim), null);
  const afterQ = await page.evaluate(() => {
    const s = window.__kivaState;
    return { position: s.units.find((unit) => unit.id === s.player_id).position, cooldown: s.abilities[0].cooldown, paused: s.paused };
  });
  assert.equal(afterQ.paused, false);
  assert(afterQ.cooldown > 0);
  assert(Math.hypot(...afterQ.position.map((value, i) => value - before.position[i])) < 0.1,
    "HUD and invalid target clicks must not issue walking orders");

  await clickUi(page, '[data-slot="1"]');
  await clickWorld(page, [3, -2]);
  const rooted = await status(page, "Root");
  assert.equal(rooted.health, 205);
  assert(rooted.statuses.some((effect) => effect.kind === "Slow"));
  assert.equal(rooted.control.can_move, false);
  assert.equal(rooted.control.can_attack, true);
  assert.equal(rooted.control.can_cast, true);
  assert(Math.abs(rooted.attributes.move_speed - 3.85) < 0.001,
    "status and final movement speed must agree in the same published snapshot");
  await page.waitForFunction(() => {
    const target = window.__kivaState.units.find((unit) => unit.name === "训练哨兵");
    return !target.statuses.some((effect) => effect.kind === "Root")
      && target.statuses.some((effect) => effect.kind === "Slow") && target.control.can_move;
  });
  await page.waitForFunction(() => {
    const target = window.__kivaState.units.find((unit) => unit.name === "训练哨兵");
    return target.statuses.length === 0 && target.attributes.move_speed === 5.5;
  });

  // Keep the latest rendered body projection independently of the 80ms HUD.
  assert.notEqual(await page.evaluate(() => window.__kivaState.selected), stunned.id,
    "the airborne click must produce a new selection");
  await page.evaluate(() => {
    window.__testLabels = new Map();
    window.addEventListener("kiva-labels", (event) => {
      const labels = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
      for (const label of labels) window.__testLabels.set(label.id, label);
    });
    // Freeze via the real capture-loss path as soon as selection is published.
    // Host/CDP round trips must not consume an 800ms animation before pausing.
    let pending = true;
    window.addEventListener("kiva-state", (event) => {
      const state = JSON.parse(event.detail);
      const target = state.units.find((unit) => unit.name === "训练哨兵");
      if (pending && state.selected === target?.id && target.visual_height > 0
        && target.statuses.some((effect) => effect.kind === "Airborne")) {
        pending = false;
        document.exitPointerLock();
      }
    });
  });
  await clickUi(page, '[data-slot="3"]');
  await page.waitForFunction(() => window.__kivaState.aim === 3);
  assert.equal(await page.evaluate(() => window.__kivaState.abilities[3].cooldown), 0,
    "selecting directional R must not self-cast");
  await clickWorld(page, [3, -2]);
  // Pre-position during the projectile's flight, before the short airborne
  // window starts. Four units is the authored demo push distance checked below;
  // y=2.6 intersects the elevated body after that horizontal motion finishes.
  const dx = 3 - before.position[0];
  const dz = -2 - before.position[1];
  const length = Math.hypot(dx, dz);
  const landing = [3 + dx / length * 4, -2 + dz / length * 4];
  await movePointer(page, ...await worldPoint(page, [...landing, 2.6]));
  const airborne = await status(page, "Airborne");
  assert.equal(airborne.health, 205, "control-only R must not invent damage");
  assert.equal(airborne.control.can_cast, false);
  await page.waitForFunction((id) => {
    const target = window.__kivaState.units.find((unit) => unit.id === id);
    // Horizontal displacement must finish before sampling the click point:
    // software rendering can advance a full movement step between CDP calls.
    return !target.control.is_displaced && target.visual_height > 0.3
      && window.__testLabels.get(id)?.body_screen;
  }, airborne.id);
  await clickPointer(page, "left", 0);
  await page.waitForFunction(() => window.__kivaState.paused);
  assert.equal(await page.evaluate(() => window.__kivaState.selected), airborne.id);
  const paused = await page.evaluate((id) => {
    const s = window.__kivaState;
    return { tick: s.tick, target: s.units.find((unit) => unit.id === id) };
  }, airborne.id);
  assert(paused.target.statuses.some((effect) => effect.kind === "Airborne"),
    "pause must occur while airborne so this verifies the visible frozen pose");
  assert(paused.target.visual_height > 0, "pausing must preserve the already rendered airborne height");
  await page.waitForTimeout(400);
  const still = await page.evaluate((id) => {
    const s = window.__kivaState;
    return { tick: s.tick, target: s.units.find((unit) => unit.id === id) };
  }, airborne.id);
  assert.equal(still.tick, paused.tick);
  assert.equal(still.target.visual_height, paused.target.visual_height);
  assert.deepEqual(still.target.statuses, paused.target.statuses);
  assert.deepEqual(still.target.screen, paused.target.screen);
  assert(await page.locator('#target-statuses [data-status="Airborne"]').count() > 0);
  if (process.env.KIVA_BROWSER_HEADED === "1") {
    const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
    await fs.mkdir(directory, { recursive: true });
    const size = page.viewportSize();
    const dpr = await page.evaluate(() => devicePixelRatio);
    // Inspect the frozen scene; restore the real pause overlay immediately.
    const inspectionStyle = await page.addStyleTag({ content: "#entry { visibility: hidden !important; }" });
    try {
      await page.screenshot({ path: path.join(directory, `airborne-${size.width}x${size.height}-dpr${dpr}.png`) });
    } finally {
      await inspectionStyle.evaluate((element) => element.remove());
    }
  }
  await resumeGame(page);
  await page.waitForFunction((id) => {
    const target = window.__kivaState.units.find((unit) => unit.id === id);
    return target.statuses.length === 0 && target.visual_height === 0 && !target.control.is_displaced;
  }, airborne.id);
  const landed = await page.evaluate((id) => window.__kivaState.units.find((unit) => unit.id === id), airborne.id);
  assert(Math.abs(Math.hypot(landed.position[0] - 3, landed.position[1] + 2) - 4) < 0.08);

  await clickUi(page, '[data-slot="2"]');
  await clickWorld(page, [-3, 6]);
  await page.waitForFunction(() => {
    const s = window.__kivaState;
    const hero = s.units.find((unit) => unit.id === s.player_id);
    return Math.hypot(hero.position[0] + 3, hero.position[1] - 6) < 0.1 && s.abilities[2].cooldown > 0;
  });
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
    for (const config of cases) {
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(20000);
      await enterGame(page, site.url);
      await exerciseSkills(page);
      console.log(`PASS Q/W/E/R, invalid enemy aim, composed controls, slow restoration, airborne picking and pause ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
      await context.close();
    }
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
