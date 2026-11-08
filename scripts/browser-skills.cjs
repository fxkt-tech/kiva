const { resetToSelection } = require("./browser-support.cjs");
// Captured skill-slot interaction across the actual champion kits. Exact old
// generic Root/Stun/Dash/channel behavior remains in native and equipment tests;
// browser-champions covers the new field objects, vision and compound casts.
const assert=require("node:assert/strict");
const {launchBrowser,serveBuild,installFrameClock,selectAndEnter,selectCharacter,resumeGame,pause,state,hero,
  nearHome,clickUi,clickWorld,reportFailure}=require("./browser-lane-support.cjs");
async function chooseNext(page,id){
  await pause(page);await resetToSelection(page);
  await page.waitForFunction(()=>window.__kivaState.session.phase==="Selecting"&&!document.getElementById("enter").disabled);
  await selectCharacter(page,id);await resumeGame(page);return state(page);
}
async function exercise(page,id){
  const before=await state(page);
  const directional=id===1?0:1;
  const names=before.abilities.map((ability)=>ability.name);
  assert.deepEqual(await page.locator('[data-slot] small').allTextContents(),names);
  await clickUi(page,`[data-slot="${directional}"]`,1200);
  await page.waitForFunction((slot)=>window.__kivaState.aim===slot,directional);
  const aimed=await state(page);
  assert.equal(aimed.abilities[directional].cooldown,0);
  assert(Math.hypot(...hero(aimed).position.map((v,i)=>v-hero(before).position[i]))<0.1,"held HUD input must not walk");
  await clickWorld(page,hero(before).position,"right");
  await page.waitForFunction(()=>window.__kivaState.aim===null);
  assert.equal((await state(page)).abilities[directional].cooldown,0,"cancelled aim must not spend cooldown");
  await clickUi(page,'[data-slot="2"]');await page.waitForFunction(()=>window.__kivaState.aim===2);
  const destination=nearHome(before,4,-1);
  await clickWorld(page,destination);
  await page.waitForFunction(()=>window.__kivaState.abilities[2].cooldown>0);
  const point=await state(page);
  assert(Math.hypot(...hero(point).position.map((v,i)=>v-hero(before).position[i]))<0.1,"point skills must not leak a movement command");
  if(id===1)await page.waitForFunction(()=>window.__kivaState.objects.some((object)=>object.kind==="Standard"&&object.active));
  else {
    assert.equal(point.abilities[2].mechanics.charges,0);
    assert.equal(point.abilities[0].available,false);
    assert.equal(point.abilities[0].mechanics.focus,0);
  }
  await page.keyboard.press(id===1?"q":"w");
  await page.waitForFunction((slot)=>window.__kivaState.aim===slot,directional);
  await clickWorld(page,nearHome(before,7,-2));
  await page.waitForFunction((slot)=>window.__kivaState.abilities[slot].cooldown>0,directional);
  if(id===1){await page.keyboard.press("w");await page.waitForFunction(()=>window.__kivaState.units.find((unit)=>unit.id===window.__kivaState.player_id).shield>0);}
  else {
    await page.keyboard.press("q");
    assert.equal((await state(page)).aim,null,"zero-focus self skill cannot arm an old target mode");
  }
  await pause(page);
  const frozen=await state(page);
  const clock=await page.evaluate(()=>performance.now());
  await page.evaluate(()=>window.__laneSetFrameStep(250));
  try {await page.waitForFunction((clock)=>performance.now()>clock+500,clock);} finally {await page.evaluate(()=>window.__laneSetFrameStep(1000/60));}
  const stable=await state(page);
  assert.equal(stable.tick,frozen.tick);
  assert.deepEqual(stable.abilities,frozen.abilities,"all real cooldown/charge/recast/focus state freezes");
  assert.equal(hero(stable).shield,hero(frozen).shield);
  assert.equal(hero(stable).visual_height,hero(frozen).visual_height);
  await resumeGame(page);assert.equal((await state(page)).player_id,frozen.player_id);
  console.log(`PASS ${id===1?"Jarvan":"Ashe"} real mouse/keyboard slots, held UI isolation, cancellation and paused state ${page.viewportSize().width}x${page.viewportSize().height}`);
}
async function main(){
  const browser=await launchBrowser();let site,page;
  try{
    site=await serveBuild(process.env.KIVA_SITE_DIR||"dist");
    const cases=[{viewport:{width:1280,height:800},deviceScaleFactor:1},{viewport:{width:800,height:600},deviceScaleFactor:1},{viewport:{width:1000,height:720},deviceScaleFactor:2}];
    const requested=process.env.KIVA_SKILLS_CASE;
    const selected=cases.filter((c)=>!requested||`${c.viewport.width}x${c.viewport.height}@${c.deviceScaleFactor}`===requested);
    assert(selected.length,"unknown KIVA_SKILLS_CASE");
    for(const config of selected){
      const context=await browser.newContext(config);page=await context.newPage();page.setDefaultTimeout(120000);
      const errors=[];page.on("pageerror",(error)=>errors.push(error.message));await installFrameClock(page);
      await selectAndEnter(page,site.url,1);await exercise(page,1);
      await chooseNext(page,2);await exercise(page,2);
      assert.deepEqual(errors,[]);await context.close();page=null;
    }
  }catch(error){if(page)await reportFailure(page);throw error;}
  finally{try{await browser.close();}finally{if(site)await site.close();}}
}
main().catch((error)=>{console.error(error);process.exitCode=1;});
