/* Settings presentation only. Capture, pause and gameplay belong to the HUD. */
(() => {
  "use strict";
  window.createSettingsUi = ({ requestClose, onToggle = () => {} }) => {
    const root = document.documentElement;
    const dialog = document.getElementById("settings-dialog");
    const toggle = document.getElementById("settings-toggle");
    const closeButton = document.getElementById("settings-close");
    const error = document.getElementById("settings-error");
    const tabs = Array.from(dialog.querySelectorAll("[data-settings-tab]"));
    const panels = Array.from(dialog.querySelectorAll("[data-settings-panel]"));
    let opened = false;
    let busy = false;
    let category = "controls";

    function select(next) {
      if (!tabs.some((tab) => tab.dataset.settingsTab === next)) return false;
      category = next;
      dialog.dataset.category = category;
      for (const tab of tabs) {
        const selected = tab.dataset.settingsTab === category;
        tab.setAttribute("aria-selected", String(selected));
        tab.tabIndex = selected ? 0 : -1;
      }
      for (const panel of panels) panel.hidden = panel.dataset.settingsPanel !== category;
      return true;
    }

    function setBusy(value) {
      busy = Boolean(value);
      closeButton.disabled = busy;
      closeButton.textContent = busy ? "正在返回…" : "完成";
      dialog.setAttribute("aria-busy", String(busy));
    }

    function setError(message) {
      error.textContent = message || "";
      error.hidden = !error.textContent;
    }

    function open() {
      if (opened) return;
      opened = true;
      root.dataset.settingsOpen = "true";
      dialog.hidden = false;
      toggle.setAttribute("aria-expanded", "true");
      setError("");
      select(category);
      onToggle();
      tabs.find((tab) => tab.dataset.settingsTab === category)?.focus();
    }

    function close() {
      if (!opened) return;
      opened = false;
      root.dataset.settingsOpen = "false";
      dialog.hidden = true;
      toggle.setAttribute("aria-expanded", "false");
      setBusy(false);
      setError("");
      onToggle();
    }

    closeButton.addEventListener("click", () => {
      if (opened && !busy) requestClose();
    });
    for (const [index, tab] of tabs.entries()) {
      tab.addEventListener("click", () => select(tab.dataset.settingsTab));
      tab.addEventListener("keydown", (event) => {
        const next = event.key === "ArrowDown" ? (index + 1) % tabs.length
          : event.key === "ArrowUp" ? (index + tabs.length - 1) % tabs.length
          : event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : null;
        if (next === null) return;
        event.preventDefault();
        select(tabs[next].dataset.settingsTab);
        tabs[next].focus();
      });
    }
    root.dataset.settingsOpen = "false";
    select(category);
    setBusy(false);
    setError("");
    return Object.freeze({ open, close, isOpen: () => opened, select, setBusy, setError });
  };
})();
