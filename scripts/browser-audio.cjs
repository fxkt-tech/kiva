const { resetToSelection } = require("./browser-support.cjs");
// Real application inputs, simulation commits and decoded browser audio.
// Diagnostics are read-only; no fabricated simulation events or World writes.
const assert = require("node:assert/strict");
const {
  launchBrowser, serveBuild, installFrameClock, openSelection, selectCharacter,
  resumeGame, pause, state, hero, clickWorld, reportFailure,
} = require("./browser-lane-support.cjs");

async function installObservation(page) {
  await page.addInitScript(() => {
    window.__audioTrace = { cues: [], maxPeak: 0, trustedGestures: 0 };
    document.addEventListener("pointerdown", (event) => {
      if (event.isTrusted) window.__audioTrace.trustedGestures++;
    }, { capture: true });
    document.addEventListener("keydown", (event) => {
      if (event.isTrusted) window.__audioTrace.trustedGestures++;
    }, { capture: true });
    window.addEventListener("kiva-audio", (event) => {
      const packet = JSON.parse(event.detail);
      const trace = window.__audioTrace;
      for (const cue of packet.cues) trace.cues.push({ ...cue, session: packet.session });
      trace.cues = trace.cues.slice(-200);
      trace.maxPeak = Math.max(trace.maxPeak, window.__kivaAudio?.peak || 0);
    });
  });
}

const audio = (page) => page.evaluate(() => window.__kivaAudio);

async function ownCue(page, cue, action, audible) {
  const before = await page.evaluate(() => ({
    player: String(window.__kivaState.player_id),
    session: window.__kivaAudio.session,
    played: window.__kivaAudio.played_by_cue,
    sequence: window.__audioTrace.cues.findLast((cue) =>
      cue.session === window.__kivaAudio.session)?.sequence || 0,
  }));
  await page.evaluate(() => { window.__audioTrace.maxPeak = 0; });
  await action();
  await page.waitForFunction(({ cue, before }) => window.__audioTrace.cues.some((event) =>
    event.session === before.session && event.source === before.player
    && event.sequence > before.sequence && event.cue === cue), { cue, before });
  if (audible) {
    await page.waitForFunction(({ cue, before }) =>
      (window.__kivaAudio.played_by_cue[cue] || 0) > (before.played[cue] || 0)
      && window.__audioTrace.maxPeak > 0.00001, { cue, before });
  } else {
    // The authoritative packet has already been synchronously consumed by the
    // production owner before this observation resumes in the next task.
    assert.equal((await audio(page)).played_by_cue[cue] || 0, before.played[cue] || 0,
      "a muted committed skill must not create a sound or a delayed queue");
  }
}

async function directional(page, key) {
  const current = await state(page);
  const origin = hero(current).position;
  const delta = current.map.enemy_base.map((value, i) => value - origin[i]);
  const length = Math.hypot(...delta);
  const target = origin.map((value, i) => value + delta[i] * 4 / length);
  await page.keyboard.press(key);
  await page.waitForFunction(() => window.__kivaState.aim !== null);
  await clickWorld(page, target);
}

async function main() {
  const browser = await launchBrowser();
  let site, page;
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
    page = await context.newPage();
    page.setDefaultTimeout(120000);
    const errors = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await installFrameClock(page);
    await installObservation(page);
    await openSelection(page, site.url);
    await selectCharacter(page, 1); // A genuine pointer gesture unlocks audio.
    await page.waitForFunction(() => window.__kivaAudio?.ready
      && window.__kivaAudio.context_state === "running");
    assert.equal((await audio(page)).expected_files, 25);
    assert.equal((await audio(page)).volume, 0.65);
    assert.deepEqual((await audio(page)).errors, []);
    assert(await page.evaluate(() => window.__audioTrace.trustedGestures > 0));
    await resumeGame(page);
    await page.waitForFunction(() => window.__kivaAudio.session && !window.__kivaAudio.paused);

    await ownCue(page, "jarvan.w", () => page.keyboard.press("w"), true);
    await pause(page);
    assert.equal((await audio(page)).active, 0, "pause must disconnect all voices");
    await page.locator('[data-settings-tab="audio"]').click();
    await page.locator("#audio-muted").check();
    assert.equal((await audio(page)).muted, true);
    await resumeGame(page);
    await ownCue(page, "jarvan.e", () => directional(page, "e"), false);
    await pause(page);
    await page.locator("#audio-muted").uncheck();
    // Range keyboard input is trusted and uses the normal settings binding.
    await page.locator("#audio-volume").focus();
    await page.keyboard.press("Home");
    assert.equal((await audio(page)).volume, 0);
    await page.keyboard.press("End");
    assert.equal((await audio(page)).volume, 1);
    const mutedE = (await audio(page)).played_by_cue["jarvan.e"] || 0;
    await resumeGame(page);
    await ownCue(page, "jarvan.q", () => directional(page, "q"), true);
    assert.equal((await audio(page)).played_by_cue["jarvan.e"] || 0, mutedE,
      "unmuting and resuming must not replay the discarded E");

    await pause(page);
    const oldSession = (await audio(page)).session;
    await page.locator('[data-settings-tab="characters"]').click();
    await resetToSelection(page);
    await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting"
      && window.__kivaAudio.session === null);
    assert.equal((await audio(page)).active, 0);
    await selectCharacter(page, 2);
    await resumeGame(page);
    await page.waitForFunction((old) => window.__kivaAudio.session
      && window.__kivaAudio.session !== old, oldSession);
    await ownCue(page, "ashe.w", () => directional(page, "w"), true);
    await pause(page);
    assert.equal((await audio(page)).active, 0);
    assert.equal((await audio(page)).context_state, "running");
    assert.deepEqual((await audio(page)).errors, []);
    assert.deepEqual(errors, []);
    console.log("PASS real Web Audio trusted activation / 25 decoded files / Jarvan & PROJECT Ashe cues / analyser output / mute / volume / pause / reselection");
    await context.close();
    page = null;
  } catch (error) {
    if (page) {
      console.log("Audio diagnostics:", JSON.stringify(await page.evaluate(() => ({
        audio: window.__kivaAudio, trace: window.__audioTrace,
      }))));
      await reportFailure(page);
    }
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}

module.exports = { main };
if (require.main === module) main().catch((error) => { console.error(error); process.exitCode = 1; });
