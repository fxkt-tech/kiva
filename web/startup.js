/* Install synchronously before Trunk's module; the DOM may not exist yet. */
(() => {
  "use strict";
  let state = "loading";
  let detail = Object.freeze({ phase: "download", loaded: 0, total: 0 });
  const phases = ["download", "initialize", "render"];
  const subscribers = new Set();
  window.__kivaStartup = state;

  function notify() {
    for (const subscriber of subscribers) subscriber(state, detail);
  }

  function transition(next, reason) {
    // Engine readiness wins over earlier or delayed loader notifications.
    // A failed load can still recover if the engine actually becomes ready.
    if (state === "ready" || state === next) return;
    state = next;
    window.__kivaStartup = state;
    detail = Object.freeze({ phase: detail.phase, loaded: detail.loaded, total: detail.total,
      ...(next === "failed" ? { reason } : {}) });
    notify();
  }

  const failed = (reason) => transition("failed", typeof reason === "string" && reason.trim()
    ? reason.trim().slice(0, 160) : "加载失败。请使用支持 WebGPU 的浏览器，再重新加载。");
  window.kivaStartup = Object.freeze({
    subscribe(subscriber) {
      subscribers.add(subscriber);
      subscriber(state, detail);
      return () => subscribers.delete(subscriber);
    },
    progress(phase, loaded, total) {
      if (state !== "loading" || !phases.includes(phase)
        || phases.indexOf(phase) < phases.indexOf(detail.phase)) return;
      loaded = Number.isFinite(loaded) && loaded >= 0 ? loaded : 0;
      // A missing or inconsistent denominator cannot support a percentage.
      total = Number.isFinite(total) && total > 0 && total >= loaded ? total : 0;
      if (phase === detail.phase && loaded === detail.loaded && total === detail.total) return;
      detail = Object.freeze({ phase, loaded, total });
      notify();
    },
    fail: failed,
  });
  window.addEventListener("TrunkApplicationError", failed);
  window.addEventListener("error", failed);
  window.addEventListener("unhandledrejection", failed);
  window.addEventListener("kiva-ready", () => transition("ready"), { once: true });
  if (!navigator.gpu) failed();
})();
