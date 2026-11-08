// Exercise actual packaged GLBs/images and the hero death lifecycle through
// normal controls. The fixture clock advances every simulation step; it does
// not replace the separate native-clock healthbar/performance checks.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, enterGame, resumeGame, selectCharacter, installFrameClock,
  state, hero, pause, clickWorld, movePointer, clickPointer, nearHome, reportFailure, serveBuild,
} = require("./browser-lane-support.cjs");

async function modelsReady(page, characterId) {
  await page.waitForFunction((id) => {
    const s = window.__kivaState;
    const player = s?.units.find((unit) => unit.id === s.player_id);
    return player?.character_id === id && player.model?.ready && !player.model.error
      && s.units.filter((unit) => unit.team === "ally" && unit.role !== "Base")
        .every((unit) => unit.model?.ready && !unit.model.error);
  }, characterId);
  const snapshot = await state(page);
  assert.equal(hero(snapshot).model.skin, characterId === 1 ? "Original" : "PROJECT");
  assert.equal(hero(snapshot).model.name, characterId === 1 ? "Jarvan IV" : "Ashe");
  assert(hero(snapshot).model.animation, "the ready model must report its actual clip");
  await page.waitForFunction(() => {
    const images = [...document.querySelectorAll(".ability-art, #hero-portrait")];
    return images.length === 5 && images.every((image) => image.complete && image.naturalWidth > 0);
  });
  const icons = await page.locator(".ability-art").evaluateAll((images) => images.map((image) => image.getAttribute("src")));
  assert.equal(new Set(icons).size, 4, "each actual skill has its own packaged icon");
  const portraitBounds = await page.locator("#hero-portrait").evaluate((image) => {
    const rect = image.getBoundingClientRect();
    const parent = image.parentElement.getBoundingClientRect();
    return { width: rect.width, height: rect.height, parentWidth: parent.width, parentHeight: parent.height };
  });
  assert(portraitBounds.width <= portraitBounds.parentWidth && portraitBounds.height <= portraitBounds.parentHeight,
    "the tall source portrait must stay inside its existing HUD frame");
  // Select the actual loaded skinned hero through production geometry. This
  // complements the enemy-base qualification checks in browser-picking.
  const player = hero(snapshot);
  assert(player.body_screen, "the displayed model has an on-screen body anchor");
  await movePointer(page, ...player.body_screen);
  await page.waitForFunction((id) => window.__kivaState.hovered === id, player.id);
  await clickPointer(page, "left", 0);
  await page.waitForFunction((id) => window.__kivaState.selected === id, player.id);
  return snapshot;
}

