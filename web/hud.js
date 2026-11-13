/* Read-only snapshots: simulation owns targets, health, action availability and all timers.
   The page never keeps a clock: every number comes from the latest snapshot, and the only
   motion over time (the damage trail, the link's breathing) is CSS. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
  // Health per block; above MAX_BLOCKS a bar is drawn as one continuous block.
  const BLOCK = 100;
  const MAX_BLOCKS = 20;
  const ENERGY_SEGMENTS = 10;
  const RAGE_SEGMENTS = 20;
  const DURABILITY_SEGMENTS = 20;
  const LOW_DURABILITY = 0.3;
  // Simulation cooldown durations (combat.rs), for the water level only.
  const COOLDOWN = { melee: 3, ranged: 4 };
  const ORDINALS = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];

  let latest = null;
  let target = null;
  let cue = { kind: 0, left: 0, total: 0, chain: 0, swings: 0, resolved: 0 };
  let lock = { mode: 0, x: 0, y: 0, distance: 0, side: 0 };
  let loadout = { set: 1, maximum: 1000, attack: false, melee: false, ranged: false, guard: false, combo: false };
  let resources = { stamina: 100, maximum: 100, melee: 0, ranged: 0, axeAway: false, defense: 0 };
  let energy = { sun: 0, sunMaximum: 10, moon: 0, moonMaximum: 10, rage: 0, rageMaximum: 20, enraged: false, ready: false };
  let durability = { values: [], maximum: 50 };
  let mounted = document.readyState !== "loading";
  const nonnegative = value => Number.isFinite(value) ? Math.max(0, value) : 0;
  const label = value => typeof value === "string" ? value : "";
  const number = value => Math.ceil(nonnegative(value)).toLocaleString("en-US");
  const fraction = (value, maximum) => maximum > 0 ? Math.min(1, nonnegative(value) / maximum) : 0;
  const percent = value => `${Math.round(Math.min(1, nonnegative(value)) * 1000) / 10}%`;
  const node = id => document.getElementById(id);
  const sets = Object.freeze({
    1: { melee: "伏浪锚", meleeKind: "锤", ranged: "引冰落", rangedKind: "铳", starter: "锤击 · 范围击飞", remote: "白灵淼 · 单发引冰落", finisher: "妹妹三枪", condition: "锤 E 击飞原目标后可用" },
    2: { melee: "谪星·摧山", meleeKind: "斧", ranged: "羿之影", rangedKind: "弓", starter: "链斧横扫 · 减速", remote: "白灵淼 · 射出链斧", finisher: "拉近 · 妹妹三箭", condition: "弓 Q 僵直原目标后可用" },
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
  function style(id, name, value) {
    node(id)?.style.setProperty(name, value);
  }

  // Children of `id`, rebuilt only when their number changes.
  function items(id, count, make) {
    const container = node(id);
    if (!container) return [];
    if (container.kivaItems?.length !== count) {
      const made = Array.from({ length: count }, make);
      container.replaceChildren(...made.map(item => item.root));
      container.kivaItems = made;
    }
    return container.kivaItems;
  }
  function segments(id, count, filled, hit = () => false) {
    items(id, count, () => ({ root: document.createElement("i") })).forEach((item, index) => {
      item.root.classList.toggle("on", index < filled);
      item.root.classList.toggle("hit", hit(index));
    });
  }
  // One block per BLOCK of `maximum`; the last may be shorter. `current` fills from the left.
  function blocks(id, current, maximum) {
    const count = maximum > 0 ? Math.ceil(maximum / BLOCK) : 0;
    const continuous = count > MAX_BLOCKS;
    const made = items(id, continuous ? 1 : count, () => {
      const root = document.createElement("i");
      const trail = document.createElement("b");
      const fill = document.createElement("b");
      trail.className = "trail";
      fill.className = "fill";
      root.append(trail, fill);
      return { root, trail, fill };
    });
    made.forEach((item, index) => {
      const start = continuous ? 0 : index * BLOCK;
      const size = continuous ? maximum : Math.min(BLOCK, maximum - start);
      const filled = percent(fraction(current - start, size));
      item.root.style.setProperty("--hud-block", String(continuous ? 1 : size / BLOCK));
      item.fill.style.setProperty("width", filled);
      item.trail.style.setProperty("width", filled);
    });
  }

  function key(id, cooldown, total, available, alive, empowered, description) {
    text(`${id}-time`, cooldown > 0 ? cooldown.toFixed(1) : "");
    toggle(id, "cooling", alive && cooldown > 0);
    style(id, "--cooldown-fill", `${Math.max(0, 1 - cooldown / total) * 100}%`);
    const ready = alive && available && cooldown <= 0;
    toggle(id, "ready", ready);
    toggle(id, "empowered", alive && empowered);
    attribute(id, "aria-disabled", !ready);
    const restriction = !alive ? "倒地中不可用" : cooldown > 0 ? `冷却 ${cooldown.toFixed(1)}s` : !available ? "当前条件不可用" : "就绪";
    attribute(id, "aria-label", `${description}${empowered ? " · 强化" : ""} · ${restriction}`);
  }

  function paintTarget(alive) {
    const selected = alive ? target : null;
    hidden("hud-target", !selected);
    if (!selected) {
      hidden("hud-reticle", true);
      hidden("hud-offscreen", true);
      return;
    }
    text("hud-target-name", selected.name);
    text("hud-target-phase", selected.phases > 1 ? `${ORDINALS[selected.phase - 1] || selected.phase}阶段` : "");
    text("hud-target-health", `${number(selected.health)} / ${number(selected.maximum)}`);
    attribute("hud-target-meter", "aria-valuenow", fraction(selected.health, selected.maximum).toFixed(3));
    attribute("hud-target-meter", "aria-valuetext", `${number(selected.health)} / ${number(selected.maximum)}`);
    // Two phases: the later phase on the left, the current one draining first from the right.
    const twoPhases = selected.phases > 1;
    const first = selected.phase <= 1;
    blocks("hud-target-later", twoPhases && first ? selected.maximum : selected.health, selected.maximum);
    blocks("hud-target-first", twoPhases ? (first ? selected.health : 0) : 0, twoPhases ? selected.maximum : 0);
    toggle("hud-target-first", "spent", twoPhases && !first);
    hidden("hud-target-split", !twoPhases);

    const striking = cue.kind === 2;
    const cueShown = cue.kind === 1 || striking;
    hidden("hud-boss-cue", !cueShown);
    hidden("hud-boss-state", cueShown || !selected.status);
    text("hud-boss-state", selected.status);
    if (cueShown) {
      toggle("hud-boss-cue", "striking", striking);
      text("hud-boss-chain", cue.chain > 1 ? `连招 · 第${ORDINALS[cue.chain - 1] || cue.chain}式` : "");
      text("hud-boss-word", striking ? "出招" : "蓄势");
      text("hud-boss-time", striking ? "" : cue.left.toFixed(1));
      style("hud-boss-count-fill", "width", percent(fraction(cue.left, cue.total)));
      segments("hud-boss-swings", cue.swings, 0, index => (cue.resolved & (1 << index)) !== 0);
    }

    hidden("hud-reticle", lock.mode !== 1);
    hidden("hud-offscreen", lock.mode !== 2 && lock.mode !== 3);
    if (lock.mode === 1) {
      style("hud-reticle", "left", percent(lock.x));
      style("hud-reticle", "top", percent(lock.y));
      text("hud-reticle-distance", `${lock.distance.toFixed(1)}m`);
    } else if (lock.mode === 2 || lock.mode === 3) {
      toggle("hud-offscreen", "right", lock.side > 0);
      text("hud-offscreen-label", lock.mode === 3 ? "身后" : "侧方");
    }
  }

  function paint() {
    if (!mounted || !latest) return;
    const set = sets[loadout.set];
    const alive = latest.health > 0;

    // Center: health blocks, stamina, the link.
    text("hud-status", latest.context);
    blocks("hud-health-blocks", latest.health, loadout.maximum);
    text("hud-health-text", `${number(latest.health)} / ${number(loadout.maximum)}`);
    attribute("hud-health-blocks", "aria-valuenow", latest.fraction.toFixed(3));
    attribute("hud-health-blocks", "aria-valuetext", `${number(latest.health)} / ${number(loadout.maximum)}`);
    const stamina = fraction(resources.stamina, resources.maximum);
    style("hud-stamina-fill", "width", percent(stamina));
    attribute("hud-stamina", "aria-valuenow", stamina.toFixed(3));
    const linked = alive && loadout.combo && latest.combo > 0;
    toggle("hud-link", "ready", linked);
    toggle("hud-link", "wait", !linked);
    attribute("hud-link", "aria-disabled", !linked);
    style("hud-link", "--left", linked ? percent(latest.combo / Math.max(latest.combo, latest.window || latest.combo)) : "0%");
    text("hud-link-text", linked ? set.finisher : set.condition);
    text("hud-link-time", linked ? latest.combo.toFixed(1) : "");
    attribute("hud-link", "aria-label", `Tab · 联动 · ${set.finisher} · ${linked ? `剩余 ${latest.combo.toFixed(1)}s` : set.condition}`);

    // Left: the sister.
    text("hud-sister-weapon", `${set.ranged} · ${set.rangedKind}`);
    text("hud-key-q-glyph", set.rangedKind);
    const moonFull = energy.moonMaximum > 0 && energy.moon >= energy.moonMaximum;
    segments("hud-moon", ENERGY_SEGMENTS, Math.floor(fraction(energy.moon, energy.moonMaximum) * ENERGY_SEGMENTS + 1e-6));
    text("hud-moon-value", String(Math.floor(energy.moon)));
    toggle("hud-moon-row", "full", moonFull);
    hidden("hud-moon-tag", !(moonFull || energy.enraged));
    const wear = fraction(durability.values[loadout.set - 1], durability.maximum);
    segments("hud-durability", DURABILITY_SEGMENTS, Math.floor(wear * DURABILITY_SEGMENTS + 1e-6));
    text("hud-durability-value", number(durability.values[loadout.set - 1]));
    toggle("hud-durability-row", "low", wear <= LOW_DURABILITY);
    key("hud-key-q", resources.ranged, COOLDOWN.ranged, loadout.ranged, alive, energy.enraged || moonFull, `Q · ${set.remote}`);

    // Right: the brother.
    text("hud-brother-weapon", `${set.melee} · ${set.meleeKind}${resources.axeAway ? " · 回收中" : ""}`);
    text("hud-key-e-glyph", set.meleeKind);
    const sunFull = energy.sunMaximum > 0 && energy.sun >= energy.sunMaximum;
    segments("hud-sun", ENERGY_SEGMENTS, Math.floor(fraction(energy.sun, energy.sunMaximum) * ENERGY_SEGMENTS + 1e-6));
    text("hud-sun-value", String(Math.floor(energy.sun)));
    toggle("hud-sun-row", "full", sunFull);
    segments("hud-rage", RAGE_SEGMENTS, Math.ceil(fraction(energy.rage, energy.rageMaximum) * RAGE_SEGMENTS - 1e-6));
    text("hud-rage-value", String(Math.ceil(energy.rage)));
    toggle("hud-rage-row", "ready", alive && energy.ready);
    toggle("hud-rage-row", "enraged", energy.enraged);
    attribute("hud-rage-row", "aria-disabled", !(alive && energy.ready));
    attribute("hud-rage-row", "aria-label", `X · 暴怒技 · ${energy.enraged ? `暴怒中 · 剩余 ${Math.ceil(energy.rage)}s` : energy.ready ? "可用" : "暴怒值未满"}`);
    key("hud-key-e", resources.melee, COOLDOWN.melee, loadout.melee, alive, energy.enraged || sunFull, `E · ${set.starter}`);

    paintTarget(alive);
  }

  window.kivaHud = Object.freeze({
    render(persona, weapon, healthFraction, health, combo, cooldown, context) {
      const remaining = nonnegative(combo);
      latest = {
        persona: label(persona), weapon: label(weapon),
        fraction: Math.min(1, nonnegative(healthFraction)), health: nonnegative(health),
        combo: remaining, cooldown: nonnegative(cooldown), context: label(context),
        // The longest window seen in this link, so the drain line has a full length.
        window: remaining > 0 ? Math.max(remaining, latest?.combo > 0 ? latest.window : 0) : 0,
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
    // `phase` of `phases`: a Boss has two, each with `maximum` health.
    renderTarget(name, current, maximum, status = "", phase = 1, phases = 1) {
      const health = nonnegative(current);
      const max = nonnegative(maximum);
      const count = phases === 2 ? 2 : 1;
      target = label(name) && health > 0 && max > 0
        ? { name: label(name), health: Math.min(health, max), maximum: max, status: label(status), phases: count, phase: count === 2 && phase === 2 ? 2 : 1 }
        : null;
      paint();
    },
    // The Boss's strike in progress: kind 0 none, 1 telegraph (left / total seconds), 2 striking.
    // `resolved` has one bit per swing already resolved against the player.
    renderBossCue(kind, left, total, chain, swings, resolved) {
      cue = {
        kind: kind === 1 || kind === 2 ? kind : 0,
        left: nonnegative(left), total: nonnegative(total),
        chain: Number.isInteger(chain) && chain > 0 ? chain : 0,
        swings: Number.isInteger(swings) && swings > 0 ? Math.min(swings, 31) : 0,
        resolved: Number.isInteger(resolved) && resolved >= 0 ? resolved : 0,
      };
      paint();
    },
    // Where the locked target is: mode 0 none, 1 on screen at (x, y) as fractions of the
    // frame, 2 off screen to one side, 3 behind the camera; side is -1 left or 1 right.
    renderLock(mode, x, y, distance, side) {
      lock = {
        mode: mode === 1 || mode === 2 || mode === 3 ? mode : 0,
        x: Math.min(1, nonnegative(x)), y: Math.min(1, nonnegative(y)),
        distance: nonnegative(distance), side: side > 0 ? 1 : -1,
      };
      paint();
    },
  });
  if (!mounted) document.addEventListener("DOMContentLoaded", () => {
    mounted = true;
    paint();
  }, { once: true });
})();
