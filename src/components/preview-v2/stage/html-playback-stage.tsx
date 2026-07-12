import { Img } from "remotion";
import { easeOutCubic } from "@/components/preview/shot-engine/animation";
import { subtitleWindows } from "@/components/preview/shot-engine/subtitles";
import type {
  RenderablePlayer,
  ShotFrame,
} from "@/components/preview/shot-engine/types";
import type { HtmlFrameViewModel } from "../composition/frame-view-model";
import {
  identityTextStyle,
  presenterIdentityTone,
  roleIdentityTone,
} from "./identity-palette";
import {
  stagePaletteForPhase,
  type StagePalette,
  type StageVisualPhase,
} from "./stage-palette";
import {
  transcriptPresentationForScene,
  transcriptSpeakerIdentity,
  transcriptTextSegments,
  type TranscriptSpeakerIdentity,
} from "./stage-copy";
import { StageEventVisual } from "./stage-event-visual";

export function HtmlPlaybackStage({
  viewModel,
}: {
  readonly viewModel: HtmlFrameViewModel | null;
}) {
  if (!viewModel) {
    return <EmptyStage />;
  }

  const { shot, assets } = viewModel;
  const backgroundUrl =
    shot.scene.phase === "night"
      ? assets.nightBackgroundUrl
      : assets.dayBackgroundUrl;
  const enter = easeOutCubic(shot.clock.enterProgress);
  const exit = easeOutCubic(shot.clock.exitProgress);
  const opacity = stageOpacity(shot, enter, exit);
  const visualPhase: StageVisualPhase =
    shot.scene.phase === "night" ? "night" : "day";
  const palette = stagePaletteForPhase(visualPhase);
  const leftPlayers = shot.players.filter((player) => player.seatNo <= 6);
  const rightPlayers = shot.players.filter((player) => player.seatNo > 6);

  return (
    <main
      className="relative h-[1080px] w-[1920px] overflow-hidden bg-[#080a09] text-[#eee8dc]"
      style={{
        fontFamily: '"Kiva Noto Sans SC", sans-serif',
        opacity,
        color: palette.text,
      }}
    >
      <StageBackground backgroundUrl={backgroundUrl} />
      <div className="absolute inset-0 grid grid-cols-[480px_minmax(0,1fr)_480px] grid-rows-[112px_minmax(0,1fr)_212px] gap-x-6 gap-y-4 px-10 pb-8 pt-7">
        <StageHeader palette={palette} shot={shot} />
        <SeatTrack
          avatarUrls={assets.avatarUrls}
          palette={palette}
          phase={visualPhase}
          players={leftPlayers}
          side="left"
        />
        <StageEventVisual
          avatarUrls={assets.avatarUrls}
          palette={palette}
          phase={visualPhase}
          shot={shot}
        />
        <SeatTrack
          avatarUrls={assets.avatarUrls}
          palette={palette}
          phase={visualPhase}
          players={rightPlayers}
          side="right"
        />
        <SubtitleBand
          avatarUrls={assets.avatarUrls}
          palette={palette}
          phase={visualPhase}
          shot={shot}
        />
      </div>
    </main>
  );
}

export function stageOpacity(
  shot: Pick<ShotFrame, "scene" | "items">,
  enter: number,
  exit: number,
): number {
  const sceneIndex = shot.items.indexOf(shot.scene);
  const previousScene = shot.items[sceneIndex - 1];
  const nextScene = shot.items[sceneIndex + 1];
  const enterOpacity = switchesDayNight(previousScene, shot.scene) ? enter : 1;
  const exitOpacity = switchesDayNight(shot.scene, nextScene) ? exit : 1;

  return 0.16 + Math.min(enterOpacity, exitOpacity) * 0.84;
}

function switchesDayNight(
  from: ShotFrame["scene"] | undefined,
  to: ShotFrame["scene"] | undefined,
): boolean {
  return Boolean(
    from && to && (from.phase === "night") !== (to.phase === "night"),
  );
}

function EmptyStage() {
  return (
    <div className="grid h-[1080px] w-[1920px] place-items-center bg-[#080a09] text-[#d8d0be]">
      <div className="border border-[#8e7450]/40 px-12 py-8 text-3xl tracking-[0.28em]">
        等待案卷记录
      </div>
    </div>
  );
}

function StageBackground({
  backgroundUrl,
}: {
  readonly backgroundUrl: string | null;
}) {
  return (
    <div className="absolute inset-0">
      {backgroundUrl ? (
        <Img
          className={STAGE_BACKGROUND_IMAGE_CLASS_NAME}
          src={backgroundUrl}
        />
      ) : (
        <div className="h-full w-full bg-[radial-gradient(circle_at_50%_20%,#273229,#0b0e0c_62%,#050606)]" />
      )}
    </div>
  );
}

