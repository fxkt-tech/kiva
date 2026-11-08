/* Full-map projection and pointer gestures; Rust owns the actual camera. */
(() => {
  "use strict";
  const INSET = 8;
  const clamp = (value, min, max) => Math.max(min, Math.min(max, value));
  const finitePoint = (point) => Array.isArray(point) && point.length === 2 && point.every(Number.isFinite);

  window.createMinimap = (canvas) => {
    const root = document.documentElement;
    const game = document.getElementById("game");
    const stage = document.getElementById("minimap-stage");
    const viewport = document.getElementById("minimap-viewport");
    const context = canvas.getContext("2d");
    let state = null;
    let playerId = null;
    let phase = null;
    let camera = null;
    let frame = null;
    let drag = null;
    canvas.dataset.mapInset = String(INSET);

    function geometry() {
      const rect = canvas.getBoundingClientRect();
      const half = state?.map.half_size;
      if (!(half > 0) || !Number.isFinite(half) || rect.width <= INSET * 2 || rect.height <= INSET * 2) return null;
      const scale = [(rect.width - INSET * 2) / (half * 2), (rect.height - INSET * 2) / (half * 2)];
      return {
        rect, half, scale,
        project: ([x, z]) => [INSET + (x + half) * scale[0], INSET + (z + half) * scale[1]],
        unproject: ([x, y]) => [(x - rect.left - INSET) / scale[0] - half,
          (y - rect.top - INSET) / scale[1] - half],
      };
    }

    function permitted() {
      return phase === "Active" && Boolean(playerId) && root.dataset.paused === "false"
        && root.dataset.pointerLocked === "true" && document.pointerLockElement === game && document.hasFocus();
    }

    function finish(discard = true) {
      drag = null;
      window.kivaPointer.setUiCapture(false);
      delete root.dataset.minimapDragging;
      if (discard) delete root.dataset.cameraRequest;
    }

    function updateViewport() {
      const map = geometry();
      const outline = camera?.footprint;
      frame = null;
      viewport.hidden = true;
      if (!map || !Array.isArray(outline) || outline.length !== 4 || !outline.every(finitePoint)) return;
      const corners = outline.map(map.project);
      const left = clamp(Math.min(...corners.map(([x]) => x)), INSET, map.rect.width - INSET);
      const right = clamp(Math.max(...corners.map(([x]) => x)), INSET, map.rect.width - INSET);
      const top = clamp(Math.min(...corners.map(([, y]) => y)), INSET, map.rect.height - INSET);
      const bottom = clamp(Math.max(...corners.map(([, y]) => y)), INSET, map.rect.height - INSET);
      if (right <= left || bottom <= top) return;
      // Perspective ground rays form a trapezoid. The UI rectangle encloses
      // those real corners; it never substitutes a fixed or predicted view.
      frame = { left, right, top, bottom };
      viewport.style.left = `${left}px`;
      viewport.style.top = `${top}px`;
      viewport.style.width = `${right - left}px`;
      viewport.style.height = `${bottom - top}px`;
      viewport.hidden = false;
    }

    function drawMap() {
      const map = geometry();
      if (!context || !map) return;
      const { rect, scale, project } = map;
      const ratio = Math.max(1, window.devicePixelRatio || 1);
      const pixels = [Math.round(rect.width * ratio), Math.round(rect.height * ratio)];
      if (canvas.width !== pixels[0] || canvas.height !== pixels[1]) [canvas.width, canvas.height] = pixels;
      context.setTransform(canvas.width / rect.width, 0, 0, canvas.height / rect.height, 0, 0);
      context.fillStyle = "#122923";
      context.fillRect(0, 0, rect.width, rect.height);
      context.fillStyle = "#294333";
      context.fillRect(INSET, INSET, rect.width - INSET * 2, rect.height - INSET * 2);
      context.save();
      context.beginPath();
      context.rect(INSET, INSET, rect.width - INSET * 2, rect.height - INSET * 2);
      context.clip();
      context.strokeStyle = "#63775b33";
      context.lineWidth = 1;
      context.beginPath();
      for (let division = 1; division < 4; division++) {
        const x = INSET + (rect.width - INSET * 2) * division / 4;
        const y = INSET + (rect.height - INSET * 2) * division / 4;
        context.moveTo(x, INSET); context.lineTo(x, rect.height - INSET);
        context.moveTo(INSET, y); context.lineTo(rect.width - INSET, y);
      }
      context.stroke();
      const ally = project(state.map.ally_base);
      const enemy = project(state.map.enemy_base);
      context.strokeStyle = "#718069";
      context.lineWidth = state.map.lane_width * scale[0];
      context.beginPath(); context.moveTo(...ally); context.lineTo(...enemy); context.stroke();
      context.fillStyle = "#718069";
      for (const [x, z, hx, hz] of state.obstacles) {
        const start = project([x - hx, z - hz]);
        context.fillRect(...start, hx * 2 * scale[0], hz * 2 * scale[1]);
      }
      for (const [location, color] of [[ally, "#8cddc3"], [enemy, "#df9a7c"]]) {
        context.fillStyle = color === "#8cddc3" ? "#476a54" : "#705347";
        context.beginPath(); context.arc(...location, 8, 0, Math.PI * 2); context.fill();
        context.strokeStyle = color; context.lineWidth = 1.5;
        context.strokeRect(location[0] - 4.5, location[1] - 4.5, 9, 9);
      }
      for (const unit of state.units) {
        if (!unit.alive) continue;
        context.fillStyle = unit.team === "ally" ? "#8cddc3" : "#df9a7c";
        const location = project(unit.position);
        const radius = unit.role === "Hero" ? 3.5 : unit.role === "Base" ? 4 : 1.5;
        context.beginPath(); context.arc(...location, radius, 0, Math.PI * 2); context.fill();
        if (unit.id === playerId) {
          context.strokeStyle = "#efe0ac"; context.lineWidth = 1;
          context.beginPath(); context.arc(...location, radius + 2, 0, Math.PI * 2); context.stroke();
        }
      }
      context.restore();
      context.strokeStyle = "#a7b18a";
      context.lineWidth = 1;
      context.strokeRect(INSET, INSET, rect.width - INSET * 2, rect.height - INSET * 2);
    }

    function submit(center) {
      if (!finitePoint(center)) return;
      // Clamp after applying the grab offset. Clipping the pointer first
      // would make the opposite map edge unreachable for an off-center grab.
      const half = state.map.half_size;
      root.dataset.cameraRequest = JSON.stringify({ player_id: playerId,
        center: center.map((value) => clamp(value, -half, half)) });
    }

    function begin(event) {
      if (!event.isTrusted || event.pointerType !== "mouse" || event.button !== 0 || !permitted()) return;
      // pointer.js runs first in document capture order. Native clientX/Y
      // belongs to the locked system cursor, not our visible game pointer.
      const point = window.kivaPointer.position();
      const map = geometry();
      if (!finitePoint(point) || !map || !stage.contains(document.elementFromPoint(...point))) return;
      const world = map.unproject(point);
      const local = [point[0] - map.rect.left, point[1] - map.rect.top];
      const grabbed = frame && finitePoint(camera?.center) && local[0] >= frame.left && local[0] <= frame.right
        && local[1] >= frame.top && local[1] <= frame.bottom;
      drag = { offset: grabbed ? camera.center.map((value, axis) => value - world[axis]) : [0, 0] };
      window.kivaPointer.setUiCapture(true);
      root.dataset.minimapDragging = "true";
      if (!grabbed) submit(world);
      event.preventDefault();
    }

    document.addEventListener("pointerdown", begin, true);

    document.addEventListener("pointermove", (event) => {
      if (!event.isTrusted || event.pointerType !== "mouse") return;
      if (!drag) {
        // A left press while another button is held is delivered as motion.
        if (event.button === 0 && (event.buttons & 1)) begin(event);
        return;
      }
      if (!permitted()) { finish(); return; }
      if (!(event.buttons & 1)) {
        // A chorded left release is a normal release, unlike losing the
        // button state without a release event. Keep its final valid target.
        finish(event.button !== 0);
        return;
      }
      const point = window.kivaPointer.position();
      const map = geometry();
      if (!finitePoint(point) || !map) { finish(); return; }
      submit(map.unproject(point).map((value, axis) => value + drag.offset[axis]));
      event.preventDefault();
    }, true);
    document.addEventListener("pointerup", (event) => {
      if (event.isTrusted && event.pointerType === "mouse" && event.button === 0 && drag) finish(!permitted());
    }, true);
    document.addEventListener("pointercancel", () => finish(), true);
    document.addEventListener("pointerlockchange", () => { if (!permitted()) finish(); });
    window.addEventListener("blur", () => finish());
    window.addEventListener("resize", () => { finish(); drawMap(); updateViewport(); });
    new MutationObserver(() => { if (!permitted()) finish(); }).observe(root, {
      attributes: true, attributeFilter: ["data-paused", "data-pointer-locked", "data-session-phase"],
    });
    window.addEventListener("kiva-camera", (event) => {
      camera = JSON.parse(event.detail);
      updateViewport();
    });

    return {
      render(next) {
        const changed = playerId !== next.player_id || phase !== next.session.phase;
        state = next;
        playerId = next.player_id;
        phase = next.session.phase;
        if (changed) finish();
        if (!camera || changed) camera = { center: next.camera_center, footprint: next.camera_footprint };
        if (!permitted()) finish();
        drawMap();
        updateViewport();
      },
    };
  };
})();
