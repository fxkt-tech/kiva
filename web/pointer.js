/* Web pointer coordinates belong here; Rust consumes event-time snapshots. */
(() => {
  "use strict";
  const root = document.documentElement;
  const canvas = document.getElementById("game");
  const cursor = document.getElementById("game-cursor");
  let active = false;
  let point = [0, 0];
  let size = [canvas.clientWidth, canvas.clientHeight];
  let previous = null;
  let calibrationDpr = devicePixelRatio;
  let cssMotion = 0;
  let rawMotion = 0;
  let hovered = null;
  const presses = [];

  const captured = () => document.pointerLockElement === canvas;
  const permitted = () => captured() && document.hasFocus()
    && root.dataset.pointerLocked === "true" && root.dataset.paused === "false";
  const gain = () => rawMotion > 0 ? cssMotion / rawMotion : 1;

  function clamp() {
    point = point.map((value, axis) => Math.max(0, Math.min(Math.max(1, size[axis]) - 1, value)));
  }

  function updateHover() {
    const next = active && permitted()
      ? document.elementFromPoint(...point)?.closest("[data-ui] button") : null;
    if (next === hovered) return;
    hovered?.removeAttribute("data-virtual-hover");
    next?.setAttribute("data-virtual-hover", "");
    hovered = next;
  }

  function draw() {
    cursor.style.transform = `translate3d(${point[0]}px,${point[1]}px,0)`;
    cursor.style.setProperty("--flip-x", point[0] > size[0] - 24 ? "-1" : "1");
    cursor.style.setProperty("--flip-y", point[1] > size[1] - 24 ? "-1" : "1");
    cursor.hidden = false;
  }

  function stop() {
    if (active) previous = null;
    active = false;
    presses.length = 0;
    cursor.hidden = true;
    updateHover();
  }

  function resetCalibration() {
    if (calibrationDpr === devicePixelRatio) return;
    calibrationDpr = devicePixelRatio;
    cssMotion = rawMotion = 0;
    previous = null;
  }

  function motionSamples(event) {
    const samples = event.getCoalescedEvents?.();
    return samples?.length ? samples : [event];
  }

  function calibrate(event, samples) {
    resetCalibration();
    if (previous) {
      const css = [event.clientX - previous[0], event.clientY - previous[1]];
      const raw = samples.reduce((sum, sample) => [sum[0] + sample.movementX, sum[1] + sample.movementY], [0, 0]);
      // Learn units from the browser's normal pointer, including its existing
      // OS acceleration. Ignore a stationary/clipped axis independently.
      const current = [event.clientX, event.clientY];
      for (let axis = 0; axis < 2; axis++) {
        // A final partially clipped sample still has nonzero client motion.
        const coordinates = [previous[axis], current[axis], ...samples.map((sample) => axis === 0 ? sample.clientX : sample.clientY)];
        if (coordinates.some((value) => value <= 0 || value >= size[axis] - 1)) continue;
        const dot = css[axis] * raw[axis], square = raw[axis] ** 2;
        if (Number.isFinite(dot) && Number.isFinite(square) && dot > 0 && square > 0) {
          cssMotion += dot;
          rawMotion += square;
        }
      }
    }
    previous = [event.clientX, event.clientY];
  }

  function press(event) {
    if (!active || !permitted()) return;
    // Additional buttons generate pointermove rather than another pointerdown.
    const mask = [1, 4, 2, 8, 16][event.button];
    if (event.type === "pointerdown" || (mask && (event.buttons & mask))) {
      presses.push({ button: event.button, point: [...point] });
    }
  }

  document.addEventListener("pointermove", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse") return;
    const samples = motionSamples(event);
    if (!document.pointerLockElement) {
      if (!document.hasFocus()) { previous = null; return; }
      calibrate(event, samples);
      return;
    }
    previous = null;
    if (!active || !permitted()) return;
    resetCalibration();
    const scale = gain();
    for (const sample of samples) {
      const delta = [sample.movementX, sample.movementY];
      if (!delta.every(Number.isFinite)) continue;
      point = point.map((value, axis) => value + delta[axis] * scale);
      // Discard overshoot per sample, including a reversal in the same batch.
      clamp();
    }
    draw();
    press(event);
    updateHover();
  }, true);
  document.addEventListener("pointerdown", (event) => {
    if (event.isTrusted && event.pointerType === "mouse") press(event);
  }, true);
  document.addEventListener("pointerleave", () => { previous = null; });
  document.addEventListener("pointerlockchange", () => {
    previous = null;
    if (!captured()) stop();
  });
  window.addEventListener("blur", () => { previous = null; stop(); });
  window.addEventListener("resize", () => {
    size = [canvas.clientWidth, canvas.clientHeight];
    resetCalibration();
    clamp();
    if (active && permitted()) draw();
  });
  window.addEventListener("kiva-state", updateHover);
  new MutationObserver(() => { if (!permitted()) stop(); }).observe(root, {
    attributes: true, attributeFilter: ["data-paused", "data-pointer-locked"],
  });

  window.kivaPointer = Object.freeze({
    sync(enabled) {
      if (!enabled || !permitted()) { stop(); return; }
      if (!active) {
        active = true;
        presses.length = 0;
        size = [canvas.clientWidth, canvas.clientHeight];
        point = size.map((value) => value / 2);
        clamp();
        draw();
      }
    },
    position: () => active && permitted() ? [...point] : [],
    takePress(button) {
      const next = presses.shift();
      if (!active || !permitted() || next?.button !== button) {
        presses.length = 0;
        return [];
      }
      return next.point;
    },
    clearPresses() { presses.length = 0; },
  });
})();
