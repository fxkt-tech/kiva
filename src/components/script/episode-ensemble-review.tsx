import type { EpisodeScriptSnapshot } from "@/core/episode-script";
import type { PlayerSnapshot } from "@/core/player";

export function EpisodeEnsembleReview({
  script,
  players,
}: {
  readonly script: EpisodeScriptSnapshot;
  readonly players: readonly PlayerSnapshot[];
}) {
  const playersById = new Map(
    players.map((player) => [player.playerId, player]),
  );

  return (
    <section className="mt-5" aria-labelledby="ensemble-review-heading">
      <div>
        <h3 id="ensemble-review-heading" className="text-sm font-semibold">
          群像与人物弧线
        </h3>
        <p className="mt-1 text-xs leading-5 text-muted">
          主次权重来自合法轨迹，每位角色仍有一个可识别的标志性时刻。以下内容是导演方向，不是对局事实。
        </p>
      </div>

      <div className="mt-3 grid gap-3 md:grid-cols-2">
        {script.castDirections.map((direction) => {
          const player = playersById.get(direction.playerId);
          return (
            <article
              key={direction.playerId}
              className="rounded border border-border bg-background/50 p-4"
            >
              <div className="flex items-start justify-between gap-3">
                <div>
                  <p className="text-[11px] text-subtle">
                    {player ? `${player.seatNo} 号` : direction.playerId}
                  </p>
                  <h4 className="mt-0.5 text-sm font-semibold">
                    {player?.actor.identity.name ?? direction.playerId}
                  </h4>
                </div>
                <span className="rounded-full border border-border px-2 py-1 text-[10px] text-muted">
                  {direction.dramaticWeight === "primary"
                    ? "主要角色"
                    : "支持角色"}
                </span>
              </div>

              <p className="mt-3 text-sm leading-6 text-foreground">
                {direction.dramaticFunction}
              </p>
              <dl className="mt-3 space-y-2 text-xs leading-5 text-muted">
                <DirectionRow label="起点" value={direction.baseline} />
                <DirectionRow label="受压" value={direction.pressure} />
                <DirectionRow label="变化" value={direction.change} />
                <DirectionRow label="兑现" value={direction.payoff} />
                <DirectionRow
                  label={`标志时刻 #${direction.signatureMoment.stepIndex}`}
                  value={direction.signatureMoment.description}
                />
              </dl>
            </article>
          );
        })}
      </div>

      <div className="mt-5">
        <h4 className="text-sm font-semibold">本局关系推进</h4>
        {script.relationships.length > 0 ? (
          <div className="mt-2 space-y-2">
            {script.relationships.map((relationship) => {
              const [leftId, rightId] = relationship.playerIds;
              const left = playersById.get(leftId);
              const right = playersById.get(rightId);
              return (
                <article
                  key={[...relationship.playerIds].sort().join(":")}
                  className="rounded border border-border bg-background/40 p-3"
                >
                  <div className="flex flex-wrap items-center gap-2 text-xs">
                    <span className="font-medium text-foreground">
                      {playerLabel(left, leftId)} × {playerLabel(right, rightId)}
                    </span>
                    <span className="rounded-full bg-surface px-2 py-0.5 text-subtle">
                      {relationshipKindLabel(relationship.kind)}
                    </span>
                  </div>
                  <dl className="mt-2 grid gap-2 text-xs leading-5 text-muted sm:grid-cols-3">
                    <DirectionRow label="铺垫" value={relationship.setup} />
                    <DirectionRow label="发展" value={relationship.development} />
                    <DirectionRow label="兑现" value={relationship.payoff} />
                  </dl>
                </article>
              );
            })}
          </div>
        ) : (
          <p className="mt-2 text-xs text-muted">本候选没有单独设置关系推进线。</p>
        )}
      </div>
    </section>
  );
}

function DirectionRow({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div>
      <dt className="font-medium text-subtle">{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function playerLabel(
  player: PlayerSnapshot | undefined,
  fallback: string,
): string {
  return player
    ? `${player.seatNo} 号 ${player.actor.identity.name}`
    : fallback;
}

function relationshipKindLabel(
  kind: EpisodeScriptSnapshot["relationships"][number]["kind"],
): string {
  switch (kind) {
    case "rivalry":
      return "竞争";
    case "alliance":
      return "联盟";
    case "contrast":
      return "反差";
    case "trust_shift":
      return "信任变化";
  }
}
