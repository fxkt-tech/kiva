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
    <section className="flex min-h-0 flex-col rounded-lg border border-border bg-surface/45">
      <div className="flex shrink-0 items-center justify-between border-b border-border px-3 py-2">
        <h2 className="text-sm font-semibold text-foreground">Board</h2>
        <div className="text-xs text-subtle">
          {state.currentPhase} · Day {state.dayNumber}
        </div>
      </div>
      <div className="grid min-h-0 grid-cols-1 gap-2 p-2 sm:grid-cols-2 xl:grid-cols-3">
        {[...game.players]
          .sort((left, right) => left.seatNo - right.seatNo)
          .map((player) => {
            const dead = deadPlayerIds.has(player.playerId);

            return (
              <article
                key={player.playerId}
                className="min-w-0 rounded-md border border-border bg-background/55 px-3 py-2"
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="text-xs text-subtle">
                      Seat {player.seatNo}
                    </div>
                    <div className="mt-1 truncate text-sm font-medium text-foreground">
                      {player.actor.identity.name}
                    </div>
                  </div>
                  <span
                    className={
                      dead
                        ? "rounded-full bg-danger-badge px-2.5 py-1 text-xs text-danger-badge-foreground"
                        : "rounded-full bg-good-badge px-2.5 py-1 text-xs text-good-badge-foreground"
                    }
                  >
                    {dead ? "dead" : "alive"}
                  </span>
                </div>
                <div className="mt-2 text-xs uppercase tracking-[0.14em] text-muted">
                  {player.ruleRole.id}
                </div>
              </article>
            );
          })}
      </div>
    </section>
  );
}
