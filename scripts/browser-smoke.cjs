// Optional browser integration test. Uses an ALREADY RUNNING dev server.
// Requires playwright and pngjs on NODE_PATH; see README.md.
const assert = require('node:assert/strict');
const { chromium } = require('playwright');
const { PNG } = require('pngjs');

function healthBar(buffer, side) {
  const image = PNG.sync.read(buffer);
  const color = side === 'player' ? [90, 208, 194] : [228, 159, 95];
  const start = side === 'player' ? 20 : image.width - 320;
  let longest = 0;
  for (let y = 85; y < 150; y++) {
    let run = 0;
    for (let x = start; x < start + 300; x++) {
      const index = (y * image.width + x) * 4;
      run = color.every((value, channel) => Math.abs(image.data[index + channel] - value) < 3) ? run + 1 : 0;
      longest = Math.max(longest, run);
    }
  }
  return longest;
}

(async () => {
  const args = ['--no-sandbox'];
  if (process.env.KIVA_BROWSER_SOFTWARE === '1') {
    // ANGLE-only SwiftShader fails to allocate WebGPU swap-chain shared images
    // in headless Linux Chromium. Enable its Vulkan compositor as well.
    args.push('--enable-unsafe-webgpu', '--use-angle=swiftshader', '--enable-features=Vulkan',
      '--use-vulkan=swiftshader', '--disable-vulkan-surface', '--disable-gpu-watchdog');
  }
  const browser = await chromium.launch({ headless: true, args });
  try {
    const page = await browser.newPage({ viewport: { width: 960, height: 640 } });
    const errors = [];
    let webgpu = false;
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => {
      if (message.text().includes('backend: BrowserWebGpu')) webgpu = true;
      if (message.type() === 'error' || message.text().includes('%cERROR%c')) errors.push(message.text());
    });
    await page.addInitScript(() => {
      window.soundCues = [];
      window.addEventListener('kiva-sound', event => window.soundCues.push(event.detail));
    });
    await page.goto(process.env.KIVA_TEST_URL || 'http://localhost:8080', { waitUntil: 'networkidle', timeout: 90000 });
    await page.locator('#enter').waitFor({ state: 'visible', timeout: 60000 });
    await page.waitForTimeout(3500);
    await page.locator('#enter').click();
    await page.waitForFunction(() => document.pointerLockElement?.id === 'game');
    const cdp = await page.context().newCDPSession(page);
    const button = async (name, down) => {
      // CDP absolute viewport coordinates produce artificial cursor-warp deltas
      // under pointer lock. Zero coordinates preserve real relative-look state.
      await cdp.send('Input.dispatchMouseEvent', {
        type: down ? 'mousePressed' : 'mouseReleased', button: name,
        buttons: down ? (name === 'left' ? 1 : 2) : 0, x: 0, y: 0, clickCount: 1,
      });
    };
    const attack = async () => {
      await button('left', true);
      await page.waitForTimeout(45);
      await button('left', false);
    };
    const reset = async () => {
      await page.keyboard.press('KeyR', { delay: 80 });
      await page.waitForTimeout(350);
      // Software WebGPU may still be compiling its first rendering pipelines.
      // Wait for the rendered bars, with the same health assertion and a bound.
      const deadline = Date.now() + 20000;
      while (true) {
        const resetImage = await page.screenshot();
        if (healthBar(resetImage, 'dummy') > 240 && healthBar(resetImage, 'player') > 240) break;
        assert(Date.now() < deadline, 'R must reset both health bars');
        await page.waitForTimeout(250);
      }
    };
    const screenshot = async name => page.screenshot(name && process.env.KIVA_SCREENSHOT_DIR
      ? { path: `${process.env.KIVA_SCREENSHOT_DIR}/${name}.png` } : {});
    await page.waitForTimeout(350);
    await reset();
    const initial = await screenshot('idle');
    const playerFull = healthBar(initial, 'player');
    const dummyFull = healthBar(initial, 'dummy');
    assert(playerFull > 230 && dummyFull > 230, 'WebGPU must actually render both full health bars');
    await attack();
    await page.waitForTimeout(800);
    const hit = await screenshot('single-hit');
    assert(Math.abs(healthBar(hit, 'dummy') / dummyFull - 0.90) < 0.015, 'one swing deals exactly 10 damage');

    await reset();
    const comboAttack = async () => {
      await attack();
      const deadline = Date.now() + 6000;
      while (!(await page.locator('#game').getAttribute('aria-label')).includes('CUT 3 /')) {
        assert(Date.now() < deadline, 'repeated clicks should reach the third cut');
        await page.waitForTimeout(35);
        await attack();
      }
      await page.waitForFunction(() => document.getElementById('game').getAttribute('aria-label').includes('Player: 100 HP; READY'), undefined, { timeout: 5000 });
    };
    await comboAttack();
    const combo = await screenshot('combo');
    assert(Math.abs(healthBar(combo, 'dummy') / dummyFull - 0.50) < 0.02, 'three-cut combo deals 10 + 15 + 25');

    const downDeadline = Date.now() + 10000;
    while (!(await page.locator('#game').getAttribute('aria-label')).includes('Dummy: 0 HP')) {
      assert(Date.now() < downDeadline, 'repeated attacks should down the dummy');
      await page.waitForFunction(() => document.getElementById('game').getAttribute('aria-label').includes('Player: 100 HP; READY'), undefined, { timeout: 3000 });
      await attack();
      await page.waitForTimeout(650);
    }
    assert(healthBar(await screenshot('down'), 'dummy') < 20, 'dummy falls when health reaches zero');
    await page.waitForTimeout(2300);
    assert(healthBar(await screenshot('respawn'), 'dummy') > dummyFull - 3, 'dummy respawns with full health');

    await reset();
    await button('right', true);
    await page.waitForTimeout(200);
    await page.keyboard.press('KeyT', { delay: 60 });
    await page.waitForTimeout(1300);
    const guarded = await screenshot('guard');
    assert(Math.abs(healthBar(guarded, 'player') / playerFull - 0.96) < 0.015, 'frontal guard reduces 20 damage to 4');
    await button('right', false);

    await reset();
    await page.keyboard.press('KeyT', { delay: 60 });
    await page.waitForTimeout(1300);
    const unguarded = await screenshot('unguarded');
    assert(Math.abs(healthBar(unguarded, 'player') / playerFull - 0.80) < 0.015, 'unguarded strike deals full 20 damage');
    await reset();
    assert.equal(await page.evaluate(() => window.KivaAudio.state), 'running', 'audio unlocks on entering training');
    await page.keyboard.press('Space', { delay: 60 });
    await page.waitForFunction(() => document.getElementById('game').getAttribute('aria-label').includes('AIRBORNE'));
    await page.keyboard.press('Space', { delay: 40 });
    await page.waitForFunction(() => document.getElementById('game').getAttribute('aria-label').includes('AIRBORNE 2/2'));
    await page.keyboard.press('Space', { delay: 40 });
    await screenshot('jump');
    await page.waitForFunction(() => document.getElementById('game').getAttribute('aria-label').includes('Player: 100 HP; READY'));
    assert.equal(await page.evaluate(() => window.soundCues.filter(cue => cue === 'jump').length), 2, 'two jumps allowed, third airborne jump rejected');
    assert((await page.evaluate(() => window.soundCues)).includes('land'), 'landing sound is emitted');

    await page.keyboard.down('KeyD');
    await page.keyboard.press('ShiftLeft', { delay: 70 });
    await page.waitForFunction(() => document.getElementById('game').getAttribute('aria-label').includes('DODGING'));
    await screenshot('dodge');
    await page.keyboard.up('KeyD');
    await page.waitForFunction(() => document.getElementById('game').getAttribute('aria-label').includes('Player: 100 HP; READY'));
    const dodgeCues = await page.evaluate(() => window.soundCues.filter(cue => cue === 'dodge').length);
    assert(dodgeCues >= 1, 'left Shift starts a dodge');

    await page.keyboard.press('KeyM');
    assert.equal(await page.evaluate(() => window.KivaAudio.muted), true, 'M mutes sound');
    await page.keyboard.press('KeyM');
    assert.equal(await page.evaluate(() => window.KivaAudio.muted), false, 'M restores sound');
    const cues = await page.evaluate(() => window.soundCues);
    for (const cue of ['swing1', 'swing2', 'swing3', 'hit', 'guard', 'down', 'revive', 'jump', 'land', 'dodge']) {
      assert(cues.includes(cue), `missing sound cue: ${cue}`);
    }
    await page.evaluate(() => document.exitPointerLock());
    await page.locator('#entry').waitFor({ state: 'visible' });
    await page.waitForTimeout(100);
    assert(webgpu, 'engine must use BrowserWebGpu, not a WebGL fallback');
    assert.deepEqual(errors, [], 'no browser or renderer errors');
    console.log('PASS WebGPU render, mouse capture, attacks, combo, respawn, guard, jump, dodge, audio, mute and pause');
  } finally {
    await browser.close();
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
