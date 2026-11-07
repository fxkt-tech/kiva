// Character selection uses real DOM controls and Pointer Lock. Malformed/stale
// cases exercise the production intent transport; none can mutate the World.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, openSelection, selectCharacter, resumeGame, clickWorld,
  reportFailure, serveBuild, installFrameClock, nearHome,
} = require("./browser-lane-support.cjs");

const fields = ["attack_damage", "ability_power", "armor", "magic_resistance", "move_speed", "attack_speed", "attack_range"];
const arcanist = {
  attack_damage: 30, ability_power: 40, armor: 0, magic_resistance: 0,
  move_speed: 5.2, attack_speed: 1, attack_range: 6,
};
const state = (page) => page.evaluate(() => window.__kivaState);
const nextState = (page) => page.evaluate(() => new Promise((resolve) => {
  window.addEventListener("kiva-state", (event) => resolve(JSON.parse(event.detail)), { once: true });
}));

async function traceCapture(page) {
  await page.addInitScript(() => {
    window.__testCaptureTrace = [];
    const record = (event) => window.__testCaptureTrace.push({
      event, time: performance.now(), captured: document.pointerLockElement?.id || null,
      paused: document.documentElement?.dataset.paused, focus: document.hasFocus(),
      phase: window.__kivaState?.session?.phase,
    });
    window.__recordTestCapture = record;
    for (const type of ["pointerlockchange", "pointerlockerror"]) {
      document.addEventListener(type, () => record(type));
    }
    for (const type of ["blur", "focus"]) window.addEventListener(type, () => record(type));
    for (const type of ["keydown", "keyup"]) window.addEventListener(type, (event) => record(`${type}-${event.key}`));
    const exit = Document.prototype.exitPointerLock;
    Document.prototype.exitPointerLock = function (...args) {
      record(`exitPointerLock: ${new Error().stack}`);
      return exit.apply(this, args);
    };
    const request = HTMLCanvasElement.prototype.requestPointerLock;
    HTMLCanvasElement.prototype.requestPointerLock = function (...args) {
      record("native-request");
      const promise = request.apply(this, args);
      promise?.then?.(() => record("native-resolved"),
        (error) => record(`native-error-${error.name}: ${error.message}`));
      return promise;
    };
    document.addEventListener("DOMContentLoaded", () => {
      new MutationObserver(() => record("paused-attribute")).observe(document.documentElement, {
        attributes: true, attributeFilter: ["data-paused"],
      });
    });
  });
}

// Chromium may update pointerLockElement before dispatching pointerlockchange.
// Hold that notification across real animation frames, with native capture still
// held, so the pre-acknowledgement interval is deterministic on every machine.
async function deferCaptureNotification(page) {
  await page.evaluate(() => {
    window.__testDeferredCapture = null;
    function defer(event) {
      if (!event.isTrusted || document.pointerLockElement?.id !== "game") return;
      document.removeEventListener("pointerlockchange", defer, true);
      event.stopImmediatePropagation();
      const before = window.__kivaState;
      const evidence = { phase: before.session.phase, player: before.player_id, tick: before.tick, frames: [] };
      window.__testDeferredCapture = evidence;
      function frame() {
        const snapshot = window.__kivaState;
        evidence.frames.push({
          captured: document.pointerLockElement?.id,
          acknowledged: document.documentElement.dataset.pointerLocked,
          paused: snapshot.paused, tick: snapshot.tick,
          phase: snapshot.session.phase, player: snapshot.player_id,
        });
        if (evidence.frames.length < 4) {
          requestAnimationFrame(frame);
        } else {
          document.documentElement.dataset.skill = "0";
          // Deliver the deferred notification; the capture itself remains the
          // result of the real trusted button click and native browser request.
          document.dispatchEvent(new Event("pointerlockchange"));
        }
      }
      requestAnimationFrame(frame);
    }
    document.addEventListener("pointerlockchange", defer, true);
  });
}

