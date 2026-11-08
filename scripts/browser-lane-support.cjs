// Read-only observations and a display-frame clock for the live lane mode.
// Input still uses the production captured-pointer helpers; no World writes.
const assert = require("node:assert/strict");
const support = require("./browser-support.cjs");

async function installFrameClock(page) {
  await page.addInitScript(() => {
    const nativeNow = performance.now.bind(performance);
    const nativeFrame = requestAnimationFrame.bind(window);
    let now = nativeNow();
    let lastFrame;
    let step = 1000 / 60;
    const diagnostics = { frames: 0, states: 0, labels: 0, step };
    window.__laneDiagnostics = diagnostics;
    Object.defineProperty(performance, "now", { value: () => now });
    window.requestAnimationFrame = (callback) => nativeFrame((timestamp) => {
      if (timestamp !== lastFrame) {
        lastFrame = timestamp;
        now += step;
        diagnostics.frames++;
      }
      callback(now);
    });
    window.__laneSetFrameStep = (milliseconds) => {
      if (![1000 / 60, 250].includes(milliseconds)) throw new Error("Unsupported fixture frame step");
      step = milliseconds;
      diagnostics.step = step;
    };
    window.__laneStarts = new Map();
    window.addEventListener("kiva-state", (event) => {
      diagnostics.states++;
      const snapshot = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
      if (snapshot.match?.phase === "Running" && !window.__laneStarts.has(snapshot.match.id)) {
        window.__laneStarts.set(snapshot.match.id, snapshot);
      }
    });
    window.__laneLabels = new Map();
    window.addEventListener("kiva-labels", (event) => {
      diagnostics.labels++;
      const labels = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
      window.__laneLabels = new Map(labels.map((label) => [label.id, label]));
    });
  });
}

const state = (page) => page.evaluate(() => window.__kivaState);
const hero = (snapshot) => snapshot.units.find((unit) => unit.id === snapshot.player_id);
const enemyHero = (snapshot) => snapshot.units.find((unit) => unit.role === "Hero" && unit.team === "enemy");
const base = (snapshot, team) => snapshot.units.find((unit) => unit.role === "Base" && unit.team === team);
const minions = (snapshot, team) => snapshot.units.filter((unit) =>
  (!team || unit.team === team) && ["MeleeMinion", "RangedMinion"].includes(unit.role));

async function observed(page, predicate, argument) {
  const result = await page.waitForFunction(predicate, argument);
  try { return await result.jsonValue(); }
  finally { await result.dispose(); }
}

async function waitTicks(page, count = 6) {
  const initial = await page.evaluate(() => window.__kivaState.tick);
  await page.waitForFunction(({ initial, count }) => window.__kivaState.tick >= initial + count,
    { initial, count });
}

async function pause(page) {
  await page.keyboard.press("Escape");
  // Esc opens live settings. Long combat fixtures explicitly use the existing
  // pause intent when they need a frozen observation, rather than changing UI.
  await page.evaluate(() => { document.documentElement.dataset.paused = "true"; });
  await page.waitForFunction(() => window.__kivaState.paused && !document.pointerLockElement);
}

// Freeze a real visible skill through the existing pause intent. All selection
// and combat still happen through normal input; the observer never writes World.
async function pauseOnSkillVisual(page, id, phase, action, delayTicks = 0, minLayers = 1) {
  await page.evaluate(({id,phase,delayTicks,minLayers}) => {
    window.__vfxCaptured = null;
    let firstTick;
    function observe(event) {
      const s = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
      if (s.paused || !s.effects?.active.some((effect) => effect.ability_id === id
        && effect.phase === phase && effect.layers >= minLayers)) return;
      firstTick ??= s.tick;
      if(s.tick < firstTick + delayTicks)return;
      window.__vfxCaptured = s;
      document.documentElement.dataset.paused = "true";
      document.exitPointerLock();
      window.removeEventListener("kiva-state", observe);
    }
    window.addEventListener("kiva-state", observe);
  }, {id,phase,delayTicks,minLayers});
  await action();
  await page.waitForFunction(() => Boolean(window.__vfxCaptured) && window.__kivaState.paused);
  const frozen = await state(page);
  const frame = await page.evaluate(() => window.__laneDiagnostics.frames);
  await page.waitForFunction((frame) => window.__laneDiagnostics.frames >= frame+4, frame);
  const stable = await state(page);
  assert.equal(stable.tick, frozen.tick);
  assert.deepEqual(stable.effects.active, frozen.effects.active, "pause freezes actual effect instances");
  assert.deepEqual(stable.effects.errors, []);
  return frozen;
}