export const STAGE_BACKGROUND_IMAGE_CLASS_NAME =
  "h-full w-full object-cover";

function StageHeader({
  palette,
  shot,
}: {
  readonly palette: StagePalette;
  readonly shot: ShotFrame;
}) {
  return (
    <header
      className="relative col-start-2 row-start-1 flex min-w-0 items-center justify-center overflow-hidden border px-12 py-3"
      style={{
        background: palette.majorSurface,
        borderColor: palette.majorBorder,
        boxShadow: palette.majorShadow,
      }}
    >
      <div className="flex w-full min-w-0 items-center gap-7">
        <HeaderRule palette={palette} side="left" />
        <div className="min-w-0 shrink-0 text-center">
          <h1
            className="max-w-[760px] truncate text-[38px] font-black leading-[1.15] tracking-[0.08em]"
            style={{ color: palette.text, textShadow: palette.titleShadow }}
          >
            {stageHeaderTitle(shot.gameTitle)}
          </h1>
          <p
            className="mt-1 max-w-[760px] truncate text-[20px] font-medium leading-none tracking-[0.16em]"
            style={{ color: palette.accent, opacity: 0.82 }}
          >
            {shot.scene.title}
          </p>
        </div>
        <HeaderRule palette={palette} side="right" />
      </div>
    </header>
  );
}

function HeaderRule({
  palette,
  side,
}: {
  readonly palette: StagePalette;
  readonly side: "left" | "right";
}) {
  const line = (
    <div
      className="h-px min-w-0 flex-1"
      style={{
        background: `linear-gradient(${side === "left" ? "90deg" : "270deg"}, transparent, ${palette.rule})`,
      }}
    />
  );
  const diamond = (
    <div
      className="h-2.5 w-2.5 shrink-0 rotate-45 border"
      style={{ borderColor: palette.accent }}
    />
  );

  return (
    <div aria-hidden="true" className="flex min-w-0 flex-1 items-center gap-3">
      {side === "left" ? line : diamond}
      {side === "left" ? diamond : line}
    </div>
  );
}

export function stageHeaderTitle(gameTitle: string): string {
  return gameTitle || "未命名游戏";
}

function SeatTrack({
  avatarUrls,
  palette,
  phase,
  players,
  side,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly palette: StagePalette;
  readonly phase: StageVisualPhase;
  readonly players: readonly RenderablePlayer[];
  readonly side: "left" | "right";
}) {
  return (
    <aside
      className={[
        "row-span-3 grid min-h-0 content-between grid-rows-[repeat(6,148px)] gap-3",
        side === "left"
          ? "col-start-1 row-start-1"
          : "col-start-3 row-start-1",
      ].join(" ")}
    >
      {Array.from({ length: 6 }, (_, index) => {
        const seatNo = index + (side === "left" ? 1 : 7);
        const player =
          players.find((candidate) => candidate.seatNo === seatNo) ?? null;
        const avatarUrl =
          player?.avatar
            ? avatarUrls[player.avatar] ?? player.avatar
            : null;
        return (
          <SeatCard
            avatarUrl={avatarUrl}
            key={player?.playerId ?? side + "-empty-" + index}
            palette={palette}
            phase={phase}
            player={player}
            side={side}
          />
        );
      })}
    </aside>
  );
}

