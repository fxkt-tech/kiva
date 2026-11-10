/* Mode controls submit a selection; Rust owns the roster and enemy behavior. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
  let current = 1;
  let pending = 0;

  function paint() {
    const selected = pending || current;
    document.getElementById("enemy-mode-target")?.setAttribute("aria-pressed", String(selected === 1));
    document.getElementById("enemy-mode-combat")?.setAttribute("aria-pressed", String(selected === 2));
    const hint = document.getElementById("enemy-mode-hint");
    if (hint) hint.textContent = selected === 1
      ? "1 个静止靶子 · 100 万生命 · 不会攻击"
      : "愁铃客镇守 · 两阶段对战 · 弹反制造破绽";
    const status = document.getElementById("hud-enemy-mode");
    if (status) status.textContent = `${current === 1 ? "靶子" : "实战"}模式 · F2 切换`;
  }

  function select(mode) {
    pending = mode;
    paint();
  }

  window.kivaEnemies = Object.freeze({
    takeMode() {
      const requested = pending;
      pending = 0;
      return requested;
    },
    render(mode) {
      if (mode === 1 || mode === 2) current = mode;
      paint();
    },
  });

  document.addEventListener("keydown", (event) => {
    if (event.code !== "F2" || event.repeat || !event.isTrusted || !window.kivaInput?.isActive()) return;
    event.preventDefault();
    select((pending || current) === 1 ? 2 : 1);
  }, true);

  function mount() {
    document.getElementById("enemy-mode-target")?.addEventListener("click", () => select(1));
    document.getElementById("enemy-mode-combat")?.addEventListener("click", () => select(2));
    paint();
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount, { once: true });
  } else mount();
})();
