// Optional browser integration test. Connects to an ALREADY RUNNING dev server.
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
  const args = ['--no-sandbox', '--force-device-scale-factor=1'];
  if (process.env.KIVA_BROWSER_SOFTWARE === '1') {
    args.push('--enable-unsafe-webgpu', '--use-angle=swiftshader', '--enable-features=Vulkan',
      '--use-vulkan=swiftshader', '--disable-vulkan-surface', '--disable-gpu-watchdog');
  }
  const browser = await chromium.launch({ headless: process.env.KIVA_BROWSER_HEADED !== '1', args,
    channel: process.env.KIVA_BROWSER_CHANNEL || 'chromium' });
  try {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
    const errors = new Set();
    let webgpu = false;
    page.on('pageerror', error => { errors.add(error.message); console.error(error.message); });
    page.on('console', message => {
      if (message.text().includes('backend: BrowserWebGpu')) webgpu = true;
      if (message.type() === 'error' || message.text().includes('%cERROR%c')) {
        if (!errors.has(message.text()) && errors.size < 3) console.error(message.text());
        errors.add(message.text());
      }
    });
    await page.addInitScript(() => {
      window.soundCues = [];
      window.addEventListener('kiva-sound', event => window.soundCues.push(event.detail));
    });
    await page.goto(process.env.KIVA_TEST_URL || 'http://localhost:8080', { timeout: 90000 });
    await page.locator('#enter').waitFor({ state: 'visible', timeout: 60000 });
    // Activate the document before requesting real pointer lock.
    await page.bringToFront();
    await page.keyboard.press('Tab');
    await page.locator('#enter').click();
    await page.waitForFunction(() => document.pointerLockElement?.id === 'game');
    const cdp = await page.context().newCDPSession(page);
    const button = async (name, down, buttons = down ? (name === 'left' ? 1 : 2) : 0) => {
      await cdp.send('Input.dispatchMouseEvent', {
        type: down ? 'mousePressed' : 'mouseReleased', button: name, buttons,
        x: 0, y: 0, clickCount: 1,
      });
    };
    const tap = async (name = 'left') => {
      await button(name, true); await page.waitForTimeout(50); await button(name, false);
    };
    const contains = text => page.waitForFunction(text =>
      document.getElementById('game').getAttribute('aria-label').includes(text), text, { timeout: 6000 });
    const summary = () => page.locator('#game').getAttribute('aria-label');
    const hp = async side => Number((await summary()).match(new RegExp(`${side}: (\\d+) HP`))[1]);
    const reset = async () => {
      await page.keyboard.press('F8', { delay: 50 });
      await contains('Dummy: 100 HP; READY');
      await contains('Player: 100 HP; READY');
      await page.waitForTimeout(150);
    };
    const shot = async name => page.screenshot(process.env.KIVA_SCREENSHOT_DIR
      ? { path: `${process.env.KIVA_SCREENSHOT_DIR}/${name}.png` } : {});
    await reset();
    // Require actual rendering as well as state updates.
    const deadline = Date.now() + 25000;
    while (true) {
      const initial = await shot('idle');
      if (healthBar(initial, 'player') > 230 && healthBar(initial, 'dummy') > 230) break;
      assert(Date.now() < deadline, 'WebGPU must render both health bars');
      await page.waitForTimeout(250);
    }
    await tap('right');
    await contains('VERTICAL 1');
    await contains('Dummy: 90 HP');
    await reset();
    await tap('left');
    await contains('HORIZONTAL 1');
    await contains('Dummy: 90 HP');

    await reset();
    await tap('left');
    await contains('HORIZONTAL 1');
    await page.waitForTimeout(310);
    await tap('right');
    await contains('VERTICAL 2');
    await page.waitForTimeout(390);
    await tap('left');
    await contains('HORIZONTAL 3 / BLUE');
    await contains('Dummy: 50 HP');

    await reset();
    await button('right', true);
    await contains('VERTICAL CHARGE / BLUE READY');
    await button('right', false);
    await contains('VERTICAL SWORD ENERGY');
    await contains('Dummy: 70 HP');
    await shot('sword-energy');

    await reset();
    await button('left', true);
    await contains('HORIZONTAL CHARGE / BLUE READY');
    await page.keyboard.press('KeyC', { delay: 60 });
    await button('left', false);
    await page.waitForTimeout(700);
    assert.equal(await hp('Dummy'), 100, 'crouch-cancel must not release the held sword energy');

    await reset();
    await button('left', true);
    await button('right', true, 3);
    await contains('PARRY / ACTIVE');
    await button('right', false, 1); await button('left', false);
    await page.waitForTimeout(700);
    assert.equal(await hp('Dummy'), 100, 'combined parry must not queue a normal attack');

    await reset();
    await page.keyboard.press('F6', { delay: 50 });
    await page.waitForTimeout(420);
    await page.keyboard.press('KeyG', { delay: 40 });
    await contains('RIPOSTE READY');
    assert.equal(await hp('Player'), 100, 'timed parry stops the blue strike');
    await tap('left');
    await contains('RIPOSTE / GOLD');
    await contains('Dummy: 65 HP');
    await shot('riposte');

    await reset();
    await page.keyboard.press('F5', { delay: 40 });
    await tap('left');
    await contains('Dummy: 90 HP');

    await reset();
    await page.keyboard.press('F7', { delay: 50 });
    await page.waitForTimeout(420);
    await page.keyboard.press('KeyG', { delay: 40 });
    await contains('Player: 80 HP');

    await reset();
    await page.keyboard.down('KeyC');
    await tap('right');
    await page.keyboard.up('KeyC');
    await contains('UPPERCUT');
    await contains('Dummy: 86 HP');
    await shot('uppercut');

    await reset();
    await page.keyboard.press('Space', { delay: 50 });
    await contains('AIRBORNE 1/2');
    await tap('left');
    await contains('AIR SLASH');
    await shot('air-slash');

    await reset();
    await page.keyboard.press('Space', { delay: 50 });
    await page.keyboard.press('Space', { delay: 50 });
    await contains('AIRBORNE 2/2');
    const jumps = await page.evaluate(() => window.soundCues.filter(cue => cue === 'jump').length);
    await page.keyboard.press('Space', { delay: 50 });
    assert.equal(await page.evaluate(() => window.soundCues.filter(cue => cue === 'jump').length), jumps, 'third jump rejected');

    await reset();
    await page.keyboard.press('KeyC', { delay: 50 });
    await contains('CROUCHING');
    await page.waitForTimeout(100);
    assert((await summary()).includes('CROUCHING'), 'releasing C keeps crouch toggled on');
    await page.keyboard.press('KeyC', { delay: 50 });
    await contains('READY');
    await page.keyboard.down('KeyD');
    await page.keyboard.down('ShiftLeft');
    await contains('DODGING');
    await contains('SPRINTING');
    await page.keyboard.up('ShiftLeft');
    await page.waitForTimeout(100);
    await contains('SPRINTING');
    await page.keyboard.down('KeyW');
    await page.keyboard.up('KeyD');
    await contains('SPRINTING');
    await page.keyboard.up('KeyW');
    await contains('READY');
    await page.keyboard.down('KeyD');
    await page.keyboard.down('ShiftLeft');
    await contains('SPRINTING');
    await page.keyboard.up('ShiftLeft');
    await page.keyboard.press('KeyC', { delay: 50 });
    await contains('SLIDING');
    await contains('CROUCHING');
    await page.keyboard.up('KeyD');
    await reset();
    await page.keyboard.press('KeyR'); await page.keyboard.press('KeyM');
    assert.equal(await page.evaluate(() => window.KivaAudio.muted), false, 'M is reserved for map, not mute');
    assert.equal(await page.evaluate(() => window.KivaAudio.state), 'running');
    // DOM click avoids browser focus changes while the mouse is captured.
    await page.locator('#sound-toggle').evaluate(button => button.click());
    assert.equal(await page.evaluate(() => window.KivaAudio.muted), true);
    await page.locator('#sound-toggle').evaluate(button => button.click());
    await button('left', true);
    await contains('HORIZONTAL CHARGE');
    await page.evaluate(() => document.exitPointerLock());
    await page.locator('#entry').waitFor({ state: 'visible' });
    await button('left', false);
    await page.locator('#enter').click();
    await contains('Player: 100 HP; READY');
    const cues = await page.evaluate(() => window.soundCues);
    for (const cue of ['charge-ready', 'sword-energy', 'parry', 'guard', 'jump', 'dodge']) assert(cues.includes(cue), `missing ${cue} sound`);
    assert(webgpu, 'engine must use BrowserWebGpu');
    assert.deepEqual([...errors], [], 'no browser or renderer errors');
    console.log('PASS WebGPU rendering, horizontal/vertical, charged wave, cancellation, combined parry, riposte, normal punishes parry, uppercut, air attack, jump, dash/sprint/slide, audio and pause');
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
