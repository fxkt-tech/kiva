/* Match presentation consumes authoritative Rust projections only. */
(() => {
  "use strict";
  const $ = (id) => document.getElementById(id);
  window.createMatchUi = ({ restart, reselect }) => {
    let state = null;
    let pending = false;
    $("restart").addEventListener("click", () => {
      if (pending || state?.phase !== "Finished") return;
      pending = restart(state.id);
      syncButtons();
    });
    $("result-reselect").addEventListener("click", () => {
      if (pending || state?.phase !== "Finished") return;
      pending = reselect();
      syncButtons();
    });

    function syncButtons() {
      $("restart").disabled = pending;
      $("result-reselect").disabled = pending;
      $("restart").textContent = pending ? "正在准备…" : "再来一局";
    }

    function renderBase(team, base) {
      const element = $(`${team}-base-health`);
      element.textContent = base ? `${Math.ceil(base.health)} / ${base.max_health}` : "—";
      const fill = $(`${team}-base-fill`);
      fill.style.width = base ? `${100 * base.health / base.max_health}%` : "0%";
    }

    function render(next) {
      if (state?.id !== next.id || next.phase !== "Finished") pending = false;
      state = next;
      const active = next.phase !== "Waiting";
      $("match-hud").hidden = !active;
      const seconds = Math.floor(next.elapsed_seconds);
      $("match-clock").textContent = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
      $("wave-count").textContent = String(next.wave_number);
      $("next-wave").textContent = next.next_wave_seconds == null ? "—" : `${Math.ceil(next.next_wave_seconds)}s`;
      renderBase("ally", next.bases.ally);
      renderBase("enemy", next.bases.enemy);
      const ended = next.phase === "Finished";
      $("match-result").hidden = !ended;
      if (ended) {
        const names = { Victory: "胜利", Defeat: "失败", Draw: "平局" };
        $("match-result-title").textContent = names[next.outcome] || "对局结束";
        $("match-result").dataset.outcome = next.outcome || "";
        $("match-result-detail").textContent = next.outcome === "Draw"
          ? "双方基地同时被摧毁。" : next.outcome === "Victory"
            ? "敌方基地已被摧毁。" : "我方基地已被摧毁。";
      }
      syncButtons();
    }

    return Object.freeze({
      render,
      ended: () => state?.phase === "Finished",
      blocked: () => state?.phase === "Finished" || Boolean(state?.error),
      pending: () => pending,
      rejected: () => { pending = false; syncButtons(); },
    });
  };
})();