async function captureDeath(page, output) {
  await page.evaluate(() => {
    const capture = (event) => {
      const snapshot = JSON.parse(event.detail);
      const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
      if (!player || player.alive || !(player.respawn_remaining > 0.6 && player.respawn_remaining <= 1.8)) return;
      window.__presentationDeath = snapshot;
      document.documentElement.dataset.paused = "true";
      document.exitPointerLock();
      window.removeEventListener("kiva-state", capture);
    };
    window.addEventListener("kiva-state", capture);
  });
  const initial = await state(page);
  await clickWorld(page, nearHome(initial, 35, 0), "right");
  await page.keyboard.press("y");
  await page.evaluate(() => window.__laneSetFrameStep(250));
  await page.waitForFunction(() => Boolean(window.__presentationDeath) && window.__kivaState.paused);
  const dead = await state(page);
  const body = hero(dead);
  assert.equal(body.alive, false);
  assert.equal(body.model.ready, true);
  assert.equal(body.model.animation, "Death");
  assert(body.respawn_remaining > 0 && body.respawn_remaining <= 1.8);
  assert.equal(await page.locator("#respawn-status").isVisible(), true);
  assert.match(await page.locator("#respawn-status").textContent(), /复活倒计时 [12] 秒/);
  // Hide only the existing paused overlay to inspect the corpse screenshot;
  // no state, animation or time is changed by this presentation-only step.
  await page.locator("#entry").evaluate((element) => { element.style.visibility = "hidden"; });
  await page.screenshot({ path: path.join(output, "project-ashe-corpse.png") });
  await page.locator("#entry").evaluate((element) => { element.style.visibility = ""; });
  await page.waitForFunction((tick) => window.__laneDiagnostics.frames > tick, await page.evaluate(() => window.__laneDiagnostics.frames + 4));
  const frozen = await state(page);
  assert.equal(frozen.tick, dead.tick);
  assert.equal(hero(frozen).respawn_remaining, body.respawn_remaining);
  assert.deepEqual(hero(frozen).position, body.position);
  await resumeGame(page);
  await page.waitForFunction(() => {
    const s = window.__kivaState;
    const player = s.units.find((unit) => unit.id === s.player_id);
    return player?.alive && player.respawn_remaining === null;
  });
  const reborn = await state(page);
  assert.equal(reborn.player_id, dead.player_id, "respawn keeps the authoritative unit");
  assert(Math.hypot(...hero(reborn).position.map((value, index) => value - reborn.map.ally_base[index])) < 4);
  assert.equal(await page.locator("#respawn-status").isHidden(), true);
  await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
  console.log("PASS real combat death, retained corpse, authoritative paused countdown and respawn", JSON.stringify({ tick: dead.tick, remaining: body.respawn_remaining, model: body.model }));
}

async function caseRun(browser, site, sample, output) {
  const page = await browser.newPage({ viewport: { width: sample.width, height: sample.height }, deviceScaleFactor: sample.dpr });
  page.setDefaultTimeout(180000);
  const errors = [];
  const assets = new Set();
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => {
    if (response.url().includes("/assets/") && response.status() === 200) assets.add(response.url());
  });
  try {
    await installFrameClock(page);
    await enterGame(page, site.url, sample.character);
    await modelsReady(page, sample.character);
    const name = `${sample.width}x${sample.height}-dpr${sample.dpr}`;
    await page.screenshot({ path: path.join(output, `models-${name}.png`) });
    assert([...assets].some((url) => url.endsWith(".glb")), "the browser must fetch real model files");
    if (sample.death) await captureDeath(page, output);
    await pause(page);
    const old = (await state(page)).player_id;
    await page.locator("#reselect").click();
    await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting" && window.__kivaState.units.length === 0);
    await selectCharacter(page, sample.character === 1 ? 2 : 1);
    await resumeGame(page);
    const fresh = await modelsReady(page, sample.character === 1 ? 2 : 1);
    assert.notEqual(fresh.player_id, old);
    assert(!fresh.units.some((unit) => unit.id === old));
    assert.equal(await page.locator("#respawn-status").isHidden(), true);
    assert.deepEqual(errors, []);
    console.log("PASS packaged models/icons and clean reselection", name);
  } catch (error) {
    await reportFailure(page, errors);
    throw error;
  } finally { await page.close(); }
}

async function main() {
  const output = process.env.KIVA_ARTIFACT_DIR || "/tmp/kiva-browser-tests/artifacts/battle-presentation";
  await fs.mkdir(output, { recursive: true });
  const browser = await launchBrowser();
  const site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
  try {
    const cases = [
      { width: 1280, height: 800, dpr: 1, character: 2, death: true },
      { width: 800, height: 600, dpr: 1, character: 1 },
      { width: 1000, height: 720, dpr: 2, character: 2 },
    ];
    const selected = process.env.KIVA_PRESENTATION_CASE;
    const run = selected ? cases.filter((sample) => `${sample.width}x${sample.height}@${sample.dpr}` === selected) : cases;
    assert(run.length > 0, "unknown presentation viewport case");
    for (const sample of run) await caseRun(browser, site, sample, output);
  } finally { await browser.close(); await site.close(); }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
