import {
  Cross,
  Eye,
  FlaskConical,
  Shield,
  Swords,
  Trophy,
} from "lucide-react";
import { Img } from "remotion";
import type {
  RenderablePlayer,
  ShotFrame,
} from "@/components/preview/shot-engine/types";
import type {
  StageAction,
  StageActionResult,
  StagePresentation,
} from "@/core/playback";
import type { StagePalette, StageVisualPhase } from "./stage-palette";

type StageEventVisualProps = {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly palette: StagePalette;
  readonly phase: StageVisualPhase;
  readonly shot: ShotFrame;
};

export function StageEventVisual({
  avatarUrls,
  palette,
  phase,
  shot,
}: StageEventVisualProps) {
  const stage = shot.scene.stage;
  if (!stage) {
    return <section aria-label="视觉舞台" className="col-start-2 row-start-2 min-h-0" />;
  }

  const enter = progressBetween(shot.clock.sceneMs, 0, 350);
  const exit = progressBetween(
    shot.clock.sceneMs,
    Math.max(0, shot.clock.durationMs - 350),
    shot.clock.durationMs,
  );
  const lineProgress = progressBetween(shot.clock.sceneMs, 250, 750);
  const resultProgress = progressBetween(shot.clock.sceneMs, 650, 1050);

  return (
    <section
      aria-label="视觉舞台"
      className="col-start-2 row-start-2 flex min-h-0 items-center justify-center"
      style={{
        opacity: enter * (1 - exit * 0.22),
        translate: `0 ${(1 - enter) * 18}px`,
      }}
    >
      <div
        className="flex w-full max-w-[840px] flex-col items-center justify-center overflow-hidden border px-8 py-7"
        style={{
          background: palette.majorSurface,
          borderColor: palette.majorBorder,
          boxShadow: palette.majorShadow,
        }}
      >
        <div
          className="mb-7 text-[24px] font-black tracking-[0.22em]"
          style={{ color: palette.accent, textShadow: palette.textShadow }}
        >
          {stageTitle(stage)}
        </div>
        {stage.kind === "action" ? (
          <ActionVisual
            avatarUrls={avatarUrls}
            lineProgress={lineProgress}
            palette={palette}
            phase={phase}
            players={shot.players}
            resultProgress={resultProgress}
            stage={stage}
          />
        ) : stage.kind === "night_result" ? (
          <NightResult
            avatarUrls={avatarUrls}
            palette={palette}
            players={shot.players}
            resultProgress={resultProgress}
            stage={stage}
          />
        ) : stage.kind === "vote_result" ? (
          <VoteResult
            avatarUrls={avatarUrls}
            palette={palette}
            players={shot.players}
            resultProgress={resultProgress}
            stage={stage}
          />
        ) : (
          <GameResult
            palette={palette}
            resultProgress={resultProgress}
            stage={stage}
          />
        )}
      </div>
    </section>
  );
}

function ActionVisual({
  avatarUrls,
  lineProgress,
  palette,
  phase,
  players,
  resultProgress,
  stage,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly lineProgress: number;
  readonly palette: StagePalette;
  readonly phase: StageVisualPhase;
  readonly players: readonly RenderablePlayer[];
  readonly resultProgress: number;
  readonly stage: Extract<StagePresentation, { kind: "action" }>;
}) {
  const actorPlayerId = stage.actor.kind === "player" ? stage.actor.playerId : null;
  const actor = actorPlayerId
    ? players.find((player) => player.playerId === actorPlayerId) ?? null
    : null;
  const target = stage.targetPlayerId
    ? players.find((player) => player.playerId === stage.targetPlayerId) ?? null
    : null;
  const action = actionPresentation(stage.action);

  return (
    <div className="flex w-full flex-col items-center">
      <div className="grid w-full grid-cols-[220px_minmax(180px,1fr)_220px] items-center gap-5">
        {stage.actor.kind === "wolves" ? (
          <SyntheticParticipant label="狼队" palette={palette} symbol="狼" />
        ) : (
          <PlayerParticipant
            avatarUrls={avatarUrls}
            fallback="行动者"
            palette={palette}
            player={actor}
          />
        )}
        <div className="flex min-w-0 flex-col items-center gap-4">
          <div
            className="grid h-20 w-20 place-items-center rounded-full border"
            style={{
              borderColor: palette.accent,
              color: action.color(phase),
              scale: 0.9 + lineProgress * 0.1,
              boxShadow: `0 0 ${Math.round(26 * lineProgress)}px ${palette.accentGlow}`,
            }}
          >
            {action.icon}
          </div>
          <div
            className="text-[32px] font-black tracking-[0.12em]"
            style={{ color: palette.text, textShadow: palette.textShadow }}
          >
            {action.label}
          </div>
          <div className="h-1.5 w-full overflow-hidden" style={{ background: palette.ruleSoft }}>
            <div
              className="h-full origin-left"
              style={{ background: palette.accent, scale: `${lineProgress} 1` }}
            />
          </div>
        </div>
        {target ? (
          <PlayerParticipant
            avatarUrls={avatarUrls}
            fallback="目标"
            palette={palette}
            player={target}
          />
        ) : (
          <SyntheticParticipant
            label={stage.result === "skipped" ? "未使用" : "未确定"}
            palette={palette}
            symbol="—"
          />
        )}
      </div>
      <ResultStamp
        label={actionResultLabel(stage.action, stage.result)}
        palette={palette}
        progress={resultProgress}
        tone={resultTone(stage.action, stage.result)}
      />
    </div>
  );
}

