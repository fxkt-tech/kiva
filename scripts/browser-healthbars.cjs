// Measure actual DOM movement on browser animation frames, not HUD event counts.
// The static fixture listens on an ephemeral port and is closed after the test.
const assert = require("node:assert/strict");
const { launchBrowser, enterGame, clickWorld, worldPoint, panTo, state, hero, base,
  reportFailure, serveBuild } = require("./browser-lane-support.cjs");

async function measureTracking(page, selector, duration) {
  return page.evaluate(({ selector, duration }) => new Promise((resolve) => {
    const label = document.querySelector(selector);
    if (!label) throw new Error(`Missing tracking label: ${selector}`);
    let lastPosition = "";
    let frames = 0;
    let updates = 0;
    let stale = 0;
    let maxStale = 0;
    let hudEvents = 0;
    let labelEvents = 0;
    let visibleFrames = 0;
    let start;
    const onHud = () => hudEvents++;
    const onLabels = () => labelEvents++;
    window.addEventListener("kiva-state", onHud);
    window.addEventListener("kiva-labels", onLabels);
    function sample(now) {
      start ??= now;
      const position = `${label.style.left},${label.style.top},${label.style.transform}`;
      if (lastPosition) {
        frames++;
        const bounds = label.getBoundingClientRect();
        if (!label.hidden && bounds.x + bounds.width > 0 && bounds.x < innerWidth
          && bounds.y + bounds.height > 0 && bounds.y < innerHeight) visibleFrames++;
        if (position !== lastPosition) { updates++; stale = 0; }
        else { stale++; maxStale = Math.max(maxStale, stale); }
      }
      lastPosition = position;
      // Software WebGPU can run below 20 fps; collect enough real frames
      // instead of failing merely because the test machine renders slowly.
      if (now - start < duration || (frames < 20 && now - start < 10000)) requestAnimationFrame(sample);
      else {
        window.removeEventListener("kiva-state", onHud);
        window.removeEventListener("kiva-labels", onLabels);
        resolve({ frames, updates, maxStale, hudEvents, labelEvents, visibleFrames,
          elapsedMs: now - start, updateRatio: updates / frames });
      }
    }
    requestAnimationFrame(sample);
  }), { selector, duration });
}

function assertTracking(metrics, scenario) {
  console.log(`${scenario}:`, JSON.stringify(metrics));
  assert(metrics.frames >= 20, "enough browser frames must be sampled");
  assert(metrics.visibleFrames >= metrics.frames * 0.8, `${scenario}: measured labels must remain on screen`);
  assert(metrics.updateRatio >= 0.8,
    `${scenario}: health bars updated on only ${(metrics.updateRatio * 100).toFixed(1)}% of frames`);
  assert(metrics.hudEvents <= Math.ceil(metrics.elapsedMs / 80) + 2,
    "full HUD snapshots must remain throttled");
}

async function main() {
  const browser = await launchBrowser();
  let site;
  let page;
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
    page.setDefaultTimeout(120000);
    await enterGame(page, site.url);
    assert.equal(await page.evaluate(() => typeof window.__laneSetFrameStep), "undefined",
      "tracking evidence must use the native browser clock");
    await page.locator('.unit-label.ally[data-role="Hero"]').waitFor({ state: "visible" });
    // Keep both endpoints visible from a real intermediate free camera. The
    // long flank route gives the human hero several native seconds of actual
    // movement; AI stop/cast decisions are not a continuous-motion fixture.
    const initial = await state(page);
    const destination = [-9, 0];
    const midpoint = hero(initial).position.map((value, i) => (value + destination[i]) / 2);
    await panTo(page, midpoint); // No fixture clock is installed on this page.
    const point = await worldPoint(page, destination);
    assert(point[0] > 0 && point[0] < 1280 && point[1] > 0 && point[1] < 800,
      "the movement destination must be genuinely visible before clicking");
    await clickWorld(page, destination, "right");
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      return s.units.find((u) => u.id === s.player_id).order === "Moving";
    });
    assertTracking(await measureTracking(page, '.unit-label.ally[data-role="Hero"]', 1500), "Moving unit");

    // Start the static allied base toward the right edge, then pan right across
    // it. This leaves enough map-boundary room and visible travel for 20 real
    // frames; the enemy base is legitimately far outside the initial camera.
    const current = await state(page);
    const home = base(current, "ally").position;
    const bound = current.map.half_size - 6;
    const cameraStart = [home[0] - 18 * 2 / Math.sqrt(5), home[1] + 18 / Math.sqrt(5)]
      .map((value) => Math.max(-bound, Math.min(bound, value)));
    await panTo(page, cameraStart);
    await page.locator('.unit-label.ally[data-role="Base"]').waitFor({ state: "visible" });
    const beforePan = (await state(page)).camera_center;
    await page.keyboard.down("ArrowRight");
    try {
      assertTracking(await measureTracking(page, '.unit-label.ally[data-role="Base"]', 800), "Moving camera");
    } finally {
      await page.keyboard.up("ArrowRight");
    }
    const afterPan = await state(page);
    assert(Math.hypot(...afterPan.camera_center.map((value, i) => value - beforePan[i])) > 0.8,
      "the camera sample must contain actual translation");
    assert(afterPan.camera_center.every((value) => Math.abs(value) < afterPan.map.half_size - 5 - 0.2),
      "camera tracking must not spend its sample clamped at a map boundary");
    console.log("PASS per-frame health-bar tracking and throttled HUD snapshots");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
