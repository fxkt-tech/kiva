/* Resolve once before installing any game/viewer bridge or creating the App. */
(() => {
  "use strict";
  const viewer = new URLSearchParams(window.location.search).get("view") === "model";
  Object.defineProperty(window, "kivaPage", { value: Object.freeze({ isViewer: () => viewer }) });
  document.documentElement.setAttribute("data-kiva-view", viewer ? "model" : "game");
})();
