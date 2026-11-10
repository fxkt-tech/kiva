/* Appearance requests affect player visuals; Rust owns the applied selection. */
(() => {
  "use strict";
  let current = 1;
  let pending = 0;

  function paint() {
    const selected = pending || current;
    document.getElementById("appearance-choulingke")?.setAttribute("aria-pressed", String(selected === 1));
    document.getElementById("appearance-lihuowang")?.setAttribute("aria-pressed", String(selected === 2));
    const hint = document.getElementById("appearance-hint");
    if (hint) hint.textContent = selected === 1
      ? "愁铃客 · 身体与服饰 · F3 切换"
      : "李火旺 · 身体与服饰 · F3 切换";
    const status = document.getElementById("hud-appearance");
    if (status) status.textContent = `外观 · ${current === 1 ? "愁铃客" : "李火旺"} · F3 切换`;
  }

  function select(appearance) {
    pending = appearance;
    paint();
  }

  window.kivaAppearance = Object.freeze({
    takeAppearance() {
      const requested = pending;
      pending = 0;
      return requested;
    },
    render(appearance) {
      if (appearance === 1 || appearance === 2) current = appearance;
      paint();
    },
  });

  document.addEventListener("keydown", (event) => {
    if (event.code !== "F3" || event.repeat || !event.isTrusted || !window.kivaInput?.isActive()) return;
    event.preventDefault();
    select((pending || current) === 1 ? 2 : 1);
  }, true);

  function mount() {
    document.getElementById("appearance-choulingke")?.addEventListener("click", () => select(1));
    document.getElementById("appearance-lihuowang")?.addEventListener("click", () => select(2));
    paint();
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount, { once: true });
  } else mount();
})();
