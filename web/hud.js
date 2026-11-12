/* Read-only snapshots: simulation owns targets, health, action availability and all timers. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
  let latest = null;
  let target = null;
  let loadout = { set: 1, maximum: 1000, attack: false, melee: false, ranged: false, guard: false, combo: false };
  let resources = { stamina: 100, maximum: 100, melee: 0, ranged: 0, axeAway: false, defense: 0 };
  let energy = { sun: 0, sunMaximum: 10, moon: 0, moonMaximum: 10, rage: 0, rageMaximum: 20, enraged: false, ready: false };
  let durability = { values: [], maximum: 50 };
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

  const fraction = (value, maximum) => maximum > 0 ? Math.min(1, nonnegative(value) / maximum) : 0;
  // The edge meter is a clipped outline: empty hides it, full drops the clip.
  function meter(key, value) {
    const percent = Math.round(Math.min(1, nonnegative(value)) * 100);
    attribute(`fill-${key}`, "y", 1 - percent / 100);
    attribute(`fill-${key}`, "height", percent / 100 + 0.1);
    style(`edge-${key}`, "clip-path", percent >= 100 ? "none" : "");
    style(`edge-${key}`, "visibility", percent <= 0 ? "hidden" : "visible");
    attribute(`meter-${key}`, "aria-valuenow", percent);
    toggle(`meter-${key}`, "full", percent >= 100);
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
      attribute(id, "aria-label", `套装 ${pair} · ${sets[pair].melee} / ${sets[pair].ranged} · 耐久 ${number(durability.values[pair - 1])} / ${number(durability.maximum)}${active && resources.axeAway ? " · 回收中" : ""}`);
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
      const empowered = energy.enraged || (key === "E" ? energy.sun >= energy.sunMaximum : energy.moon >= energy.moonMaximum);
      const description = `${key} · ${caption}${empowered ? " · 强化" : ""} · ${restriction}`;
      toggle(id, "empowered", empowered);
      attribute(id, "title", description);
      attribute(id, "aria-label", description);
    }
    ready("hud-skill-combo", alive && loadout.combo && latest.combo > 0);
    attribute("hud-skill-combo", "title", `Tab · 联动 · ${set.finisher} · ${set.condition}`);
    attribute("hud-skill-combo", "aria-label", `Tab · 联动 · ${set.finisher} · ${set.condition}`);
    // Sun fills the melee skill, moon the ranged skill; rage is the stored
    // meter, or what is left of the enraged state.
    for (const [id, key, value, maximum] of [
      ["melee-energy", "meleeEnergy", energy.sun, energy.sunMaximum],
      ["ranged-energy", "rangedEnergy", energy.moon, energy.moonMaximum],
      ["rage-value", "rageValue", energy.rage, energy.rageMaximum],
    ]) {
      progress(`hud-${id}-bar`, fraction(value, maximum), value, maximum);
      toggle(`hud-${id}`, "full", maximum > 0 && value >= maximum);
      meter(key, fraction(value, maximum));
    }
    style("hud-rage-value", "--rage-fraction", String(fraction(energy.rage, energy.rageMaximum)));
    toggle("hud-rage-value", "enraged", energy.enraged);
    ready("hud-skill-rage", alive && energy.ready);
    toggle("hud-skill-rage", "enraged", energy.enraged);
    attribute("hud-skill-rage", "aria-label", `X · 暴怒技 · ${energy.enraged ? `暴怒中 · 剩余 ${Math.ceil(energy.rage)}s` : energy.ready ? "可用" : "暴怒值未满"}`);
    for (const pair of [1, 2]) {
      meter(`pair${pair}Durability`, fraction(durability.values[pair - 1], durability.maximum));
    }

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
    renderEnergy(sun, sunMaximum, moon, moonMaximum, rage, rageMaximum, enraged, rageReady) {
      const bounded = (value, maximum) => Math.min(nonnegative(value), nonnegative(maximum));
      energy = {
        sun: bounded(sun, sunMaximum), sunMaximum: nonnegative(sunMaximum),
        moon: bounded(moon, moonMaximum), moonMaximum: nonnegative(moonMaximum),
        rage: bounded(rage, rageMaximum), rageMaximum: nonnegative(rageMaximum),
        enraged: enraged === true, ready: rageReady === true,
      };
      paint();
    },
    // One value per weapon pair, in the page's pair-slot order.
    renderDurability(values, maximum) {
      const max = nonnegative(maximum);
      durability = { values: Array.from(values || [], value => Math.min(nonnegative(value), max)).slice(0, 2), maximum: max };
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
