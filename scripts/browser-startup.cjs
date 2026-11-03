// Requires Playwright on NODE_PATH and an already running Trunk server.
const assert = require('node:assert/strict');
const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({
    headless: true,
    channel: process.env.KIVA_BROWSER_CHANNEL || 'chromium',
    args: ['--force-device-scale-factor=1'],
  });
  try {
    const page = await browser.newPage();
    await page.goto(process.env.KIVA_TEST_URL || 'http://localhost:8080');
    // The engine can be running behind the loading overlay with older Trunk.
    await page.waitForFunction(() => document.getElementById('game')
      .getAttribute('aria-label').includes('Player: 100 HP'), undefined, { timeout: 30000 });
    await page.locator('#enter').waitFor({ state: 'visible', timeout: 5000 });
    assert(await page.locator('#loading').isHidden(), 'running game must dismiss loading');
    console.log('PASS engine started, loading dismissed, entry visible');
    await page.close();

    const failed = await browser.newPage();
    await failed.route('**/*.wasm', route => route.abort());
    await failed.goto(process.env.KIVA_TEST_URL || 'http://localhost:8080');
    await failed.waitForFunction(() => document.getElementById('loading')
      .textContent.startsWith('Unable to start.'), undefined, { timeout: 5000 });
    assert(await failed.locator('#entry').isHidden(), 'failed startup must not offer entry');
    console.log('PASS failed WASM download shows an error instead of loading forever');
  } finally {
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
