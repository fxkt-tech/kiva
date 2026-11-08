/* Browser input owns capture and pixel units; Bevy owns direction and movement. */
(() => {
  "use strict";
  let ready = false;
  let confirmed = false;
  let request = null;
  let look = 0;
  let keys = 0;
  const movementKeys = Object.freeze({ KeyW: 1, KeyS: 2, KeyA: 4, KeyD: 8 });
  let error = "";
  let previous = null;
  let pixelRatio = window.devicePixelRatio;
  let cross = 0;
  let square = 0;
  let gain = 1;
  const canvas = () => document.getElementById("game");
  const isActive = () => ready && confirmed && document.hasFocus()
    && document.pointerLockElement === canvas();

  function clearCapture() {
    confirmed = false;
    request = null;
    look = 0;
    keys = 0;
    previous = null;
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
      look = 0;
      previous = null;
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
    if (event.isTrusted && !event.repeat && isActive()) keys |= movementKeys[event.code] || 0;
  }, true);
  document.addEventListener("keyup", (event) => {
    if (event.isTrusted) keys &= ~(movementKeys[event.code] || 0);
  }, true);
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
    takeLook() {
      const delta = isActive() ? look : 0;
      look = 0;
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
