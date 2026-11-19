(() => {
  const scroll = document.querySelector("#scroll");
  const track = document.querySelector("#track");
  const opening = document.querySelector(".opening");
  const station = document.querySelector(".station");
  const sortie = document.querySelector("#sortie");

  const brush = sortie.querySelector(".brush");
  const seal = sortie.querySelector(".seal");
  const stageButtons = Array.from(station.querySelectorAll("[data-stage]"));
  const heading = station.querySelector("#stage-name");
  const goal = station.querySelector("#stage-goal");
  const battle = station.querySelector("#battle");
  const feedback = station.querySelector("#battle-status");
  const retry = station.querySelector("#retry-game");
  // The standalone print also works when framed by a preview site.
  const host = window.frameElement && window.parent.kivaHandscroll;
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)");
  let menuOpen = true;
  const canAnimate = () => menuOpen && !document.hidden && !reduced.matches;
  let selected = stageButtons.find((button) => button.getAttribute("aria-pressed") === "true");
  const armory = new HandscrollArmory(document.querySelector(".armory"), host);
  const stageRoute = {
    name: "station", leaf: station, trigger: sortie, origin: brush,
    back: document.querySelector("#return"), focus: () => selected,
  };
  const bookmark = document.querySelector("#armory-open");
  const armoryRoute = {
    name: "armory", leaf: armory.page, trigger: bookmark, origin: bookmark,
    back: document.querySelector("#armory-back"), focus: () => armory.selectedButton,
    enter: () => armory.inspect(armory.equipped),
  };
  const ink = new HandscrollInk(document.querySelector("#ink-field"), document.querySelector("#ink-threshold"));
  const canReveal = () =>
    position === 1 && !animation && !inkLayer && !pointer?.dragging && canAnimate();
  const persona = new HandscrollPersona(document.querySelector(".persona"), armory, ink, canReveal);
  const personaBookmark = document.querySelector("#persona-open");
  const personaRoute = {
    name: "persona", leaf: persona.page, trigger: personaBookmark, origin: personaBookmark,
    back: document.querySelector("#persona-back"), focus: () => persona.selectedButton,
    enter: () => persona.inspect(persona.selectedButton),
    finishInner: () => persona.finishTransition(),
  };
  const bestiary = new HandscrollBestiary(document.querySelector(".bestiary"), ink, canReveal);
  const birdBookmark = document.querySelector("#bestiary-open");
  const bestiaryRoute = {
    name: "bestiary", leaf: bestiary.page, trigger: birdBookmark, origin: birdBookmark,
    back: document.querySelector("#bestiary-back"), focus: () => bestiary.focusControl,
    enter: () => bestiary.showOverview(), finishInner: () => bestiary.finishTransition(),
  };
  const supply = new HandscrollSupply(document.querySelector(".supply"), ink, canReveal);
  const supplyBookmark = document.querySelector("#supply-open");
  const supplyRoute = {
    name: "supply", leaf: supply.page, trigger: supplyBookmark, origin: supplyBookmark,
    back: document.querySelector("#supply-back"), focus: () => supply.stallButton,
    finishInner: () => supply.reveal.finish(),
  };
  // Each route is one facing leaf paired with the same opening.
  let route = stageRoute;
  let nextRoute = null;

  const clamp = (value) => Math.max(0, Math.min(1, value));
  let position = 0;
  let animation = 0;
  let destination = 0;
  let pointer = null;
  let completedTouch = null;
  let suppressClick = false;
  let inkLayer = null;

  function paintScroll() {
    // Open the last fold as it leaves view; feather only the strip shared by both paintings.
    const lastFold = clamp((position - .5) / .5);
    const unfolded = 12 * lastFold * lastFold * (3 - 2 * lastFold);
    // At rest align the map directly to the viewport. Fractional opposing
    // compositor translations would resample the detailed print twice.
    route.leaf.hidden = position === 0;
    route.leaf.style.left = position === 1 ? "0" : "";
    track.style.transform = position === 1 ? "none" : `translate3d(${position * 88}%,0,0)`;
    opening.style.transform = `translate3d(${unfolded}%,0,0)`;
    opening.hidden = position === 1;
    opening.style.setProperty("--mist-drift", `${-4 * position}cqw`);
    track.style.setProperty("--feather", `${(12 - unfolded) * clamp(position / .16)}%`);
    track.style.setProperty("--home-ui", clamp(1 - position / .4));
    track.style.setProperty("--station-ui", clamp((position - .52) / .4));
  }

  function paint(value) {
    position = clamp(value);
    if (inkLayer) ink.advance(inkLayer === opening ? 1 - position : position);
    else paintScroll();
    const inside = position >= .5;
    const name = inside ? route.name : "opening";
    if (scroll.dataset.page === name) return;
    const oldPage = inside ? opening : route.leaf;
    const newControl = inside ? route.focus() : route.trigger;
    const transferFocus = oldPage.contains(document.activeElement) && document.activeElement.matches(":focus-visible");
    if (!inside && !host) feedback.textContent = "";
    opening.inert = inside;
    route.leaf.inert = !inside;
    scroll.dataset.page = name;
    if (menuOpen && transferFocus) newControl.focus({ preventScroll: true });
  }

  function stopAnimation() {
    cancelAnimationFrame(animation);
    animation = 0;
  }

  function finishAt(to) {
    if (inkLayer) {
      inkLayer.classList.remove("revealing");
      inkLayer.style.removeProperty("mask-image");
      track.classList.remove("inking");
      inkLayer = null;
    }
    paint(to);
    if (to === 0) {
      // A fresh manual pull from home always opens the chapter map.
      route = stageRoute;
      if (nextRoute) {
        const pending = nextRoute;
        nextRoute = null;
        visit(pending.route, pending.event);
      }
    }
  }

  function go(to) {
    if (animation && destination === to) return;
    destination = to;
    stopAnimation();
    if (!to) {
      sortie.classList.remove("impressed");
      if (!host) feedback.textContent = "";
    }
    const from = position;
    const distance = Math.abs(to - from);
    if (!canAnimate() || distance < .001) {
      finishAt(to);
      return;
    }
    const start = performance.now();
    const duration = (inkLayer ? HandscrollInk.duration : 1350) * Math.sqrt(distance);
    function step(now) {
      const t = clamp((now - start) / duration);
      if (t === 1) {
        animation = 0;
        finishAt(to);
        return;
      }
      const eased = inkLayer ? t : t * t * (3 - 2 * t);
      paint(from + (to - from) * eased);
      animation = requestAnimationFrame(step);
    }
    animation = requestAnimationFrame(step);
  }

  function navigate(to, event) {
    if (position === to && !animation && !inkLayer) return;
    route.finishInner?.();
    // Reverse a partially unrolled scroll or a wet edge in place. Only
    // an explicit departure from a resting page creates a fresh ink field.
    if (position === 1 - to && !inkLayer && canAnimate()) {
      ink.prepareFrom(scroll, to ? route.origin : route.back, event);
      inkLayer = to ? route.leaf : opening;
      route.leaf.hidden = false;
      track.classList.add("inking");
      track.style.transform = "none";
      opening.style.transform = "none";
      opening.style.setProperty("--mist-drift", "0cqw");
      opening.hidden = false;
      inkLayer.classList.add("revealing");
      inkLayer.style.maskImage = "url(#ink-wash)";
    }
    sortie.classList.toggle("impressed", Boolean(to) && route === stageRoute);
    go(to);
  }

  function visit(target, event) {
    nextRoute = null;
    if (route !== target && position !== 0) {
      // A tap can land on the newly visible home label during the return.
      // Finish the current pair before revealing a different painting.
      nextRoute = { route: target, event };
      navigate(0);
      return;
    }
    if (position === 0 && (!inkLayer || target !== route)) {
      stopAnimation();
      finishAt(0);
      route = target;
      route.enter?.();
    }
    navigate(1, event);
  }

  for (const target of [stageRoute, armoryRoute, personaRoute, bestiaryRoute, supplyRoute]) {
    target.trigger.addEventListener("click", (event) => visit(target, event));
    target.back.addEventListener("click", (event) => {
      nextRoute = null;
      navigate(0, event);
    });
  }

  function selectStage(button) {
    selected.setAttribute("aria-pressed", "false");
    selected = button;
    selected.setAttribute("aria-pressed", "true");
    const name = selected.querySelector(".name").textContent;
    const characters = Array.from(name, (character) => {
      const span = document.createElement("span");
      span.textContent = character;
      return span;
    });
    heading.replaceChildren(...characters);
    heading.setAttribute("aria-label", name);
    const locked = host && selected.dataset.stage !== "1-1";
    goal.previousElementSibling.textContent = locked ? "状态" : "目标";
    const description = host ? selected.dataset.gameGoal : selected.dataset.goal;
    goal.textContent = locked ? "尚未开放" : description;
    seal.textContent = name;
    sortie.setAttribute("aria-label", `出击，前往第一章灰原的${name}`);
    battle.setAttribute("aria-label", `进入战斗，${name}`);
    if (host) refreshGame();
    else feedback.textContent = "";
  }

  function refreshGame() {
    const state = host.getState();
    const open = host.isOpen();
    if (menuOpen !== open) {
      menuOpen = open;
      document.documentElement.toggleAttribute("data-menu-closed", !open);
      if (!open) settleImmediately();
      window.dispatchEvent(new Event("handscroll-visibilitychange"));
    }
    if (state.pair !== armory.equipped) {
      armory.setEquipped(state.pair);
      persona.inspect(persona.selectedButton);
    }
    const available = selected.dataset.stage === "1-1";
    const name = selected.querySelector(".name").textContent;
    const label = available && state.started ? "继续战斗" : "进入战斗";
    if (battle.textContent !== label) battle.textContent = label;
    battle.setAttribute("aria-label", `${name}，${label}`);
    battle.disabled = !available || !state.ready || state.failed || !open;
    const message = available ? state.message : "";
    if (feedback.textContent !== message) feedback.textContent = message;
    retry.hidden = !available || !state.failed;
  }

  for (const button of stageButtons) {
    button.addEventListener("click", () => selectStage(button));
  }
  battle.addEventListener("click", (event) => {
    if (host) {
      if (!event.isTrusted || battle.disabled || selected.dataset.stage !== "1-1") return;
      host.enter("1-1");
      refreshGame();
      return;
    }
    const chapter = document.createElement("span");
    chapter.textContent = `出战预览 · ${selected.dataset.stage} ${selected.querySelector(".name").textContent}`;
    const weapons = document.createElement("span");
    weapons.textContent = `随行　${armory.equippedNames}`;
    feedback.replaceChildren(chapter, " ", weapons);
  });

  function releasePointer() {
    if (!pointer) return;
    const id = pointer.id;
    pointer = null;
    scroll.classList.remove("dragging");
    if (scroll.hasPointerCapture(id)) scroll.releasePointerCapture(id);
  }

  scroll.addEventListener("pointerdown", (event) => {
    completedTouch = null;
    if (!event.isPrimary || event.button !== 0) return;
    // Keep the native click target until the gesture becomes a drag.
    pointer = { id: event.pointerId, start: event.clientX, last: event.clientX,
      time: event.timeStamp, speed: 0, origin: position, dragging: false };
    suppressClick = false;
  });

  function movePointer(event) {
    if (!pointer || pointer.id !== event.pointerId) return;
    const delta = event.clientX - pointer.start;
    if (!pointer.dragging) {
      if (Math.abs(delta) < 5) return;
      route.finishInner?.();
      stopAnimation();
      nextRoute = null;
      // A running automatic scroll may have advanced since pointerdown.
      pointer.origin = position;
      pointer.dragging = true;
      scroll.classList.add("dragging");
      scroll.setPointerCapture(event.pointerId);
      suppressClick = true;
    }
    const travel = scroll.clientWidth * .88;
    const elapsed = Math.max(1, event.timeStamp - pointer.time);
    pointer.speed = (event.clientX - pointer.last) / elapsed / travel;
    pointer.last = event.clientX;
    pointer.time = event.timeStamp;
    paint(pointer.origin + delta / travel);
  }
  window.addEventListener("pointermove", movePointer);

  function finishPointer(event) {
    if (!pointer || pointer.id !== event.pointerId) return;
    const { dragging, speed, time } = pointer;
    if (!dragging && event.type === "pointerup" && event.pointerType === "touch") {
      completedTouch = event;
    }
    releasePointer();
    if (!dragging) return;
    const momentum = event.type === "pointerup" && event.timeStamp - time < 100 ? speed * 140 : 0;
    go(position + momentum >= .5 ? 1 : 0);
  }
  window.addEventListener("pointerup", finishPointer);
  window.addEventListener("pointercancel", finishPointer);
  scroll.addEventListener("lostpointercapture", finishPointer);
  scroll.addEventListener("click", (event) => {
    // Mobile browsers may snap a synthesized click to a nearby label.
    // Keep the physical tap with this exact click, including queued navigation.
    if (completedTouch && event.detail) ink.tapOrigins.set(event, completedTouch);
    completedTouch = null;
    if (!suppressClick || event.detail === 0) return;
    event.stopImmediatePropagation();
    suppressClick = false;
  }, true);

  document.addEventListener("keydown", (event) => {
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    if (!["ArrowLeft", "ArrowRight", "Escape"].includes(event.key)) return;
    event.preventDefault();
    releasePointer();
    nextRoute = null;
    if (event.key !== "ArrowLeft" && route === bestiaryRoute && bestiary.inDetail && position >= .5) {
      bestiary.showOverview(event);
    } else if (event.key === "ArrowLeft") visit(route);
    else navigate(0);
  });

  function settleImmediately() {
    route.finishInner?.();
    releasePointer();
    nextRoute = null;
    const target = animation ? destination : Math.round(position);
    stopAnimation();
    finishAt(target);
  }
  reduced.addEventListener("change", () => { if (reduced.matches) settleImmediately(); });
  document.addEventListener("visibilitychange", () => { if (document.hidden) settleImmediately(); });
  window.addEventListener("blur", () => {
    if (pointer) {
      releasePointer();
      go(Math.round(position));
    }
  });
  paint(0);
  if (host) {
    for (const notice of station.querySelectorAll(".stage-access")) notice.hidden = false;
    window.parent.addEventListener("kiva-handscroll-change", refreshGame);
    window.addEventListener("pagehide", (event) => {
      if (!event.persisted) window.parent.removeEventListener("kiva-handscroll-change", refreshGame);
    });
    window.addEventListener("kiva-handscroll-open", () => {
      settleImmediately();
      finishAt(0);
      sortie.classList.remove("impressed");
      sortie.focus({ preventScroll: true });
    });
    retry.addEventListener("click", () => host.retry());
    selectStage(selected);
  }
})();