async function assertDeferredCapture(page) {
  const evidence = await page.evaluate(() => window.__testDeferredCapture);
  assert.equal(evidence.frames.length, 4);
  for (const frame of evidence.frames) {
    assert.equal(frame.captured, "game", "capture must survive until its event is delivered");
    assert.equal(frame.acknowledged, "false");
    assert.equal(frame.paused, true);
    assert.equal(frame.tick, evidence.tick, "the pending notification cannot advance the simulation");
    assert.equal(frame.phase, evidence.phase);
    assert.equal(frame.player, evidence.player);
  }
  assert.equal((await state(page)).aim, null, "a stale menu skill cannot cross the capture acknowledgement");
}

function close(actual, expected, message) {
  assert(Math.abs(actual - expected) < 0.001, `${message}: ${actual} != ${expected}`);
}

function assertUnstarted(snapshot) {
  assert.equal(snapshot.session.phase, "Selecting");
  assert.equal(snapshot.player_id, null);
  assert.equal(snapshot.paused, true);
  assert.deepEqual(snapshot.abilities, []);
  assert.equal(snapshot.units.length, 0, "selection must not create a hidden match roster");
  assert.equal(snapshot.match.phase, "Waiting");
}

async function sendIntents(page, intents) {
  await page.evaluate((intents) => {
    const root = document.documentElement;
    const queued = JSON.parse(root.dataset.sessionRequests || "[]");
    root.dataset.sessionRequests = JSON.stringify([...queued, ...intents]);
  }, intents);
}

async function inspectPreview(page, expectedId) {
  const snapshot = await state(page);
  assertUnstarted(snapshot);
  const preview = snapshot.session.selection.selected;
  assert.equal(preview.id, expectedId);
  const values = await page.evaluate((fields) => Object.fromEntries(fields.map((field) =>
    [field, document.querySelector(`#character-preview [data-preview-stat="${field}"]`)?.textContent])), fields);
  assert.equal(await page.locator("#character-preview [data-preview-stat]").count(), 7);
  for (const field of fields) close(Number(values[field]), preview.attributes[field], `preview ${field}`);
  assert.equal(await page.locator(`[data-character-id="${expectedId}"]`).getAttribute("aria-pressed"), "true");
  return preview;
}

async function screenshot(page, name) {
  if (process.env.KIVA_BROWSER_HEADED !== "1") return;
  const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
  await fs.mkdir(directory, { recursive: true });
  const { width, height } = page.viewportSize();
  const dpr = await page.evaluate(() => devicePixelRatio);
  await page.screenshot({ path: path.join(directory, `${name}-${width}x${height}-dpr${dpr}.png`) });
}

async function rejectCapturedLaunch(page, mode) {
  await page.evaluate((mode) => {
    window.__testCaptureObserved = false;
    const current = window.__kivaState.session.selection;
    const character = current.selected.id;
    const revision = current.selection_revision;
    function replaceIntent() {
      if (document.pointerLockElement?.id !== "game") return;
      window.__testCaptureObserved = true;
      document.removeEventListener("pointerlockchange", replaceIntent);
      // The production listener already queued its Launch. Replace only those
      // external intents before Rust consumes them, while real capture is held.
      const root = document.documentElement;
      if (mode === "stray") {
        delete root.dataset.sessionRequests;
        root.dataset.paused = "false";
      } else {
        root.dataset.sessionRequests = JSON.stringify(mode === "failed-choice" ? [
          { type: "SelectCharacter", character_id: 65535, request_id: 900001 },
          { type: "Launch", character_id: character, selection_revision: revision },
        ] : [{ type: "Launch", character_id: character, selection_revision: revision - 1 }]);
      }
    }
    document.addEventListener("pointerlockchange", replaceIntent);
  }, mode);
  await page.locator("#enter").click();
  await page.waitForFunction(() => window.__testCaptureObserved
    && window.__kivaState?.session?.phase === "Selecting" && !document.pointerLockElement);
  const rejected = await state(page);
  assertUnstarted(rejected);
  assert.equal(rejected.tick, 0);
  if (mode !== "stray") assert(await page.locator("#selection-error").isVisible());
  return rejected;
}

