/* Browser presentation only. Rust owns game rules and authoritative state. */
(() => {
  "use strict";
  const root = document.documentElement;
  const $ = (id) => document.getElementById(id);
  const canvas = $("game");
  const labels = new Map();
  const actionNames = {
    Ready: "准备就绪", Attacking: "攻击中", Casting: "施法中",
    Channeling: "引导中", Recovery: "收招中",
  };
  let ready = false;
  let entered = false;
  window.__kivaStartup = "loading";

  function setPaused(paused) {
    root.dataset.paused = String(paused);
  }

  function syncMenu() {
    $("entry").hidden = !ready || root.dataset.paused === "false";
    if (entered) {
      $("entry-title").textContent = "片刻停留，再出发。";
      $("enter").textContent = "继续试炼 →";
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
  $("enter").addEventListener("click", () => {
    entered = true;
    setPaused(false);
    canvas.focus();
  });
  canvas.addEventListener("contextmenu", (event) => event.preventDefault());
  canvas.addEventListener("pointerdown", () => canvas.focus());
  window.addEventListener("blur", () => {
    setPaused(true);
    delete root.dataset.skill;
  });
  $("help-toggle").addEventListener("click", toggleHelp);
  window.addEventListener("keydown", (event) => {
    if (event.key === "?") toggleHelp();
  });
  document.querySelectorAll("[data-slot]").forEach((button) => {
    button.addEventListener("click", () => {
      if (root.dataset.paused === "false") {
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

  function renderHero(player) {
    if (!player) return;
    $("health-fill").style.width = `${100 * player.health / player.max_health}%`;
    $("mana-fill").style.width = `${100 * player.mana / player.max_mana}%`;
    $("health-text").textContent = `${Math.ceil(player.health)} / ${player.max_health}`;
    $("mana-text").textContent = `${Math.floor(player.mana)} / ${player.max_mana}`;
    $("hero-action").textContent = player.alive
      ? (actionNames[player.action] || player.action) : "等待重生";
  }

  function renderAbilities(state, player) {
    for (const ability of state.abilities) {
      const button = document.querySelector(`[data-slot="${ability.slot}"]`);
      if (!button) continue;
      button.classList.toggle("cooling", ability.cooldown > 0.01);
      button.classList.toggle("aiming", state.aim === ability.slot);
      button.classList.toggle("no-energy", Boolean(player && player.mana < ability.cost));
      button.querySelector(".cooldown").textContent = ability.cooldown >= 1
        ? String(Math.ceil(ability.cooldown)) : ability.cooldown.toFixed(1);
      button.title = `${ability.name} · 能量 ${ability.cost} · 冷却 ${ability.total_cooldown}s · 射程 ${ability.range}`;
    }
    $("aim-hint").hidden = state.aim === null && !state.attack_move;
    $("aim-text").textContent = state.attack_move
      ? "攻击移动：选择前进方向"
      : `${state.abilities.find((a) => a.slot === state.aim)?.name || ""}：选择目标`;
  }

  function renderUnits(state) {
    const present = new Set();
    for (const unit of state.units) {
      present.add(unit.id);
      let label = labels.get(unit.id);
      if (!label) {
        label = document.createElement("div");
        label.className = `unit-label ${unit.team}`;
        const name = document.createElement("strong");
        name.textContent = unit.id === state.player_id ? "旅者" : unit.name;
        const bar = document.createElement("div");
        bar.className = "bar";
        bar.appendChild(document.createElement("i"));
        label.append(name, bar);
        $("unit-labels").appendChild(label);
        labels.set(unit.id, label);
      }
      label.hidden = !unit.alive || !unit.screen;
      if (unit.screen) {
        label.style.left = `${unit.screen[0]}px`;
        label.style.top = `${unit.screen[1]}px`;
      }
      label.querySelector("i").style.width = `${100 * unit.health / unit.max_health}%`;
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
  }

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
    const player = state.units.find((unit) => unit.id === state.player_id);
    $("session-status").textContent = state.paused ? "试炼暂停" : "本地试炼 · WEBGPU";
    $("camera-mode").textContent = state.camera_locked ? "跟随镜头" : "自由镜头";
    $("notice").textContent = state.notice;
    renderHero(player);
    renderAbilities(state, player);
    renderUnits(state);
    renderMinimap(state);
  });
})();
