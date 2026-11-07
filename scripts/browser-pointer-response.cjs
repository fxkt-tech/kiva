// Compare locked motion to the browser's own unlocked motion; no World writes.
const assert = require("node:assert/strict");
const { launchBrowser, serveBuild, openSelection, selectCharacter, resumeGame,
  moveRelative, movePointer, clickPointer, worldPoint, reportFailure } = require("./browser-support.cjs");

const { installFrameClock, state, nearHome } = require("./browser-lane-support.cjs");

async function observe(page, physicalUnits) {
  await page.addInitScript(({ physicalUnits }) => {
    // Exercise a browser reporting physical deltas as well as this Chromium's
    // native units. Events, pointer lock and client coordinates remain native.
    if (physicalUnits) {
      for (const key of ["movementX", "movementY"]) {
        const descriptor = Object.getOwnPropertyDescriptor(MouseEvent.prototype, key);
        Object.defineProperty(MouseEvent.prototype, key, { ...descriptor,
          get() { return descriptor.get.call(this) * devicePixelRatio; } });
      }
    }
    const position = () => {
      const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
      return [matrix.m41, matrix.m42];
    };
    const delta = (event) => {
      const samples = event.getCoalescedEvents?.();
      return (samples?.length ? samples : [event]).reduce((sum, sample) =>
        [sum[0] + sample.movementX, sum[1] + sample.movementY], [0, 0]);
    };
    let previous;
    const samplesByEvent = new WeakMap();
    window.__pointerSamples = [];
    document.addEventListener("pointermove", (event) => {
      if (document.pointerLockElement?.id !== "game") {
        const movement = delta(event);
        if (previous && movement[0]) {
          const ratio = (event.clientX - previous[0]) / movement[0];
          if (Number.isFinite(ratio) && ratio > 0) window.__normalPointerRatio = ratio;
        }
        previous = [event.clientX, event.clientY];
        return;
      }
      if (!event.movementX && !event.movementY) return;
      const sample = { before: position(), delta: delta(event) };
      window.__pointerSamples.push(sample);
      samplesByEvent.set(event, sample);
    }, true);
    // Microtasks can run between native listeners. Observe at window bubble,
    // after the pointer owner's document-capture handler, still in this event.
    window.addEventListener("pointermove", (event) => {
      if (document.pointerLockElement?.id === "game") {
        const sample = samplesByEvent.get(event);
        if (sample) sample.after = position();
      }
    });
  }, { physicalUnits });
}

async function holdFrames(page) {
  return page.evaluate(async () => {
    let labels = 0;
    window.addEventListener("kiva-labels", () => labels++);
    window.__heldPointerLabels = () => labels;
    const nativeFrame = window.requestAnimationFrame.bind(window);
    let held = true;
    const pending = [];
    window.requestAnimationFrame = (callback) => nativeFrame((timestamp) => {
      if (held) pending.push(callback); else callback(timestamp);
    });
    window.__resumePointerFrames = () => {
      held = false;
      for (const callback of pending.splice(0)) nativeFrame(callback);
    };
    await new Promise((resolve) => nativeFrame(() => nativeFrame(resolve)));
    if (!pending.length) throw new Error("The fixture must hold a real game frame");
    return labels;
  });
}

async function exercise(page, url) {
  await openSelection(page, url);
  await page.mouse.move(200, 300);
  await page.mouse.move(248, 300);
  const ratio = await page.evaluate(() => window.__normalPointerRatio);
  assert(Number.isFinite(ratio) && ratio > 0, "observe actual unlocked CSS movement first");
  await selectCharacter(page, 1);
  await resumeGame(page);
  await page.evaluate(() => { window.__pointerSamples = []; });
  for (const delta of [[24, 0], [24, 16], [-24, 0], [-24, -16]]) await moveRelative(page, ...delta);
  const samples = await page.evaluate(() => window.__pointerSamples);
  assert(samples.length >= 4);
  for (const sample of samples) {
    assert(sample.after, "observe the same native event after its handlers");
    for (let axis = 0; axis < 2; axis++) {
      assert(Math.abs(sample.after[axis] - sample.before[axis] - sample.delta[axis] * ratio) < 0.7,
        `cursor must apply normal pointer gain during the event: ${JSON.stringify({ ratio, sample })}`);
    }
  }

  // Rendering may be late. The cursor must still move, and an earlier click
  // must retain its own position even after subsequent motion in the same batch.
  await page.keyboard.press("s");
  const snapshot = await state(page);
  const destinationWorld = nearHome(snapshot, 2.2, -1.3);
  const destination = await worldPoint(page, destinationWorld);
  const laterPoint = await worldPoint(page, nearHome(snapshot, 4, -2.5));
  const heldLabels = await holdFrames(page);
  try {
    await moveRelative(page, 24, 0);
    await movePointer(page, ...destination);
    await clickPointer(page, "right", 0);
    await movePointer(page, ...laterPoint);
    assert.equal(await page.evaluate(() => window.__heldPointerLabels()), heldLabels,
      "WASM must remain between frames while the cursor and press snapshots change");
  } finally {
    await page.evaluate(() => window.__resumePointerFrames());
  }
  await page.waitForFunction(([x, z]) => {
    const state = window.__kivaState;
    const player = state.units.find((unit) => unit.id === state.player_id);
    return Math.hypot(player.position[0] - x, player.position[1] - z) < 0.1;
  }, destinationWorld);
  await page.keyboard.press("Escape");
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
  assert(await page.locator("#game-cursor").isHidden());
  await resumeGame(page);
  const center = await page.evaluate(() => {
    const matrix = new DOMMatrix(getComputedStyle(document.getElementById("game-cursor")).transform);
    return [matrix.m41, matrix.m42];
  });
  assert.deepEqual(center, [500, 360]);
  console.log(`PASS normal gain=${ratio}, event-time display, stalled rendering, press position and resume`);
}

async function main() {
  const browser = await launchBrowser();
  let site, page;
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    for (const [scale, physicalUnits] of [[1, false], [1.5, false], [2, false], [1.5, true]]) {
      const context = await browser.newContext({ viewport: { width: 1000, height: 720 }, deviceScaleFactor: scale });
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      page.on("pageerror", (error) => console.error("Pointer test page error:", error.message));
      page.on("requestfailed", (request) => console.error("Pointer test failed request:", request.url(), request.failure()?.errorText));
      page.on("console", (message) => { if (message.type() === "error") console.error("Pointer test browser error:", message.text()); });
      await installFrameClock(page);
      await observe(page, physicalUnits);
      await exercise(page, site.url);
      console.log(`PASS pointer response DPR${scale}, ${physicalUnits ? "physical" : "native"} movement units`);
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
