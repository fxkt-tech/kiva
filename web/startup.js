/* Install synchronously before Trunk's module; the HUD may not exist yet. */
(() => {
  "use strict";
  let state = "loading";
  const subscribers = new Set();
  window.__kivaStartup = state;

  function transition(next) {
    // Engine readiness wins over earlier or delayed loader notifications.
    // A failed load can still recover if the engine actually becomes ready.
    if (state === "ready" || state === next) return;
    state = next;
    window.__kivaStartup = state;
    for (const subscriber of subscribers) subscriber(state);
  }

  window.kivaStartup = Object.freeze({
    subscribe(subscriber) {
      subscribers.add(subscriber);
      subscriber(state);
      return () => subscribers.delete(subscriber);
    },
  });
  const failed = () => transition("failed");
  window.addEventListener("TrunkApplicationError", failed);
  window.addEventListener("error", failed);
  window.addEventListener("unhandledrejection", failed);
  window.addEventListener("kiva-ready", () => transition("ready"), { once: true });
  if (!navigator.gpu) failed();
})();
