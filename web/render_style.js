/* Render-style requests are presentation only; Rust owns the applied style and
   which styles the level registers. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
  const labels = { 1: "写实", 2: "岩彩" };
  let current = 1;
  let count = 1;
  let pending = 0;

  function paint() {
    const status = document.getElementById("hud-render-style");
    if (!status) return;
    const text = count > 1
      ? `渲染 · ${labels[current]} · F8 切换`
      : `渲染 · ${labels[current]} · 本关仅此一种`;
    // Rust renders every frame; touch the DOM only when the text changes.
    if (status.textContent !== text) status.textContent = text;
  }

  window.kivaRenderStyle = Object.freeze({
    takeStyle() {
      const requested = pending;
      pending = 0;
      return requested;
    },
    render(style, styles) {
      if (style in labels) current = style;
      if (Number.isInteger(styles) && styles > 0) count = styles;
      paint();
    },
  });

  document.addEventListener("keydown", (event) => {
    if (event.code !== "F8" || event.repeat || !event.isTrusted || !window.kivaInput?.isActive()) return;
    event.preventDefault();
    // One "next style" request; Rust skips it on single-style levels.
    pending = 1;
  }, true);

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", paint, { once: true });
  } else paint();
})();