function NightResult({
  avatarUrls,
  palette,
  players,
  resultProgress,
  stage,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly palette: StagePalette;
  readonly players: readonly RenderablePlayer[];
  readonly resultProgress: number;
  readonly stage: Extract<StagePresentation, { kind: "night_result" }>;
}) {
  if (stage.deaths.length === 0) {
    return (
      <ResultHero
        icon={<Shield size={82} strokeWidth={1.6} />}
        label="平安夜"
        palette={palette}
        progress={resultProgress}
        tone="good"
      />
    );
  }

  return (
    <div className="flex w-full flex-col items-center gap-7">
      <div className="flex max-w-full flex-wrap justify-center gap-5">
        {stage.deaths.map((death) => {
          const player = players.find((entry) => entry.playerId === death.playerId) ?? null;
          return (
            <div key={death.playerId} className="flex flex-col items-center gap-3">
              <PlayerParticipant
                avatarUrls={avatarUrls}
                fallback="死亡玩家"
                palette={palette}
                player={player}
              />
              {death.reason ? (
                <div className="text-[20px] font-bold text-[#FF8A8D]">
                  {death.reason === "wolf_kill" ? "狼人袭击" : "女巫毒杀"}
                </div>
              ) : null}
            </div>
          );
        })}
      </div>
      <ResultStamp label="确认死亡" palette={palette} progress={resultProgress} tone="danger" />
    </div>
  );
}

