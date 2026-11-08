/* Equipment presentation and intents. Definitions and state come from Rust. */
(() => {
  "use strict";
  // Rust publishes static definitions once. Retain them even when its first
  // snapshot precedes the deferred HUD; no DOM exists at this early boundary.
  let initialCatalog = null;
  function retainCatalog(event) {
    const snapshot = typeof event.detail === "string" ? JSON.parse(event.detail) : event.detail;
    if (!Array.isArray(snapshot.equipment_catalog)) return;
    initialCatalog = snapshot.equipment_catalog;
    window.removeEventListener("kiva-state", retainCatalog);
  }
  window.addEventListener("kiva-state", retainCatalog);

  window.createEquipmentUi = ({ blocked, onToggle = () => {} }) => {
    const root = document.documentElement;
    const $ = (id) => document.getElementById(id);
    let state = null;
    let catalog = initialCatalog || [];
    if (initialCatalog) window.__kivaEquipmentCatalog = catalog;
    let opened = false;
    let requestId = 0;
    let pending = null;
    let catalogSignature = "";

    function close() {
      if (!opened) return;
      opened = false;
      $("equipment-editor").hidden = true;
      $("equipment-toggle").setAttribute("aria-expanded", "false");
      renderSlots();
      onToggle();
    }

    function open() {
      if (opened || blocked() || state?.session.phase !== "Active" || state.match?.phase === "Finished") return;
      opened = true;
      $("equipment-editor").hidden = false;
      $("equipment-toggle").setAttribute("aria-expanded", "true");
      renderSlots();
      renderEditor();
      onToggle();
      if (document.pointerLockElement !== $("game")) $("equipment-close").focus();
    }

    function submit(request) {
      if (pending !== null || !state?.equipment?.can_manage || !state.player_id) return;
      try {
        const queued = JSON.parse(root.dataset.equipmentRequests || "[]");
        if (!Array.isArray(queued) || queued.length >= 32) throw new Error("full intent queue");
        requestId = requestId === 0xffffffff ? 1 : requestId + 1;
        queued.push({ ...request, player_id: state.player_id, request_id: requestId });
        root.dataset.equipmentRequests = JSON.stringify(queued);
        pending = requestId;
      } catch {
        $("equipment-status").textContent = "请求暂未完成，请稍后重试。";
        $("equipment-status").classList.add("error");
        return;
      }
      renderSlots();
      renderEditor();
    }

    function itemDefinition(id) { return catalog.find((item) => item.id === id); }

    function descriptionLines(item) {
      return [
        item.bonuses.join(" · "),
        ...item.passives.map((passive) => `${passive.name}：${passive.description}`),
        ...(item.active ? [`主动 · ${item.active.name}：${item.active.description}`] : []),
      ].filter(Boolean);
    }

    function renderCatalog() {
      const signature = JSON.stringify(catalog);
      if (signature === catalogSignature) return;
      catalogSignature = signature;
      $("equipment-catalog").replaceChildren(...catalog.map((item) => {
        const card = document.createElement("article");
        card.className = "equipment-card";
        card.dataset.equipmentId = item.id;
        const name = document.createElement("strong");
        const description = document.createElement("p");
        name.textContent = item.name;
        description.textContent = item.description;
        card.append(name, description);
        for (const [index, text] of descriptionLines(item).entries()) {
          const line = document.createElement("p");
          line.className = index === 0 && item.bonuses.length ? "equipment-bonuses" : "equipment-effect";
          line.textContent = text;
          card.append(line);
        }
        if (item.active) {
          const hint = document.createElement("p");
          hint.textContent = `${item.active.target_hint} · 能量 ${item.active.cost} · 冷却 ${item.active.total_cooldown}s`;
          card.append(hint);
        }
        const equip = document.createElement("button");
        equip.dataset.equipId = item.id;
        equip.setAttribute("aria-label", `装备${item.name}`);
        equip.addEventListener("click", () => {
          const slot = state?.equipment?.first_empty_slot;
          if (slot != null) submit({ type: "Equip", equipment_id: item.id, slot });
        });
        card.append(equip);
        return card;
      }));
    }

    function renderEditor() {
      if (!opened) return;
      renderCatalog();
      const equipment = state?.equipment;
      const waiting = pending !== null;
      const enabled = Boolean(equipment?.can_manage) && !waiting;
      const full = equipment?.first_empty_slot == null;
      for (const button of $("equipment-catalog").querySelectorAll("[data-equip-id]")) {
        button.disabled = !enabled || full;
        button.textContent = full ? "装备格已满" : "装备";
      }
      const error = !waiting && equipment?.ack?.error;
      $("equipment-status").classList.toggle("error", Boolean(error));
      $("equipment-status").textContent = waiting ? "正在确认装备调整…" : error
        || (!equipment?.can_manage ? "正在准备装备配置…" : full
          ? "装备格已满，右键底部装备可卸下。" : "装备即时生效 · 右键底部装备可卸下 · T 关闭");
    }

    function renderSlots() {
      const slots = state?.equipment?.slots || [];
      $("equipment-count").textContent = `装备 ${slots.filter(({ item }) => item).length} / 6`;
      for (const button of document.querySelectorAll("[data-equipment-slot]")) {
        const slot = Number(button.dataset.equipmentSlot);
        const item = slots[slot]?.item;
        const active = item?.active;
        const definition = item && itemDefinition(item.id);
        button.classList.toggle("filled", Boolean(item));
        button.classList.toggle("cooling", Boolean(active && active.cooldown > 0.01));
        button.classList.toggle("aiming", state?.equipment_aim === slot);
        button.classList.toggle("unavailable", Boolean(item && (!active || !active.available)));
        const removable = opened && item && state?.equipment?.can_manage && pending === null;
        button.disabled = !removable && (!active || state.paused || !active.available);
        button.classList.toggle("manageable", Boolean(removable));
        button.querySelector(".equipment-symbol").textContent = item?.name.slice(0, 1) || "◇";
        button.querySelector(".equipment-cooldown").textContent = active
          ? active.cooldown >= 1 ? String(Math.ceil(active.cooldown)) : active.cooldown.toFixed(1) : "";
        button.title = item ? [item.name, ...(definition ? descriptionLines(definition) : []),
          ...(removable ? ["右键卸下"] : [])].join("\n") : `空装备格 ${slot + 1}`;
        button.setAttribute("aria-label", `${slot + 1} ${item?.name || "空装备格"}${active ? ` · ${active.target_hint}` : ""}`);
      }
    }

    function render(nextState) {
      const wasActive = state?.session.phase === "Active";
      const ownerChanged = state?.player_id !== nextState.player_id;
      const ended = nextState.match?.phase === "Finished";
      if (ownerChanged || ended || nextState.session.phase !== "Active"
        || (!state?.paused && nextState.paused)) close();
      state = nextState;
      if (Array.isArray(state.equipment_catalog)) {
        catalog = state.equipment_catalog;
        window.__kivaEquipmentCatalog = catalog; // Read-only diagnostic projection.
      }
      if (state.session.phase !== "Active" || ownerChanged || ended) {
        pending = null;
        if (wasActive && (ownerChanged || state.session.phase !== "Active")) requestId = 0;
        delete root.dataset.equipmentSkill;
      } else if (pending !== null && !root.hasAttribute("data-equipment-requests")) {
        const ack = state.equipment?.ack;
        if (ack?.request_id === pending || (ack?.request_id == null && ack?.error)) pending = null;
      }
      $("equipment-toggle").hidden = state.session.phase !== "Active" || ended;
      $("equipment-toggle").disabled = blocked();
      renderSlots();
      renderEditor();
    }

    function detail() {
      if (!state || opened) return null;
      const hovered = document.querySelector("[data-equipment-slot][data-virtual-hover]");
      const slot = hovered ? Number(hovered.dataset.equipmentSlot) : state.equipment_aim;
      if (slot == null) return null;
      const item = state.equipment?.slots[slot]?.item;
      const definition = item && itemDefinition(item.id);
      if (!definition) return null;
      return { name: definition.name, target_hint: item.active?.target_hint || "被动装备",
        description: descriptionLines(definition).join("；"),
        note: item.active?.unavailable_reason || (item.active
          ? `能量 ${item.active.cost} · 冷却 ${item.active.total_cooldown}s · 射程 ${item.active.display_range}` : "该装备没有主动能力") };
    }

    $("equipment-toggle").addEventListener("click", open);
    $("equipment-close").addEventListener("click", close);
    window.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && opened) {
        event.preventDefault();
        close();
      } else if (event.code === "KeyT" && !event.repeat && !event.ctrlKey && !event.metaKey && !event.altKey
        && !event.target?.closest?.("input, textarea, select, [contenteditable]")) {
        event.preventDefault();
        if (opened) close(); else open();
      }
    });
    window.addEventListener("wheel", (event) => {
      if (!opened || document.pointerLockElement !== $("game")) return;
      const point = window.kivaPointer?.position();
      if (!point?.length) return;
      const scroll = $("equipment-catalog-scroll");
      if (!scroll.contains(document.elementFromPoint(point[0], point[1]))) return;
      const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? scroll.clientHeight : 1;
      scroll.scrollTop += event.deltaY * unit;
      event.preventDefault();
    }, { passive: false });
    for (const button of document.querySelectorAll("[data-equipment-slot]")) {
      button.addEventListener("click", () => {
        if (!state || state.session.phase !== "Active" || root.dataset.paused !== "false") return;
        const slot = Number(button.dataset.equipmentSlot);
        const item = state.equipment?.slots[slot]?.item;
        if (!item?.active?.available) return;
        root.dataset.equipmentSkill = JSON.stringify({ slot, instance_id: item.instance_id });
        $("game").focus();
      });
      const remove = (event) => {
        event.preventDefault();
        if (!opened || blocked()) return;
        const slot = Number(button.dataset.equipmentSlot);
        const item = state?.equipment?.slots[slot]?.item;
        if (item) submit({ type: "Unequip", slot, instance_id: item.instance_id });
      };
      button.addEventListener("contextmenu", remove);
      button.addEventListener("kiva-contextmenu", remove);
    }
    return { render, detail, close, isOpen: () => opened, isPending: () => pending !== null };
  };
})();
