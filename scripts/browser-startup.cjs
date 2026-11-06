const assert = require("node:assert/strict");
const { launchBrowser } = require("./browser-support.cjs");

async function main() {
  const browser = await launchBrowser();
  try {
    const page = await browser.newPage();
    await page.route("**/*.wasm", (route) => route.abort());
    await page.goto(process.env.KIVA_TEST_URL || "http://127.0.0.1:3221");
    await page.waitForFunction(() => window.__kivaStartup === "failed", undefined, { timeout: 30000 });
    assert.equal(await page.locator("#loading").getAttribute("role"), "alert");
    assert(await page.locator("#retry").isVisible());
    assert(await page.locator("#entry").isHidden());
    await page.unroute("**/*.wasm");
    await page.locator("#retry").click();
    await page.locator("#enter").waitFor({ state: "visible", timeout: 120000 });
    assert.equal(await page.evaluate(() => window.__kivaStartup), "ready");
    console.log("PASS failed WASM download, error message and retry");
  } finally {
    await browser.close();
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
