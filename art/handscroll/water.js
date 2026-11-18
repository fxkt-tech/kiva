// These coordinates are measured on the approved 1672 × 941 painting.
(() => {
  "use strict";

  const canvas = document.getElementById("water");
  const ctx = canvas?.getContext("2d");
  if (!ctx) return;

  const artWidth = 1672;
  const artHeight = 941;
  const eaveX = 742;
  const eaveY = 154;
  const impactY = 792;
  const perspective = 0.19;
  const impactTime = 1.45;
  const waveDuration = 2.65;
  const cycleLength = 6.8;
  const tau = 2 * Math.PI;
  const still = matchMedia("(prefers-reduced-motion: reduce)");
  const art = new Image();

  // Stay below the platform lip, clear of the broken sleepers and foreground grass.
  const pond = new Path2D(
    "M 240 941 L 280 836 L 278 785 L 345 776 L 389 685 L 462 640 " +
    "L 557 619 L 688 650 L 854 665 L 1014 678 L 1160 694 " +
    "L 1320 712 L 1330 810 L 1250 834 L 1200 941 Z"
  );

  let inView = false;
  let started = null;
  let frame = 0;
  let timer = 0;
  let lastPaint = -Infinity;

  function clear() {
    ctx.clearRect(0, 0, artWidth, artHeight);
  }

  function fitCanvas() {
    const density = Math.min(2, window.devicePixelRatio || 1);
    // The painting bounds the useful spatial resolution of its reflected copy.
    const width = Math.max(1, Math.min(artWidth, Math.round(canvas.clientWidth * density)));
    const height = Math.max(1, Math.round(width * artHeight / artWidth));
    if (width === canvas.width && height === canvas.height) return;
    canvas.width = width;
    canvas.height = height;
    ctx.setTransform(width / artWidth, 0, 0, height / artHeight, 0, 0);
  }

  function drawDrop(time) {
    const fall = Math.max(0, (time - 0.78) / (impactTime - 0.78));
    const size = Math.min(1, Math.max(0, (time - 0.22) / 0.55));
    if (!size) return;
    const y = eaveY + (impactY - eaveY) * fall * fall;
    const radius = size * 2.85;
    const head = radius * 1.6;
    const tail = radius * (2 + 6 * fall);
    ctx.beginPath();
    ctx.moveTo(eaveX, y - tail);
    ctx.bezierCurveTo(eaveX - radius * 0.2, y - radius, eaveX - 2 * radius, y + radius, eaveX, y + head);
    ctx.bezierCurveTo(eaveX + 1.8 * radius, y + radius, eaveX + radius * 0.3, y - radius, eaveX, y - tail);
    ctx.fillStyle = "rgba(225, 233, 225, .9)";
    ctx.fill();
    ctx.strokeStyle = "rgba(63, 76, 90, .65)";
    ctx.lineWidth = 0.95;
    ctx.stroke();
    ctx.beginPath();
    ctx.ellipse(eaveX - 0.45, y + radius * 0.42, radius * 0.36, radius * 0.67, 0.24, 0, tau);
    ctx.fillStyle = "#ffeed1";
    ctx.fill();
  }

  function drawImpact(age) {
    const p = age / 0.29;
    if (p >= 1) return;
    ctx.fillStyle = `rgba(246, 225, 184, ${0.8 * (1 - p)})`;
    for (const spread of [-21, 11, 28]) {
      const y = impactY - (1 - Math.abs(spread) / 65) * 54 * p * (1 - p);
      ctx.beginPath();
      ctx.ellipse(eaveX + spread * p, y, 1.75 - p, 2.7 - p, spread / 42, 0, tau);
      ctx.fill();
    }
  }

  function radiusAt(age) {
    return 260 * Math.log1p(2.7 * age);
  }

  function drawWave(age) {
    const front = radiusAt(age);
    const decay = Math.pow(1 - age / waveDuration, 1.35);
    ctx.save();
    ctx.clip(pond);

    // Sample the original painting in thin elliptical bands around the impact.
    // Local file images may be drawn this way without reading pixels back.
    const packetWidth = 120;
    const band = 3;
    for (let behind = 0; behind < Math.min(front, packetWidth); behind += band) {
      const outer = front - behind;
      const inner = Math.max(0, outer - band);
      const envelope = Math.sin(Math.PI * behind / packetWidth) ** 2;
      const slope = Math.sin(behind * tau / 36) * envelope * decay;
      if (Math.abs(slope) < 0.025) continue;

      ctx.save();
      ctx.beginPath();
      ctx.ellipse(eaveX, impactY, outer, outer * perspective, 0, 0, tau);
      if (inner > 0) {
        ctx.moveTo(eaveX + inner, impactY);
        ctx.ellipse(eaveX, impactY, inner, inner * perspective, 0, 0, tau);
      }
      ctx.clip("evenodd");

      // A wave tilts the local mirror: shift its reflected pigment vertically,
      // and stretch its horizontal span by a few source pixels at the crest.
      const scale = 1 + 6 * slope / Math.max(outer, 20);
      ctx.drawImage(
        art, eaveX * (1 - scale), 7.5 * slope, artWidth * scale, artHeight
      );
      ctx.restore();
    }

    // Broken highlights echo the wet brushwork already in the painting.
    for (const [lag, start, end] of [[0, 0.12, 2.05], [6, 3.42, 5.48], [36, 1.75, 3.04], [72, 3.72, 5.85]]) {
      const radius = front - lag;
      if (radius < 2) continue;
      ctx.beginPath();
      ctx.ellipse(eaveX, impactY + 0.6, radius, radius * perspective, -0.003, start, end);
      ctx.strokeStyle = `rgba(47, 73, 91, ${decay * 0.52})`;
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.beginPath();
      ctx.ellipse(eaveX, impactY - 0.4, radius, radius * perspective, -0.003, start + 0.08, end - 0.05);
      ctx.strokeStyle = `rgba(248, 223, 178, ${decay * 0.86})`;
      ctx.lineWidth = 1.6;
      ctx.stroke();
    }
    drawImpact(age);
    ctx.restore();
  }

  function animate(now) {
    frame = 0;
    started ??= now;
    const time = ((now - started) / 1000) % cycleLength;
    if (time >= impactTime + waveDuration) {
      clear();
      timer = setTimeout(() => {
        timer = 0;
        frame = requestAnimationFrame(animate);
      }, (cycleLength - time) * 1000);
      return;
    }
    if (now - lastPaint >= 1000 / 36) {
      clear();
      if (time < impactTime) drawDrop(time);
      else drawWave(time - impactTime);
      lastPaint = now;
    }
    frame = requestAnimationFrame(animate);
  }

  function syncPlayback() {
    const active = inView && !document.hidden && !still.matches && art.complete && art.naturalWidth > 0;
    if (active) {
      if (!frame && !timer) frame = requestAnimationFrame(animate);
      return;
    }
    cancelAnimationFrame(frame);
    clearTimeout(timer);
    frame = 0;
    timer = 0;
    started = null;
    lastPaint = -Infinity;
    clear();
  }

  new ResizeObserver(fitCanvas).observe(canvas);
  new IntersectionObserver(([entry]) => {
    inView = entry.isIntersecting && entry.intersectionRatio >= 0.025;
    syncPlayback();
  }, { threshold: [0, 0.025] }).observe(canvas);
  document.addEventListener("visibilitychange", syncPlayback);
  still.addEventListener("change", syncPlayback);
  art.addEventListener("load", syncPlayback);
  art.src = new URL("duangui-key-art.png", document.currentScript.src).href;
  fitCanvas();
})();
