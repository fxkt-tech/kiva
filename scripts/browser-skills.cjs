// Real Pointer Lock interaction against a temporary static fixture. State is
// observed through Rust projections; tests never mutate the simulation world.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, enterGame, resumeGame, movePointer, clickPointer, clickAndMovePointer,
  clickUi, clickWorld, worldPoint, reportFailure, serveBuild,
} = require("./browser-support.cjs");

async function status(page, kind) {
  // Return the snapshot that satisfied the condition. A second CDP round trip
  // can otherwise sample after a short control effect has already expired.
  const snapshot = await page.waitForFunction((kind) => {
    const unit = window.__kivaState.units.find((unit) => unit.name === "训练哨兵");
    return unit?.statuses.some((effect) => effect.kind === kind) ? unit : false;
  }, kind);
  try { return await snapshot.jsonValue(); }
  finally { await snapshot.dispose(); }
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

  // Cap elapsed time per native display frame for this short animation only.
  // CDP/software-rendering delays must not consume its remaining control ticks.
  await page.evaluate(() => window.__testUseFrameClock());
  // Virtual time advances per rendered frame. Give slow software GPUs a wall
  // time budget without extending any production control duration/assertion.
  page.setDefaultTimeout(120000);
  // Keep the latest rendered body projection independently of the 80ms HUD.
  assert.notEqual(await page.evaluate(() => window.__kivaState.selected), stunned.id,
    "the airborne click must produce a new selection");
  await page.evaluate(() => {
    window.__testLabels = new Map();
    window.__testSkillTrace = [];
    window.__recordSkillTrace = (event) => {
      const state = window.__kivaState;
      const target = state.units.find((unit) => unit.name === "训练哨兵");
      window.__testSkillTrace.push({ event, time: performance.now(), tick: state.tick,
        selected: state.selected, height: target.visual_height,
        remaining: target.statuses.find((effect) => effect.kind === "Airborne")?.remaining,
        displaced: target.control.is_displaced });
    };
    window.addEventListener("kiva-state", () => {
      window.__testLastSnapshotAt = performance.now();
      if (window.__kivaState.units.some((unit) => unit.statuses.some((effect) => effect.kind === "Airborne"))) {
        window.__recordSkillTrace("airborne-state");
      }
    });
    let pauseAfterPick = false;
    document.addEventListener("pointerdown", (event) => {
      window.__recordSkillTrace("pointerdown");
      if (window.__testPauseOnPick && event.isTrusted && event.button === 0) {
        window.__testPauseOnPick = false;
        pauseAfterPick = true;
      }
    });
    window.addEventListener("kiva-labels", (event) => {
      const labels = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
      for (const label of labels) window.__testLabels.set(label.id, label);
      // This display frame follows consumption of the real pick input. Submit
      // pause now, then release capture; waiting for the throttled HUD loses time.
      if (pauseAfterPick) {
        pauseAfterPick = false;
        window.__recordSkillTrace("pause-after-pick");
        // Use the existing pause intent before releasing the native lock, just
        // as Escape does. Native unlock notification can arrive a frame later.
        document.documentElement.dataset.paused = "true";
        document.exitPointerLock();
      }
    });
  });
  await clickUi(page, '[data-slot="3"]');
  await page.waitForFunction(() => window.__kivaState.aim === 3);
  assert.equal(await page.evaluate(() => window.__kivaState.abilities[3].cooldown), 0,
    "selecting directional R must not self-cast");
  // Pre-position during the projectile's flight, before the short airborne
  // window starts. Four units is the authored demo push distance checked below;
  // y=2.6 intersects the elevated body after that horizontal motion finishes.
  const dx = 3 - before.position[0];
  const dz = -2 - before.position[1];
  const length = Math.hypot(dx, dz);
  const landing = [3 + dx / length * 4, -2 + dz / length * 4];
  // Prepare both projections before committing R. Keep host projection work
  // and the helper's usual held click outside the 800ms control window.
  const castPoint = await worldPoint(page, [3, -2]);
  const airbornePoint = await worldPoint(page, [...landing, 2.6]);
  await movePointer(page, ...castPoint);
  await clickAndMovePointer(page, ...airbornePoint);
  await page.waitForFunction(({ id, point }) => {
    const target = window.__kivaState.units.find((unit) => unit.id === id);
    const cursor = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    // Horizontal displacement must finish before sampling the click point:
    // software rendering can advance a full movement step between CDP calls.
    const ready = target.statuses.some((effect) => effect.kind === "Airborne")
      && !target.control.is_displaced && target.visual_height > 0.3
      && Math.abs(cursor.m41 - point[0]) < 2 && Math.abs(cursor.m42 - point[1]) < 2
      && window.__testLabels.get(id)?.body_screen;
    if (ready) {
      window.__recordSkillTrace("pick-ready");
      window.__testPauseOnPick = true;
    }
    return ready;
  }, { id: stunned.id, point: airbornePoint });
  // Inspect control and damage after the real selection has frozen the pose;
  // reading them through extra host round trips first consumes the click window.
  await clickPointer(page, "left", 0);
  await page.waitForFunction(() => window.__kivaState.paused);
  assert.equal(await page.evaluate(() => window.__kivaState.selected), stunned.id);
  const paused = await page.evaluate((id) => {
    const s = window.__kivaState;
    return { tick: s.tick, clock: performance.now(), target: s.units.find((unit) => unit.id === id) };
  }, stunned.id);
  assert.equal(paused.target.health, 205, "control-only R must not invent damage");
  assert.equal(paused.target.control.can_cast, false);
  assert(paused.target.statuses.some((effect) => effect.kind === "Airborne"),
    "pause must occur while airborne so this verifies the visible frozen pose");
  assert(paused.target.visual_height > 0, "pausing must preserve the already rendered airborne height");
  await page.waitForFunction((pausedAt) => window.__testLastSnapshotAt >= pausedAt + 400, paused.clock);
  const still = await page.evaluate((id) => {
    const s = window.__kivaState;
    return { tick: s.tick, clock: performance.now(), target: s.units.find((unit) => unit.id === id) };
  }, stunned.id);
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
  }, stunned.id);
  const landed = await page.evaluate((id) => window.__kivaState.units.find((unit) => unit.id === id), stunned.id);
  assert(Math.abs(Math.hypot(landed.position[0] - 3, landed.position[1] + 2) - 4) < 0.08);

  const dash = await page.evaluate(() => {
    const s = window.__kivaState;
    return { origin: s.units.find((unit) => unit.id === s.player_id).position, range: s.abilities[2].range };
  });
  const destination = [dash.origin[0] - dash.range, dash.origin[1]];
  await clickUi(page, '[data-slot="2"]');
  await clickWorld(page, [dash.origin[0] - dash.range * 2, dash.origin[1]]);
  await page.waitForFunction((destination) => {
    const s = window.__kivaState;
    const hero = s.units.find((unit) => unit.id === s.player_id);
    return Math.hypot(hero.position[0] - destination[0], hero.position[1] - destination[1]) < 0.1 && s.abilities[2].cooldown > 0;
  }, destination);
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
    const requestedCase = process.env.KIVA_SKILLS_CASE;
    const selected = cases.filter(({ viewport, deviceScaleFactor }) => !requestedCase
      || `${viewport.width}x${viewport.height}@${deviceScaleFactor}` === requestedCase);
    assert(selected.length, `Unknown KIVA_SKILLS_CASE: ${requestedCase}`);
    for (const config of selected) {
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(20000);
      // Preserve native timers, animation delivery and trusted input. Only the
      // monotonic time read by WASM is capped after the short-animation phase
      // starts. Install before Bevy captures performance.now; do not write World.
      await page.addInitScript(() => {
        const nativeNow = performance.now.bind(performance);
        const nativeFrame = requestAnimationFrame.bind(window);
        let controlled = false;
        let time = 0;
        let lastFrame;
        Object.defineProperty(performance, "now", { value: () => controlled ? time : nativeNow() });
        window.__testUseFrameClock = () => { time = nativeNow(); controlled = true; };
        window.requestAnimationFrame = (callback) => nativeFrame((timestamp) => {
          if (controlled && timestamp !== lastFrame) {
            lastFrame = timestamp;
            time += 1000 / 60;
          }
          callback(controlled ? time : timestamp);
        });
      });
      await enterGame(page, site.url);
      await exerciseSkills(page);
      console.log(`PASS Q/W/E/R, invalid enemy aim, composed controls, slow restoration, airborne picking and pause ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
      await context.close();
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
