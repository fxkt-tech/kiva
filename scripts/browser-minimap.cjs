// Real captured-pointer minimap navigation; no camera/World writes.
// The display-frame clock stabilizes interaction timing, not performance.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const {
  launchBrowser, enterGame, resumeGame, selectCharacter, movePointer, clickPointer,
  installFrameClock, state, hero, waitTicks, pause, reportFailure, serveBuild,
} = require("./browser-lane-support.cjs");

const distance = (a, b) => Math.hypot(...a.map((value, index) => value - b[index]));

async function geometry(page) {
  return page.evaluate(() => {
    const canvas = document.getElementById("minimap");
    const box = canvas.getBoundingClientRect();
    const frame = document.getElementById("minimap-viewport");
    const rect = frame.getBoundingClientRect();
    return {
      map: { x: box.x, y: box.y, width: box.width, height: box.height },
      frame: { x: rect.x - box.x, y: rect.y - box.y, width: rect.width, height: rect.height },
      hidden: frame.hidden,
      inset: Number(canvas.dataset.mapInset),
      backing: [canvas.width, canvas.height],
      dpr: devicePixelRatio,
      camera: window.__minimapCamera,
    };
  });
}

function assertFrame(view, half) {
  assert(!view.hidden && view.camera?.footprint?.length === 4, "a real camera rectangle must be visible");
  const limits = [view.map.width, view.map.height];
  for (const [axis, offset, extent] of [[0, "x", "width"], [1, "y", "height"]]) {
    const coordinates = view.camera.footprint.map((corner) =>
      view.inset + (corner[axis] + half) / (2 * half) * (limits[axis] - 2 * view.inset));
    const start = Math.max(view.inset, Math.min(limits[axis] - view.inset, Math.min(...coordinates)));
    const end = Math.max(view.inset, Math.min(limits[axis] - view.inset, Math.max(...coordinates)));
    assert(Math.abs(view.frame[offset] - start) < 1.1, `rectangle ${offset} must follow the actual camera`);
    assert(Math.abs(view.frame[extent] - (end - start)) < 1.1, `rectangle ${extent} must follow the actual camera`);
  }
  assert(Math.abs(view.backing[0] / view.map.width - view.dpr) < 0.05, "DPR only scales canvas backing pixels");
}

async function mapPoint(page, world) {
  const view = await geometry(page);
  const half = (await state(page)).map.half_size;
  return [view.map.x + view.inset + (world[0] + half) / (2 * half) * (view.map.width - 2 * view.inset),
    view.map.y + view.inset + (world[1] + half) / (2 * half) * (view.map.height - 2 * view.inset)];
}

async function waitCenter(page, expected, tolerance = 0.04) {
  await page.waitForFunction(({ expected, tolerance }) => {
    const center = window.__kivaState.camera_center;
    return Math.hypot(...center.map((value, axis) => value - expected[axis])) < tolerance;
  }, { expected, tolerance });
}

async function clickMap(page, world) {
  const point = await mapPoint(page, world);
  const view = await geometry(page);
  assert(point[0] < view.map.x + view.frame.x || point[0] > view.map.x + view.frame.x + view.frame.width
    || point[1] < view.map.y + view.frame.y || point[1] > view.map.y + view.frame.y + view.frame.height,
  "fixture destination must be outside the current frame so clicking navigates");
  await movePointer(page, ...point);
  const actual = await page.evaluate(() => window.kivaPointer.position());
  const half = (await state(page)).map.half_size;
  const target = actual.map((value, axis) => {
    const start = axis === 0 ? view.map.x : view.map.y;
    const size = axis === 0 ? view.map.width : view.map.height;
    return Math.max(-half, Math.min(half, (value - start - view.inset) / (size - view.inset * 2) * half * 2 - half));
  });
  await clickPointer(page, "left", 0);
  const bound = half - 5;
  await waitCenter(page, target.map((value) => Math.max(-bound, Math.min(bound, value))));
  return target;
}

