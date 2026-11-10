/* Read-only snapshots: simulation owns targets, health, action availability and all timers. */
(() => {
  "use strict";
  let latest = null;
  let target = null;
  let loadout = { set: 1, maximum: 1000, attack: false, melee: false, ranged: false, guard: false, combo: false };
  let resources = { stamina: 100, maximum: 100, melee: 0, ranged: 0, axeAway: false, defense: 0 };
  let mounted = document.readyState !== "loading";
  const nonnegative = value => Number.isFinite(value) ? Math.max(0, value) : 0;
  const label = value => typeof value === "string" ? value : "";
  const number = value => Math.ceil(nonnegative(value)).toLocaleString("en-US");
  const node = id => document.getElementById(id);
  const sets = Object.freeze({
    1: { melee: "锤子", ranged: "引冰落", meleeIcon: "hammer", rangedIcon: "musket", starter: "锤击 · 范围击飞", remote: "白灵淼 · 单发引冰落", finisher: "妹妹三枪", condition: "锤 E 原目标浮空时按 Tab" },
    2: { melee: "链斧", ranged: "羿之影", meleeIcon: "axe", rangedIcon: "bow", starter: "链斧横扫 · 减速", remote: "白灵淼 · 射出链斧", finisher: "拉近 · 妹妹三箭", condition: "弓 Q 成功僵直原目标后按 Tab" },
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
    text("hud-melee-name", resources.axeAway ? "链斧 · 回收中" : set.melee);
    text("hud-ranged-name", set.ranged);
    icon("hud-melee-icon", set.meleeIcon);
    icon("hud-ranged-icon", set.rangedIcon);
    icon("hud-start-icon", set.meleeIcon);
    icon("hud-combo-icon", set.rangedIcon);
    for (const [id, active] of [["hud-melee", !resources.axeAway], ["hud-ranged", true], ["hud-pair1", loadout.set === 1], ["hud-pair2", loadout.set === 2]]) {
      toggle(id, "active", active);
      attribute(id, "aria-current", active);
    }

    progress("hud-stamina-bar", resources.maximum > 0 ? resources.stamina / resources.maximum : 0, resources.stamina, resources.maximum);
    text("hud-stamina", number(resources.stamina));
    text("hud-stamina-max", number(resources.maximum));
    text("hud-ranged-caption", set.remote);
    icon("hud-ranged-skill-icon", set.rangedIcon);
    for (const [id, cooldownId, cooldown, available, key, caption] of [
      ["hud-skill-start", "hud-cooldown", resources.melee, loadout.melee, "E", set.starter],
      ["hud-skill-ranged", "hud-ranged-cooldown", resources.ranged, loadout.ranged, "Q", set.remote],
    ]) {
      text(cooldownId, cooldown.toFixed(1));
      hidden(cooldownId, cooldown <= 0);
      toggle(id, "cooling", cooldown > 0);
      ready(id, alive && available && cooldown <= 0);
      const restriction = !alive ? "倒地中不可用" : cooldown > 0 ? `冷却 ${cooldown.toFixed(1)}s` : !available ? "当前条件不可用" : "就绪";
      attribute(id, "title", `${key} · ${caption} · ${restriction}`);
      attribute(id, "aria-label", `${key} · ${caption} · ${restriction}`);
    }
    ready("hud-skill-attack", alive && loadout.attack && !resources.axeAway);
    ready("hud-skill-guard", alive && loadout.guard);
    ready("hud-skill-combo", alive && loadout.combo && latest.combo > 0);
    const defenseLabels = ["按下拆招 · 长按防御 · 松开弹反", "拆招窗口", "等待防御", "防御中", "弹反窗口"];
    attribute("hud-skill-guard", "aria-label", `右键 · ${defenseLabels[resources.defense]}`);
    toggle("hud-skill-guard", "active", resources.defense > 0);
    attribute("hud-skill-combo", "title", `Tab · ${set.finisher} · ${set.condition}`);
    attribute("hud-skill-combo", "aria-label", `Tab · ${set.finisher} · ${set.condition}`);

    const hasCombo = alive && latest.combo > 0;
    hidden("hud-combo", !hasCombo);
    hidden("hud-status", hasCombo);
    text("hud-combo-name", set.finisher);
    text("hud-combo-time", `${latest.combo.toFixed(1)}s`);
  }

  window.kivaHud = Object.freeze({
    render(persona, weapon, healthFraction, health, combo, cooldown, context) {
      latest = {
        persona: label(persona), weapon: label(weapon),
        fraction: Math.min(1, nonnegative(healthFraction)), health: nonnegative(health),
        combo: nonnegative(combo), cooldown: nonnegative(cooldown), context: label(context),
      };
      paint();
    },
    renderLoadout(set, maximum, attackReady, meleeReady, rangedReady, guardReady, comboReady) {
      loadout = {
        set: set === 2 ? 2 : 1, maximum: nonnegative(maximum),
        attack: attackReady === true, melee: meleeReady === true, ranged: rangedReady === true, guard: guardReady === true, combo: comboReady === true,
      };
      paint();
    },
    renderResources(stamina, maximum, meleeCooldown, rangedCooldown, axeAway, defense) {
      resources = { stamina: Math.min(nonnegative(stamina), nonnegative(maximum)), maximum: nonnegative(maximum), melee: nonnegative(meleeCooldown), ranged: nonnegative(rangedCooldown), axeAway: axeAway === true, defense: Number.isInteger(defense) && defense >= 0 && defense <= 4 ? defense : 0 };
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
