// Ink reaches each point on the paper at a fixed time. The SVG filter moves
// only that threshold; the fibers and the paintings stay put between frames.
class HandscrollInk {
  static duration = 400;

  constructor(fieldImage, threshold) {
    this.fieldImage = fieldImage;
    this.threshold = threshold;
    this.tapOrigins = new WeakMap();
    this.canvas = document.createElement("canvas");
    this.canvas.width = 640;
    this.canvas.height = 360;
    this.context = this.canvas.getContext("2d");
    this.pigment = this.context.createImageData(this.canvas.width, this.canvas.height);
    this.arrivals = new Float32Array(this.canvas.width * this.canvas.height);
    this.paper = new Float32Array(this.arrivals.length * 2);
    this.makePaper();
  }

  makePaper() {
    const grid = new Float32Array(128 * 128);
    let seed = 431;
    for (let i = 0; i < grid.length; i++) {
      seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
      grid[i] = seed / 0x100000000 - .5;
    }
    function noise(x, y) {
      const ix = Math.floor(x), iy = Math.floor(y);
      let fx = x - ix, fy = y - iy;
      fx *= fx * (3 - 2 * fx);
      fy *= fy * (3 - 2 * fy);
      const top = (iy & 127) * 128, bottom = ((iy + 1) & 127) * 128;
      const left = ix & 127, right = (ix + 1) & 127;
      const a = grid[top + left] + fx * (grid[top + right] - grid[top + left]);
      const b = grid[bottom + left] + fx * (grid[bottom + right] - grid[bottom + left]);
      return a + fy * (b - a);
    }
    function grain(x, y) {
      return .44 * noise(x * 3.7, y * 3.7) + .17 * noise(x * 9, y * 9)
        + .065 * noise(x * 21, y * 23) + .026 * noise(x * 45, y * 53)
        + .015 * noise(x * 72, y * 140);
    }
    const { width, height } = this.canvas;
    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const i = (row * width + col) * 2;
        const x = (col + .5) / height, y = (row + .5) / height;
        this.paper[i] = x + grain(x, y);
        this.paper[i + 1] = y + grain(x + 7.12, y + 3.38);
      }
    }
  }

  prepareFrom(frame, control, event) {
    const bounds = frame.getBoundingClientRect();
    const rect = control.getBoundingClientRect();
    const pointerClick = event?.detail > 0;
    const point = this.tapOrigins.get(event) || event;
    const x = pointerClick ? point.clientX : rect.left + rect.width / 2;
    const y = pointerClick ? point.clientY : rect.top + rect.height / 2;
    const clamp = (value) => Math.max(0, Math.min(1, value));
    this.prepare(clamp((x - bounds.left) / bounds.width), clamp((y - bounds.top) / bounds.height));
  }

  prepare(u, v) {
    const { width, height } = this.canvas;
    const cx = u * width, cy = v * height;
    const col = Math.min(width - 1, Math.floor(cx)), row = Math.min(height - 1, Math.floor(cy));
    const center = (row * width + col) * 2;
    const px = this.paper[center], py = this.paper[center + 1];
    let maximum = 0;
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const i = y * width + x;
        const dx = this.paper[i * 2] - px, dy = this.paper[i * 2 + 1] - py;
        const sx = (x + .5 - cx) / height, sy = (y + .5 - cy) / height;
        const arrival = .76 * Math.sqrt(dx * dx + dy * dy) + .24 * Math.sqrt(sx * sx + sy * sy);
        this.arrivals[i] = arrival;
        maximum = Math.max(maximum, arrival);
      }
    }
    const pixels = this.pigment.data;
    for (let i = 0; i < this.arrivals.length; i++) {
      const shade = Math.round(255 * this.arrivals[i] / maximum);
      pixels[i * 4] = shade;
      pixels[i * 4 + 1] = shade;
      pixels[i * 4 + 2] = shade;
      pixels[i * 4 + 3] = 255;
    }
    this.context.putImageData(this.pigment, 0, 0);
    this.fieldImage.setAttribute("href", this.canvas.toDataURL("image/png"));
    this.advance(0);
  }

  advance(progress) {
    // Alpha rises over a short range, like a wet edge sinking into the paper.
    this.threshold.setAttribute("values",
      `0 0 0 0 1  0 0 0 0 1  0 0 0 0 1  -82 0 0 0 ${1.2 + progress * 82}`);
  }
}

// Both leaves stay fixed while one wet edge reveals the destination.
class HandscrollReveal {
  constructor(page, leaves, ink, canAnimate, revealClass) {
    this.page = page;
    this.leaves = leaves;
    this.ink = ink;
    this.canAnimate = canAnimate;
    this.revealClass = revealClass;
    this.masked = null;
    this.visible = leaves.find((leaf) => !leaf.hidden);
    this.frame = 0;
    this.progress = 0;
  }

  finish() {
    cancelAnimationFrame(this.frame);
    this.frame = 0;
    if (this.masked) {
      this.masked.classList.remove(this.revealClass);
      this.masked.style.removeProperty("mask-image");
      this.masked = null;
    }
    for (const leaf of this.leaves) leaf.hidden = leaf !== this.visible;
  }

  show(target, origin, event) {
    if (!event || !this.canAnimate()) {
      this.visible = target;
      this.finish();
      return;
    }
    if (target === this.visible) return;
    // A third leaf needs a fresh field; the two visible leaves can reverse in place.
    if (this.masked && target.hidden) this.finish();
    this.visible = target;
    cancelAnimationFrame(this.frame);
    if (!this.masked) {
      this.ink.prepareFrom(this.page, origin, event);
      this.masked = target;
      this.progress = 0;
      target.hidden = false;
      target.classList.add(this.revealClass);
      target.style.maskImage = "url(#ink-wash)";
    }
    // An interrupted turn can retreat along its current edge without moving either leaf.
    const from = this.progress;
    const to = target === this.masked ? 1 : 0;
    const duration = HandscrollInk.duration * Math.sqrt(Math.abs(to - from));
    if (!duration) {
      this.finish();
      return;
    }
    const start = performance.now();
    const step = (now) => {
      const t = Math.max(0, Math.min(1, (now - start) / duration));
      if (t === 1 || !this.canAnimate()) {
        this.finish();
        return;
      }
      this.progress = from + (to - from) * t;
      this.ink.advance(this.progress);
      this.frame = requestAnimationFrame(step);
    };
    this.frame = requestAnimationFrame(step);
  }
}
