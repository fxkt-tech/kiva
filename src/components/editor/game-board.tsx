import type { Game } from "@/core/game";
import { deriveGameState } from "@/core/state";
import type { GameEvent } from "@/core/events";

type GameBoardProps = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
};

export function GameBoard({ game, events }: GameBoardProps) {
  const state = deriveGameState(game.players, events);
  const deadPlayerIds = new Set(state.deadPlayerIds);

  return (
    <section className="flex min-h-0 flex-col rounded-lg border border-zinc-800 bg-zinc-900/45">
      <div className="flex shrink-0 items-center justify-between border-b border-zinc-800 px-4 py-3">
        <h2 className="text-sm font-semibold text-zinc-100">Board</h2>
        <div className="text-xs text-zinc-500">
          {state.currentPhase} · Day {state.dayNumber}
        </div>
      </div>
      <div className="grid min-h-0 flex-1 grid-cols-1 gap-2 overflow-y-auto p-3 sm:grid-cols-2 xl:grid-cols-3">
        {[...game.players]
          .sort((left, right) => left.seatNo - right.seatNo)
          .map((player) => {
            const dead = deadPlayerIds.has(player.playerId);

            return (
              <article
                key={player.playerId}
                className="min-w-0 rounded-md border border-zinc-800 bg-zinc-950/55 p-3"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="text-xs text-zinc-500">
                      Seat {player.seatNo}
                    </div>
                    <div className="mt-1 truncate text-sm font-medium text-zinc-100">
                      {player.name}
                    </div>
                  </div>
                  <span
                    className={
                      dead
                        ? "rounded-full border border-red-900/70 px-2 py-1 text-xs text-red-300"
                        : "rounded-full border border-emerald-900/70 px-2 py-1 text-xs text-emerald-300"
                    }
                  >
                    {dead ? "dead" : "alive"}
                  </span>
                </div>
                <div className="mt-3 text-xs uppercase tracking-[0.14em] text-zinc-400">
                  {player.gameRole}
                </div>
              </article>
            );
          })}
      </div>
    </section>
  );
}
