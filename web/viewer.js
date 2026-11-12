/* Inspection DOM owns commands only. Rust owns pose, playback, camera and catalog. */
(() => {
  "use strict";
  if (!window.kivaPage?.isViewer()) return;
  const MAX_COMMANDS = 128;
  const MAX_BYTES = 16384;
  const id = (value) => typeof value === "string" && value.length > 0 && value.length <= 160
    && !/[\u0000-\u0020\u007f]/.test(value);
  const finite = (value) => typeof value === "number" && Number.isFinite(value);
  const modes = ["game", "source", "reference"];
  const views = ["front", "back", "left", "right", "top", "three-quarter"];
  const debugNames = ["grid", "axes", "bones", "grips", "bounds"];
  const coalesced = new Set(["preset", "appearance", "weapon", "action", "seek", "speed", "loop"]);
  let commands = [];
  let snapshot = null;
  let catalog = null;
  let bound = false;
  let engineReady = false;
  let renderFailure = "";
  let startupState = "loading";
  let startupDetail = { phase: "download", loaded: 0, total: 0 };
  let drag = null;
  let timelineEditing = false;
  let actionKey = "";
  let partKey = "";
  let catalogKey = "";
  let search = "";
  let copyRevision = 0;
  const node = (name) => document.getElementById(`viewer-${name}`);
  const text = (name, value) => { const item = node(name); if (item) item.textContent = value; };
  const string = (value, length = 8192) => typeof value === "string" ? value.slice(0, length) : "";

  function command(name, ...args) {
    if (args.some((value) => typeof value === "number" ? !finite(value) : !id(value))) return;
    const line = [name, ...args].join("\t");
    if (line.length > 512) return;
    if (coalesced.has(name)) commands = commands.filter((item) => !item.startsWith(`${name}\t`));
    if (commands.length >= MAX_COMMANDS || commands.join("\n").length + line.length + 1 > MAX_BYTES) return;
    commands.push(line);
  }

  function safeCatalog(raw) {
    if (!raw || typeof raw !== "object") return null;
    const result = {};
    for (const key of ["appearances", "weapons", "presets", "actions"]) {
      if (!Array.isArray(raw[key]) || raw[key].length > 2000) return null;
      result[key] = raw[key].filter((entry) => entry && id(entry.id) && typeof entry.label === "string")
        .map((entry) => ({ id: entry.id, label: string(entry.label, 240),
          ...(key === "actions" ? { mode: modes.includes(entry.mode) ? entry.mode : "source",
            group: string(entry.group, 160), duration: finite(entry.duration) ? entry.duration : 0 } : {}) }));
    }
    return result;
  }

  function safeSnapshot(raw) {
    if (!raw || raw.version !== 1 || !Number.isSafeInteger(raw.revision) || raw.revision < 0
      || !id(raw.appearance) || !id(raw.weapon) || !modes.includes(raw.mode)
      || !finite(raw.time) || !finite(raw.duration) || raw.time < 0 || raw.duration < 0
      || !Number.isSafeInteger(raw.frame) || raw.frame < 0
      || !finite(raw.speed) || raw.speed <= 0 || raw.speed > 2) return null;
    const parts = Array.isArray(raw.parts) ? raw.parts.slice(0, 2000)
      .filter((entry) => entry && id(entry.id) && typeof entry.label === "string")
      .map((entry) => ({ id: entry.id, label: string(entry.label, 240), visible: entry.visible === true })) : [];
    return { version: 1, revision: raw.revision, ready: raw.ready === true, failed: raw.failed === true,
      appearance: raw.appearance, weapon: raw.weapon, preset: id(raw.preset) ? raw.preset : "custom",
      mode: raw.mode, action: id(raw.action) ? raw.action : "", time: raw.time, duration: raw.duration,
      frame: raw.frame, playing: raw.playing === true, speed: raw.speed, looping: raw.looping === true,
      trace: string(raw.trace), warning: string(raw.warning), status: string(raw.status, 240),
      camera: raw.camera && typeof raw.camera === "object" ? raw.camera : {},
      debug: Object.fromEntries(debugNames.map((name) => [name, raw.debug?.[name] === true])),
      parts, inspection: string(raw.inspection, 32768) };
  }

  function options(name, entries, value, extra) {
    const select = node(name);
    if (!select) return;
    select.replaceChildren();
    if (extra) {
      const option = document.createElement("option"); option.value = extra.id;
      option.textContent = extra.label; select.appendChild(option);
    }
    for (const entry of entries) {
      const option = document.createElement("option"); option.value = entry.id;
      option.textContent = entry.label; select.appendChild(option);
    }
    select.value = value;
  }

  function renderActions() {
    if (!snapshot || !catalog) return;
    const mode = snapshot.mode === "reference" ? "game" : snapshot.mode;
    const entries = catalog.actions.filter((entry) => entry.mode === mode
      && `${entry.label} ${entry.id} ${entry.group}`.toLocaleLowerCase().includes(search));
    const key = JSON.stringify([mode, search, entries]);
    if (actionKey !== key) {
      options("action", entries.map((entry) => ({ ...entry,
        label: `${entry.group ? `${entry.group} · ` : ""}${entry.label}` })), snapshot.action,
      entries.length ? null : { id: "", label: "没有匹配的动作" });
      actionKey = key;
    } else if (!timelineEditing) node("action").value = snapshot.action;
    node("action").disabled = snapshot.mode === "reference" || entries.length === 0;
  }

  function renderParts() {
    const key = JSON.stringify(snapshot.parts.map((part) => [part.id, part.label]));
    if (key !== partKey) {
      const list = node("parts"); list.replaceChildren();
      for (const part of snapshot.parts) {
        const label = document.createElement("label"); label.className = "viewer-check";
        const input = document.createElement("input"); input.type = "checkbox";
        input.dataset.part = part.id; input.checked = part.visible;
        input.addEventListener("change", () => command("part", part.id, input.checked ? 1 : 0));
        const name = document.createElement("span"); name.textContent = part.label;
        label.appendChild(input); label.appendChild(name); list.appendChild(label);
      }
      partKey = key;
    }
    for (const input of node("parts").querySelectorAll("input[data-part]")) {
      const part = snapshot.parts.find((part) => part.id === input.dataset.part);
      if (part) input.checked = part.visible;
      input.disabled = !snapshot.ready;
    }
  }

  function loading() {
    if (!bound) return;
    const panel = node("loading");
    const progress = node("progress");
    const failed = startupState === "failed" && !engineReady || Boolean(renderFailure) || snapshot?.failed;
    const loadingScene = failed || !engineReady || !snapshot?.ready;
    panel.hidden = !loadingScene;
    node("retry").hidden = !failed;
    if (failed) {
      text("loading-label", "画面准备失败");
      text("loading-detail", renderFailure || snapshot?.failed && snapshot.status || string(startupDetail.reason, 240) || "请重新加载后重试。");
      progress.removeAttribute("value");
    } else if (engineReady) {
      text("loading-label", snapshot?.status || "正在准备所选模型…");
      text("loading-detail", "正在等待所选模型的渲染资源准备完成。");
      progress.removeAttribute("value");

    } else if (startupDetail.phase === "download") {
      const total = finite(startupDetail.total) && startupDetail.total > 0 ? startupDetail.total : 0;
      const loaded = finite(startupDetail.loaded) && startupDetail.loaded >= 0 ? startupDetail.loaded : 0;
      text("loading-label", total ? `正在下载 ${Math.floor(loaded / total * 100)}%` : "正在下载检查程序…");
      text("loading-detail", loaded ? `已接收 ${(loaded / 1048576).toFixed(1)} MB · 模型和动作随后加载` : "模型和动作准备后即可检查，无需捕获鼠标。");
      if (total) { progress.max = total; progress.value = loaded; } else progress.removeAttribute("value");
    } else if (startupDetail.phase === "initialize" && startupDetail.total > 0) {
      // The engine reports completed / required files of the default model.
      text("loading-label", `正在加载模型与动作 ${startupDetail.loaded} / ${startupDetail.total}`);
      text("loading-detail", "其余角色和武器在选中时加载。");
      progress.max = startupDetail.total; progress.value = startupDetail.loaded;
    } else {
      text("loading-label", startupDetail.phase === "render" ? "正在准备模型画面…" : "正在初始化检查场景…");
      text("loading-detail", "使用游戏相同的模型、蒙皮和武器挂接计算。");
      progress.removeAttribute("value");
    }
  }

  function draw() {
    if (!bound || !snapshot) return;
    if (catalog) {
      const key = JSON.stringify(catalog);
      if (catalogKey !== key) {
        options("preset", catalog.presets, snapshot.preset, { id: "custom", label: "自定义搭配（诊断）" });
        options("appearance", catalog.appearances, snapshot.appearance);
        options("weapon", catalog.weapons, snapshot.weapon);
        catalogKey = key;
      }
      for (const name of ["preset", "appearance", "weapon"]) node(name).value = snapshot[name];
      const label = (entries, value) => entries.find((entry) => entry.id === value)?.label || value;
      text("current", `${label(catalog.appearances, snapshot.appearance)} · ${label(catalog.weapons, snapshot.weapon)}`);
      renderActions();
    }
    text("status", snapshot.status || (snapshot.ready ? "模型已就绪" : "正在准备模型…"));
    text("context", `${{ game: "游戏表现检查", source: "源片段对照", reference: "角色参考姿态" }[snapshot.mode]} / ${snapshot.playing ? "播放中" : "暂停"}`);
    text("mode-help", snapshot.mode === "source" ? "源片段已做现有根运动与骨架适配；不叠加游戏持握修正，默认隐藏手持。"
      : snapshot.mode === "reference" ? "当前资产参考姿态；它不一定等于蒙皮绑定姿态。" : "检查游戏采用的裁剪、混合和持握结果。");
    node("game-mode").setAttribute("aria-pressed", String(snapshot.mode === "game"));
    node("source-mode").setAttribute("aria-pressed", String(snapshot.mode === "source"));
    node("reference").setAttribute("aria-pressed", String(snapshot.mode === "reference"));
    text("warning", snapshot.warning || "使用实际游戏配置与共享显示计算。");
    text("trace", snapshot.trace || "当前姿态暂无额外采样信息。");
    text("time", `${snapshot.time.toFixed(3)} / ${snapshot.duration.toFixed(3)} s · 第 ${snapshot.frame} 帧`);
    text("play", snapshot.playing ? "暂停" : "播放");
    node("play").setAttribute("aria-pressed", String(snapshot.playing));
    node("loop").checked = snapshot.looping;
    node("speed").value = String(snapshot.speed);
    node("timeline").max = String(snapshot.duration);
    if (!timelineEditing) node("timeline").value = String(snapshot.time);
    for (const control of document.querySelectorAll("[data-debug]")) control.checked = snapshot.debug[control.dataset.debug];
    renderParts(); loading();
  }

  function clearDrag() {
    if (drag) {
      const previous = drag;
      drag = null;
      try { previous.canvas.releasePointerCapture(previous.id); } catch (_) { /* Already released on cancel. */ }
    }
    timelineEditing = false;
  }

  async function copyInspection() {
    if (!snapshot) return;
    const revision = ++copyRevision;
    const content = snapshot.inspection || JSON.stringify(snapshot, null, 2);
    try {
      if (!navigator.clipboard?.writeText) throw new Error("Clipboard unavailable");
      await navigator.clipboard.writeText(content);
      if (revision === copyRevision) text("copy-status", "已复制当前检查信息");
    } catch (_) {
      if (revision === copyRevision) text("copy-status", "复制未成功，请允许剪贴板访问后重试。");
    }
  }

  function bind() {
    if (bound) return;
    bound = true;
    document.title = "Kiva · 模型、武器与动作验收";
    document.getElementById("viewer").hidden = false;
    const canvas = document.getElementById("game");
    node("viewport").appendChild(canvas);
    canvas.setAttribute("aria-label", "模型检查：左键拖动环绕，右键拖动平移，滚轮缩放。");
    for (const name of ["preset", "appearance", "weapon", "action"]) {
      node(name).addEventListener("change", (event) => {
        if (event.target.value !== "custom") command(name, event.target.value);
      });
    }
    node("game-mode").addEventListener("click", () => command("mode", "game"));
    node("source-mode").addEventListener("click", () => command("mode", "source"));
    node("reference").addEventListener("click", () => command("mode", snapshot?.mode === "reference" ? "game" : "reference"));
    node("search").addEventListener("input", (event) => { search = string(event.target.value, 160).toLocaleLowerCase(); renderActions(); });
    node("play").addEventListener("click", () => command("play", snapshot?.playing ? 0 : 1));
    node("restart").addEventListener("click", () => { command("seek", 0); command("play", 1); });
    node("step-back").addEventListener("click", () => command("step", -1));
    node("step-next").addEventListener("click", () => command("step", 1));
    node("speed").addEventListener("change", (event) => command("speed", Number(event.target.value)));
    node("loop").addEventListener("change", (event) => command("loop", event.target.checked ? 1 : 0));
    node("timeline").addEventListener("pointerdown", () => { timelineEditing = true; });
    node("timeline").addEventListener("input", (event) => command("seek", Number(event.target.value)));
    node("timeline").addEventListener("change", () => { timelineEditing = false; });
    for (const control of document.querySelectorAll("[data-debug]")) {
      control.addEventListener("change", () => command("debug", control.dataset.debug, control.checked ? 1 : 0));
    }
    for (const button of document.querySelectorAll("[data-view]")) {
      button.addEventListener("click", () => { if (views.includes(button.dataset.view)) command("view", button.dataset.view); });
    }
    node("fit").addEventListener("click", () => command("fit"));
    node("reset").addEventListener("click", () => command("reset"));
    node("copy").addEventListener("click", copyInspection);
    node("retry").addEventListener("click", () => window.location.reload());
    canvas.addEventListener("contextmenu", (event) => event.preventDefault());
    canvas.addEventListener("pointerdown", (event) => {
      if (event.button !== 0 && event.button !== 2 || !finite(event.clientX) || !finite(event.clientY)) return;
      clearDrag(); canvas.focus();
      drag = { id: event.pointerId, x: event.clientX, y: event.clientY, pan: event.button === 2, canvas };
      try { canvas.setPointerCapture(event.pointerId); } catch (_) { drag = null; }
      event.preventDefault();
    });
    canvas.addEventListener("pointermove", (event) => {
      if (!drag || drag.id !== event.pointerId || !finite(event.clientX) || !finite(event.clientY)) return;
      const x = Math.max(-2000, Math.min(2000, event.clientX - drag.x));
      const y = Math.max(-2000, Math.min(2000, event.clientY - drag.y));
      drag.x = event.clientX; drag.y = event.clientY;
      if (x || y) command(drag.pan ? "pan" : "orbit", x, y);
      event.preventDefault();
    });
    canvas.addEventListener("pointerup", clearDrag);
    canvas.addEventListener("pointercancel", clearDrag);
    canvas.addEventListener("lostpointercapture", clearDrag);
    canvas.addEventListener("wheel", (event) => {
      if (!finite(event.deltaY)) return;
      const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.clientHeight : 1;
      command("zoom", Math.max(-2000, Math.min(2000, event.deltaY * unit)));
      event.preventDefault();
    }, { passive: false });
    window.addEventListener("blur", clearDrag);
    document.addEventListener("pointerup", () => { timelineEditing = false; });
    draw(); loading();
  }

  window.kivaViewer = Object.freeze({
    takeCommands() { const result = commands.join("\n"); commands = []; return result; },
    render(encoded) {
      if (typeof encoded !== "string" || encoded.length > 1048576) return;
      let raw; try { raw = JSON.parse(encoded); } catch (_) { return; }
      const next = safeSnapshot(raw);
      if (!next || snapshot && next.revision < snapshot.revision) return;
      if (raw.catalog) { const nextCatalog = safeCatalog(raw.catalog); if (!nextCatalog) return; catalog = nextCatalog; }
      if (next.ready && !next.failed) renderFailure = "";
      snapshot = next; draw();
    },
    fail(reason) {
      renderFailure = string(reason, 240) || "画面准备失败，请重新加载后重试。";
      clearDrag(); loading();
    },
    ready() {
      if (engineReady) return;
      engineReady = true;
      window.dispatchEvent(new CustomEvent("kiva-ready"));
      loading();
    },
  });
  window.kivaStartup.subscribe((state, detail) => { startupState = state; startupDetail = detail; loading(); });
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind, { once: true });
  else bind();
})();