async function firstWaveStart(page) {
  const snapshot = await page.evaluate(() => window.__laneStarts?.get(window.__kivaState.match.id));
  assert(snapshot, "the fixture must observe the first authoritative Running projection");
  assert(snapshot.tick < 30, "the first trusted start projection must precede the second scheduled pair");
  assert.equal(snapshot.match.phase, "Running");
  assert.equal(snapshot.match.wave_number, 1);
  assert.equal(snapshot.units.length, 3, "first observation contains our base, hero and first minion; unseen enemies stay hidden");
  assert.equal(snapshot.units.filter((unit)=>unit.team==="enemy").length,0);
  assert.equal(snapshot.match.bases.enemy,null,"enemy base health is unknown before vision");
  for (const team of ["ally"]) {
    const first = minions(snapshot, team);
    assert.equal(first.length, 1);
    assert.equal(first[0].role, "MeleeMinion");
    assert.equal(first[0].formation_slot, 0);
    assert.equal(first[0].max_health, 465);
  }
  assert.equal(snapshot.camera_locked, false);
  assert(Math.hypot(...snapshot.camera_center.map((value, i) => value - hero(snapshot).position[i])) < 0.15,
    "a fresh match must center its free camera on the new player once");
  return snapshot;
}

async function waitForFirstWave(page) {
  await page.evaluate(() => window.__laneSetFrameStep(250));
  let snapshot;
  try {
    snapshot = await observed(page, () => window.__kivaState.tick >= 150 ? window.__kivaState : false);
  } finally { await page.evaluate(() => window.__laneSetFrameStep(1000 / 60)); }
  assert.equal(snapshot.match.wave_number, 1);
  assert.equal(snapshot.units.filter((unit)=>unit.team==="ally").length, 8, "our six scheduled minions, hero and base are observed by tick150");
  assert(snapshot.units.every((unit)=>unit.team==="ally"),"camera position cannot reveal the distant enemy roster");
  for (const team of ["ally"]) {
    const wave = minions(snapshot, team).filter((unit) => unit.wave === 1);
    assert.deepEqual(wave.map((unit) => unit.formation_slot).sort((a,b) => a-b), [0,1,2,3,4,5]);
    assert.equal(wave.filter((unit) => unit.role === "MeleeMinion").length, 3);
    assert.equal(wave.filter((unit) => unit.role === "RangedMinion").length, 3);
    for (const unit of wave) assert.equal(unit.max_health, unit.role === "MeleeMinion" ? 465 : 284);
  }
  return snapshot;
}

async function selectAndEnter(page, url, characterId = 1) {
  await support.openSelection(page, url);
  await support.selectCharacter(page, characterId);
  await support.resumeGame(page);
  await firstWaveStart(page);
  return waitForFirstWave(page);
}

async function visibleWorldPoint(page, position) {
  const point = await support.worldPoint(page, position);
  const { width, height } = page.viewportSize();
  return point[0] > 8 && point[0] < width - 8 && point[1] > 110 && point[1] < height - 165;
}

async function panTo(page, position) {
  const diagnostics = process.env.KIVA_PAN_DIAGNOSTICS === "1";
  const sample = () => page.evaluate(() => ({ date_now: Date.now(), performance_now: performance.now(),
    tick: window.__kivaState.tick, center: window.__kivaState.camera_center,
    locked: window.__kivaState.camera_locked, paused: window.__kivaState.paused,
    dom_paused: document.documentElement.dataset.paused, captured: document.pointerLockElement?.id,
    fixture: window.__laneDiagnostics }));
  if (diagnostics) console.log("PAN begin", JSON.stringify({ position, ...(await sample()) }));
  if ((await state(page)).camera_locked) {
    await page.keyboard.press("y");
    await page.waitForFunction(() => !window.__kivaState.camera_locked);
  }
  const { width, height } = page.viewportSize();
  await support.movePointer(page, width / 2, height / 2);
  await page.evaluate(() => window.__laneSetFrameStep?.(250));
  try {
    for (const [axis, positive, negative] of [
      [[2 / Math.sqrt(5), -1 / Math.sqrt(5)], "ArrowRight", "ArrowLeft"],
      [[1 / Math.sqrt(5), 2 / Math.sqrt(5)], "ArrowDown", "ArrowUp"],
    ]) {
      const center = (await state(page)).camera_center;
      const remaining = (position[0] - center[0]) * axis[0] + (position[1] - center[1]) * axis[1];
      if (Math.abs(remaining) < 2.5) continue;
      const sign = Math.sign(remaining);
      const key = sign > 0 ? positive : negative;
      if (diagnostics) console.log("PAN axis", JSON.stringify({ key, remaining, center }));
      const timer = diagnostics ? setInterval(async () => {
        try { console.log("PAN waiting", JSON.stringify({ key, ...(await sample()) })); }
        catch (error) { console.error("PAN diagnostic error", error.message); }
      }, 5000) : null;
      await page.keyboard.down(key);
      try {
        await page.waitForFunction(({ position, axis, sign }) => {
          const center = window.__kivaState.camera_center;
          return ((position[0] - center[0]) * axis[0] + (position[1] - center[1]) * axis[1]) * sign <= 2.5;
        }, { position, axis, sign });
      } finally {
        if (timer) clearInterval(timer);
        await page.keyboard.up(key);
      }
      if (diagnostics) console.log("PAN reached", JSON.stringify({ key, ...(await sample()) }));
    }
  } finally { await page.evaluate(() => window.__laneSetFrameStep?.(1000 / 60)); }
  assert(await visibleWorldPoint(page, position), "real arrow-key panning must bring the world target into the viewport");
}

async function clickWorld(page, position, button = "left") {
  if (!await visibleWorldPoint(page, position)) await panTo(page, position);
  return support.clickWorld(page, position, button);
}

async function clickUnit(page, id, button = "left") {
  const unit = (await state(page)).units.find((unit) => unit.id === id);
  assert(unit, `Unit ${id} must still exist`);
  if (!await visibleWorldPoint(page, [...unit.position, 1])) await panTo(page, unit.position);
  const point = await page.evaluate((id) => {
    const label = window.__laneLabels?.get(id);
    const unit = window.__kivaState.units.find((unit) => unit.id === id);
    return label?.body_screen || unit?.body_screen;
  }, id);
  const viewport = page.viewportSize();
  assert(point && point[0] > 0 && point[0] < viewport.width && point[1] > 0 && point[1] < viewport.height,
    `Unit ${id} must have an actual on-screen body projection after camera movement`);
  await support.movePointer(page, ...point);
  await support.clickPointer(page, button, 0);
}

async function moveTo(page, destination) {
  if (!await visibleWorldPoint(page, destination)) await panTo(page, destination);
  // Walking fixtures do not sample short control windows. Run every fixed step
  // while avoiding a minute of host time for a few seconds of software-GPU
  // walking, then restore the fine clock before testing combat input.
  await page.evaluate(() => window.__laneSetFrameStep(250));
  try {
    await support.clickWorld(page, destination, "right");
    await page.waitForFunction(([x, z]) => {
      const snapshot = window.__kivaState;
      const player = snapshot.units.find((unit) => unit.id === snapshot.player_id);
      if (!player?.alive) throw new Error("The live opponent defeated the player during the movement order");
      return Math.hypot(player.position[0] - x, player.position[1] - z) < 0.15;
    }, destination);
  } finally {
    await page.evaluate(() => window.__laneSetFrameStep(1000 / 60));
  }
}

function nearHome(snapshot, forward = 2, lateral = -2) {
  const home = snapshot.map.ally_base;
  const away = snapshot.map.enemy_base;
  const length = Math.hypot(away[0] - home[0], away[1] - home[1]);
  const direction = [(away[0] - home[0]) / length, (away[1] - home[1]) / length];
  return [home[0] + direction[0] * forward - direction[1] * lateral,
    home[1] + direction[1] * forward + direction[0] * lateral];
}

function nearMiddle(snapshot, forwardOffset = -0.65, lateral = -3) {
  const length = Math.hypot(...snapshot.map.enemy_base.map((value, i) => value - snapshot.map.ally_base[i]));
  return nearHome(snapshot, length / 2 + forwardOffset, lateral);
}

module.exports = { ...support, installFrameClock, state, hero, enemyHero, base, minions,
  observed, waitTicks, pause, pauseOnSkillVisual, selectAndEnter, firstWaveStart, waitForFirstWave,
  clickUnit, clickWorld, moveTo, panTo, nearHome, nearMiddle };