function VoteResult({
  avatarUrls,
  palette,
  players,
  resultProgress,
  stage,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly palette: StagePalette;
  readonly players: readonly RenderablePlayer[];
  readonly resultProgress: number;
  readonly stage: Extract<StagePresentation, { kind: "vote_result" }>;
}) {
  const candidates = relevantVoteCandidates(stage);
  return (
    <div className="flex w-full flex-col items-center gap-6">
      {candidates.length > 0 ? (
        <div className="flex max-w-full flex-wrap justify-center gap-5">
          {candidates.map((candidate) => (
            <div key={candidate.playerId} className="flex flex-col items-center gap-3">
              <PlayerParticipant
                avatarUrls={avatarUrls}
                fallback="候选玩家"
                palette={palette}
                player={players.find((entry) => entry.playerId === candidate.playerId) ?? null}
              />
              <div className="font-mono text-[32px] font-black" style={{ color: palette.number }}>
                {candidate.votes} 票
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="grid h-32 w-32 place-items-center rounded-full border text-[64px]" style={{ borderColor: palette.rule, color: palette.rule }}>
          —
        </div>
      )}
      <ResultStamp
        label={stage.outcome === "exiled" ? "放逐出局" : stage.outcome === "tied" ? "平票" : "无人出局"}
        palette={palette}
        progress={resultProgress}
        tone={stage.outcome === "exiled" ? "danger" : "neutral"}
      />
      {stage.abstentions > 0 ? (
        <div className="text-[22px] font-medium" style={{ color: palette.rule }}>
          弃票 {stage.abstentions}
        </div>
      ) : null}
    </div>
  );
}

function GameResult({
  palette,
  resultProgress,
  stage,
}: {
  readonly palette: StagePalette;
  readonly resultProgress: number;
  readonly stage: Extract<StagePresentation, { kind: "game_result" }>;
}) {
  const winner = stage.winner === "good" ? "好人阵营胜利" : "狼人阵营胜利";
  return (
    <div className="flex flex-col items-center gap-7 text-center">
      <Trophy size={112} strokeWidth={1.5} style={{ color: palette.accent }} />
      <div className="text-[54px] font-black tracking-[0.08em]" style={{ color: palette.text, textShadow: palette.titleShadow }}>
        {winner}
      </div>
      <ResultStamp
        label={gameReasonLabel(stage.reason)}
        palette={palette}
        progress={resultProgress}
        tone={stage.winner === "good" ? "good" : "danger"}
      />
    </div>
  );
}

function PlayerParticipant({
  avatarUrls,
  fallback,
  palette,
  player,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly fallback: string;
  readonly palette: StagePalette;
  readonly player: RenderablePlayer | null;
}) {
  const avatarUrl = player?.avatar
    ? avatarUrls[player.avatar] ?? player.avatar
    : null;
  return (
    <div className="flex min-h-[210px] w-[220px] flex-col items-center justify-center border px-4 py-4 text-center" style={{ background: palette.cardSurface, borderColor: palette.cardBorder, boxShadow: palette.cardShadow }}>
      <div className="grid h-28 w-28 place-items-center overflow-hidden rounded-full border text-[38px] font-black" style={{ borderColor: palette.avatarBorder, color: palette.number }}>
        {avatarUrl ? <Img className="h-full w-full object-cover" src={avatarUrl} /> : player ? String(player.seatNo).padStart(2, "0") : "?"}
      </div>
      <div className="mt-4 max-w-full truncate text-[30px] font-black" style={{ color: palette.text, textShadow: palette.textShadow }}>
        {player ? `${player.seatNo}号 ${player.name}` : fallback}
      </div>
      {player ? <div className="mt-1 text-[20px] font-bold" style={{ color: palette.rule }}>{player.roleName}</div> : null}
    </div>
  );
}

function SyntheticParticipant({ label, palette, symbol }: { readonly label: string; readonly palette: StagePalette; readonly symbol: string }) {
  return (
    <div className="flex min-h-[210px] w-[220px] flex-col items-center justify-center border px-4 py-4 text-center" style={{ background: palette.cardSurface, borderColor: palette.cardBorder, boxShadow: palette.cardShadow }}>
      <div className="grid h-28 w-28 place-items-center rounded-full border text-[48px] font-black" style={{ borderColor: palette.avatarBorder, color: palette.number }}>{symbol}</div>
      <div className="mt-4 text-[30px] font-black" style={{ color: palette.text }}>{label}</div>
    </div>
  );
}

function ResultHero({ icon, label, palette, progress, tone }: { readonly icon: React.ReactNode; readonly label: string; readonly palette: StagePalette; readonly progress: number; readonly tone: ResultTone }) {
  const color = toneColor(tone, palette);
  return (
    <div className="flex flex-col items-center gap-6" style={{ opacity: progress, scale: 0.92 + progress * 0.08, color }}>
      {icon}
      <div className="text-[64px] font-black tracking-[0.12em]" style={{ textShadow: palette.titleShadow }}>{label}</div>
    </div>
  );
}

type ResultTone = "good" | "danger" | "neutral";

function ResultStamp({ label, palette, progress, tone }: { readonly label: string; readonly palette: StagePalette; readonly progress: number; readonly tone: ResultTone }) {
  const color = toneColor(tone, palette);
  return (
    <div className="mt-7 border-2 px-8 py-2 text-[30px] font-black tracking-[0.16em]" style={{ borderColor: color, color, opacity: progress, scale: 0.9 + progress * 0.1, boxShadow: `0 0 ${Math.round(22 * progress)}px ${color}40` }}>
      {label}
    </div>
  );
}

function actionPresentation(action: StageAction): { readonly label: string; readonly icon: React.ReactNode; readonly color: (phase: StageVisualPhase) => string } {
  switch (action) {
    case "protect": return { label: "守护", icon: <Shield size={46} />, color: () => "#80C8FB" };
    case "attack": return { label: "袭击", icon: <Swords size={46} />, color: () => "#FF767B" };
    case "inspect": return { label: "查验", icon: <Eye size={46} />, color: () => "#76E2B6" };
    case "antidote": return { label: "解药", icon: <Cross size={46} />, color: () => "#CBA4F4" };
    case "poison": return { label: "毒药", icon: <FlaskConical size={46} />, color: () => "#CBA4F4" };
  }
}

function actionResultLabel(action: StageAction, result: StageActionResult): string {
  if (result === "skipped") return "未使用";
  if (result === "unresolved") return "未确定";
  if (result === "good") return "好人";
  if (result === "wolves") return "狼人";
  if (action === "protect") return "已守护";
  if (action === "antidote") return "已救";
  if (action === "poison") return "已毒";
  if (action === "inspect") return "查验中";
  return "已锁定";
}

function resultTone(action: StageAction, result: StageActionResult): ResultTone {
  if (result === "wolves" || action === "attack" || action === "poison") return "danger";
  if (result === "good" || action === "protect" || action === "antidote") return "good";
  return "neutral";
}

function stageTitle(stage: StagePresentation): string {
  if (stage.kind === "action") return actionPresentation(stage.action).label;
  if (stage.kind === "night_result") return "夜间结算";
  if (stage.kind === "vote_result") return stage.voteType === "pk" ? "PK 结算" : "放逐结算";
  return "对局结算";
}

function relevantVoteCandidates(stage: Extract<StagePresentation, { kind: "vote_result" }>) {
  if (stage.exiledPlayerId) {
    return stage.candidates.filter((candidate) => candidate.playerId === stage.exiledPlayerId);
  }
  const highest = stage.candidates[0]?.votes ?? 0;
  return stage.candidates.filter((candidate) => candidate.votes === highest).slice(0, 4);
}

function gameReasonLabel(reason: Extract<StagePresentation, { kind: "game_result" }>["reason"]): string {
  switch (reason) {
    case "all_wolves_dead": return "所有狼人出局";
    case "all_gods_dead": return "所有神职出局";
    case "all_villagers_dead": return "所有村民出局";
    case "all_good_dead": return "所有好人出局";
  }
}

function toneColor(tone: ResultTone, palette: StagePalette): string {
  if (tone === "good") return "#76E2B6";
  if (tone === "danger") return "#FF767B";
  return palette.accent;
}

function progressBetween(value: number, start: number, end: number): number {
  if (end <= start) return value >= end ? 1 : 0;
  return Math.min(1, Math.max(0, (value - start) / (end - start)));
}
