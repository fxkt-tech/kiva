/* Keep the one-time readonly catalog even when the deferred HUD arrives late. */
(() => {
  "use strict";
  let catalog = Object.freeze([]);
  function retainCatalog(event) {
    const snapshot = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
    if (!Array.isArray(snapshot.character_catalog)) return;
    catalog = Object.freeze(snapshot.character_catalog);
    window.removeEventListener("kiva-state", retainCatalog);
  }
  window.addEventListener("kiva-state", retainCatalog);
  window.kivaCharacters = Object.freeze({ catalog: () => catalog });
})();
