/* Browser input owns capture and pixel units; Bevy owns direction and movement. */
(() => {
  "use strict";
  let ready = false;
  let confirmed = false;
  let request = null;
  let look = 0;
  let aim = 0;
  let keys = 0;
  let actions = 0;
  let blocking = false;
  let lastWheel = null;
  const pressed = new Set();
  const heldKeys = Object.freeze({ KeyW: 1, KeyS: 2, KeyA: 4, KeyD: 8, ShiftLeft: 16, ShiftRight: 16 });
  const actionKeys = Object.freeze({ Space: 1, KeyE: 8, Tab: 16, Digit1: 64, Digit2: 128, KeyQ: 256 });
  let error = "";
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
    aim = 0;
    keys = 0;
    actions = 0;
    blocking = false;
    lastWheel = null;
    pressed.clear();
    previous = null;
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
    entry.hidden = isActive();
    document.getElementById("enter").hidden = !ready;
    document.getElementById("retry").hidden = !failed;
    document.getElementById("entry-message").textContent = error || (ready
      ? "进入庭院，自由移动并观察四周。" : failed
        ? "加载失败。请使用支持 WebGPU 的浏览器，再重新加载。"
        : "正在准备场景与游戏引擎…");
  }

  function enter() {
    if (!ready || request || isActive()) return;
    const target = canvas();
    if (!target) return;
    target.focus();
    const attempt = {};
    request = attempt;
    error = "";
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
    if (heldKeys[event.code] || actionKeys[event.code]) event.preventDefault?.();
    if (event.repeat || pressed.has(event.code)) return;
    const firstShift = heldKeys[event.code] === 16 && (keys & 16) === 0;
    pressed.add(event.code);
    refreshKeys();
    if (firstShift) actions |= 2;
    // A rapid change of pair has one unambiguous final selection.
    if (event.code === "Digit1" || event.code === "Digit2") actions &= ~(64 | 128);
    actions |= actionKeys[event.code] || 0;
  }, true);
  document.addEventListener("keyup", (event) => {
    if (!event.isTrusted) return;
    if (isActive() && (heldKeys[event.code] || actionKeys[event.code])) event.preventDefault?.();
    pressed.delete(event.code);
    refreshKeys();
  }, true);
  document.addEventListener("pointerdown", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse" || !isActive()) return;
    if (event.button === 0) actions |= 4;
    if (event.button === 2) blocking = true;
    if (event.button === 0 || event.button === 2) event.preventDefault?.();
  }, true);
  document.addEventListener("pointerup", (event) => {
    if (event.isTrusted && event.pointerType === "mouse" && event.button === 2) blocking = false;
  }, true);
  document.addEventListener("pointercancel", (event) => {
    if (event.isTrusted) blocking = false;
  });
  document.addEventListener("contextmenu", (event) => {
    if (event.isTrusted && isActive()) event.preventDefault?.();
  }, true);
  document.addEventListener("wheel", (event) => {
    if (!event.isTrusted || !isActive()) return;
    event.preventDefault?.();
    if (!Number.isFinite(event.deltaY) || event.deltaY === 0) return;
    const now = Number.isFinite(event.timeStamp) ? event.timeStamp : Date.now();
    // Trackpad inertia is one gesture until the stream is quiet for 160ms.
    if (lastWheel === null || now - lastWheel >= 160) actions |= 32;
    lastWheel = now;
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
      aim += samples.reduce((sum, sample) => sum + (Number.isFinite(sample.movementY) ? sample.movementY : 0), 0) * gain;
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
    keys: () => isActive() ? keys | (blocking ? 32 : 0) : 0,
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
    takeAim() {
      const delta = isActive() ? aim : 0;
      aim = 0;
      return delta;
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
    window.kivaStartup.subscribe(render);
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind, { once: true });
  else bind();
})();
