/* Equipment presentation and intents. Definitions and state come from Rust. */
(() => {
  "use strict";
  window.createEquipmentUi = ({ pause, blocked }) => {
    const root = document.documentElement;
    const $ = (id) => document.getElementById(id);
    let state = null;
    let catalog = [];
    let opened = false;
    let requestId = 0;
    let pending = null;
    let inventorySignature = "";
    let catalogSignature = "";

    function close() {
      opened = false;
      $("equipment-editor").hidden = true;
      $("equipment-toggle").setAttribute("aria-expanded", "false");
      $("equipment-manage").focus();
    }

    function open() {
      if (blocked() || state?.session.phase !== "Active") return;
      opened = true;
      pause();
      $("equipment-editor").hidden = false;
      $("equipment-toggle").setAttribute("aria-expanded", "true");
      renderEditor();
      $("equipment-close").focus();
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
      renderEditor();
    }

    function itemDefinition(id) { return catalog.find((item) => item.id === id); }

    function renderInventory() {
      const slots = state?.equipment?.slots || [];
      const signature = JSON.stringify(slots.map(({ slot, item }) => [slot, item?.id, item?.instance_id]));
      if (signature === inventorySignature) return;
      inventorySignature = signature;
      $("equipment-inventory").replaceChildren(...slots.map(({ slot, item }) => {
        const cell = document.createElement("div");
        cell.className = "inventory-slot";
        cell.dataset.inventorySlot = slot;
        const number = document.createElement("span");
        const name = document.createElement("strong");
        number.textContent = `装备格 ${slot + 1}`;
        name.textContent = item?.name || "空槽";
        cell.append(number, name);
        if (item) {
          const remove = document.createElement("button");
          remove.textContent = "卸下";
          remove.dataset.unequipSlot = slot;
          remove.setAttribute("aria-label", `卸下装备格 ${slot + 1} 的${item.name}`);
          remove.addEventListener("click", () => submit({ type: "Unequip", slot, instance_id: item.instance_id }));
          cell.append(remove);
        }
        return cell;
      }));
    }

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
      renderInventory();
      renderCatalog();
      const equipment = state?.equipment;
      const waiting = pending !== null;
      const enabled = Boolean(equipment?.can_manage) && !waiting;
      const full = equipment?.first_empty_slot == null;
      for (const button of $("equipment-catalog").querySelectorAll("[data-equip-id]")) {
        button.disabled = !enabled || full;
        button.textContent = full ? "装备格已满" : "装备";
      }
      for (const button of $("equipment-inventory").querySelectorAll("[data-unequip-slot]")) button.disabled = !enabled;
      const error = !waiting && equipment?.ack?.error;
      $("equipment-status").classList.toggle("error", Boolean(error));
      $("equipment-status").textContent = waiting ? "正在确认装备调整…" : error
        || (!equipment?.can_manage ? "正在准备装备配置…" : full
          ? "六个装备格已满，请先卸下一件装备。" : "装备调整即时生效，返回后可继续试炼。");
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
        button.disabled = !active || state.paused || !active.available;
        button.querySelector(".equipment-symbol").textContent = item?.name.slice(0, 1) || "◇";
        button.querySelector(".equipment-cooldown").textContent = active
          ? active.cooldown >= 1 ? String(Math.ceil(active.cooldown)) : active.cooldown.toFixed(1) : "";
        button.title = item ? [item.name, ...(definition ? descriptionLines(definition) : [])].join("\n") : `空装备格 ${slot + 1}`;
        button.setAttribute("aria-label", `${slot + 1} ${item?.name || "空装备格"}${active ? ` · ${active.target_hint}` : ""}`);
      }
    }

    function render(nextState) {
      const wasActive = state?.session.phase === "Active";
      state = nextState;
      if (Array.isArray(state.equipment_catalog)) {
        catalog = state.equipment_catalog;
        window.__kivaEquipmentCatalog = catalog; // Read-only diagnostic projection.
      }
      if (state.session.phase !== "Active") {
        opened = false;
        pending = null;
        if (wasActive) requestId = 0;
        delete root.dataset.equipmentSkill;
        $("equipment-editor").hidden = true;
        $("equipment-toggle").setAttribute("aria-expanded", "false");
      } else if (pending !== null && !root.hasAttribute("data-equipment-requests")) {
        const ack = state.equipment?.ack;
        if (ack?.request_id === pending || (ack?.request_id == null && ack?.error)) pending = null;
      }
      $("equipment-toggle").hidden = state.session.phase !== "Active";
      $("equipment-toggle").disabled = blocked();
      $("equipment-manage").disabled = blocked();
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
          ? `能量 ${item.active.cost} · 冷却 ${item.active.total_cooldown}s · 射程 ${item.active.range}` : "该装备没有主动能力") };
    }

    $("equipment-toggle").addEventListener("click", open);
    $("equipment-manage").addEventListener("click", open);
    $("equipment-close").addEventListener("click", close);
    window.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && opened) {
        event.preventDefault();
        close();
      } else if (event.code === "KeyP" && !event.repeat && !event.ctrlKey && !event.metaKey && !event.altKey) {
        event.preventDefault();
        if (opened) close(); else open();
      }
    });
    for (const button of document.querySelectorAll("[data-equipment-slot]")) {
      button.addEventListener("click", () => {
        if (!state || state.session.phase !== "Active" || root.dataset.paused !== "false") return;
        const slot = Number(button.dataset.equipmentSlot);
        const item = state.equipment?.slots[slot]?.item;
        if (!item?.active?.available) return;
        root.dataset.equipmentSkill = JSON.stringify({ slot, instance_id: item.instance_id });
        $("game").focus();
      });
    }
    return { render, detail, isOpen: () => opened, isPending: () => pending !== null };
  };
})();
