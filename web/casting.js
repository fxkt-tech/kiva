/* Local input preference only; targeting and cast validation stay in Rust. */
(() => {
  "use strict";
  const root = document.documentElement;
  const storageKey = "kiva-cast-mode";
  const valid = (mode) => mode === "normal" || mode === "quick";
  let mode = "quick";
  try {
    const saved = localStorage.getItem(storageKey);
    if (valid(saved)) mode = saved;
  } catch {}
  // Install before WASM starts, so its first input frame sees the preference.
  root.dataset.castMode = mode;

  document.addEventListener("DOMContentLoaded", () => {
    const select = document.getElementById("cast-mode");
    select.value = mode;
    select.addEventListener("change", () => {
      if (!valid(select.value)) { select.value = mode; return; }
      mode = select.value;
      root.dataset.castMode = mode;
      try { localStorage.setItem(storageKey, mode); } catch {}
    });
  }, { once: true });
})();
