// Real champion kits through normal captured input; no simulation write bridge.
const assert=require("node:assert/strict");
const fs=require("node:fs/promises");
const path=require("node:path");
const {launchBrowser,serveBuild,installFrameClock,selectAndEnter,selectCharacter,resumeGame,pause,state,hero,enemyHero,
  nearHome,nearMiddle,moveTo,clickWorld,clickUnit,clickUi,observed,waitTicks,reportFailure}=require("./browser-lane-support.cjs");

async function fresh(page,id) {
  const old=await state(page);
  if (!old.paused) await pause(page);
  await page.locator("#reselect").click();
  await page.waitForFunction(()=>window.__kivaState.session.phase==="Selecting"&&!document.getElementById("enter").disabled);
  await selectCharacter(page,id); await resumeGame(page);
  assert.notEqual((await state(page)).player_id,old.player_id);
  return state(page);
}
async function shot(page,name) {
  if(process.env.KIVA_BROWSER_HEADED!=="1")return;
  const directory=process.env.KIVA_SCREENSHOT_DIR||"/tmp/kiva-browser-tests/artifacts/champions";
  await fs.mkdir(directory,{recursive:true});
  const size=page.viewportSize();
  const style=await page.addStyleTag({content:"#entry{visibility:hidden!important}"});
  try {await page.screenshot({path:path.join(directory,`${name}-${size.width}x${size.height}.png`)});}
  finally {await style.evaluate((node)=>node.remove());}
}
async function approach(page,offset=0,lateral=-3) {
  await moveTo(page,nearMiddle(await state(page),offset,lateral));
  await page.keyboard.press("s");
  await page.evaluate(()=>window.__laneSetFrameStep(250));
  try {
    return await observed(page,()=>{
      const snapshot=window.__kivaState;
      const player=snapshot.units.find((unit)=>unit.id===snapshot.player_id);
      if(!player.alive)throw new Error("The live opponent defeated the player while approaching");
      const enemy=snapshot.units.find((unit)=>unit.team==="enemy"&&unit.role==="Hero"&&unit.alive);
      return enemy&&player.control.can_cast&&Math.hypot(...enemy.position.map((v,i)=>v-player.position[i]))<6.3?snapshot:false;
    });
  } finally {await page.evaluate(()=>window.__laneSetFrameStep(1000/60));}
}
async function jarvan(page,url,deep) {
  const initial=await selectAndEnter(page,url,1);
  assert.equal(hero(initial).name,"德玛西亚皇子");
  assert.equal(hero(initial).display_attributes.move_speed,340);
  assert.equal(hero(initial).display_attributes.attack_range,175);
  assert(Math.abs(hero(initial).attributes.attack_speed-0.7896)<0.001);
  assert.deepEqual(initial.abilities.map((ability)=>ability.name),["巨龙撞击","黄金圣盾","德邦军旗","天崩地裂"]);
  assert.equal(initial.match.bases.enemy,null);
  assert((await page.locator("#enemy-base-health").textContent()).includes("未知"));
  await page.keyboard.press("w");
  const shielded=await observed(page,()=>window.__kivaState.units.find((u)=>u.id===window.__kivaState.player_id).shield>0?window.__kivaState:false);
  assert.equal(shielded.aim,null);
  assert(await page.locator("#shield-text").isVisible());
  await pause(page); await shot(page,"jarvan-spear-shield"); await resumeGame(page);
  const origin=hero(await state(page)).position;
  const far=nearHome(initial,20,1.4);
  await clickUi(page,'[data-slot="2"]');
  await page.waitForFunction(()=>window.__kivaState.aim===2);
  await clickWorld(page,far);
  const planted=await observed(page,()=>window.__kivaState.objects.some((o)=>o.kind==="Standard"&&o.active)?window.__kivaState:false);
  const flag=planted.objects.find((o)=>o.kind==="Standard"&&o.owner===planted.player_id);
  assert(flag);
  assert(Math.abs(Math.hypot(...flag.position.map((v,i)=>v-origin[i]))-planted.abilities[2].range)<0.08,"far E must plant at maximum range");
  assert(Math.hypot(...hero(planted).position.map((v,i)=>v-origin[i]))<0.1,"E placement must not walk");

  const reset=await fresh(page,1);
  const start=hero(reset).position;
  const length=Math.hypot(...reset.map.enemy_base.map((v,i)=>v-reset.map.ally_base[i]));
  const direction=reset.map.enemy_base.map((v,i)=>(v-reset.map.ally_base[i])/length);
  const close=start.map((v,i)=>v+direction[i]*5.5);
  await page.keyboard.press("e"); await page.waitForFunction(()=>window.__kivaState.aim===2);
  await clickWorld(page,close);
  await page.waitForFunction(()=>window.__kivaState.objects.some((o)=>o.kind==="Standard"&&o.active));
  assert(hero(await state(page)).attributes.attack_speed>0.9,"actual flag aura stacks with learned passive");
  await page.evaluate(({start,goal})=>{
    window.__comboShieldQueued=null;
    function duringLink(event){
      const snapshot=typeof event.detail==="string"?JSON.parse(event.detail):event.detail;
      const player=snapshot.units.find((u)=>u.id===snapshot.player_id);
      if(snapshot.abilities[0].cooldown>0&&Math.hypot(...player.position.map((v,i)=>v-start[i]))>0.1
        &&Math.hypot(...player.position.map((v,i)=>v-goal[i]))>0.4){
        window.removeEventListener("kiva-state",duringLink);
        window.__comboShieldQueued={tick:snapshot.tick,position:player.position};
        document.documentElement.dataset.skill="1"; // Existing public skill intent, during actual committed motion.
      }
    }
    window.addEventListener("kiva-state",duringLink);
  },{start,goal:close});
  await clickUi(page,'[data-slot="0"]'); await clickWorld(page,close);
  await page.waitForFunction((goal)=>{
    const snapshot=window.__kivaState;
    const player=snapshot.units.find((u)=>u.id===snapshot.player_id);
    return snapshot.abilities[0].cooldown>0&&player.shield>0&&Math.hypot(...player.position.map((v,i)=>v-goal[i]))<0.3;
  },close);
  assert(await page.evaluate(()=>Boolean(window.__comboShieldQueued)),"W must be queued during real EQ motion");
  await pause(page); await shot(page,"jarvan-flag-combo"); await resumeGame(page);
  if(deep){
    await defensiveEquipment(page);
    // Stop outside R range. The only remaining movement toward the enemy
    // must come from the targeted cast itself, through normal captured input.
    await moveTo(page,nearMiddle(await state(page),-8,-7));
    await page.keyboard.press("s");
    // Wait for the opponent's actual visible mana to fall below her arrow's
    // cost. Waiting for a stun is unreliable: her real arrow can miss. No AI,
    // health or mana state is changed; normal armor items protect this setup.
    let ready;
    await page.evaluate(()=>window.__laneSetFrameStep(250));
    try {
      ready=await observed(page,()=>{
        const s=window.__kivaState;
        const p=s.units.find((u)=>u.id===s.player_id);
        if(!p.alive)throw new Error("Jarvan died before the targeted-cast scenario");
        const enemy=s.units.find((u)=>u.team==="enemy"&&u.role==="Hero"&&u.alive);
        return enemy&&enemy.mana<=75&&p.control.can_cast&&p.control.can_dash
          &&Math.hypot(...enemy.position.map((v,i)=>v-p.position[i]))>s.abilities[3].range+0.1?s:false;
      });
    } finally {await page.evaluate(()=>window.__laneSetFrameStep(1000/60));}
    const enemy=enemyHero(ready);
    await page.evaluate((target)=>{
      window.__targetedApproach=null;
      window.__targetedTrace=[];
      let previous;
      function recordApproach(event){
        const s=typeof event.detail==="string"?JSON.parse(event.detail):event.detail;
        const p=s.units.find((u)=>u.id===s.player_id);
        const enemy=s.units.find((u)=>u.id===target);
        const signature=JSON.stringify([p?.order,p?.action,p?.alive,p?.control.can_cast,
          p?.statuses.map((status)=>status.kind),s.abilities[3].cooldown>0,s.abilities[3].mechanics.recast]);
        if(signature!==previous){
          previous=signature;
          window.__targetedTrace.push({tick:s.tick,order:p?.order,action:p?.action,
            health:p?.health,mana:p?.mana,position:p?.position,target:enemy?.position,
            statuses:p?.statuses,notice:s.notice,cooldown:s.abilities[3].cooldown});
        }
        if(p?.order!=="Approaching cast")return;
        window.removeEventListener("kiva-state",recordApproach);
        window.__targetedApproach??={tick:s.tick,position:p.position,mana:p.mana,
          cooldown:s.abilities[3].cooldown,aim:s.aim,
          distance:enemy&&Math.hypot(...enemy.position.map((v,i)=>v-p.position[i]))};
      }
      window.addEventListener("kiva-state",recordApproach);
    },enemy.id);
    await page.keyboard.press("r"); await page.waitForFunction(()=>window.__kivaState.aim===3);
    const minion=ready.units.find((u)=>u.team==="enemy"&&u.role?.includes("Minion")&&u.alive);
    if(minion){await clickUnit(page,minion.id);assert.equal((await state(page)).aim,3,"R cannot select a minion");}
    await clickUnit(page,enemy.id);
    const approaching=await observed(page,()=>window.__targetedApproach);
    assert(approaching.distance>ready.abilities[3].range,"R must actually approach from outside its range");
    assert.equal(approaching.cooldown,0,"walking into cast range must not start cooldown");
    assert.equal(approaching.aim,null,"one enemy click submits the purpose and exits aim");
    assert(approaching.mana>=hero(ready).mana-1,"walking into cast range must not pay R mana");
    const enclosed=await observed(page,()=>window.__kivaState.objects.some((o)=>o.kind==="Terrain"&&o.owner===window.__kivaState.player_id)?window.__kivaState:false);
    assert(Math.hypot(...hero(enclosed).position.map((v,i)=>v-hero(ready).position[i]))>0.1,"R order must move Jarvan before the real terrain lands");
    assert(enclosed.abilities[3].mechanics.recast);
    assert(enclosed.abilities[3].cooldown===0,"recast affordance overrides the long normal cooldown");
    await pause(page); await shot(page,"jarvan-terrain"); await resumeGame(page);
    await page.waitForFunction(()=>window.__kivaState.abilities[3].mechanics.recast_ready);
    const before=hero(await state(page)).mana;
    await clickUi(page,'[data-slot="3"]');
    await page.waitForFunction(()=>!window.__kivaState.abilities[3].mechanics.recast);
    const removed=await state(page);
    assert(!removed.objects.some((o)=>o.kind==="Terrain"&&o.owner===removed.player_id));
    assert(Math.abs(hero(removed).mana-before)<4,"terrain removal has no second mana cost");
    assert(removed.abilities[3].cooldown>100);
  }
  console.log(`PASS Jarvan real spear/shield/max E/aura/EQ-midcast${deep?"/outside-range hero R approach/recast":""}`);
}
async function defensiveEquipment(page) {
  const initial=hero(await state(page));
  await page.keyboard.press("p");
  await page.waitForFunction(()=>window.__kivaState.equipment.can_manage&&!document.getElementById("equipment-editor").hidden);
  await page.evaluate(()=>window.__laneSetFrameStep(250));
  try {
    for(let count=1;count<=6;count++) {
      await page.locator('[data-equip-id="4"]').click();
      await page.waitForFunction((count)=>window.__kivaState.equipment.slots.filter((slot)=>slot.item?.id===4).length===count,count);
    }
  } finally {await page.evaluate(()=>window.__laneSetFrameStep(1000/60));}
  const equipped=await state(page);
  assert.equal(hero(equipped).max_health,initial.max_health,"survival uses real defense items rather than changing hero health");
  assert.equal(hero(equipped).attributes.armor,initial.attributes.armor+150);
  await page.locator("#equipment-close").click();await resumeGame(page);
}