async function exerciseChoiceFailures(page) {
  const before = await state(page);
  const selected = before.session.selection.selected.id;
  const other = selected === 1 ? 2 : 1;
  await sendIntents(page, [
    { type: "SelectCharacter", character_id: other, request_id: 900000 },
    { type: "Launch", character_id: selected, selection_revision: before.session.selection.selection_revision,
      stats: { attack_damage: 999999 } },
  ]);
  await page.waitForFunction(() => Boolean(window.__kivaState.session.selection?.error));
  const malformed = await state(page);
  assertUnstarted(malformed);
  assert.equal(malformed.session.selection.selected.id, selected,
    "a malformed batch must be rejected before its valid prefix is applied");
  await selectCharacter(page, 2);

  const failedChoice = await rejectCapturedLaunch(page, "failed-choice");
  assert.equal(failedChoice.session.selection.selected.id, 2, "failed choice must preserve the last preview");
  assert.equal(failedChoice.session.selection.selection_ack.accepted, false);
  assert.equal(failedChoice.session.selection.can_start, false,
    "Rust must reject an old Launch after a failed Select even with real capture");
  await selectCharacter(page, 2);
  const stale = await rejectCapturedLaunch(page, "stale");
  assert.equal(stale.session.selection.selection_ack.accepted, true);
  assert.equal(stale.session.selection.can_start, true, "a failed Launch must preserve valid retry eligibility");
  const stray = await rejectCapturedLaunch(page, "stray");
  assert.equal(stray.session.selection.can_start, true);
  console.log("PASS strict intent batch, failed selection + old Launch, stale revision and capture without Launch");
}

