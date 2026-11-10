/* Read-only snapshots: simulation owns targets, health, action availability and all timers. */
(() => {
  "use strict";
  let latest = null;
  let target = null;
  let melee = "愁铃长刀";
  let loadout = { set: 1, mode: 0, maximum: 100, attack: false, skill: false, guard: false, combo: false };
  let mounted = document.readyState !== "loading";
  const nonnegative = value => Number.isFinite(value) ? Math.max(0, value) : 0;
  const label = value => typeof value === "string" ? value : "";
  const number = value => Math.ceil(nonnegative(value)).toLocaleString("en-US");
  const node = id => document.getElementById(id);
  const sets = Object.freeze({
    1: { ranged: "狙击枪", meleeIcon: "blade", rangedIcon: "sniper", starter: "重刀震击", finisher: "狙击追击", starterMode: 0 },
    2: { ranged: "火炮", meleeIcon: "blade", rangedIcon: "cannon", starter: "火炮跃进", finisher: "空中旋刀", starterMode: 1 },
  });

  function text(id, value) {
    const element = node(id);
    if (element) element.textContent = value;
  }
  function hidden(id, value) {
    const element = node(id);
    if (element) element.hidden = value;
  }
  function attribute(id, name, value) {
    node(id)?.setAttribute(name, String(value));
  }
  function toggle(id, name, value) {
    node(id)?.classList.toggle(name, value);
  }
  function ready(id, value) {
    toggle(id, "ready", value);
    attribute(id, "aria-disabled", !value);
  }
  function progress(id, fraction, current, maximum) {
    const bar = node(id);
    if (!bar) return;
    bar.max = 1;
    bar.value = Math.min(1, nonnegative(fraction));
    bar.setAttribute("aria-valuetext", `${number(current)} / ${number(maximum)}`);
  }
  function icon(id, name) {
    attribute(id, "href", `#hud-icon-${name}`);
  }

  function paint() {
    if (!mounted || !latest) return;
    const set = sets[loadout.set];
    const alive = latest.health > 0;
    text("hud-persona", latest.persona);
    text("hud-weapon", latest.weapon);
    text("hud-health", number(latest.health));
    text("hud-health-max", number(loadout.maximum));
    progress("hud-health-bar", latest.fraction, latest.health, loadout.maximum);
    text("hud-status", latest.context);

    const selected = alive ? target : null;
    hidden("hud-target", !selected);
    text("hud-target-name", selected?.name || "");
    text("hud-target-status", selected?.status || "");
    text("hud-target-health", number(selected?.health));
    text("hud-target-max", number(selected?.maximum));
    progress("hud-target-bar", selected ? selected.health / selected.maximum : 0, selected?.health, selected?.maximum);

    text("hud-skill-caption", set.starter);
    text("hud-melee-name", melee);
    text("hud-ranged-name", set.ranged);
    icon("hud-melee-icon", set.meleeIcon);
    icon("hud-ranged-icon", set.rangedIcon);
    icon("hud-start-icon", set.starterMode === 0 ? set.meleeIcon : set.rangedIcon);
    icon("hud-combo-icon", loadout.set === 1 ? "sniper" : set.meleeIcon);
    for (const [id, active] of [["hud-melee", loadout.mode === 0], ["hud-ranged", loadout.mode === 1], ["hud-pair1", loadout.set === 1], ["hud-pair2", loadout.set === 2]]) {
      toggle(id, "active", active);
      attribute(id, "aria-current", active);
    }

    const cooling = latest.cooldown > 0;
    text("hud-cooldown", latest.cooldown.toFixed(1));
    hidden("hud-cooldown", !cooling);
    toggle("hud-skill-start", "cooling", cooling);
    ready("hud-skill-start", alive && loadout.skill && !cooling);
    ready("hud-skill-attack", alive && loadout.attack);
    ready("hud-skill-guard", alive && loadout.guard);
    ready("hud-skill-combo", alive && loadout.combo && latest.combo > 0);
    const restriction = !alive ? "倒地中不可用" : cooling ? `冷却 ${latest.cooldown.toFixed(1)}s`
      : loadout.mode !== set.starterMode ? `切换${set.starterMode === 0 ? "近战" : "远程"}使用`
      : !loadout.skill ? "当前动作中不可用" : "就绪";
    attribute("hud-skill-start", "title", `E · ${set.starter} · ${restriction}`);
    attribute("hud-skill-combo", "title", `Tab · ${set.finisher}`);
    attribute("hud-skill-start", "aria-label", `E · ${set.starter} · ${restriction}`);
    attribute("hud-skill-combo", "aria-label", `Tab · ${set.finisher}`);

    const hasCombo = alive && latest.combo > 0;
    hidden("hud-combo", !hasCombo);
    hidden("hud-status", hasCombo);
    text("hud-combo-name", set.finisher);
    text("hud-combo-time", `${latest.combo.toFixed(1)}s`);
  }

  window.kivaHud = Object.freeze({
    renderMelee(name) {
      if (label(name)) melee = name;
      paint();
    },
    render(persona, weapon, healthFraction, health, combo, cooldown, context) {
      latest = {
        persona: label(persona), weapon: label(weapon),
        fraction: Math.min(1, nonnegative(healthFraction)), health: nonnegative(health),
        combo: nonnegative(combo), cooldown: nonnegative(cooldown), context: label(context),
      };
      paint();
    },
    renderLoadout(set, mode, maximum, attackReady, skillReady, guardReady, comboReady) {
      loadout = {
        set: set === 2 ? 2 : 1, mode: mode === 1 ? 1 : 0, maximum: nonnegative(maximum),
        attack: attackReady === true, skill: skillReady === true, guard: guardReady === true, combo: comboReady === true,
      };
      paint();
    },
    renderTarget(name, current, maximum, status = "锁定目标") {
      const health = nonnegative(current);
      const max = nonnegative(maximum);
      target = label(name) && health > 0 && max > 0
        ? { name: label(name), health: Math.min(health, max), maximum: max, status: label(status) } : null;
      paint();
    },
  });
  if (!mounted) document.addEventListener("DOMContentLoaded", () => {
    mounted = true;
    paint();
  }, { once: true });
})();