async function ashe(page,deep){
  const initial=await fresh(page,2);
  assert.equal(hero(initial).name,"寒冰射手");
  assert.equal(hero(initial).display_attributes.move_speed,325);
  assert.equal(hero(initial).display_attributes.attack_range,600);
  assert.deepEqual(initial.abilities.map((ability)=>ability.name),["射手的专注","万箭齐发","鹰击长空","魔法水晶箭"]);
  assert.equal(initial.abilities[0].available,false);
  assert.equal(initial.abilities[0].mechanics.focus,0);
  assert.equal(initial.abilities[2].mechanics.charges,1);
  assert.equal(await page.locator('[data-slot="0"] .skill-state').textContent(),"0/4");
  await page.keyboard.press("q");
  await waitTicks(page,6);
  assert.equal(hero(await state(page)).empowered,false);
  await clickUi(page,'[data-slot="1"]'); await page.waitForFunction(()=>window.__kivaState.aim===1);
  await clickWorld(page,nearHome(initial,8,-3));
  await page.waitForFunction(()=>window.__kivaState.abilities[1].cooldown>0);
  assert.equal((await state(page)).aim,null);
  await page.keyboard.press("e"); await page.waitForFunction(()=>window.__kivaState.aim===2);
  await clickWorld(page,initial.map.enemy_base);
  await page.waitForFunction(()=>window.__kivaState.abilities[2].mechanics.charges===0);
  assert.equal(await page.locator('[data-slot="2"] .skill-state').textContent(),"0/2");
  await page.evaluate(()=>window.__laneSetFrameStep(250));
  try {await page.waitForFunction(()=>window.__kivaState.match.bases.enemy!==null);} finally {await page.evaluate(()=>window.__laneSetFrameStep(1000/60));}
  const revealed=await state(page);
  assert(revealed.units.some((u)=>u.team==="enemy"&&u.role==="Base"),"scout must reveal actual enemy units");
  assert(revealed.objects.some((o)=>o.kind==="Scout"||o.kind==="Reveal"));
  await pause(page); await shot(page,"ashe-bow-scout"); await resumeGame(page);
  if(deep){
    await fresh(page,2);
    await defensiveEquipment(page);
    const readyFocus=await approach(page,-3,-4);
    const focusTarget=enemyHero(readyFocus);
    assert(focusTarget?.alive,"focus needs an actual visible hero with enough health for four attacks");
    await clickUnit(page,focusTarget.id,"right");
    await page.evaluate(()=>window.__laneSetFrameStep(250));
    try {await page.waitForFunction((start)=>{
      const s=window.__kivaState;const p=s.units.find((u)=>u.id===s.player_id);
      if(!p.alive)throw new Error("Defensively equipped Ashe died before focus completed");
      if(s.tick>start+800)throw new Error("Real attack orders did not generate four focus stacks");
      return s.abilities[0].mechanics.focus===4;
    },readyFocus.tick);} finally {await page.evaluate(()=>window.__laneSetFrameStep(1000/60));}
    await page.keyboard.press("s");
    assert.equal((await state(page)).abilities[0].mechanics.focus,4);
    await clickUi(page,'[data-slot="0"]');
    await page.waitForFunction(()=>window.__kivaState.abilities[0].mechanics.empowered);
    const empowered=await state(page);
    assert.equal(empowered.aim,null);
    assert(hero(empowered).attributes.attack_speed>0.78);
    await pause(page); await shot(page,"ashe-focus"); await resumeGame(page);
    const ready=await state(page); const target=enemyHero(ready);
    assert(target?.alive,"actual R needs a visible enemy hero");
    await page.keyboard.press("r"); await page.waitForFunction(()=>window.__kivaState.aim===3);
    await clickWorld(page,target.position);
    const stunned=await observed(page,(id)=>{
      const s=window.__kivaState; const target=s.units.find((u)=>u.id===id);
      return target?.statuses.some((status)=>status.kind==="Stun")&&s.abilities[3].cooldown>0?s:false;
    },target.id);
    assert(stunned.units.find((u)=>u.id===target.id).health<target.health);
  }
  console.log(`PASS Ashe actual bow/zero-focus/W/charges/scout vision${deep?"/focus empowerment/hero arrow":""}`);
}
async function main(){
  const browser=await launchBrowser();let site,page;
  try{
    site=await serveBuild(process.env.KIVA_SITE_DIR||"dist");
    const cases=[{viewport:{width:1280,height:800},deviceScaleFactor:1},{viewport:{width:800,height:600},deviceScaleFactor:1},{viewport:{width:1000,height:720},deviceScaleFactor:2}];
    const requested=process.env.KIVA_CHAMPION_CASE;
    const selectedHero=process.env.KIVA_CHAMPION_HERO;
    assert(!selectedHero||["1","2"].includes(selectedHero),"unknown KIVA_CHAMPION_HERO");
    const selected=cases.filter((c)=>!requested||`${c.viewport.width}x${c.viewport.height}@${c.deviceScaleFactor}`===requested);
    assert(selected.length,"unknown KIVA_CHAMPION_CASE");
    for(const config of selected){
      const context=await browser.newContext(config);page=await context.newPage();page.setDefaultTimeout(120000);
      let failBrowser;
      const browserFailure=new Promise((_,reject)=>{failBrowser=reject;});
      const errors=[];page.on("pageerror",(error)=>{errors.push(error.message);failBrowser(error);});
      page.on("console",(message)=>{
        if(message.type()==="error"||/%c(?:WARN|ERROR)%c/.test(message.text())){
          console.log(`Browser console ${message.type()}:`,message.text());
        }
        if(message.text().includes("Quitting the application due to"))failBrowser(new Error(message.text()));
      });
      await installFrameClock(page);
      await Promise.race([(async()=>{
        if(selectedHero!=="2")await jarvan(page,site.url,config.viewport.width===1280);
        else await selectAndEnter(page,site.url,2);
        if(selectedHero!=="1")await ashe(page,config.viewport.width===1280);
      })(),browserFailure]);
      assert.deepEqual(errors,[]);await context.close();page=null;
    }
  }catch(error){if(page){
    console.log("Targeted cast diagnostics:",JSON.stringify(await page.evaluate(()=>({
      approach:window.__targetedApproach,trace:window.__targetedTrace,frames:window.__laneDiagnostics,
    }))));
    await reportFailure(page);
  }throw error;}
  finally{try{await browser.close();}finally{if(site)await site.close();}}
}
module.exports={main};
if(require.main===module)main().catch((error)=>{console.error(error);process.exitCode=1;});
