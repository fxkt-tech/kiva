// Mouse-only skill regression. An ephemeral static fixture is closed in finally;
// no development service is started. Build first: trunk build --dist <directory>.
const assert = require("node:assert/strict");
const { launchBrowser, enterGame, clickWorld, reportFailure, serveBuild } = require("./browser-support.cjs");


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