async function exerciseSelection(page, url, config, failures) {
  await openSelection(page, url);
  const initial = await state(page);
  assertUnstarted(initial);
  assert.equal(initial.tick, 0);
  assert.equal(initial.session.selection.selected.id, 1, "Traveller remains the default selection");
  assert.equal(await page.locator("[data-character-id]").count(), 2);
  assert(await page.locator("#hero-hud").isHidden());
  await page.keyboard.press("q");
  await page.keyboard.press("a");
  await page.keyboard.press("s");
  await page.waitForTimeout(250);
  const idle = await state(page);
  assertUnstarted(idle);
  assert.equal(idle.tick, initial.tick);
  assert.equal(idle.aim, null);
  assert.equal(idle.attack_move, false);
  assert.deepEqual(idle.units.map(({ id, position }) => ({ id, position })),
    initial.units.map(({ id, position }) => ({ id, position })), "the match roster must not advance before launch");
  await inspectPreview(page, 1);

  // A single browser turn produces multiple normal card clicks, exercising
  // latest-request acknowledgement rather than serially waiting after each.
  const last = await page.evaluate(() => {
    for (const id of [2, 1, 2]) document.querySelector(`[data-character-id="${id}"]`).click();
    return JSON.parse(document.documentElement.dataset.sessionRequests).at(-1);
  });
  await page.waitForFunction((requestId) => {
    const selection = window.__kivaState.session.selection;
    return selection.selected.id === 2 && selection.selection_ack.request_id === requestId
      && selection.selection_ack.accepted && !document.getElementById("enter").disabled;
  }, last.request_id);
  await selectCharacter(page, 1);
  const previousAck = (await state(page)).session.selection.selection_ack.request_id;
  await page.locator('[data-character-id="2"]').focus();
  await page.keyboard.press("Enter");
  await page.waitForFunction((previousAck) => {
    const selection = window.__kivaState.session.selection;
    return selection.selected.id === 2 && selection.selection_ack.accepted
      && selection.selection_ack.request_id !== previousAck && !document.getElementById("enter").disabled;
  }, previousAck);
  const preview = await inspectPreview(page, 2);
  assert.equal(preview.max_health, 480);
  assert.equal(preview.max_mana, 360);
  for (const field of fields) close(preview.attributes[field], arcanist[field], `Arcanist ${field}`);
  assert.deepEqual(preview.abilities.map(({ targeting }) => targeting), ["Direction", "Point", "Point", "SelfCast"]);
  await screenshot(page, "character-selection");
  if (failures) await exerciseChoiceFailures(page);

  await page.evaluate(() => {
    function leaveStaleSkill() {
      if (document.pointerLockElement?.id !== "game") return;
      document.removeEventListener("pointerlockchange", leaveStaleSkill);
      document.documentElement.dataset.skill = "0";
    }
    document.addEventListener("pointerlockchange", leaveStaleSkill);
  });
  if (failures) await deferCaptureNotification(page);
  await resumeGame(page, config.deviceScaleFactor === 2 ? "keyboard" : "mouse");
  if (failures) await assertDeferredCapture(page);
  const active = await state(page);
  const hero = active.units.find((unit) => unit.id === active.player_id);
  assert.equal(active.session.phase, "Active");
  assert.equal(active.session.selection, null);
  assert.equal(hero.character_id, 2);
  assert.equal(active.units.filter((unit) => unit.character_id !== null).length, 2);
  assert.equal(hero.name, preview.name);
  assert.equal(hero.health, preview.max_health);
  assert.equal(hero.mana, preview.max_mana);
  assert.equal(hero.order, "Idle");
  assert.equal(hero.action, "Ready");
  assert.equal(active.aim, null, "selection-stage Q must not replay at launch");
  for (const field of fields) close(hero.attributes[field], preview.attributes[field], `initial live ${field}`);
  assert.deepEqual(active.abilities.map(({ name }) => name), preview.abilities.map(({ name }) => name));
  const liveDom = await page.evaluate((fields) => Object.fromEntries(fields.map((field) =>
    [field, document.querySelector(`#hero-stats [data-stat="${field}"]`).textContent])), fields);
  for (const field of fields) close(Number(liveDom[field]), arcanist[field], `actual HUD ${field}`);
  assert((await page.locator("#hero-stats").boundingBox()).width <= 145);
  assert(await page.locator("#character-select").isHidden());
  await screenshot(page, "character-arcanist");

  await page.keyboard.press("q");
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  await clickWorld(page, nearHome(active, 5, -1));
  await page.waitForFunction(() => window.__kivaState.abilities[0].cooldown > 0);
  assert((await state(page)).units.find((unit) => unit.id === active.player_id).mana < preview.max_mana,
    "the selected character's direction spell must commit its real resource cost");
  await page.keyboard.press("r");
  await page.waitForFunction(() => {
    const s = window.__kivaState;
    return s.aim === null && s.abilities[3].cooldown > 0
      && s.units.find((unit) => unit.id === s.player_id).action === "Channeling";
  });
  const committed = await state(page);
  await sendIntents(page, [
    { type: "Launch", character_id: 1, selection_revision: 1 },
    { type: "Launch", character_id: 2, selection_revision: 1 },
    { type: "SelectCharacter", character_id: 1, request_id: 900002 },
  ]);
  await nextState(page);
  const repeated = await nextState(page);
  assert.equal(repeated.player_id, committed.player_id);
  assert.equal(repeated.units.filter((unit) => unit.character_id !== null).length, 2);
  assert.equal(repeated.units.find((unit) => unit.id === repeated.player_id).character_id, 2);
  assert(repeated.abilities[3].cooldown <= committed.abilities[3].cooldown + 0.001,
    "repeated launch must not restart an ability cooldown");
  assert(repeated.abilities[3].cooldown > 0);
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  const paused = await state(page);
  await page.waitForTimeout(250);
  assert.equal((await state(page)).tick, paused.tick);
  assert(await page.locator("#character-select").isHidden());
  if (failures) await deferCaptureNotification(page);
  await resumeGame(page);
  if (failures) await assertDeferredCapture(page);
  const resumed = await state(page);
  assert.equal(resumed.player_id, paused.player_id);
  const resumedHero = resumed.units.find((unit) => unit.id === resumed.player_id);
  assert.equal(resumedHero.character_id, 2);
  assert.equal(resumedHero.max_health, 480);
  assert(resumed.abilities[3].cooldown > 0 && resumed.abilities[3].cooldown <= paused.abilities[3].cooldown,
    "resume preserves the committed cooldown instead of creating a fresh character");
  console.log(`PASS no-player freeze, latest selection, preview/live values, keyboard choice/start, Arcanist Q/R, duplicate requests and same-instance resume ${config.viewport.width}×${config.viewport.height} DPR${config.deviceScaleFactor}`);
}

