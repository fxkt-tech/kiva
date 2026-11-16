/* Read-only snapshots: simulation owns targets, health, action availability and all timers.
   The page never keeps a clock: every number comes from the latest frame, and the only
   motion over time (the damage trail, the link's breathing) is CSS.
   Rust sends one named-field frame per update (client/hud.rs `HudFrame`); the shape is pinned
   by the fixtures under tests/fixtures/hud, which the Node tests start from. */
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
  // The page reads at most this many Boss swings, one bit each.
  const MAX_SWINGS = 31;
  const ORDINALS = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十"];
  const WORDING = ["melee", "meleeGlyph", "ranged", "rangedGlyph", "meleeSkill", "rangedSkill", "combo", "comboCondition"];
  const LOCKS = ["onScreen", "aside", "behind"];

  // The latest frame, normalised; null until Rust sends one.
  let latest = null;
  // Each pair's wording by page slot, from Rust's loadout rows (defineLoadouts).
  let sets = {};
  let mounted = document.readyState !== "loading";
  const nonnegative = value => Number.isFinite(value) ? Math.max(0, value) : 0;
  const label = value => typeof value === "string" ? value : "";
  const flag = value => value === true;
  const count = (value, maximum) => Number.isInteger(value) && value > 0 ? Math.min(value, maximum) : 0;
  const object = value => value && typeof value === "object" ? value : {};
  const number = value => Math.ceil(nonnegative(value)).toLocaleString("en-US");
  const fraction = (value, maximum) => maximum > 0 ? Math.min(1, nonnegative(value) / maximum) : 0;
  const percent = value => `${Math.round(Math.min(1, nonnegative(value)) * 1000) / 10}%`;
  const node = id => document.getElementById(id);

  // A value out of its maximum, never above it.
  function meter(value) {
    const { current, maximum } = object(value);
    const max = nonnegative(maximum);
    return { current: Math.min(nonnegative(current), max), maximum: max };
  }
  function clock(value) {
    const { left, total } = object(value);
    return { left: nonnegative(left), total: nonnegative(total) };
  }
  // Every field bounded; anything missing or malformed reads as empty, never throws.
  function normalize(frame) {
    const f = object(frame);
    const loadout = object(f.loadout);
    const energy = object(f.energy);
    const durability = object(f.durability);
    const durabilityMaximum = nonnegative(durability.maximum);
    const target = f.target && typeof f.target === "object" ? meter(f.target) : null;
    const lock = object(f.lock);
    const cue = f.cue && typeof f.cue === "object" ? f.cue : null;
    return {
      context: label(f.context),
      health: meter(f.health),
      stamina: meter(f.stamina),
      loadout: {
        set: count(loadout.set, Number.MAX_SAFE_INTEGER),
        melee: flag(loadout.meleeReady), ranged: flag(loadout.rangedReady), combo: flag(loadout.comboReady),
      },
      melee: clock(f.melee), ranged: clock(f.ranged), combo: clock(f.combo),
      axeAway: flag(f.axeAway),
      energy: {
        ...Object.fromEntries(["sun", "moon", "rage"].flatMap(name => {
          const { current, maximum } = meter({ current: energy[name], maximum: energy[`${name}Maximum`] });
          return [[name, current], [`${name}Maximum`, maximum]];
        })),
        enraged: flag(energy.enraged), rageReady: flag(energy.rageReady),
      },
      durability: {
        values: Array.isArray(durability.values) ? durability.values.map(value => Math.min(nonnegative(value), durabilityMaximum)) : [],
        maximum: durabilityMaximum,
      },
      target: target && label(f.target.name) && target.current > 0 && target.maximum > 0 ? {
        name: label(f.target.name), health: target.current, maximum: target.maximum, status: label(f.target.status),
        phases: f.target.phases === 2 ? 2 : 1, phase: f.target.phases === 2 && f.target.phase === 2 ? 2 : 1,
      } : null,
      cue: cue ? {
        striking: flag(cue.striking), left: nonnegative(cue.left), total: nonnegative(cue.total),
        chain: count(cue.chain, Number.MAX_SAFE_INTEGER), swings: count(cue.swings, MAX_SWINGS),
        resolved: Number.isInteger(cue.resolved) && cue.resolved >= 0 ? cue.resolved : 0,
      } : null,
      lock: LOCKS.includes(lock.kind) ? {
        kind: lock.kind, x: Math.min(1, nonnegative(lock.x)), y: Math.min(1, nonnegative(lock.y)),
        distance: nonnegative(lock.distance), right: flag(lock.right),
      } : null,
    };
  }

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
    style(id, "--cooldown-fill", `${(1 - fraction(cooldown, total)) * 100}%`);
    const ready = alive && available && cooldown <= 0;
    toggle(id, "ready", ready);
    toggle(id, "empowered", alive && empowered);
    attribute(id, "aria-disabled", !ready);
    const restriction = !alive ? "倒地中不可用" : cooldown > 0 ? `冷却 ${cooldown.toFixed(1)}s` : !available ? "当前条件不可用" : "就绪";
    attribute(id, "aria-label", `${description}${empowered ? " · 强化" : ""} · ${restriction}`);
  }

  function paintTarget(f, alive) {
    const selected = alive ? f.target : null;
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

    const cue = f.cue;
    hidden("hud-boss-cue", !cue);
    hidden("hud-boss-state", Boolean(cue) || !selected.status);
    text("hud-boss-state", selected.status);
    if (cue) {
      toggle("hud-boss-cue", "striking", cue.striking);
      text("hud-boss-chain", cue.chain > 1 ? `连招 · 第${ORDINALS[cue.chain - 1] || cue.chain}式` : "");
      text("hud-boss-word", cue.striking ? "出招" : "蓄势");
      text("hud-boss-time", cue.striking ? "" : cue.left.toFixed(1));
      style("hud-boss-count-fill", "width", percent(fraction(cue.left, cue.total)));
      segments("hud-boss-swings", cue.swings, 0, index => (cue.resolved & (1 << index)) !== 0);
    }

    const lock = f.lock;
    const onScreen = lock?.kind === "onScreen";
    hidden("hud-reticle", !onScreen);
    hidden("hud-offscreen", !lock || onScreen);
    if (onScreen) {
      style("hud-reticle", "left", percent(lock.x));
      style("hud-reticle", "top", percent(lock.y));
      text("hud-reticle-distance", `${lock.distance.toFixed(1)}m`);
    } else if (lock) {
      toggle("hud-offscreen", "right", lock.right);
      text("hud-offscreen-label", lock.kind === "behind" ? "身后" : "侧方");
    }
  }

  function paint() {
    if (!mounted || !latest) return;
    const f = latest;
    const set = sets[f.loadout.set];
    if (!set) return;
    const alive = f.health.current > 0;
    const health = `${number(f.health.current)} / ${number(f.health.maximum)}`;

    // Center: health blocks, stamina, the link.
    text("hud-status", f.context);
    blocks("hud-health-blocks", f.health.current, f.health.maximum);
    text("hud-health-text", health);
    attribute("hud-health-blocks", "aria-valuenow", fraction(f.health.current, f.health.maximum).toFixed(3));
    attribute("hud-health-blocks", "aria-valuetext", health);
    const stamina = fraction(f.stamina.current, f.stamina.maximum);
    style("hud-stamina-fill", "width", percent(stamina));
    attribute("hud-stamina", "aria-valuenow", stamina.toFixed(3));
    const linked = alive && f.loadout.combo && f.combo.left > 0;
    toggle("hud-link", "ready", linked);
    toggle("hud-link", "wait", !linked);
    attribute("hud-link", "aria-disabled", !linked);
    style("hud-link", "--left", linked ? percent(fraction(f.combo.left, f.combo.total)) : "0%");
    text("hud-link-text", linked ? set.combo : set.comboCondition);
    text("hud-link-time", linked ? f.combo.left.toFixed(1) : "");
    attribute("hud-link", "aria-label", `Tab · 联动 · ${set.combo} · ${linked ? `剩余 ${f.combo.left.toFixed(1)}s` : set.comboCondition}`);

    const energy = f.energy;
    // Left: the sister.
    text("hud-sister-weapon", `${set.ranged} · ${set.rangedGlyph}`);
    text("hud-key-q-glyph", set.rangedGlyph);
    const moonFull = energy.moonMaximum > 0 && energy.moon >= energy.moonMaximum;
    segments("hud-moon", ENERGY_SEGMENTS, Math.floor(fraction(energy.moon, energy.moonMaximum) * ENERGY_SEGMENTS + 1e-6));
    text("hud-moon-value", String(Math.floor(energy.moon)));
    toggle("hud-moon-row", "full", moonFull);
    hidden("hud-moon-tag", !(moonFull || energy.enraged));
    const durability = f.durability.values[f.loadout.set - 1];
    const wear = fraction(durability, f.durability.maximum);
    segments("hud-durability", DURABILITY_SEGMENTS, Math.floor(wear * DURABILITY_SEGMENTS + 1e-6));
    text("hud-durability-value", number(durability));
    toggle("hud-durability-row", "low", wear <= LOW_DURABILITY);
    key("hud-key-q", f.ranged.left, f.ranged.total, f.loadout.ranged, alive, energy.enraged || moonFull, `Q · ${set.rangedSkill}`);

    // Right: the brother.
    text("hud-brother-weapon", `${set.melee} · ${set.meleeGlyph}${f.axeAway ? " · 回收中" : ""}`);
    text("hud-key-e-glyph", set.meleeGlyph);
    const sunFull = energy.sunMaximum > 0 && energy.sun >= energy.sunMaximum;
    segments("hud-sun", ENERGY_SEGMENTS, Math.floor(fraction(energy.sun, energy.sunMaximum) * ENERGY_SEGMENTS + 1e-6));
    text("hud-sun-value", String(Math.floor(energy.sun)));
    toggle("hud-sun-row", "full", sunFull);
    segments("hud-rage", RAGE_SEGMENTS, Math.ceil(fraction(energy.rage, energy.rageMaximum) * RAGE_SEGMENTS - 1e-6));
    text("hud-rage-value", String(Math.ceil(energy.rage)));
    toggle("hud-rage-row", "ready", alive && energy.rageReady);
    toggle("hud-rage-row", "enraged", energy.enraged);
    attribute("hud-rage-row", "aria-disabled", !(alive && energy.rageReady));
    attribute("hud-rage-row", "aria-label", `X · 暴怒技 · ${energy.enraged ? `暴怒中 · 剩余 ${Math.ceil(energy.rage)}s` : energy.rageReady ? "可用" : "暴怒值未满"}`);
    key("hud-key-e", f.melee.left, f.melee.total, f.loadout.melee, alive, energy.enraged || sunFull, `E · ${set.meleeSkill}`);

    paintTarget(f, alive);
  }

  // JSON text from Rust; anything unreadable is ignored and the last good value kept.
  function parse(json) {
    try { return JSON.parse(json); } catch { return null; }
  }

  window.kivaHud = Object.freeze({
    // Every pair's wording, keyed by page slot; a pair with any field missing is dropped.
    defineLoadouts(json) {
      const parsed = parse(json);
      if (!parsed || typeof parsed !== "object") return;
      const defined = {};
      for (const [slot, row] of Object.entries(parsed)) {
        if (!/^[1-9]\d*$/.test(slot) || !row || typeof row !== "object") continue;
        if (WORDING.every(name => typeof row[name] === "string")) {
          defined[slot] = Object.freeze(Object.fromEntries(WORDING.map(name => [name, row[name]])));
        }
      }
      sets = Object.freeze(defined);
      paint();
    },
    // The whole HUD for one update.
    frame(json) {
      const parsed = parse(json);
      if (!parsed || typeof parsed !== "object") return;
      latest = normalize(parsed);
      paint();
    },
  });
  if (!mounted) document.addEventListener("DOMContentLoaded", () => {
    mounted = true;
    paint();
  }, { once: true });
})();