function SeatCard({
  avatarUrl,
  palette,
  phase,
  player,
  side,
}: {
  readonly avatarUrl: string | null;
  readonly palette: StagePalette;
  readonly phase: StageVisualPhase;
  readonly player: RenderablePlayer | null;
  readonly side: "left" | "right";
}) {
  if (!player) {
    return <div />;
  }

  const dead = player.status === "dead";
  const active = player.emphasis === "active";
  const highlighted = player.emphasis === "highlighted";
  const layout = seatCardLayout(side);
  const surface = dead
    ? phase === "night"
      ? "rgba(4,7,10,0.40)"
      : "rgba(10,12,10,0.46)"
    : active
      ? palette.activeCard
      : palette.cardSurface;

  return (
    <article
      className={[
        "relative grid min-h-0 items-center gap-4 overflow-hidden border px-3 py-2.5",
        layout.grid,
      ].join(" ")}
      style={{
        background: surface,
        borderColor: dead
          ? palette.ruleSoft
          : active
            ? palette.activeBorder
            : highlighted
              ? palette.rule
              : palette.cardBorder,
        boxShadow: active ? palette.activeShadow : palette.cardShadow,
      }}
    >
      <div className={["relative z-10", layout.avatar].join(" ")}>
        <Avatar
          avatarUrl={avatarUrl}
          borderColor={palette.avatarBorder}
          muted={dead}
          name={player.name}
        />
      </div>
      <div
        className={["relative z-10 col-start-2 min-w-0", layout.text].join(" ")}
      >
        <div
          className={[
            "line-clamp-2 font-black leading-[1.08]",
            nameSizeClass(player.name),
            dead
              ? "text-[#77736c] line-through decoration-[#76665d]/45"
              : "",
          ].join(" ")}
          style={{
            color: dead ? "#77736C" : palette.text,
            textShadow: dead ? "none" : palette.textShadow,
          }}
        >
          {player.name}
        </div>
        <div
          className="mt-2 truncate text-[27px] font-black leading-[1.15] tracking-[0.04em]"
          style={{
            ...identityTextStyle(roleIdentityTone(player.roleName, phase)),
            opacity: dead ? 0.45 : 1,
          }}
        >
          {player.roleName}
        </div>
      </div>
      <div className={["relative z-10 self-center", layout.seat].join(" ")}>
        <div
          className="font-mono text-[80px] font-black leading-none tracking-[-0.08em]"
          style={{
            color: dead ? "#6D6557" : active ? palette.active : palette.number,
            opacity: dead ? 0.45 : 1,
            textShadow: dead ? "none" : palette.numberShadow,
          }}
        >
          {String(player.seatNo).padStart(2, "0")}
        </div>
      </div>
      {active && !dead ? (
        <div
          aria-hidden="true"
          className={`pointer-events-none absolute inset-y-0 w-[180px] ${side === "left" ? "right-0" : "left-0"}`}
          style={{
            background: `linear-gradient(${side === "left" ? "270deg" : "90deg"}, ${palette.activeField}, transparent)`,
          }}
        />
      ) : null}
      <div
        aria-hidden="true"
        className={[
          active && !dead
            ? "absolute inset-y-2 w-[9px]"
            : "absolute inset-y-4 w-[4px]",
          layout.accent,
        ].join(" ")}
        style={{
          background: dead ? palette.ruleSoft : active ? palette.active : palette.edge,
          boxShadow: dead
            ? "none"
            : active
              ? palette.activeEdgeShadow
              : palette.edgeShadow,
          opacity: dead ? 0.35 : highlighted ? 0.9 : 1,
        }}
      />
    </article>
  );
}

export function seatCardLayout(side: "left" | "right") {
  return side === "left"
    ? {
        grid: "grid-cols-[104px_minmax(0,1fr)_112px]",
        avatar: "col-start-3 row-start-1",
        text: "row-start-1 text-center",
        seat: "col-start-1 row-start-1 flex h-[112px] items-center justify-start",
        accent: "right-0",
      }
    : {
        grid: "grid-cols-[112px_minmax(0,1fr)_104px]",
        avatar: "col-start-1 row-start-1",
        text: "row-start-1 text-center",
        seat: "col-start-3 row-start-1 flex h-[112px] items-center justify-end",
        accent: "left-0",
      };
}

function Avatar({
  avatarUrl,
  borderColor,
  muted,
  name,
}: {
  readonly avatarUrl: string | null;
  readonly borderColor: string;
  readonly muted: boolean;
  readonly name: string;
}) {
  return (
    <div
      className={[
        "grid h-[112px] w-[112px] place-items-center overflow-hidden border bg-[#171b17] shadow-[inset_0_0_0_3px_rgba(7,9,7,0.7)]",
        muted ? "grayscale-[0.92] opacity-35" : "",
      ].join(" ")}
      style={{ borderColor }}
    >
      {avatarUrl ? (
        <Img className="h-full w-full object-cover" src={avatarUrl} />
      ) : (
        <span className="text-[46px] font-black text-[#c9b48c]">
          {Array.from(name.trim())[0] ?? "?"}
        </span>
      )}
    </div>
  );
}

