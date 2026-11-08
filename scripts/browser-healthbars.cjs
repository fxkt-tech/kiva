// Measure actual DOM movement on browser animation frames, not HUD event counts.
// The static fixture listens on an ephemeral port and is closed after the test.
const assert = require("node:assert/strict");
const { launchBrowser, enterGame, clickWorld, worldPoint, panTo, state, hero, nearMiddle,
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
    let cameraTravel = 0;
    let lastCamera;
    let minCameraMargin = Infinity;
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
        const state = window.__kivaState;
        if (lastCamera) cameraTravel += Math.hypot(...state.camera_center.map((v, i) => v - lastCamera[i]));
        lastCamera = state.camera_center;
        minCameraMargin = Math.min(minCameraMargin, state.map.half_size - 5 - Math.max(...state.camera_center.map(Math.abs)));
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
        resolve({ frames, updates, maxStale, hudEvents, labelEvents, visibleFrames, cameraTravel, minCameraMargin,
          elapsedMs: now - start, updateRatio: updates / frames });
      }
    }
    requestAnimationFrame(sample);
  }), { selector, duration });
}

async function measureBoundedCameraPan(page, selector) {
  let active = true;
  let key = "ArrowRight";
  let pending = Promise.resolve();
  await page.exposeBinding("__trackingPanKey", (_source, requested) => {
    pending = pending.then(async () => {
      if (!active || requested === key) return;
      assert(["ArrowLeft", "ArrowRight"].includes(requested));
      await page.keyboard.down(requested);
      await page.keyboard.up(key);
      key = requested;
    });
    return pending;
  });
  await page.keyboard.down(key);
  await page.evaluate((selector) => {
    let running = true;
    let direction = "ArrowRight";
    window.__stopTrackingPan = () => { running = false; };
    function monitor() {
      if (!running) return;
      const bounds = document.querySelector(selector).getBoundingClientRect();
      const x = bounds.x + bounds.width / 2;
      const next = x < innerWidth * 0.20 ? "ArrowLeft"
        : x > innerWidth * 0.35 ? "ArrowRight" : direction;
      if (next !== direction) {
        direction = next;
        window.__trackingPanKey(next);
      }
      requestAnimationFrame(monitor);
    }
    requestAnimationFrame(monitor);
  }, selector);
  try { return await measureTracking(page, selector, 800); }
  finally {
    active = false;
    await page.evaluate(() => window.__stopTrackingPan());
    await pending;
    await page.keyboard.up("ArrowLeft");
    await page.keyboard.up("ArrowRight");
  }
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
    // The new close default is covered by browser-stats. This native-clock
    // sample needs a long route visible at once, so use the real wheel to
    // restore roughly the previous field of view before tracking 20 frames.
    await page.waitForFunction(() => window.__kivaState.camera_footprint?.length === 4);
    const beforeZoomArea = await page.evaluate(() => {
      const p = window.__kivaState.camera_footprint;
      return Math.abs(p.reduce((sum, [x,z], i) => {
        const next = p[(i + 1) % p.length]; return sum + x * next[1] - z * next[0];
      }, 0)) / 2;
    });
    await page.mouse.wheel(0, 11);
    await page.waitForFunction((before) => {
      const p = window.__kivaState.camera_footprint;
      if (!p) return false;
      const area = Math.abs(p.reduce((sum, [x,z], i) => {
        const next = p[(i + 1) % p.length]; return sum + x * next[1] - z * next[0];
      }, 0)) / 2;
      return area > before * 3.4 && area < before * 5.2;
    }, beforeZoomArea);
    // Keep both endpoints visible from a real intermediate free camera. The
    // long flank route gives the human hero several native seconds of actual
    // movement; AI stop/cast decisions are not a continuous-motion fixture.
    const initial = await state(page);
    const destination = nearMiddle(initial, -3, -(initial.map.lane_width + 2));
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

    // Reverse real arrow-key input before the label reaches the edge. On a
    // slow software GPU, 20 native frames can span several seconds; holding a
    // single direction would leave the label offscreen or hit the map bound.
    const current = await state(page);
    const home = current.map.ally_base;
    const bound = current.map.half_size - 6;
    const cameraStart = [home[0] + 6 * 2 / Math.sqrt(5), home[1] - 6 / Math.sqrt(5)]
      .map((value) => Math.max(-bound, Math.min(bound, value)));
    await panTo(page, cameraStart);
    await page.locator('.unit-label.ally[data-role="Base"]').waitFor({ state: "visible" });
    const cameraMetrics = await measureBoundedCameraPan(page, '.unit-label.ally[data-role="Base"]');
    assertTracking(cameraMetrics, "Moving camera");
    const afterPan = await state(page);
    assert(cameraMetrics.cameraTravel > 0.8,
      "the camera sample must contain actual translation");
    assert(cameraMetrics.minCameraMargin > 0.2, "the entire camera sample must stay clear of map clamping");
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
