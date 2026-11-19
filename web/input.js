/* Browser input owns capture and pixel units; Bevy owns direction and movement. */
(() => {
  "use strict";
  if (window.kivaPage?.isViewer()) return;
  const welcome = window.kivaPage?.isStation() ? "进入月台，在积水中寻找青鸾。" : "进入竹林，迎战愁铃客。";
  let ready = false;
  let confirmed = false;
  // The first-load splash stays until the player has entered once; after that the
  // entry card is the pause screen.
  let introduced = false;
  let scrollOpen = window.kivaPage.startsInHandscroll();
  let pair = "hammer";
  let queuedPair = 0;
  let awaitingPair = false;
  // The splash bar never runs backwards across loading phases.
  let shownProgress = 0;
  let request = null;
  let look = 0;
  let keys = 0;
  let actions = 0;
  let defenseEdges = [];
  let defenseCancelled = false;
  let rightBlocked = false;
  const mousePressed = new Set();
  const MAX_DEFENSE_EDGES = 64;
  // The bit layout shared with client/controls.rs; Rust writes it to
  // tests/fixtures/input/bits.json and the Node tests compare this against it.
  const HELD = Object.freeze({ forward: 1, back: 2, left: 4, right: 8, movement: 15, sprint: 16 });
  const ACTION = Object.freeze({
    jump: 1, dodge: 2, attack: 4, meleeSkill: 8, combo: 16, toggleTarget: 32, nextPair: 64,
    rage: 128, rangedSkill: 256, crouchToggle: 512, movementReleased: 1024, shiftReleased: 2048,
  });
  const DEFENSE_EDGE = Object.freeze({ pressed: 1, released: 2, cancelled: 3 });
  const pressed = new Set();
  const heldKeys = Object.freeze({
    KeyW: HELD.forward, KeyS: HELD.back, KeyA: HELD.left, KeyD: HELD.right, ShiftLeft: HELD.sprint, ShiftRight: HELD.sprint,
  });
  const actionKeys = Object.freeze({
    Space: ACTION.jump, KeyE: ACTION.meleeSkill, Tab: ACTION.combo, KeyZ: ACTION.nextPair, KeyX: ACTION.rage,
    KeyQ: ACTION.rangedSkill, KeyC: ACTION.crouchToggle,
  });
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
    actions &= ACTION.movementReleased | ACTION.shiftReleased;
    if (keys & HELD.movement) actions |= ACTION.movementReleased;
    if (keys & HELD.sprint) actions |= ACTION.shiftReleased;
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

  function notifyScroll() {
    window.dispatchEvent(new CustomEvent("kiva-handscroll-change"));
  }

  function loadingLabel() {
    if (startupDetail.phase === "render") return "正在准备画面…";
    const { loaded, total } = startupDetail;
    if (startupDetail.phase === "initialize") {
      return total > 0 ? `正在加载资源 ${loaded} / ${total}` : "正在准备游戏…";
    }
    return total > 0 ? `正在下载 ${Math.floor(100 * loaded / total)}%` : "正在下载…";
  }

  function render() {
    const entry = document.getElementById("entry");
    if (!entry) return;
    const failed = window.__kivaStartup === "failed" && !ready;
    const pending = ready && request !== null;
    const busy = (!ready && !failed) || pending;
    const measured = !ready && !failed && startupDetail.phase === "download" && startupDetail.total > 0;
    // After the program starts, the engine reports completed / required files.
    const counted = !ready && !failed && startupDetail.phase === "initialize" && startupDetail.total > 0;
    const label = loadingLabel();
    const frame = document.getElementById("handscroll");
    if (frame) {
      frame.hidden = !scrollOpen;
      if (scrollOpen && !frame.hasAttribute("src")) frame.src = "handscroll-motion-preview.html";
    }
    const game = document.getElementById("game-frame");
    if (game) game.inert = scrollOpen && !pending;
    const back = document.getElementById("return-to-handscroll");
    if (back) back.hidden = !window.kivaPage.isStation();
    const splash = document.getElementById("splash");
    if (splash) renderSplash(splash, failed, pending, label);
    const enterButton = document.getElementById("enter");
    const progress = document.getElementById("enter-progress");
    entry.hidden = scrollOpen || isActive() || (splash !== null && !introduced);
    enterButton.hidden = false;
    enterButton.disabled = !ready || pending;
    enterButton.setAttribute("aria-busy", String(busy));
    document.getElementById("enter-label").textContent = pending ? "正在进入…"
      : ready ? "点击进入" : failed ? "加载失败" : label;
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
      ? welcome : failed
        ? startupDetail.reason
        : startupDetail.phase === "render" ? "正在准备首次画面，请稍候…"
          : counted ? "正在加载模型、动作与场景贴图…"
            : startupDetail.phase === "initialize" ? "正在准备场景与游戏引擎…" : downloading);
    notifyScroll();
  }

  // Download is most of the wait; the engine then reports its files, then the first frame.
  function loadFraction() {
    if (ready) return 1;
    const { phase, loaded, total } = startupDetail;
    const part = total > 0 ? Math.min(1, loaded / total) : 0;
    const value = phase === "download" ? 0.6 * part : phase === "initialize" ? 0.6 + 0.35 * part : 0.95;
    shownProgress = Math.max(shownProgress, value);
    return shownProgress;
  }

  function renderSplash(splash, failed, pending, loadingLabel) {
    splash.hidden = introduced || scrollOpen;
    if (splash.hidden) return;
    const fraction = loadFraction();
    const percent = Math.round(fraction * 100);
    const bar = document.getElementById("splash-bar");
    if (bar) {
      bar.hidden = ready || failed;
      bar.setAttribute("aria-valuenow", String(percent));
    }
    document.getElementById("splash-fill")?.style.setProperty("width", `${percent}%`);
    const status = document.getElementById("splash-status");
    if (status) {
      status.textContent = ready ? (pending ? "正在进入…" : error) : failed ? startupDetail.reason : loadingLabel;
      status.hidden = ready && !pending && !error;
    }
    const prompt = document.getElementById("splash-prompt");
    if (prompt) prompt.hidden = !ready || pending;
    const retry = document.getElementById("splash-retry");
    if (retry) retry.hidden = !failed;
  }

  function captureDenied() {
    if (!request || confirmed) return;
    clearCapture();
    error = "未能锁定鼠标，请再次点击进入。";
    render();
    if (scrollOpen && document.hasFocus()) document.getElementById("handscroll")?.focus();
  }

  function enter(fromScroll = false) {
    if (!ready || request || isActive() || scrollOpen !== fromScroll) return;
    const target = canvas();
    if (!target) return;
    const attempt = {};
    request = attempt;
    error = "";
    render();
    const rejected = () => {
      if (request === attempt) captureDenied();
    };
    try {
      // Default pointer lock retains system mouse acceleration.
      if (fromScroll) window.kivaAudio?.unlockFromGesture();
      target.focus();
      const result = target.requestPointerLock();
      if (result?.catch) result.catch(rejected);
    } catch { rejected(); }
  }

  document.addEventListener("pointerlockchange", () => {
    if (document.pointerLockElement === canvas() && document.hasFocus() && (request || confirmed)) {
      confirmed = true;
      introduced = true;
      if (scrollOpen) {
        scrollOpen = false;
        document.documentElement.removeAttribute("data-handscroll");
      }
      request = null;
      clearInputs();
      error = "";
    } else {
      clearCapture();
      if (document.pointerLockElement === canvas()) document.exitPointerLock();
    }
    render();
  });
  document.addEventListener("pointerlockerror", captureDenied);
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
    const firstShift = heldKeys[event.code] === HELD.sprint && (keys & HELD.sprint) === 0;
    pressed.add(event.code);
    refreshKeys();
    if (firstShift) actions |= ACTION.dodge;
    if (event.code === "KeyC") actions ^= ACTION.crouchToggle;
    else actions |= actionKeys[event.code] || 0;
  }, true);
  document.addEventListener("keyup", (event) => {
    if (!event.isTrusted) return;
    if (isActive() && (heldKeys[event.code] || actionKeys[event.code])) event.preventDefault?.();
    const previouslyHeld = keys;
    pressed.delete(event.code);
    refreshKeys();
    if (isActive()) {
      if ((previouslyHeld & HELD.movement) && !(keys & HELD.movement)) actions |= ACTION.movementReleased;
      if ((previouslyHeld & HELD.sprint) && !(keys & HELD.sprint)) actions |= ACTION.shiftReleased;
    }
  }, true);
  document.addEventListener("pointerdown", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse" || !isActive()) return;
    if (![0, 1, 2].includes(event.button)) return;
    event.preventDefault?.();
    if (mousePressed.has(event.button)) return;
    mousePressed.add(event.button);
    if (event.button === 0) actions |= ACTION.attack;
    if (event.button === 1) actions ^= ACTION.toggleTarget;
    if (event.button === 2 && !rightBlocked) queueDefense(DEFENSE_EDGE.pressed);
  }, true);
  document.addEventListener("pointerup", (event) => {
    if (!event.isTrusted || event.pointerType !== "mouse" || !isActive()) return;
    if (!mousePressed.delete(event.button)) return;
    if (event.button === 2) {
      if (rightBlocked) rightBlocked = false;
      else queueDefense(DEFENSE_EDGE.released);
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
    // Read-only, for the contract test against Rust's layout.
    bits: Object.freeze({ held: HELD, actions: ACTION, defenseEdges: DEFENSE_EDGE }),
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
      const pending = defenseCancelled ? [DEFENSE_EDGE.cancelled] : [];
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

  window.kivaHandscroll = Object.freeze({
    isOpen: () => scrollOpen,
    getState() {
      const failed = !ready && window.__kivaStartup === "failed";
      const message = error || (failed ? startupDetail.reason : !ready ? loadingLabel()
        : request ? "正在锁定鼠标，成功后即可进入。" : "");
      return { ready: ready && !request, failed, message, pair, started: introduced };
    },
    enter(stageId) {
      if ((stageId === "station" || stageId === "1-1") && scrollOpen) enter(true);
    },
    retry() {
      if (scrollOpen && !ready && window.__kivaStartup === "failed") window.location.reload();
    },
    equip(choice) {
      if (!scrollOpen || (choice !== "hammer" && choice !== "axe") || choice === pair) return;
      pair = choice;
      queuedPair = choice === "hammer" ? 1 : 2;
      awaitingPair = true;
      notifyScroll();
    },
    takePair() {
      const choice = queuedPair;
      queuedPair = 0;
      return choice;
    },
    renderPair(code) {
      if (queuedPair || (code !== 1 && code !== 2)) return;
      const actual = code === 1 ? "hammer" : "axe";
      // Rust can report the old loadout until the first resumed fixed step.
      if (awaitingPair && !isActive() && actual !== pair) return;
      awaitingPair = false;
      if (actual === pair) return;
      pair = actual;
      notifyScroll();
    },
  });

  // On the splash, any key enters: not Esc, function keys or browser shortcuts.
  document.addEventListener("keydown", (event) => {
    if (!event.isTrusted || scrollOpen || introduced || !ready || event.repeat) return;
    if (!document.getElementById("splash")) return;
    if (event.code === "Escape" || /^F\d+$/.test(event.code) || event.metaKey || event.ctrlKey || event.altKey) return;
    event.preventDefault?.();
    enter();
  });

  function bind() {
    document.getElementById("enter").addEventListener("click", () => enter());
    document.getElementById("splash")?.addEventListener("click", () => enter());
    document.getElementById("splash-retry")?.addEventListener("click", () => window.location.reload());
    canvas().addEventListener("click", () => enter());
    document.getElementById("retry").addEventListener("click", () => window.location.reload());
    const frame = document.getElementById("handscroll");
    const focusScroll = () => {
      frame.focus();
      if (introduced) frame.contentWindow.dispatchEvent(new CustomEvent("kiva-handscroll-open"));
    };
    frame?.addEventListener("load", () => {
      if (scrollOpen && document.hasFocus()) focusScroll();
    });
    document.getElementById("return-to-handscroll")?.addEventListener("click", () => {
      if (!window.kivaPage.isStation() || isActive() || request || scrollOpen) return;
      scrollOpen = true;
      document.documentElement.setAttribute("data-handscroll", "open");
      error = "";
      const loaded = frame.hasAttribute("src");
      render();
      if (loaded) focusScroll();
    });
    window.kivaStartup.subscribe((_state, detail) => {
      startupDetail = detail;
      render();
    });
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", bind, { once: true });
  else bind();
})();
