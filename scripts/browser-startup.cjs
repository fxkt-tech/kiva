const assert = require("node:assert/strict");
const { launchBrowser, serveBuild, selectCharacter, resumeGame } = require("./browser-support.cjs");

async function normalLateHud(browser, url) {
  const page = await browser.newPage();
  page.setDefaultTimeout(120000);
  let releaseHud;
  try {
    await page.addInitScript(() => {
      window.__lateHudEvidence = { ready: false, firstCatalog: null };
      window.addEventListener("kiva-ready", () => { window.__lateHudEvidence.ready = true; });
      window.addEventListener("kiva-state", (event) => {
        const snapshot = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
        if (Array.isArray(snapshot.equipment_catalog) && !window.__lateHudEvidence.firstCatalog) {
          window.__lateHudEvidence.firstCatalog = { count: snapshot.equipment_catalog.length, tick: snapshot.tick };
        }
      });
    });
    const hudGate = new Promise((resolve) => { releaseHud = resolve; });
    await page.route("**/hud.js", async (route) => { await hudGate; await route.continue(); });
    await page.goto(url, { waitUntil: "commit" });
    await page.waitForFunction(() => window.__lateHudEvidence?.ready
      && window.__lateHudEvidence.firstCatalog?.count === 9);
    assert.equal(await page.evaluate(() => typeof window.__kivaState), "undefined",
      "the real first catalog must arrive while the HUD is still blocked");
    assert(await page.locator("#enter").isHidden());
    releaseHud();
    await page.waitForFunction(() => window.__kivaState?.session?.phase === "Selecting"
      && window.__kivaState.session.selection?.can_start && !document.getElementById("enter").disabled);
    assert.equal(await page.evaluate(() => window.__kivaStartup), "ready");
    await selectCharacter(page, 1);
    await resumeGame(page);
    await page.keyboard.press("p");
    await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement
      && !document.getElementById("equipment-editor").hidden);
    assert.equal(await page.locator("#equipment-catalog [data-equipment-id]").count(), 9,
      "a late HUD must retain the nine real equipment definitions sent before it subscribed");
    console.log("PASS normal WASM with early ready/catalog before HUD, late selection/capture and all nine equipment cards");
  } finally {
    releaseHud?.();
    await page.close();
  }
}

async function main() {
  const browser = await launchBrowser();
  let site;
  let releaseHud;
  try {
    if (process.env.KIVA_SITE_DIR) site = await serveBuild(process.env.KIVA_SITE_DIR);
    const page = await browser.newPage();
    // Force the real loader rejection to occur before the deferred HUD can
    // install a DOM subscriber. This is an ordering barrier, not a sleep.
    const hudGate = new Promise((resolve) => { releaseHud = resolve; });
    await page.route("**/hud.js", async (route) => { await hudGate; await route.continue(); });
    await page.route("**/*.wasm", (route) => route.abort());
    const wasmRequest = page.waitForRequest((request) => request.url().split("?")[0].endsWith(".wasm"));
    await page.goto(site?.url || process.env.KIVA_TEST_URL || "http://127.0.0.1:3221", { waitUntil: "commit" });
    await wasmRequest;
    await page.waitForFunction(() => window.__kivaStartup === "failed", undefined, { timeout: 30000 });
    assert.equal(await page.locator("#loading").getAttribute("role"), "status",
      "the early loader error must be retained before HUD initialization");
    assert(await page.locator("#retry").isHidden());
    releaseHud();
    await page.waitForFunction(() => document.getElementById("loading")?.getAttribute("role") === "alert");
    assert.equal(await page.locator("#loading").getAttribute("role"), "alert");
    assert(await page.locator("#retry").isVisible());
    assert(await page.locator("#entry").isHidden());
    await page.unroute("**/*.wasm");
    await page.unroute("**/hud.js");
    await page.locator("#retry").click();
    await page.locator("#enter").waitFor({ state: "visible", timeout: 120000 });
    assert.equal(await page.evaluate(() => window.__kivaStartup), "ready");
    await page.evaluate(() => window.dispatchEvent(new Event("TrunkApplicationError")));
    assert.equal(await page.evaluate(() => window.__kivaStartup), "ready", "a late old loader event cannot override readiness");
    assert(await page.locator("#loading").isHidden());
    assert(await page.locator("#enter").isVisible());
    console.log("PASS early WASM failure before HUD, retained error/retry, successful reload and ready-state precedence");
    await page.close();
    await normalLateHud(browser, site?.url || process.env.KIVA_TEST_URL || "http://127.0.0.1:3221");
  } finally {
    releaseHud?.();
    try { await browser.close(); }
    finally { if (site) await site.close(); }
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