async function beginFrameDrag(page) {
  const view = await geometry(page);
  const point = [view.map.x + view.frame.x + view.frame.width * 0.3,
    view.map.y + view.frame.y + view.frame.height * 0.65];
  await movePointer(page, ...point);
  const actual = await page.evaluate(() => window.kivaPointer.position());
  // Camera/frame gestures use the display-frame channel. The full HUD may
  // still hold the preceding pan sample for up to its 80 ms publication period.
  const before = await page.evaluate(() => window.__minimapCamera.center.slice());
  await page.mouse.down();
  await page.waitForFunction(() => document.documentElement.dataset.minimapDragging === "true");
  await waitTicks(page, 6);
  const after = await page.evaluate(() => window.__minimapCamera.center.slice());
  assert(distance(after, before) < 0.02,
    "grabbing inside the rectangle must not jump the camera to the pointer");
  return { view, point: actual, before };
}

async function exerciseChordRelease(page) {
  await beginFrameDrag(page);
  // Hold real game callbacks so a captured right press and gesture release
  // are both delivered before the same Rust input frame consumes either.
  const heldEvents = await page.evaluate(async () => {
    const frame = window.requestAnimationFrame.bind(window);
    const pending = [];
    let held = true;
    window.requestAnimationFrame = (callback) => frame((timestamp) => {
      if (held) pending.push(callback); else callback(timestamp);
    });
    window.__releaseMinimapFrames = () => {
      held = false;
      window.requestAnimationFrame = frame;
      for (const callback of pending.splice(0)) frame(callback);
    };
    await new Promise((resolve) => frame(() => frame(resolve)));
    if (!pending.length) throw new Error("fixture must hold an actual game frame");
    return window.__minimapCameraEvents;
  });
  const before = hero(await state(page)).position;
  try {
    const viewport = page.viewportSize();
    await movePointer(page, viewport.width / 2, viewport.height / 2);
    await page.mouse.down({ button: "right" });
    await page.mouse.up({ button: "left" });
    await page.mouse.up({ button: "right" });
    assert.notEqual(await page.evaluate(() => document.documentElement.dataset.minimapDragging), "true");
    assert.equal(await page.evaluate(() => window.__minimapCameraEvents), heldEvents,
      "all press/release events must precede the next real Rust frame");
  } finally {
    await page.mouse.up({ button: "left" });
    await page.mouse.up({ button: "right" });
    await page.evaluate(() => window.__releaseMinimapFrames());
  }
  await waitTicks(page, 6);
  const after = await state(page);
  assert.equal(after.aim, 0, "a released UI gesture must still own its queued right press");
  assert.equal(hero(after).order, "Idle", "dragging outside the map and chording right cannot move the hero");
  assert(distance(hero(after).position, before) < 0.02);
}

