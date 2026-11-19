/* Appearance requests affect player visuals; Rust owns the applied selection. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
  let current = window.kivaPage?.isStation() ? 3 : 2;
  let pending = 0;
  const choices = [
    [1, "choulingke", "愁铃客"],
    [2, "lihuowang", "李火旺"],
    [3, "zhuque", "朱雀"],
  ];
  const label = value => choices.find(choice => choice[0] === value)[2];

  function paint() {
    const selected = pending || current;
    for (const [value, id] of choices) {
      document.getElementById(`appearance-${id}`)?.setAttribute("aria-pressed", String(selected === value));
    }
    const hint = document.getElementById("appearance-hint");
    if (hint) hint.textContent = `${label(selected)} · 身体与服饰 · F3 切换`;
    const status = document.getElementById("hud-appearance");
    if (status) status.textContent = `外观 · ${label(current)} · F3 切换`;
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
      if (choices.some(choice => choice[0] === appearance)) current = appearance;
      paint();
    },
  });

  document.addEventListener("keydown", (event) => {
    if (event.code !== "F3" || event.repeat || !event.isTrusted || !window.kivaInput?.isActive()) return;
    event.preventDefault();
    const index = choices.findIndex(choice => choice[0] === (pending || current));
    select(choices[(index + 1) % choices.length][0]);
  }, true);

  function mount() {
    for (const [value, id] of choices) {
      document.getElementById(`appearance-${id}`)?.addEventListener("click", () => select(value));
    }
    paint();
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount, { once: true });
  } else mount();
})();
