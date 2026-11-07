/* Runtime badges are projections; this module owns no skill timers or rules. */
(() => {
  "use strict";
  window.renderSkillState = (button, ability) => {
    let badge = button.querySelector(".skill-state");
    if (!badge) {
      badge = document.createElement("span");
      badge.className = "skill-state";
      button.appendChild(badge);
    }
    const state = ability?.mechanics;
    const text = state?.recast ? (state.recast_ready ? "拆除" : "准备重施")
      : state?.charges != null ? `${state.charges}/${state.max_charges}`
      : state?.required_focus != null ? state.empowered ? "强化" : `${state.focus}/${state.required_focus}` : "";
    badge.textContent = text;
    badge.hidden = !text;
    button.classList.toggle("recast", Boolean(state?.recast));
    button.classList.toggle("empowered", Boolean(state?.required_focus != null && state.empowered));
    for (const [key, value] of Object.entries({ focus: state?.focus, charges: state?.charges,
      recast: state?.recast, empowered: state?.empowered })) {
      if (value == null) delete button.dataset[key]; else button.dataset[key] = String(value);
    }
    return state?.recharge != null ? `充能恢复 ${Math.ceil(state.recharge)}s` : text;
  };
})();