async function exercise(page, scenario, extended) {
  const initial = await state(page);
  await page.waitForFunction(() => window.__minimapCamera?.footprint?.length === 4);
  assertFrame(await geometry(page), initial.map.half_size);
  await page.keyboard.press("s");
  await page.keyboard.press("y");
  await page.waitForFunction(() => window.__kivaState.camera_locked);
  await page.keyboard.press("q");
  await page.waitForFunction(() => window.__kivaState.aim === 0);
  const playerBefore = hero(await state(page)).position;
  await clickMap(page, [-20, -20]);
  assert.equal((await state(page)).camera_locked, false, "minimap navigation must leave hero-follow mode");
  const centered = await clickMap(page, [0, 0]);
  const projected = await page.evaluate(([x, z]) => {
    const s = window.__kivaState, m = s.view_projection;
    const w = m[3] * x + m[11] * z + m[15];
    return [(m[0] * x + m[8] * z + m[12]) / w * s.viewport[0] / 2 + s.viewport[0] / 2,
      s.viewport[1] / 2 - (m[1] * x + m[9] * z + m[13]) / w * s.viewport[1] / 2];
  }, centered);
  const viewport = page.viewportSize();
  assert(distance(projected, [viewport.width / 2, viewport.height / 2]) < 2,
    "the real rendered camera must place the clicked ground target at screen center");
  await clickPointer(page, "right", 0);
  const isolated = await state(page);
  assert.equal(isolated.aim, 0, "minimap clicks must preserve skill aiming");
  assert.equal(isolated.abilities[0].cooldown, 0, "minimap clicks must not release a skill");
  assert.equal(hero(isolated).order, "Idle", "minimap clicks must not issue movement or attacks");
  assert(distance(hero(isolated).position, playerBefore) < 0.02, "only the camera may move");

  const drag = await beginFrameDrag(page);
  const delta = [3, 2.5];
  const pixels = delta.map((value, axis) => value / (2 * initial.map.half_size)
    * ((axis === 0 ? drag.view.map.width : drag.view.map.height) - 2 * drag.view.inset));
  await movePointer(page, drag.point[0] + pixels[0], drag.point[1] + pixels[1]);
  // Pointer Lock's native synthetic input may round each CSS endpoint. Test
  // the delivered displacement, rather than an ideal fractional destination.
  const delivered = await page.evaluate(() => window.kivaPointer.position());
  await waitCenter(page, drag.before.map((value, axis) => value
    + (delivered[axis] - drag.point[axis]) / ((axis === 0 ? drag.view.map.width : drag.view.map.height)
      - 2 * drag.view.inset) * 2 * initial.map.half_size));
  await page.mouse.up();
  assert.notEqual(await page.evaluate(() => document.documentElement.dataset.minimapDragging), "true");
  await waitTicks(page, 6);
  const released = (await state(page)).camera_center;
  await movePointer(page, drag.point[0] - 5, drag.point[1] - 5);
  await waitTicks(page, 6);
  assert(distance((await state(page)).camera_center, released) < 0.02, "released drags cannot continue panning");
  assertFrame(await geometry(page), initial.map.half_size);

  await clickMap(page, [initial.map.half_size - 0.2, initial.map.half_size - 0.2]);
  assertFrame(await geometry(page), initial.map.half_size);
  await clickMap(page, [-20, -20]);
  await clickMap(page, [0, 0]);
  await movePointer(page, viewport.width / 2, viewport.height / 2);
  const beforeWheel = await page.evaluate(() => window.__minimapCamera.footprint);
  await page.mouse.wheel(0, 11);
  await page.waitForFunction((old) => {
    const outline = window.__minimapCamera.footprint;
    const span = (points) => Math.max(...points.map(([x]) => x)) - Math.min(...points.map(([x]) => x));
    return span(outline) > span(old) * 1.5;
  }, beforeWheel);
  assertFrame(await geometry(page), initial.map.half_size);
  await page.mouse.wheel(0, -11);
  await page.waitForFunction((old) => {
    const point = window.__minimapCamera.footprint[0];
    return Math.hypot(point[0] - old[0][0], point[1] - old[0][1]) < 0.2;
  }, beforeWheel);

  await page.evaluate(() => {
    window.__minimapSamples = [];
    window.__minimapStateCount = 0;
    const count = () => window.__minimapStateCount++;
    window.addEventListener("kiva-state", count);
    const record = (event) => {
      const frame = document.getElementById("minimap-viewport");
      window.__minimapSamples.push({ camera: JSON.parse(event.detail), left: frame.style.left, top: frame.style.top,
        width: frame.style.width, height: frame.style.height });
      if (window.__minimapSamples.length === 8) {
        window.removeEventListener("kiva-camera", record);
        window.removeEventListener("kiva-state", count);
      }
    };
    window.addEventListener("kiva-camera", record);
  });
  await page.keyboard.down("ArrowRight");
  try { await page.waitForFunction(() => window.__minimapSamples.length >= 8); }
  finally { await page.keyboard.up("ArrowRight"); }
  const samples = await page.evaluate(() => ({ frames: window.__minimapSamples, states: window.__minimapStateCount }));
  assert(new Set(samples.frames.map((sample) => `${sample.left},${sample.top}`)).size >= 3,
    "the visible rectangle must update between full HUD snapshots while panning");
  assert(samples.states < samples.frames.length, "camera rendering must have an independent per-frame channel");
  const pannedView = await geometry(page);
  for (const sample of samples.frames) {
    assertFrame({ ...pannedView, camera: sample.camera, frame: {
      x: parseFloat(sample.left), y: parseFloat(sample.top),
      width: parseFloat(sample.width), height: parseFloat(sample.height),
    } }, initial.map.half_size);
  }
  assertFrame(pannedView, initial.map.half_size);

  await exerciseChordRelease(page);

  await beginFrameDrag(page);
  await pause(page);
  const pausedCenter = (await state(page)).camera_center;
  assert.equal(await page.evaluate(() => document.documentElement.hasAttribute("data-camera-request")), false);
  assert.notEqual(await page.evaluate(() => document.documentElement.dataset.minimapDragging), "true");
  await page.mouse.up();
  await resumeGame(page);
  await waitTicks(page, 6);
  assert(distance((await state(page)).camera_center, pausedCenter) < 0.02,
    "resume must preserve the camera without replaying a stale drag");
  assert.equal((await state(page)).player_id, initial.player_id);

  await page.keyboard.down("Space");
  try { await waitCenter(page, hero(await state(page)).position, 0.1); }
  finally { await page.keyboard.up("Space"); }

  const directory = process.env.KIVA_SCREENSHOT_DIR || "/tmp/kiva-browser-tests/artifacts";
  await fs.mkdir(directory, { recursive: true });
  if (process.env.KIVA_BROWSER_HEADED === "1") {
    await page.screenshot({ path: path.join(directory, `minimap-${scenario}.png`) });
  }
  if (extended) {
    await page.setViewportSize({ width: 1100, height: 740 });
    await waitTicks(page, 6);
    assertFrame(await geometry(page), initial.map.half_size);
    await pause(page);
    await page.locator("#reselect").click();
    await page.waitForFunction(() => window.__kivaState.session.phase === "Selecting"
      && !document.getElementById("enter").disabled);
    await selectCharacter(page, 2);
    await resumeGame(page);
    await waitTicks(page, 6);
    const replacement = await state(page);
    assert.notEqual(replacement.player_id, initial.player_id);
    assert(distance(replacement.camera_center, hero(replacement).position) < 0.05,
      "a new match must center on its new hero without old minimap requests");
    assertFrame(await geometry(page), initial.map.half_size);
  }
  console.log(`PASS ${scenario}: full-map click/drag, UI isolation, bounds, wheel, per-frame rectangle, pause/resume`);
}