function SubtitleBand({
  avatarUrls,
  palette,
  phase,
  shot,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly palette: StagePalette;
  readonly phase: StageVisualPhase;
  readonly shot: ShotFrame;
}) {
  const playerVoiceStarted =
    shot.scene.playerVoice !== null &&
    shot.scene.playerVoice !== undefined &&
    shot.clock.sceneMs >= shot.scene.playerVoice.startsAtOffsetMs;
  const presentationScene =
    shot.scene.playerVoice && !playerVoiceStarted
      ? {
          ...shot.scene,
          transcriptSpeaker: "presenter" as const,
          text: shot.scene.presenterCue.text,
        }
      : shot.scene;
  const presentation = transcriptPresentationForScene(
    presentationScene,
    shot.activePlayer,
  );
  const lines = shot.scene.playerVoice && playerVoiceStarted
    ? shot.subtitle?.lines ?? []
    : (() => {
        const windows = subtitleWindows(presentation.content, 22);
        const windowIndex = Math.min(
          Math.max(0, windows.length - 1),
          Math.floor(shot.clock.progress * windows.length),
        );
        return windows[windowIndex] ?? [];
      })();
  const avatarUrl = presentation.speaker.avatar
    ? avatarUrls[presentation.speaker.avatar] ?? presentation.speaker.avatar
    : null;
  const speakerName = presentation.speaker.seatNo
    ? `${String(presentation.speaker.seatNo).padStart(2, "0")}号 ${presentation.speaker.name}`
    : presentation.speaker.name;
  const identity = transcriptSpeakerIdentity(presentation);
  const identityTone =
    identity.kind === "presenter"
      ? presenterIdentityTone(phase)
      : roleIdentityTone(identity.label, phase);
  const transcriptSegments = transcriptTextSegments(
    lines.join("\n"),
    presentation.speaker.kind,
    shot.scene.players,
  );

  return (
    <footer
      className="relative col-start-2 row-start-3 grid min-w-0 grid-cols-[148px_minmax(0,1fr)] items-center gap-6 overflow-hidden border px-5 py-4"
      style={{
        background: palette.majorSurface,
        borderColor: palette.majorBorder,
        boxShadow: palette.majorShadow,
      }}
    >
      <div
        className="grid h-[148px] w-[148px] place-items-center overflow-hidden border bg-[#151914]"
        style={{
          borderColor: identityTone.foreground,
          boxShadow: `inset 0 0 0 3px rgba(6,8,6,0.7), 0 0 24px ${identityTone.glow}`,
        }}
      >
        {avatarUrl ? (
          <Img className="h-full w-full object-cover" src={avatarUrl} />
        ) : (
          <span
            className="text-[48px] font-black"
            style={identityTextStyle(identityTone)}
          >
            {Array.from(presentation.speaker.name.trim())[0] ?? "?"}
          </span>
        )}
      </div>
      <div className="grid min-w-0 self-stretch grid-rows-[auto_minmax(0,1fr)] content-center py-0.5">
        <div
          aria-label="说话者信息"
          className="flex min-w-0 items-center gap-3"
        >
          <div
            className="truncate text-[29px] font-black tracking-[0.07em]"
            style={{ color: palette.text, textShadow: palette.textShadow }}
          >
            {speakerName}
          </div>
          <div
            className="h-5 w-px shrink-0"
            style={{ background: palette.rule }}
          />
          <IdentityBadge identity={identity} phase={phase} />
          <div
            className="h-px min-w-8 flex-1"
            style={{
              background: `linear-gradient(90deg, ${palette.rule}, transparent)`,
            }}
          />
        </div>
        <div
          className="mt-2 line-clamp-3 whitespace-pre-line border-t pt-2 text-[29px] font-bold leading-[1.18] tracking-[0.01em]"
          style={{
            borderColor: palette.ruleSoft,
            color: palette.text,
            textShadow: palette.textShadow,
          }}
        >
          {transcriptSegments.map((segment, index) => (
            <span
              className={segment.roleName ? "font-black" : undefined}
              key={`${index}:${segment.text}`}
              style={
                segment.roleName
                  ? identityTextStyle(roleIdentityTone(segment.roleName, phase))
                  : undefined
              }
            >
              {segment.text}
            </span>
          ))}
        </div>
      </div>
    </footer>
  );
}

function IdentityBadge({
  identity,
  phase,
}: {
  readonly identity: TranscriptSpeakerIdentity;
  readonly phase: StageVisualPhase;
}) {
  const tone =
    identity.kind === "presenter"
      ? presenterIdentityTone(phase)
      : roleIdentityTone(identity.label, phase);

  return (
    <span className="inline-flex min-w-0 items-center gap-2">
      <span
        aria-hidden="true"
        className="h-2.5 w-2.5 shrink-0 rotate-45"
        style={{
          backgroundColor: tone.foreground,
          boxShadow: `0 0 12px ${tone.glow}`,
        }}
      />
      <span
        className="truncate text-[21px] font-black tracking-[0.05em]"
        style={identityTextStyle(tone)}
      >
        {identity.label}
      </span>
    </span>
  );
}

function nameSizeClass(name: string): string {
  const length = Array.from(name).length;
  if (length <= 4) {
    return "text-[39px]";
  }
  if (length <= 7) {
    return "text-[33px]";
  }
  return "text-[28px]";
}
