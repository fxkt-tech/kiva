/* Read-only snapshots: simulation owns targets, health, action availability and all timers. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
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
    1: { melee: "伏浪锚", kind: "锤子", ranged: "引冰落", skillLabels: ["锤", "铳"], starter: "锤击 · 范围击飞", remote: "白灵淼 · 单发引冰落", finisher: "妹妹三枪", condition: "锤 E 原目标浮空时按 Tab" },
    2: { melee: "谪星·摧山", kind: "链斧", ranged: "羿之影", skillLabels: ["斧", "弓"], starter: "链斧横扫 · 减速", remote: "白灵淼 · 射出链斧", finisher: "拉近 · 妹妹三箭", condition: "弓 Q 成功僵直原目标后按 Tab" },
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
  function style(id, name, value) {
    node(id)?.style.setProperty(name, value);
  }

  function paint() {
    if (!mounted || !latest) return;
    const set = sets[loadout.set];
    const alive = latest.health > 0;
    text("hud-persona", latest.persona);
    text("hud-weapon", latest.weapon);
    progress("hud-health-bar", latest.fraction, latest.health, loadout.maximum);
    text("hud-status", latest.context);

    const selected = alive ? target : null;
    hidden("hud-target", !selected);
    text("hud-target-name", selected?.name || "");
    progress("hud-target-bar", selected ? selected.health / selected.maximum : 0, selected?.health, selected?.maximum);

    text("hud-start-label", set.skillLabels[0]);
    text("hud-ranged-skill-label", set.skillLabels[1]);
    text("hud-combo-label", "联动");
    for (const [id, pair] of [["hud-pair1", 1], ["hud-pair2", 2]]) {
      const active = loadout.set === pair;
      toggle(id, "active", active);
      attribute(id, "aria-current", active);
      attribute(id, "aria-label", `套装 ${pair} · ${sets[pair].melee} / ${sets[pair].ranged}${active && resources.axeAway ? " · 回收中" : ""}`);
    }

    progress("hud-stamina-bar", resources.maximum > 0 ? resources.stamina / resources.maximum : 0, resources.stamina, resources.maximum);
    // These are the current simulation cooldown durations (combat.rs); no Web clock.
    for (const [id, cooldown, total, available, key, caption] of [
      ["hud-skill-start", resources.melee, 3, loadout.melee, "E", set.starter],
      ["hud-skill-ranged", resources.ranged, 4, loadout.ranged, "Q", set.remote],
    ]) {
      text(`${id}-time`, cooldown > 0 ? `${cooldown.toFixed(1)}s` : "");
      toggle(id, "cooling", alive && cooldown > 0);
      style(id, "--cooldown-fill", `${Math.max(0, 1 - cooldown / total) * 100}%`);
      ready(id, alive && available && cooldown <= 0);
      const restriction = !alive ? "倒地中不可用" : cooldown > 0 ? `冷却 ${cooldown.toFixed(1)}s` : !available ? "当前条件不可用" : "就绪";
      attribute(id, "title", `${key} · ${caption} · ${restriction}`);
      attribute(id, "aria-label", `${key} · ${caption} · ${restriction}`);
    }
    ready("hud-skill-combo", alive && loadout.combo && latest.combo > 0);
    attribute("hud-skill-combo", "title", `Tab · 联动 · ${set.finisher} · ${set.condition}`);
    attribute("hud-skill-combo", "aria-label", `Tab · 联动 · ${set.finisher} · ${set.condition}`);
    // Reserved UI: simulation does not yet own energy, durability or rage values.
    for (const id of ["hud-ranged-energy-bar", "hud-melee-energy-bar", "hud-rage-value-bar"]) {
      progress(id, 0, 0, 100);
    }
    style("hud-rage-value", "--rage-fraction", "0");
    ready("hud-skill-rage", false);

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