async function main() {
  let browser, site, page;
  const errors = [];
  try {
    site = await serveBuild(process.env.KIVA_SITE_DIR || "dist");
    const cases = [[1280, 800, 1], [800, 600, 1], [1000, 720, 2]];
    const requested = process.env.KIVA_MINIMAP_CASE;
    const selected = cases.filter(([w, h, dpr]) => !requested || `${w}x${h}@${dpr}` === requested);
    assert(selected.length, "unknown KIVA_MINIMAP_CASE");
    for (const [width, height, dpr] of selected) {
      browser = await launchBrowser();
      const context = await browser.newContext({ viewport: { width, height }, deviceScaleFactor: dpr });
      page = await context.newPage();
      page.setDefaultTimeout(120000);
      page.on("pageerror", (error) => errors.push(error.message));
      await installFrameClock(page);
      await page.addInitScript(() => {
        window.__minimapCameraEvents = 0;
        window.addEventListener("kiva-camera", (event) => {
          window.__minimapCameraEvents++;
          window.__minimapCamera = JSON.parse(event.detail);
        });
      });
      await enterGame(page, site.url);
      await exercise(page, `${width}x${height}-dpr${dpr}`, width === 1280);
      await context.close();
      page = null;
      await browser.close();
      browser = null;
    }
    assert.deepEqual(errors, [], "no uncaught browser errors");
  } catch (error) {
    if (page) await reportFailure(page);
    throw error;
  } finally {
    try { if (browser) await browser.close(); }
    finally { if (site) await site.close(); }
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1; });