async function exerciseCaptureRetry(browser, url) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();
  await installFrameClock(page);
  page.setDefaultTimeout(120000);
  try {
    await traceCapture(page);
    await page.addInitScript(() => {
      const request = HTMLCanvasElement.prototype.requestPointerLock;
      let first = true;
      HTMLCanvasElement.prototype.requestPointerLock = function (...args) {
        if (first) {
          first = false;
          return Promise.reject(new DOMException("Test refusal", "NotAllowedError"));
        }
        return request.apply(this, args);
      };
    });
    await openSelection(page, url);
    await selectCharacter(page, 2);
    await page.locator("#enter").click();
    await page.locator("#pointer-error").waitFor({ state: "visible" });
    assertUnstarted(await state(page));
    assert.equal((await state(page)).session.selection.selected.id, 2);
    await resumeGame(page);
    const entered = await state(page);
    assert.equal(entered.units.find((unit) => unit.id === entered.player_id).character_id, 2);
    console.log("PASS refused first capture keeps selection and retries with exactly one character");
  } catch (error) {
    await reportFailure(page);
    throw error;
  } finally {
    await context.close();
  }
}

async function exerciseLateCaptureCallback(browser, url) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", (error) => errors.push(error.message));
  try {
    page.setDefaultTimeout(120000);
      await installFrameClock(page);
    await traceCapture(page);
    await page.addInitScript(() => {
      const request = HTMLCanvasElement.prototype.requestPointerLock;
      let calls = 0;
      let rejectOld;
      const record = (event) => window.__recordTestCapture(event);
      HTMLCanvasElement.prototype.requestPointerLock = function (...args) {
        record(`request-${calls + 1}`);
        const nativeRequest = request.apply(this, args);
        calls++;
        nativeRequest?.catch?.((error) => record(`native-error-${error.name}: ${error.message}`));
        if (calls === 1) {
          nativeRequest?.catch?.(() => {});
          return new Promise((_, reject) => { rejectOld = reject; });
        }
        if (calls === 2) {
          record("reject-old-promise");
          rejectOld(new DOMException("Late old refusal", "NotAllowedError"));
          window.__testRejectedOldCapture = true;
        }
        return nativeRequest;
      };
    });
    await openSelection(page, url);
    await selectCharacter(page, 1);
    await resumeGame(page);
    const playerId = (await state(page)).player_id;
    await page.keyboard.press("Escape");
    await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
    await resumeGame(page);
    assert.equal(await page.evaluate(() => window.__testRejectedOldCapture), true);
    assert.equal((await state(page)).player_id, playerId);
    assert.equal(await page.evaluate(() => document.pointerLockElement?.id), "game");
    assert(await page.locator("#pointer-error").isHidden());

    const cdp = await context.newCDPSession(page);
    await cdp.send("Emulation.setFocusEmulationEnabled", { enabled: false });
    const other = await context.newPage();
    await other.bringToFront();
    await page.waitForFunction(() => !document.hasFocus() && window.__kivaState.paused);
    assert.equal((await state(page)).player_id, playerId);
    await page.bringToFront();
    await other.close();
    await cdp.send("Emulation.setFocusEmulationEnabled", { enabled: true });
    await resumeGame(page);
    assert.equal((await state(page)).player_id, playerId);
    assert.deepEqual(errors, []);
    console.log("PASS late old capture Promise cannot cancel a newer attempt; real focus loss resumes the same player");
  } catch (error) {
    console.error("Capture trace:", await page.evaluate(() => window.__testCaptureTrace));
    await reportFailure(page);
    throw error;
  } finally {
    await context.close();
  }
}

async function main() {
  const browser = await launchBrowser();
  let site;
  let page;
  const errors = [];
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    if (process.argv.includes("--late-capture")) {
      await exerciseLateCaptureCallback(browser, site.url);
      return;
    }
    const cases = [
      { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 },
      { viewport: { width: 800, height: 600 }, deviceScaleFactor: 1 },
      { viewport: { width: 1000, height: 720 }, deviceScaleFactor: 2 },
    ];
    for (const [index, config] of cases.entries()) {
      const context = await browser.newContext(config);
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      await installFrameClock(page);
      await traceCapture(page);
      page.on("pageerror", (error) => errors.push(error.message));
      await exerciseSelection(page, site.url, config, index === 0);
      await context.close();
      page = null;
    }
    await exerciseCaptureRetry(browser, site.url);
    await exerciseLateCaptureCallback(browser, site.url);
    assert.deepEqual(errors, [], "no uncaught browser errors");
  } catch (error) {
    if (page) {
      console.error("Capture trace:", JSON.stringify(await page.evaluate(() => window.__testCaptureTrace), null, 2));
      await reportFailure(page);
    }
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
