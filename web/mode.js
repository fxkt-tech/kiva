/* Resolve the inspection page and playable level before installing browser bridges. */
(() => {
  "use strict";
  const query = new URLSearchParams(window.location.search);
  const view = query.get("view");
  const viewer = view === "model";
  // Older station links use ?view=station.
  const level = query.get("level");
  const directStation = level === "station" || view === "station";
  const handscroll = !viewer && !directStation && level !== "bamboo" && view !== "game";
  const station = !viewer && (directStation || handscroll);
  Object.defineProperty(window, "kivaPage", { value: Object.freeze({
    isViewer: () => viewer, isStation: () => station, startsInHandscroll: () => handscroll,
  }) });
  document.documentElement.setAttribute("data-kiva-view", viewer ? "model" : "game");

  if (handscroll) document.documentElement.setAttribute("data-handscroll", "open");

  if (station) {
    document.title = handscroll ? "断轨听羽" : "Kiva · 废站 · 映面同行";
    const labelStation = () => {
      for (const label of document.querySelectorAll("[data-station-text]")) {
        label.textContent = label.dataset.stationText;
      }
      const link = document.getElementById("level-link");
      link.href = "?view=game";
      link.textContent = "返回幽篁竹林 →";
    };
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", labelStation, { once: true });
    } else labelStation();
  }
})();
