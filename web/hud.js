/* Browser presentation only. Rust owns game rules and authoritative state. */
(() => {
  "use strict";
  const root = document.documentElement;
  const $ = (id) => document.getElementById(id);
  const canvas = $("game");
  const labels = new Map();
  const actionNames = {
    Ready: "准备就绪", Attacking: "攻击前摇", AttackBackswing: "攻击后摇", Casting: "施法中",
    Channeling: "引导中", Recovery: "收招中",
  };
  const statusNames = { Stun: "眩晕", Root: "禁锢", Slow: "减速", Airborne: "击飞" };
  const skillKeys = ["Q", "W", "E", "R"];
  const skillIcons = {
    enemy: '<circle cx="20" cy="20" r="9"/><circle cx="20" cy="20" r="3"/><path d="M20 3v10m0 14v10M3 20h10m14 0h10"/>',
    direction: '<path d="m24 5-14 18h10l-4 12 15-20H20z"/>',
    area: '<circle cx="20" cy="20" r="12"/><circle cx="20" cy="20" r="5"/><path d="M20 3v7m0 20v7M3 20h7m20 0h7"/>',
    dash: '<path d="M5 14h20m-16 7h24M4 28h20m-2-20 12 13-12 13"/>',
    channel: '<path d="m20 4 4 11 12 5-12 4-4 12-11-4 11-5z"/><circle cx="20" cy="20" r="5"/>',
  };
  let ready = false;
  let session = null;
  let activePlayerId = null;
  let returnPending = false;
  let captureAttempt = null;
  let latestSelectionRequest = 0;
  let requestedCharacter = null;
  let selectionError = "";
  let choicesSignature = "";
  let previewSignature = "";
  let helpLoadout = "";
  const statFields = [
    ["attack_damage", 1], ["ability_power", 1], ["armor", 1], ["magic_resistance", 1],
    ["move_speed", 2], ["attack_speed", 2], ["attack_range", 2],
  ].map(([key, digits]) => ({
    key, digits,
    element: document.querySelector(`[data-stat="${key}"]`),
    previewElement: document.querySelector(`[data-preview-stat="${key}"]`),
  }));
  window.__kivaStartup = "loading";

  function setPaused(paused) {
    root.dataset.paused = String(paused);
    if (paused && document.pointerLockElement === canvas) document.exitPointerLock();
  }

  function enqueueSessionRequest(request) {
    try {
      const requests = JSON.parse(root.dataset.sessionRequests || "[]");
      if (!Array.isArray(requests) || requests.length >= 32) return false;
      requests.push(request);
      root.dataset.sessionRequests = JSON.stringify(requests);
      return true;
    } catch {
      return false;
    }
  }

  function confirmedChoice() {
    const selection = session?.selection;
    if (session?.phase !== "Selecting" || !selection?.can_start
      || !selection.selection_ack.accepted
      || selection.selection_ack.request_id !== latestSelectionRequest
      || !selection.selected || selection.selection_revision == null
      || selection.selected.id !== requestedCharacter) return null;
    return { character_id: selection.selected.id, selection_revision: selection.selection_revision };
  }

  function selectCharacter(characterId) {
    if (!ready || session?.phase !== "Selecting" || captureAttempt) return;
    const requestId = latestSelectionRequest === 0xffffffff ? 1 : latestSelectionRequest + 1;
    if (!enqueueSessionRequest({ type: "SelectCharacter", character_id: characterId, request_id: requestId })) {
      selectionError = "选择请求暂未完成，请稍后重试。";
    } else {
      latestSelectionRequest = requestId;
      requestedCharacter = characterId;
      selectionError = "";
      $("pointer-error").hidden = true;
    }
    syncMenu();
  }

  function clearCaptureAttempt(attempt) {
    if (!attempt || captureAttempt !== attempt) return;
    document.removeEventListener("pointerlockerror", attempt.onError);
    captureAttempt = null;
  }

  function enterGame() {
    if (!ready || captureAttempt || returnPending || !session) return;
    const choice = confirmedChoice();
    if (session.phase === "Selecting" && !choice) return;
    const attempt = { phase: session.phase, choice, stage: "requesting" };
    attempt.onError = () => captureFailed(attempt);
    captureAttempt = attempt;
    // Each request needs its own event acknowledgement, even when the previous
    // unlock notification has not been dispatched yet.
    root.dataset.pointerLocked = "false";
    document.addEventListener("pointerlockerror", attempt.onError);
    $("pointer-error").hidden = true;
    syncMenu();
    canvas.focus();
    try {
      // Keep the request in this trusted click. Promise settlement is not a
      // capture acknowledgement; only pointerlockchange can submit Launch.
      const request = canvas.requestPointerLock();
      if (request?.catch) request.catch(() => captureFailed(attempt));
    } catch {
      captureFailed(attempt);
    }
  }

  function captureFailed(attempt) {
    if (captureAttempt !== attempt || attempt.stage !== "requesting") return;
    clearCaptureAttempt(attempt);
    setPaused(true);
    $("pointer-error").textContent = "鼠标捕获未成功，请点击进入按钮重试。";
    $("pointer-error").hidden = false;
    syncMenu();
  }

  function returnToSelection() {
    if (!ready || session?.phase !== "Active" || !activePlayerId || returnPending
      || captureAttempt || root.dataset.paused !== "true" || document.pointerLockElement === canvas) return;
    if (enqueueSessionRequest({ type: "ReturnToSelection", player_id: activePlayerId })) {
      returnPending = true;
      delete root.dataset.skill;
      $("pointer-error").hidden = true;
    } else {
      $("pointer-error").textContent = "请求暂未完成，请重新选择角色重试。";
      $("pointer-error").hidden = false;
    }
    syncMenu();
  }

  function pointerLockChanged() {
    const captured = document.pointerLockElement === canvas;
    root.dataset.pointerLocked = String(captured);
    if (!captured) {
      clearCaptureAttempt(captureAttempt);
      delete root.dataset.skill;
      $("game-cursor").hidden = true;
      setPaused(true);
      syncMenu();
      return;
    }
    const attempt = captureAttempt;
    if (returnPending) {
      setPaused(true);
      return;
    }
    if (attempt?.stage === "launching") return;
    if (session?.phase === "Active") {
      clearCaptureAttempt(attempt);
      $("pointer-error").hidden = true;
      setPaused(false);
    } else if (session?.phase === "Selecting" && attempt?.phase === "Selecting"
      && attempt.stage === "requesting" && document.hasFocus()) {
      if (!enqueueSessionRequest({ type: "Launch", ...attempt.choice })) {
        clearCaptureAttempt(attempt);
        selectionError = "启动请求暂未完成，请稍后重试。";
        setPaused(true);
      } else {
        attempt.stage = "launching";
        document.removeEventListener("pointerlockerror", attempt.onError);
        // Rust must receive the frozen choice before observing an unpaused DOM.
        setPaused(false);
      }
    } else {
      clearCaptureAttempt(attempt);
      setPaused(true);
    }
    syncMenu();
  }

  function syncMenu() {
    const active = session?.phase === "Active";
    $("entry").hidden = !ready || (active && root.dataset.paused === "false");
    $("entry").dataset.phase = active ? "Active" : "Selecting";
    $("character-select").hidden = active;
    $("entry-title").textContent = active ? "片刻停留，再出发。" : "选择你的旅程。";
    $("entry-description").textContent = active
      ? "继续探索场地，练习走位，释放技能。"
      : "选择角色，查看初始属性与技能，再进入试炼。";
    $("enter").textContent = captureAttempt
      ? (captureAttempt.stage === "launching" ? "正在进入…" : "正在捕获鼠标…")
      : (active ? "继续试炼 →" : "进入试炼 →");
    $("enter").disabled = !ready || Boolean(captureAttempt) || returnPending || (!active && !confirmedChoice());
    $("reselection-actions").hidden = !active;
    $("reselect").textContent = returnPending ? "正在返回选角…" : "重新选择角色";
    $("reselect").disabled = !ready || !activePlayerId || returnPending || Boolean(captureAttempt)
      || root.dataset.paused !== "true" || document.pointerLockElement === canvas;
    const selection = session?.selection;
    const waiting = selection?.selection_ack.request_id !== latestSelectionRequest;
    $("selection-status").hidden = active;
    $("selection-status").textContent = captureAttempt?.stage === "launching"
      ? "正在确认开局…" : waiting ? "正在确认角色选择…"
      : confirmedChoice() ? "准备就绪，进入试炼后开始。" : "请选择一个角色。";
    $("selection-error").textContent = selectionError || selection?.error || "";
    $("selection-error").hidden = active || !$("selection-error").textContent;
    $("character-preview").setAttribute("aria-busy", String(waiting));
    for (const button of $("character-choices").querySelectorAll("[data-character-id]")) {
      button.disabled = Boolean(captureAttempt);
      button.setAttribute("aria-pressed", String(Number(button.dataset.characterId) === selection?.selected?.id));
    }
  }

  function toggleHelp() {
    $("help").hidden = !$("help").hidden;
    $("help-toggle").setAttribute("aria-expanded", String(!$("help").hidden));
  }

  function startupFailed() {
    if (ready) return;
    window.__kivaStartup = "failed";
    $("loading").classList.add("failed");
    $("loading").setAttribute("role", "alert");
    $("loading").querySelector("h2").textContent = "暂时无法进入试炼场";
    $("loading-message").textContent = "请使用支持 WebGPU 的 Chrome / Edge，开启 GPU 加速，并通过 localhost 或 HTTPS 访问。";
    $("retry").hidden = false;
  }

  new MutationObserver(syncMenu).observe(root, {
    attributes: true, attributeFilter: ["data-paused"],
  });
  $("enter").addEventListener("click", enterGame);
  $("reselect").addEventListener("click", returnToSelection);
  document.addEventListener("pointerlockchange", pointerLockChanged);
  canvas.addEventListener("contextmenu", (event) => event.preventDefault());
  canvas.addEventListener("pointerdown", () => canvas.focus());
  window.addEventListener("blur", () => {
    clearCaptureAttempt(captureAttempt);
    setPaused(true);
    delete root.dataset.skill;
    syncMenu();
  });
  $("help-toggle").addEventListener("click", toggleHelp);
  window.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && document.pointerLockElement === canvas) setPaused(true);
    if (event.key === "?") toggleHelp();
  });
  document.querySelectorAll("[data-slot]").forEach((button) => {
    button.addEventListener("click", () => {
      if (session?.phase === "Active" && root.dataset.paused === "false") {
        root.dataset.skill = button.dataset.slot;
        canvas.focus();
      }
    });
  });
  window.addEventListener("kiva-ready", () => {
    ready = true;
    window.__kivaStartup = "ready";
    $("loading").hidden = true;
    syncMenu();
  }, { once: true });
  $("retry").addEventListener("click", () => location.reload());
  window.addEventListener("TrunkApplicationError", startupFailed, { once: true });
  window.addEventListener("error", startupFailed);
  window.addEventListener("unhandledrejection", startupFailed);
  if (!navigator.gpu) startupFailed();

  function formatAttribute(value, digits) {
    if (!Number.isFinite(value)) return "—";
    const rounded = Number(value.toFixed(digits));
    return (value !== 0 && rounded === 0) || Math.abs(value) >= 1e6
      ? value.toExponential(1) : String(rounded);
  }

  function renderCharacterChoices(choices) {
    const signature = JSON.stringify(choices);
    if (choicesSignature === signature) return;
    choicesSignature = signature;
    $("character-choices").replaceChildren(...choices.map((choice) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "character-choice";
      button.dataset.characterId = choice.id;
      button.setAttribute("aria-pressed", "false");
      const name = document.createElement("strong");
      const description = document.createElement("span");
      name.textContent = choice.name;
      description.textContent = choice.description;
      button.append(name, description);
      button.addEventListener("click", () => selectCharacter(choice.id));
      return button;
    }));
  }

  function renderPreviewAbilities(abilities) {
    $("preview-abilities").replaceChildren(...skillKeys.map((key, slot) => {
      const ability = abilities.find((item) => item.slot === slot);
      const item = document.createElement("article");
      item.className = "preview-ability";
      item.dataset.previewSlot = slot;
      const name = document.createElement("strong");
      name.textContent = `${key} · ${ability?.name || "未装配"}`;
      item.append(name);
      if (ability) {
        const target = document.createElement("span");
        const description = document.createElement("p");
        const details = document.createElement("small");
        target.textContent = ability.target_hint;
        description.textContent = ability.description;
        details.textContent = `能量 ${ability.cost} · 冷却 ${ability.total_cooldown}s · 射程 ${ability.range}`;
        item.append(target, description, details);
      }
      return item;
    }));
  }

  function renderCharacterPreview(selected) {
    const signature = JSON.stringify(selected);
    if (previewSignature === signature) return;
    previewSignature = signature;
    $("preview-name").textContent = selected?.name || "请选择角色";
    $("preview-description").textContent = selected?.description || "";
    $("preview-health").textContent = formatAttribute(selected?.max_health, 1);
    $("preview-mana").textContent = formatAttribute(selected?.max_mana, 1);
    for (const { key, digits, previewElement } of statFields) {
      previewElement.textContent = formatAttribute(selected?.attributes?.[key], digits);
    }
    renderPreviewAbilities(selected?.abilities || []);
  }

  function renderSession(nextSession, playerId) {
    if (session?.phase === "Active" && nextSession.phase === "Selecting") {
      clearCaptureAttempt(captureAttempt);
      latestSelectionRequest = 0;
      requestedCharacter = null;
      selectionError = "";
      delete root.dataset.skill;
      root.dataset.pointerLocked = "false";
      setPaused(true);
      $("pointer-error").hidden = true;
    }
    if (returnPending && !root.hasAttribute("data-session-requests")) {
      returnPending = false;
      if (nextSession.phase === "Active") {
        $("pointer-error").textContent = "暂时无法重新选择角色，请重试。";
        $("pointer-error").hidden = false;
      }
    }
    session = nextSession;
    activePlayerId = playerId;
    if (session.phase === "Active") {
      if (captureAttempt?.phase === "Selecting") clearCaptureAttempt(captureAttempt);
      selectionError = "";
    } else {
      const selection = session.selection;
      if (latestSelectionRequest === 0 && requestedCharacter === null) {
        requestedCharacter = selection?.selected?.id ?? null;
      }
      renderCharacterChoices(selection?.choices || []);
      renderCharacterPreview(selection?.selected || null);
    }
    syncMenu();
  }

  function renderHero(player) {
    $("hero-hud").hidden = session?.phase !== "Active" || !player;
    for (const { key, digits, element } of statFields) {
      element.textContent = formatAttribute(player?.attributes?.[key], digits);
    }
    renderStatuses($("hero-statuses"), player);
    if (!player) return;
    $("hero-name").textContent = player.name;
    $("health-fill").style.width = `${100 * player.health / player.max_health}%`;
    $("mana-fill").style.width = `${100 * player.mana / player.max_mana}%`;
    $("health-text").textContent = `${Math.ceil(player.health)} / ${player.max_health}`;
    $("mana-text").textContent = `${Math.floor(player.mana)} / ${player.max_mana}`;
    $("hero-action").textContent = !player.alive ? "等待重生"
      : player.control?.is_displaced ? "强制位移中"
      : (actionNames[player.action] || player.action);
  }

  function renderStatuses(container, unit) {
    const statuses = unit?.statuses || [];
    const texts = statuses.map((status) => {
      const amount = status.slow_fraction == null ? "" : `${Math.round(status.slow_fraction * 100)}%`;
      return `${statusNames[status.kind] || status.kind}${amount} ${Math.ceil(status.remaining * 10) / 10}s`;
    });
    if (unit?.control?.is_displaced) texts.push("强制位移");
    const signature = texts.join("|");
    if (container.dataset.signature !== signature) {
      const badges = texts.map((text, index) => {
        const badge = document.createElement("span");
        badge.dataset.status = statuses[index]?.kind || "Displaced";
        badge.textContent = text;
        return badge;
      });
      container.replaceChildren(...badges);
      container.dataset.signature = signature;
    }
    container.hidden = texts.length === 0;
  }

  function renderHelp(abilities) {
    const signature = JSON.stringify(abilities.map(({ slot, name, target_hint, description }) => [slot, name, target_hint, description]));
    if (helpLoadout === signature) return;
    helpLoadout = signature;
    $("help-abilities").replaceChildren(...abilities.map((ability) => {
      const item = document.createElement("div");
      const name = document.createElement("strong");
      const target = document.createElement("span");
      const effects = document.createElement("small");
      name.textContent = `${skillKeys[ability.slot]} · ${ability.name}`;
      target.textContent = ability.target_hint;
      effects.textContent = ability.description;
      item.append(name, target, effects);
      return item;
    }));
  }

  function renderAbilities(state, player) {
    const slots = new Set(state.abilities.map((ability) => ability.slot));
    for (const button of document.querySelectorAll(".ability")) {
      if (slots.has(Number(button.dataset.slot))) continue;
      button.disabled = true;
      button.classList.remove("cooling", "aiming", "no-energy", "unavailable");
      button.querySelector("small").textContent = "未装配";
      button.querySelector("svg").replaceChildren();
      button.querySelector(".cooldown").textContent = "";
      button.removeAttribute("data-icon");
      button.removeAttribute("title");
      button.setAttribute("aria-label", `${skillKeys[Number(button.dataset.slot)]} 未装配技能`);
    }
    for (const ability of state.abilities) {
      const button = document.querySelector(`[data-slot="${ability.slot}"]`);
      if (!button) continue;
      button.classList.toggle("cooling", ability.cooldown > 0.01);
      button.classList.toggle("aiming", state.aim === ability.slot);
      button.classList.toggle("no-energy", Boolean(player && player.mana < ability.cost));
      button.classList.toggle("unavailable", !ability.available);
      button.disabled = state.paused || !ability.available;
      button.querySelector("small").textContent = ability.name;
      button.setAttribute("aria-label", `${skillKeys[ability.slot]} ${ability.name}`);
      if (button.dataset.icon !== ability.icon) {
        button.querySelector("svg").innerHTML = skillIcons[ability.icon] || skillIcons.enemy;
        button.dataset.icon = ability.icon;
      }
      button.querySelector(".cooldown").textContent = ability.cooldown >= 1
        ? String(Math.ceil(ability.cooldown)) : ability.cooldown.toFixed(1);
      button.title = `${ability.name} · ${ability.target_hint}\n${ability.description}\n能量 ${ability.cost} · 冷却 ${ability.total_cooldown}s · 射程 ${ability.range}${ability.unavailable_reason ? `\n${ability.unavailable_reason}` : ""}`;
    }
    $("aim-hint").hidden = state.aim === null && !state.attack_move;
    $("aim-text").textContent = state.attack_move
      ? "攻击移动：选择前进方向"
      : (() => {
        const ability = state.abilities.find((a) => a.slot === state.aim);
        return ability ? `${ability.name}：${ability.target_hint}` : "";
      })();
    const hovered = document.querySelector(".ability[data-virtual-hover]");
    const detail = state.abilities.find((ability) => ability.slot === (hovered ? Number(hovered.dataset.slot) : state.aim));
    $("ability-detail").hidden = !detail || state.paused;
    if (detail) {
      $("ability-detail").querySelector("strong").textContent = detail.name;
      $("ability-detail").querySelector("span").textContent = detail.target_hint;
      $("ability-detail").querySelector("p").textContent = detail.description;
      $("ability-detail").querySelector("small").textContent = detail.unavailable_reason || `能量 ${detail.cost} · 冷却 ${detail.total_cooldown}s · 射程 ${detail.range}`;
    }
    renderHelp(state.abilities);
  }

  function renderUnits(state) {
    const present = new Set();
    for (const unit of state.units) {
      present.add(unit.id);
      let label = labels.get(unit.id);
      if (!label) {
        label = document.createElement("div");
        label.className = `unit-label ${unit.team}`;
        label.hidden = true;
        const name = document.createElement("strong");
        name.textContent = unit.name;
        const bar = document.createElement("div");
        bar.className = "bar";
        bar.appendChild(document.createElement("i"));
        const statuses = document.createElement("div");
        statuses.className = "status-badges";
        label.append(name, bar, statuses);
        $("unit-labels").appendChild(label);
        labels.set(unit.id, label);
      }
      label.querySelector("i").style.width = `${100 * unit.health / unit.max_health}%`;
      renderStatuses(label.querySelector(".status-badges"), unit);
    }
    for (const [id, label] of labels) {
      if (!present.has(id)) { label.remove(); labels.delete(id); }
    }
    const selected = state.units.find((unit) => unit.id === state.selected);
    $("target-panel").hidden = !selected;
    if (selected) {
      $("target-name").textContent = selected.name;
      $("target-health").textContent = `${Math.ceil(selected.health)} / ${selected.max_health} HP`;
    }
    renderStatuses($("target-statuses"), selected);
  }

  // Position belongs to the render frame. The slower HUD snapshot updates
  // health, names and menus; it must never overwrite these screen coordinates.
  window.addEventListener("kiva-labels", (event) => {
    const positions = JSON.parse(event.detail);
    const present = new Set();
    for (const unit of positions) {
      present.add(unit.id);
      const label = labels.get(unit.id);
      if (!label) continue;
      label.hidden = !unit.alive || !unit.screen;
      if (unit.screen) {
        label.style.left = `${unit.screen[0]}px`;
        label.style.top = `${unit.screen[1]}px`;
      }
    }
    for (const [id, label] of labels) if (!present.has(id)) label.hidden = true;
  });

  function renderMinimap(state) {
    const ctx = $("minimap").getContext("2d");
    const scale = 180 / 64;
    ctx.fillStyle = "#20362e";
    ctx.fillRect(0, 0, 180, 180);
    ctx.strokeStyle = "#7e9b7330";
    ctx.lineWidth = 1;
    for (let i = 0; i <= 180; i += 30) {
      ctx.beginPath();
      ctx.moveTo(i, 0); ctx.lineTo(i, 180);
      ctx.moveTo(0, i); ctx.lineTo(180, i); ctx.stroke();
    }
    ctx.fillStyle = "#718069";
    for (const [x, y, hx, hy] of state.obstacles) {
      ctx.fillRect((x - hx + 32) * scale, (y - hy + 32) * scale, hx * 2 * scale, hy * 2 * scale);
    }
    ctx.strokeStyle = "#c1d8bc80";
    ctx.strokeRect((state.camera_center[0] + 32) * scale - 27, (state.camera_center[1] + 32) * scale - 20, 54, 40);
    for (const unit of state.units) {
      if (!unit.alive) continue;
      ctx.fillStyle = unit.team === "ally" ? "#8cddc3" : "#df9a7c";
      ctx.beginPath();
      ctx.arc((unit.position[0] + 32) * scale, (unit.position[1] + 32) * scale,
        unit.team === "ally" ? 3 : 2, 0, Math.PI * 2);
      ctx.fill();
    }
  }

  window.addEventListener("kiva-state", (event) => {
    const state = JSON.parse(event.detail);
    // Read-only diagnostics allow tests to assert actual simulation results.
    window.__kivaState = state;
    renderSession(state.session, state.player_id);
    const player = state.units.find((unit) => unit.id === state.player_id);
    $("session-status").textContent = state.session.phase === "Selecting"
      ? "选择角色" : state.paused ? "试炼暂停" : "本地试炼 · WEBGPU";
    $("camera-mode").textContent = state.camera_locked ? "跟随镜头" : "自由镜头";
    $("notice").textContent = state.notice;
    renderHero(player);
    renderAbilities(state, player);
    renderUnits(state);
    renderMinimap(state);
  });
})();
