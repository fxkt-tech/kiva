/* Presentation only: all health, cooldown and combo authority stays in Rust. */
(() => {
  "use strict";
  let latest = null;
  let mounted = document.readyState !== "loading";
  const nonnegative = (value) => Number.isFinite(value) ? Math.max(0, value) : 0;
  const label = (value) => typeof value === "string" ? value : "";

  function text(id, value) {
    const node = document.getElementById(id);
    if (node) node.textContent = value;
  }

  function paint() {
    if (!mounted || !latest) return;
    text("hud-persona", latest.persona);
    text("hud-weapon", latest.weapon);
    text("hud-health", String(latest.health));
    const healthBar = document.getElementById("hud-health-bar");
    if (healthBar) {
      healthBar.max = 1;
      healthBar.value = latest.fraction;
    }
    text("hud-status", latest.context);
    text("hud-cooldown", latest.cooldown > 0 ? `E · ${latest.cooldown.toFixed(1)}s` : "就绪");
    const combo = document.getElementById("hud-combo");
    if (combo) combo.hidden = latest.combo <= 0;
    text("hud-combo-time", `${latest.combo.toFixed(1)}s`);
  }

  window.kivaHud = Object.freeze({
    render(persona, weapon, healthFraction, health, combo, cooldown, context) {
      latest = {
        persona: label(persona),
        weapon: label(weapon),
        fraction: Math.min(1, nonnegative(healthFraction)),
        health: Math.ceil(nonnegative(health)),
        combo: nonnegative(combo),
        cooldown: nonnegative(cooldown),
        context: label(context),
      };
      paint();
    },
  });

  if (!mounted) document.addEventListener("DOMContentLoaded", () => {
    mounted = true;
    paint();
  }, { once: true });
})();
