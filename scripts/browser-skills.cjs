// Mouse-only skill regression. An ephemeral static fixture is closed in finally;
// no development service is started. Build first: trunk build --dist <directory>.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const { createReadStream } = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const { launchBrowser, enterGame, clickWorld, reportFailure } = require("./browser-support.cjs");

async function serveBuild(directory) {
  const root = path.resolve(directory);
  const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm" };
  const server = http.createServer(async (request, response) => {
    const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    const file = path.resolve(root, `.${pathname === "/" ? "/index.html" : pathname}`);
    if (!file.startsWith(root + path.sep)) { response.writeHead(403); response.end(); return; }
    try {
      const stat = await fs.stat(file);
      response.writeHead(200, { "Content-Type": types[path.extname(file)] || "application/octet-stream", "Content-Length": stat.size });
      createReadStream(file).pipe(response);
    } catch {
      response.writeHead(404); response.end();
    }
  });
  await new Promise((resolve, reject) => { server.once("error", reject); server.listen(0, "127.0.0.1", resolve); });
  return { url: `http://127.0.0.1:${server.address().port}`, close: () => new Promise((resolve) => server.close(resolve)) };
}

async function main() {
  const browser = await launchBrowser();
  let site;
  let page;
  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, deviceScaleFactor: 1 });
    page = await context.newPage();
    page.setDefaultTimeout(10000);
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    await enterGame(page, site.url);
    const before = await page.evaluate(() => {
      const s = window.__kivaState;
      return { position: s.units.find((u) => u.id === s.player_id).position, health: s.units.find((u) => u.name === "训练哨兵").health };
    });
    // Hold across engine updates, as a human click can: a zero-duration click
    // misses canvas-focus changes between pointerdown and click.
    await page.locator('[data-slot="0"]').click({ delay: 1200 });
    assert.equal(await page.evaluate(() => window.__kivaState.paused), false, "clicking a skill must not pause the game");
    await clickWorld(page, [3, -2]);
    await page.waitForFunction(() => window.__kivaState.units.some((u) => u.name === "训练哨兵" && u.health === 325));
    const after = await page.evaluate(() => {
      const s = window.__kivaState;
      return { position: s.units.find((u) => u.id === s.player_id).position, cooldown: s.abilities[0].cooldown };
    });
    assert.equal(before.health, 400);
    assert(after.cooldown > 0);
    assert(Math.hypot(...after.position.map((value, index) => value - before.position[index])) < 0.1, "HUD click must not move the hero");
    await page.locator('[data-slot="1"]').click({ delay: 1200 });
    assert.equal(await page.evaluate(() => window.__kivaState.paused), false);
    await clickWorld(page, [3, -2]);
    await page.waitForFunction(() => window.__kivaState.units.some((u) => u.name === "训练哨兵" && u.health === 205));
    await page.locator('[data-slot="2"]').click({ delay: 1200 });
    await clickWorld(page, [-3, 6]);
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      const p = s.units.find((u) => u.id === s.player_id).position;
      return Math.hypot(p[0] + 3, p[1] - 6) < 0.1;
    });
    await page.locator('[data-slot="3"]').click({ delay: 1200 });
    await page.waitForFunction(() => {
      const s = window.__kivaState;
      return s.abilities[3].cooldown > 0 && s.units.find((u) => u.id === s.player_id).action === "Channeling";
    });
    console.log("PASS mouse-only Q/W/E targeting, R self-cast, damage, movement and cooldowns");

    const cdp = await page.context().newCDPSession(page);
    await cdp.send("Emulation.setFocusEmulationEnabled", { enabled: false });
    const otherPage = await context.newPage();
    await otherPage.bringToFront();
    await page.waitForFunction(() => !document.hasFocus() && document.documentElement.dataset.paused === "true");
    await page.bringToFront();
    await page.waitForFunction(() => window.__kivaState.paused);
    await otherPage.close();
    await cdp.send("Emulation.setFocusEmulationEnabled", { enabled: true });
    await page.locator("#enter").click();
    await page.waitForFunction(() => !window.__kivaState.paused);
    console.log("PASS real document focus loss still pauses and resumes");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    await browser.close();
    if (site) await site.close();
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });
