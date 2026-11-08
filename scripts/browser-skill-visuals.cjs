// Packaged original assets and production targeting/effect paths. No World writes.
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const path = require("node:path");
const { launchBrowser, serveBuild, installFrameClock, enterGame, resumeGame,
  state, nearHome, clickWorld, worldPoint, movePointer, reportFailure, pauseOnSkillVisual,
} = require("./browser-lane-support.cjs");
const { fresh } = require("./browser-champions.cjs");

async function snapshot(page, output, name) {
  const style = await page.addStyleTag({content:"#entry{visibility:hidden!important}"});
  try { await page.screenshot({ path: path.join(output, `${name}.png`) }); }
  finally { await style.evaluate((node) => node.remove()); }
}

async function aim(page, slot, point, expected, output, name) {
  await page.keyboard.press("qwer"[slot]);
  await page.waitForFunction((slot) => window.__kivaState.aim === slot, slot);
  const css = await worldPoint(page, point);
  assert(css[0] > 16 && css[0] < page.viewportSize().width - 16
    && css[1] > 16 && css[1] < page.viewportSize().height - 16, "aim sample is inside the live canvas");
  await movePointer(page, ...css);
  await page.waitForFunction(({id,shape,point}) => {
    const indicator = window.__kivaState.indicator;
    return indicator?.ability_id === id && indicator.shape === shape && indicator.textured
      && indicator.status === "ready"
      && Math.hypot(...indicator.cursor_ground.map((v,i) => v-point[i])) < 0.1;
  }, {...expected,point});
  const view = (await state(page)).indicator;
  assert.equal(view.status, "ready");
  if (["line","volley","arrow"].includes(view.shape)) {
    assert(Math.abs(Math.hypot(...view.target.map((v,i) => v - view.origin[i])) - view.range) < 0.002,
      "near direction clicks still display the full actual range");
  }
  if (view.shape === "volley") assert.equal(view.rays, 7);
  if (view.shape === "arrow") assert(Math.abs(view.width - 2.6) < 0.0001);
  await snapshot(page, output, name);
  return view;
}

async function cameraRoundtrip(page) {
  const before = (await state(page)).camera_center;
  await page.keyboard.down("ArrowRight");
  await page.waitForFunction((before) => Math.hypot(...window.__kivaState.camera_center.map((v,i) => v-before[i])) > 0.2, before);
  await page.keyboard.up("ArrowRight");
  await page.waitForFunction(() => window.__kivaState.indicator?.cursor_ground);
  const error = await page.evaluate(() => {
    const s = window.__kivaState;
    const [x,z] = s.indicator.cursor_ground;
    const m = s.view_projection;
    const w = m[3]*x + m[11]*z + m[15];
    const point = [(m[0]*x+m[8]*z+m[12])/w*s.viewport[0]/2+s.viewport[0]/2,
      s.viewport[1]/2-(m[1]*x+m[9]*z+m[13])/w*s.viewport[1]/2];
    return Math.hypot(...point.map((v,i) => v-window.kivaPointer.position()[i]));
  });
  assert(error < 1, `current-camera indicator ray must roundtrip to the visible CSS pointer (${error})`);
}

async function exercise(browser, site, config, output) {
  const context = await browser.newContext(config);
  const page = await context.newPage();
  page.setDefaultTimeout(180000);
  const errors=[]; const warnings=[]; const assets=new Set();
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("response", (response) => { if(response.status()===200 && response.url().includes("/assets/vfx/"))assets.add(response.url()); });
  page.on("console", (message) => {
    const text=message.text();
    if(message.type()==="error" || /%c(?:WARN|ERROR)%c/.test(text)) {
      console.log("Browser:",text);
      if(!/selected adapter is using a driver that only supports software rendering|ScreenSpaceAmbientOcclusionPlugin not loaded/.test(text))warnings.push(text);
    }
  });
  const suffix = `${config.viewport.width}x${config.viewport.height}-dpr${config.deviceScaleFactor}`;
  try {
    await installFrameClock(page);
    await enterGame(page, site.url, 1);
    await page.waitForFunction(() => window.__kivaState.effects?.ready
      && window.__kivaState.units.find((u) => u.id===window.__kivaState.player_id).model?.ready);
    let current=await state(page); let point=nearHome(current,6,0);
    await aim(page,0,point,{id:11,shape:"line"},output,`jarvan-q-indicator-${suffix}`);
    await cameraRoundtrip(page);
    await pauseOnSkillVisual(page,11,"burst",()=>clickWorld(page,point));
    await snapshot(page,output,`jarvan-q-thrust-${suffix}`);
    await resumeGame(page);
    await pauseOnSkillVisual(page,12,"shield",()=>page.keyboard.press("w"),8);
    await snapshot(page,output,`jarvan-w-shield-${suffix}`);
    await resumeGame(page);
    await pauseOnSkillVisual(page,12,"shield",async()=>{},35);
    await snapshot(page,output,`jarvan-w-shield-only-${suffix}`);
    await resumeGame(page);
    current=await state(page); point=nearHome(current,7,-1);
    const preview=await aim(page,2,point,{id:13,shape:"standard"},output,`jarvan-e-indicator-${suffix}`);
    await pauseOnSkillVisual(page,13,"standard",()=>clickWorld(page,point));
    const planted=(await state(page)).objects.find((object)=>object.kind==="Standard"&&object.active);
    assert(planted && Math.hypot(...planted.position.map((v,i)=>v-preview.target[i]))<0.15);
    await snapshot(page,output,`jarvan-e-standard-${suffix}`);

    await fresh(page,2);
    assert.equal((await state(page)).effects.active.length,0,"new hero has no old session's effect instances");
    current=await state(page);point=nearHome(current,6,-2);
    const volley=await aim(page,1,point,{id:22,shape:"volley"},output,`ashe-w-indicator-${suffix}`);
    assert(volley.angle>0 && volley.angle<Math.PI/4);
    await pauseOnSkillVisual(page,22,"projectile",()=>clickWorld(page,point),12);
    await snapshot(page,output,`ashe-w-volley-${suffix}`);

    await fresh(page,2);
    current=await state(page);point=nearHome(current,7,3);
    await aim(page,2,point,{id:23,shape:"scout"},output,`ashe-e-indicator-${suffix}`);
    await pauseOnSkillVisual(page,23,"scout",()=>clickWorld(page,point),4);
    await snapshot(page,output,`ashe-e-drone-${suffix}`);
    const flying=await state(page);
    const scout=flying.objects.find((object)=>object.owner===flying.player_id&&object.kind==="Scout");
    assert(scout,"the screenshot follows the real scouting carrier");
    await resumeGame(page);
    await page.evaluate(()=>window.__laneSetFrameStep(250));
    await page.waitForFunction((id)=>window.__kivaState.objects.some((object)=>object.id===id&&object.kind==="Reveal")
      &&window.__kivaState.effects.active.some((effect)=>effect.ability_id===23&&effect.phase==="reveal"),scout.id);
    await page.evaluate(()=>window.__laneSetFrameStep(1000/60));
    await pauseOnSkillVisual(page,23,"reveal",async()=>{});
    await snapshot(page,output,`ashe-e-scan-${suffix}`);

    await fresh(page,2);
    current=await state(page);point=nearHome(current,6,0);
    await aim(page,3,point,{id:24,shape:"arrow"},output,`ashe-r-indicator-${suffix}`);
    await pauseOnSkillVisual(page,24,"projectile",()=>clickWorld(page,point),12);
    await snapshot(page,output,`ashe-r-crystal-arrow-${suffix}`);
    await resumeGame(page);
    await page.evaluate(()=>window.__laneSetFrameStep(250));
    await page.waitForFunction(()=>!window.__kivaState.effects.active.some((effect)=>effect.phase==="projectile"));
    await page.evaluate(()=>window.__laneSetFrameStep(1000/60));
    await fresh(page,1);
    assert.equal((await state(page)).indicator,null);
    assert.equal((await state(page)).effects.active.length,0);
    assert([...assets].some((url)=>url.endsWith(".png")) && [...assets].some((url)=>url.endsWith(".glb")));
    assert.deepEqual(errors,[]);
    assert.deepEqual(warnings,[],"no runtime warnings beyond the known software-adapter limits");
    console.log("PASS original skill textures/meshes, actual indicators, paused effects, same-frame camera and clean reselection",suffix);
  } catch(error) {
    await reportFailure(page);
    console.log("Skill diagnostics:",JSON.stringify(await page.evaluate(()=>({effects:window.__kivaState?.effects,indicator:window.__kivaState?.indicator}))));
    throw error;
  } finally {await context.close();}
}

async function main() {
  const output=process.env.KIVA_SCREENSHOT_DIR||"/tmp/kiva-browser-tests/artifacts/skill-visuals";
  await fs.mkdir(output,{recursive:true});
  const browser=await launchBrowser();const site=await serveBuild(process.env.KIVA_SITE_DIR||"dist");
  try {
    const cases=[{viewport:{width:1280,height:800},deviceScaleFactor:1},
      {viewport:{width:800,height:600},deviceScaleFactor:1},
      {viewport:{width:1000,height:720},deviceScaleFactor:2}];
    const requested=process.env.KIVA_VFX_CASE;
    const selected=cases.filter((c)=>!requested||`${c.viewport.width}x${c.viewport.height}@${c.deviceScaleFactor}`===requested);
    assert(selected.length,"unknown KIVA_VFX_CASE");
    for(const config of selected)await exercise(browser,site,config,output);
  } finally {await browser.close();await site.close();}
}
main().catch((error)=>{console.error(error);process.exitCode=1;});
