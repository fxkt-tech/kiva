// Measure actual DOM movement on browser animation frames, not HUD event counts.
// The static fixture listens on an ephemeral port and is closed after the test.
const assert = require("node:assert/strict");
const { launchBrowser, enterGame, reportFailure, serveBuild } = require("./browser-support.cjs");

async function measureTracking(page, name, duration) {
  return page.evaluate(({ name, duration }) => new Promise((resolve) => {
    const label = [...document.querySelectorAll(".unit-label")]
      .find((element) => element.querySelector("strong").textContent === name);
    let lastPosition = "";
    let frames = 0;
    let updates = 0;
    let stale = 0;
    let maxStale = 0;
    let hudEvents = 0;
    let labelEvents = 0;
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
        resolve({ frames, updates, maxStale, hudEvents, labelEvents,
          elapsedMs: now - start, updateRatio: updates / frames });
      }
    }
    requestAnimationFrame(sample);
  }), { name, duration });
}

function assertTracking(metrics, scenario) {
  console.log(`${scenario}:`, JSON.stringify(metrics));
  assert(metrics.frames >= 20, "enough browser frames must be sampled");
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
    await enterGame(page, site.url);
    await page.waitForFunction(() => [...document.querySelectorAll(".unit-label")]
      .some((label) => label.querySelector("strong").textContent === "巡游哨兵"));
    assertTracking(await measureTracking(page, "巡游哨兵", 1500), "Moving unit");
    await page.keyboard.down("ArrowRight");
    try {
      assertTracking(await measureTracking(page, "训练哨兵", 800), "Moving camera");
    } finally {
      await page.keyboard.up("ArrowRight");
    }
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
