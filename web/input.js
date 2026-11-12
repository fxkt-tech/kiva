/* Browser input owns capture and pixel units; Bevy owns direction and movement. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
  let ready = false;
  let confirmed = false;
  let request = null;
  let look = 0;
  let keys = 0;
  let actions = 0;
  let defenseEdges = [];
  let defenseCancelled = false;
  let rightBlocked = false;
  const mousePressed = new Set();
  const MAX_DEFENSE_EDGES = 64;
  const DEFENSE_PRESSED = 1;
  const DEFENSE_RELEASED = 2;
  const DEFENSE_CANCELLED = 3;
  const MOVEMENT_RELEASED = 1024;
  const SHIFT_RELEASED = 2048;
  const pressed = new Set();
  const heldKeys = Object.freeze({ KeyW: 1, KeyS: 2, KeyA: 4, KeyD: 8, ShiftLeft: 16, ShiftRight: 16 });
  const actionKeys = Object.freeze({ Space: 1, KeyE: 8, Tab: 16, Digit1: 64, Digit2: 128, KeyQ: 256, KeyC: 512 });
  let error = "";
  let startupDetail = { phase: "download", loaded: 0, total: 0 };
  let previous = null;
  let pixelRatio = window.devicePixelRatio;
  let cross = 0;
  let square = 0;
  let gain = 1;
  const canvas = () => document.getElementById("game");
  const isActive = () => ready && confirmed && document.hasFocus()
    && document.pointerLockElement === canvas();

  function clearInputs() {
    look = 0;
    // A blur and recapture may both occur before Rust samples again.
    actions &= MOVEMENT_RELEASED | SHIFT_RELEASED;
    if (keys & 15) actions |= MOVEMENT_RELEASED;
    if (keys & 16) actions |= SHIFT_RELEASED;
    keys = 0;
    cancelDefense();
    mousePressed.clear();
    rightBlocked = false;
    pressed.clear();
    previous = null;
  }

  function cancelDefense() {
    // Retain cancellation even if blur and recapture happen between Rust samples.
    defenseEdges = [];
    defenseCancelled = true;
  }

  function queueDefense(edge) {
    if (defenseEdges.length + Number(defenseCancelled) >= MAX_DEFENSE_EDGES) {
      cancelDefense();
      rightBlocked = mousePressed.has(2);
      return;
    }
    defenseEdges.push(edge);
  }

  function clearCapture() {
    confirmed = false;
    request = null;
    clearInputs();
  }

  function refreshKeys() {
    keys = 0;
    for (const code of pressed) keys |= heldKeys[code] || 0;
  }

  function render() {
    const entry = document.getElementById("entry");
    if (!entry) return;
    const failed = window.__kivaStartup === "failed" && !ready;
    const pending = ready && request !== null;
    const busy = (!ready && !failed) || pending;
    const measured = !ready && !failed && startupDetail.phase === "download" && startupDetail.total > 0;
    const percent = measured ? Math.floor(100 * startupDetail.loaded / startupDetail.total) : null;
    // After the program starts, the engine reports completed / required files.
    const counted = !ready && !failed && startupDetail.phase === "initialize" && startupDetail.total > 0;
    const loadingLabel = startupDetail.phase === "render" ? "正在准备画面…"
      : counted ? `正在加载资源 ${startupDetail.loaded} / ${startupDetail.total}`
        : startupDetail.phase === "initialize" ? "正在准备游戏…"
          : measured ? `正在下载 ${percent}%` : "正在下载…";
    const enterButton = document.getElementById("enter");
    const progress = document.getElementById("enter-progress");
    entry.hidden = isActive();
    enterButton.hidden = false;
    enterButton.disabled = !ready || pending;
    enterButton.setAttribute("aria-busy", String(busy));
    document.getElementById("enter-label").textContent = pending ? "正在进入…"
      : ready ? "点击进入" : failed ? "加载失败" : loadingLabel;
    progress.hidden = ready || failed;
    if (measured || counted) {
      progress.max = startupDetail.total;
      progress.value = startupDetail.loaded;
    } else progress.removeAttribute("value");
    document.getElementById("retry").hidden = !failed;
    const bytes = (value) => value >= 1024 * 1024 ? `${(value / (1024 * 1024)).toFixed(1)} MB`
      : `${Math.floor(value / 1024)} KB`;
    const downloading = measured ? `正在下载游戏：${bytes(startupDetail.loaded)} / ${bytes(startupDetail.total)}。`
      : startupDetail.loaded > 0 ? `正在下载游戏，已接收 ${bytes(startupDetail.loaded)}…` : "正在下载游戏…";
    document.getElementById("entry-message").textContent = error || (pending
      ? "正在锁定鼠标，成功后即可进入。" : ready
      ? "进入竹林，迎战愁铃客。" : failed
        ? startupDetail.reason
        : startupDetail.phase === "render" ? "正在准备首次画面，请稍候…"
          : counted ? "正在加载模型、动作与场景贴图…"
            : startupDetail.phase === "initialize" ? "正在准备场景与游戏引擎…" : downloading);
  }

  function enter() {
    if (!ready || request || isActive()) return;
    const target = canvas();
    if (!target) return;
    target.focus();
    const attempt = {};
    request = attempt;
    error = "";
    render();
    const rejected = () => {
      if (request !== attempt || confirmed) return;
      clearCapture();
      error = "未能锁定鼠标，请再次点击进入。";
      render();
    };
    try {
      // Default pointer lock retains system mouse acceleration.
      const result = target.requestPointerLock();
      if (result?.catch) result.catch(rejected);
    } catch { rejected(); }
  }

  document.addEventListener("pointerlockchange", () => {
    if (document.pointerLockElement === canvas() && document.hasFocus() && (request || confirmed)) {
      confirmed = true;
      request = null;
      clearInputs();
      error = "";
    } else {
      clearCapture();
      if (document.pointerLockElement === canvas()) document.exitPointerLock();
    }
    render();
  });
  document.addEventListener("pointerlockerror", () => {
    if (!request || confirmed) return;
    clearCapture();
    error = "未能锁定鼠标，请再次点击进入。";
    render();
  });
  window.addEventListener("blur", () => {
    clearCapture();
    if (document.pointerLockElement === canvas()) document.exitPointerLock();
    render();
  });
  window.addEventListener("focus", render);
  document.addEventListener("keydown", (event) => {
    if (!event.isTrusted || !isActive()) return;
    if (event.code === "Escape") {
      clearCapture();
      if (document.pointerLockElement === canvas()) document.exitPointerLock();
      render();
      return;
    }
    if (heldKeys[event.code] || actionKeys[event.code]) event.preventDefault?.();
    if (event.repeat || pressed.has(event.code)) return;
    const firstShift = heldKeys[event.code] === 16 && (keys & 16) === 0;
    pressed.add(event.code);
    refreshKeys();
    if (firstShift) actions |= 2;
    // A rapid change of pair has one unambiguous final selection.
    if (event.code === "Digit1" || event.code === "Digit2") actions &= ~(64 | 128);
    if (event.code === "KeyC") actions ^= 512;
    else actions |= actionKeys[event.code] || 0;
  }, true);
  document.addEventListener("keyup", (event) => {
    if (!event.isTrusted) return;
    if (isActive() && (heldKeys[event.code] || actionKeys[event.code])) event.preventDefault?.();
    const previouslyHeld = keys;
    pressed.delete(event.code);
    refreshKeys();
    if (isActive()) {
      if ((previouslyHeld & 15) && !(keys & 15)) actions |= MOVEMENT_RELEASED;
      if ((previouslyHeld & 16) && !(keys & 16)) actions |= SHIFT_RELEASED;
    }
  }, true);
  document.addEventListener("pointerdown", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse" || !isActive()) return;
    if (![0, 1, 2].includes(event.button)) return;
    event.preventDefault?.();
    if (mousePressed.has(event.button)) return;
    mousePressed.add(event.button);
    if (event.button === 0) actions |= 4;
    if (event.button === 1) actions ^= 32;
    if (event.button === 2 && !rightBlocked) queueDefense(DEFENSE_PRESSED);
  }, true);
  document.addEventListener("pointerup", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse" || !isActive()) return;
    if (!mousePressed.delete(event.button)) return;
    if (event.button === 2) {
      if (rightBlocked) rightBlocked = false;
      else queueDefense(DEFENSE_RELEASED);
    }
  }, true);
  document.addEventListener("pointercancel", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse" || !isActive()) return;
    cancelDefense();
    mousePressed.clear();
    rightBlocked = false;
  });
  document.addEventListener("contextmenu", (event) => {
    if (event.isTrusted && isActive()) event.preventDefault?.();
  }, true);
  document.addEventListener("wheel", (event) => {
    if (event.isTrusted && isActive()) event.preventDefault?.();
  }, { capture: true, passive: false });
  document.addEventListener("pointerleave", () => { previous = null; });

  document.addEventListener("pointermove", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse") return;
    if (pixelRatio !== window.devicePixelRatio) {
      pixelRatio = window.devicePixelRatio;
      cross = square = 0;
      gain = 1;
      previous = null;
    }
    const coalesced = event.getCoalescedEvents?.();
    const samples = coalesced?.length ? coalesced : [event];
    const raw = samples.reduce((sum, sample) => sum + (Number.isFinite(sample.movementX) ? sample.movementX : 0), 0);
    if (isActive()) {
      look += raw * gain;
      return;
    }
    // Browser movementX may use physical pixels while clientX uses CSS pixels.
    // Learn the ratio from unclipped free-cursor movement, never divide by DPR blindly.
    const width = window.innerWidth;
    const unclipped = samples.every((sample) => sample.clientX > 1 && sample.clientX < width - 2);
    if (previous !== null && unclipped && event.clientX > 1 && event.clientX < width - 2) {
      const css = event.clientX - previous;
      if (Math.abs(raw) >= 1 && css * raw > 0) {
        cross += css * raw;
        square += raw * raw;
        gain = Math.max(0.25, Math.min(4, cross / square));
      }
    }
    previous = unclipped && event.clientX > 1 && event.clientX < width - 2 ? event.clientX : null;
  }, true);

  window.kivaInput = Object.freeze({
    isActive,
    keys: () => isActive() ? keys : 0,
    takeActions() {
      const pending = isActive() ? actions : 0;
      actions = 0;
      return pending;
    },
    takeLook() {
      const delta = isActive() ? look : 0;
      look = 0;
      return delta;
    },
    takeDefenseEdges() {
      const pending = defenseCancelled ? [DEFENSE_CANCELLED] : [];
      if (isActive()) pending.push(...defenseEdges);
      defenseEdges = [];
      defenseCancelled = false;
      return pending;
    },
    ready() {
      ready = true;
      window.dispatchEvent(new CustomEvent("kiva-ready"));
      render();
    },
  });

  function bind() {
    document.getElementById("enter").addEventListener("click", enter);
    canvas().addEventListener("click", enter);
    document.getElementById("retry").addEventListener("click", () => window.location.reload());
    window.kivaStartup.subscribe((_state, detail) => {
      startupDetail = detail;
      render();
    });
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind, { once: true });
  else bind();
})();
