import { Img } from "remotion";
import { caseBoardContentForFrame } from "@/components/preview/pixi/case-board-content";
import { easeOutCubic } from "@/components/preview/shot-engine/animation";
import type {
  RenderablePlayer,
  ShotFrame,
} from "@/components/preview/shot-engine/types";
import type { HtmlFrameViewModel } from "../composition/frame-view-model";

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
  const leftPlayers = shot.players.filter((player) => player.seatNo <= 6);
  const rightPlayers = shot.players.filter((player) => player.seatNo > 6);

  return (
    <main
      className="relative h-[1080px] w-[1920px] overflow-hidden bg-[#080a09] text-[#eee8dc]"
      style={{
        fontFamily: '"Kiva Noto Sans SC", sans-serif',
        opacity: 0.16 + Math.min(enter, exit) * 0.84,
      }}
    >
      <StageBackground backgroundUrl={backgroundUrl} />
      <div className="absolute inset-0 grid grid-cols-[430px_1fr_430px] grid-rows-[104px_1fr_176px] gap-x-6 px-10 pb-8 pt-7">
        <StageHeader shot={shot} />
        <SeatTrack
          avatarUrls={assets.avatarUrls}
          players={leftPlayers}
          side="left"
        />
        <section
          className="relative min-h-0 overflow-hidden border border-[#9d8052]/35 bg-[#0d100f]/78 shadow-[0_26px_80px_rgba(0,0,0,0.48)]"
          style={{ transform: "translateY(" + (1 - enter) * 22 + "px)" }}
        >
          <div className="pointer-events-none absolute inset-3 border border-[#d6b171]/12" />
          {shot.scene.kind === "speech" ? (
            <SpeechScene
              avatarUrls={assets.avatarUrls}
              player={shot.activePlayer}
              title={shot.scene.title}
            />
          ) : (
            <CaseBoardScene shot={shot} />
          )}
        </section>
        <SeatTrack
          avatarUrls={assets.avatarUrls}
          players={rightPlayers}
          side="right"
        />
        <SubtitleBand shot={shot} />
      </div>
      <div className="pointer-events-none absolute inset-0 opacity-[0.04] [background-image:repeating-linear-gradient(0deg,transparent,transparent_3px,#d6b171_4px)]" />
    </main>
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
          className="h-full w-full object-cover opacity-70 saturate-[0.72]"
          src={backgroundUrl}
        />
      ) : (
        <div className="h-full w-full bg-[radial-gradient(circle_at_50%_20%,#273229,#0b0e0c_62%,#050606)]" />
      )}
      <div className="absolute inset-0 bg-[linear-gradient(180deg,rgba(4,7,5,0.18),rgba(5,7,6,0.58))]" />
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_50%_42%,transparent_0%,rgba(2,4,3,0.08)_58%,rgba(1,2,2,0.5)_100%)]" />
    </div>
  );
}

function StageHeader({ shot }: { readonly shot: ShotFrame }) {
  return (
    <header className="col-span-3 grid grid-cols-[1fr_auto_1fr] items-center border-y border-[#8f7650]/35 bg-[#090c0a]/74 px-7">
      <div>
        <div className="text-[13px] font-black uppercase tracking-[0.34em] text-[#9f895f]">
          Kiva / Confidential Playback
        </div>
        <h1 className="mt-1 truncate text-[27px] font-black tracking-[0.08em] text-[#f0e7d4]">
          {shot.gameTitle || "未命名对局"}
        </h1>
      </div>
      <div className="border-x border-[#8f7650]/30 px-14 text-center">
        <div className="text-[12px] font-black uppercase tracking-[0.4em] text-[#a99570]">
          Current Record
        </div>
        <div className="mt-1 max-w-[760px] truncate text-[30px] font-black tracking-[0.04em]">
          {shot.scene.title}
        </div>
      </div>
      <div className="justify-self-end text-right">
        <div className="font-mono text-[14px] tracking-[0.22em] text-[#a99570]">
          FILE {String(shot.scene.index).padStart(3, "0")}
        </div>
        <div className="mt-1 text-[20px] font-black uppercase tracking-[0.24em] text-[#d2c3a5]">
          {shot.scene.phase}
        </div>
      </div>
    </header>
  );
}

function SeatTrack({
  avatarUrls,
  players,
  side,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly players: readonly RenderablePlayer[];
  readonly side: "left" | "right";
}) {
  return (
    <aside className="grid min-h-0 grid-rows-6 gap-3 py-4">
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
  player,
  side,
}: {
  readonly avatarUrl: string | null;
  readonly player: RenderablePlayer | null;
  readonly side: "left" | "right";
}) {
  if (!player) {
    return <div className="border border-[#78684c]/10 bg-[#0a0d0b]/25" />;
  }

  const dead = player.status === "dead";
  const active = player.emphasis === "active";
  const highlighted = player.emphasis === "highlighted";
  const textAlign = side === "left" ? "text-left" : "text-right";

  return (
    <article
      className={[
        "relative grid min-h-0 grid-cols-[88px_1fr_68px] items-center gap-3 overflow-hidden border px-3 py-2",
        active
          ? "border-[#d9b36c]/80 bg-[#201a11]/90 shadow-[0_0_34px_rgba(205,164,89,0.2)]"
          : highlighted
            ? "border-[#a99062]/55 bg-[#141510]/82"
            : "border-[#7d6b4b]/24 bg-[#0b0e0c]/72",
        dead ? "grayscale-[0.78]" : "",
      ].join(" ")}
    >
      <div className={side === "right" ? "order-3" : ""}>
        <Avatar avatarUrl={avatarUrl} name={player.name} />
      </div>
      <div className={["min-w-0", textAlign].join(" ")}>
        <div
          className={[
            "line-clamp-2 font-black leading-[1.04] text-[#eee4d1]",
            nameSizeClass(player.name),
            dead ? "line-through decoration-[#9e4c44]/60" : "",
          ].join(" ")}
        >
          {player.name}
        </div>
        <div className="mt-2 truncate text-[17px] font-bold tracking-[0.12em] text-[#a89b83]">
          {player.roleName}
        </div>
      </div>
      <div className={side === "right" ? "order-1 text-left" : "text-right"}>
        <div className="font-mono text-[35px] font-black leading-none text-[#b59b69]">
          {String(player.seatNo).padStart(2, "0")}
        </div>
        <div className="mt-2 text-[10px] font-black uppercase tracking-[0.2em] text-[#7f755f]">
          {dead ? "OUT" : active ? "LIVE" : "SEAT"}
        </div>
      </div>
      {dead ? (
        <div className="absolute right-2 top-2 border border-[#a9574d]/60 bg-[#230e0c]/80 px-2 py-1 text-[10px] font-black tracking-[0.2em] text-[#d98a7f]">
          已出局
        </div>
      ) : null}
    </article>
  );
}

function Avatar({
  avatarUrl,
  name,
}: {
  readonly avatarUrl: string | null;
  readonly name: string;
}) {
  return (
    <div className="grid h-[78px] w-[78px] place-items-center overflow-hidden border border-[#aa9165]/42 bg-[#171b17]">
      {avatarUrl ? (
        <Img className="h-full w-full object-cover" src={avatarUrl} />
      ) : (
        <span className="text-[30px] font-black text-[#c9b48c]">
          {Array.from(name.trim())[0] ?? "?"}
        </span>
      )}
    </div>
  );
}

function SpeechScene({
  avatarUrls,
  player,
  title,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly player: RenderablePlayer | null;
  readonly title: string;
}) {
  const avatarUrl =
    player?.avatar ? avatarUrls[player.avatar] ?? player.avatar : null;
  return (
    <div className="relative grid h-full grid-rows-[1fr_auto]">
      <div className="relative min-h-0 overflow-hidden">
        {avatarUrl ? (
          <Img
            className="h-full w-full object-cover object-top opacity-85 saturate-[0.72]"
            src={avatarUrl}
          />
        ) : (
          <div className="grid h-full place-items-center bg-[radial-gradient(circle,#283027,#0c0f0d_68%)] text-[240px] font-black text-[#8f7c59]/35">
            {player ? Array.from(player.name)[0] : "?"}
          </div>
        )}
        <div className="absolute inset-0 bg-[linear-gradient(180deg,transparent_45%,rgba(5,7,6,0.96)_100%)]" />
        <div className="absolute left-8 top-8 border-l-4 border-[#c19b5d] pl-4">
          <div className="text-[14px] font-black uppercase tracking-[0.34em] text-[#b49c70]">
            Witness Statement
          </div>
          <div className="mt-1 text-[28px] font-black tracking-[0.08em]">
            {title}
          </div>
        </div>
      </div>
      <div className="relative grid grid-cols-[1fr_auto] items-end border-t border-[#93784e]/30 bg-[#090c0a]/96 px-10 py-7">
        <div>
          <div className="text-[15px] font-black uppercase tracking-[0.36em] text-[#9f8b66]">
            Active Speaker
          </div>
          <div className="mt-2 text-[58px] font-black leading-none tracking-[0.04em]">
            {player?.name ?? "等待发言"}
          </div>
        </div>
        <div className="text-right">
          <div className="font-mono text-[74px] font-black leading-none text-[#b9985e]">
            {player ? String(player.seatNo).padStart(2, "0") : "--"}
          </div>
          <div className="mt-1 text-[17px] font-bold tracking-[0.16em] text-[#a99d87]">
            {player?.roleName ?? "NO SUBJECT"}
          </div>
        </div>
      </div>
    </div>
  );
}

function CaseBoardScene({ shot }: { readonly shot: ShotFrame }) {
  const content = caseBoardContentForFrame(shot);
  const fixedRowCount = content.metrics.length + (content.relation ? 1 : 0);
  const rowBudget = Math.max(0, 6 - fixedRowCount);
  const hasOverflow = content.rows.length > rowBudget;
  const visibleRowCount = hasOverflow ? Math.max(0, rowBudget - 1) : rowBudget;
  const rows = content.rows.slice(0, visibleRowCount);
  const hiddenRowCount = content.rows.length - rows.length;
  const accent =
    content.tone === "danger"
      ? "#b96559"
      : content.tone === "good"
        ? "#6f9f85"
        : content.tone === "night"
          ? "#6d7fa5"
          : "#b69760";

  return (
    <div className="grid h-full grid-cols-[1.05fr_0.95fr]">
      <div className="flex min-w-0 flex-col justify-between border-r border-[#91774f]/25 p-11">
        <div>
          <div
            className="text-[15px] font-black uppercase tracking-[0.42em]"
            style={{ color: accent }}
          >
            {content.kindLabel} / Evidence Board
          </div>
          <h2 className="mt-6 line-clamp-2 text-[58px] font-black leading-[1.08] tracking-[0.02em] text-[#f0e9dc]">
            {content.title}
          </h2>
          <div className="mt-7 h-px w-28" style={{ backgroundColor: accent }} />
          <div
            className="mt-8 line-clamp-3 text-[82px] font-black leading-[1.06] tracking-[-0.02em]"
            style={{ color: accent }}
          >
            {content.hero}
          </div>
        </div>
        <p className="line-clamp-4 max-w-[640px] text-[27px] font-semibold leading-[1.55] text-[#bdb4a2]">
          {content.body || shot.scene.text || "案卷内容已归档，等待下一项记录。"}
        </p>
      </div>
      <div className="flex min-w-0 flex-col p-9">
        <div className="flex items-center justify-between border-b border-[#92784f]/25 pb-5">
          <span className="text-[13px] font-black uppercase tracking-[0.35em] text-[#9f8b67]">
            Attached Records
          </span>
          <span className="font-mono text-[13px] tracking-[0.18em] text-[#786f5f]">
            {String(shot.scene.index).padStart(3, "0")} / {content.variant}
          </span>
        </div>
        <div className="mt-6 grid flex-1 auto-rows-fr gap-3">
          {content.metrics.map((metric) => (
            <RecordRow
              key={[metric.label, metric.value].join(":")}
              label={metric.label}
              meta={metric.caption}
              text={metric.value}
            />
          ))}
          {content.relation ? (
            <RecordRow
              label={content.relation.actor?.label ?? "ACTION"}
              meta={content.relation.result}
              text={[
                content.relation.actor?.name,
                content.relation.action,
                content.relation.target?.name,
              ]
                .filter(Boolean)
                .join(" ")}
            />
          ) : null}
          {rows.map((row) => (
            <RecordRow
              key={[row.label, row.text].join(":")}
              label={row.label}
              meta={row.meta}
              text={row.text}
            />
          ))}
          {hiddenRowCount > 0 && rowBudget > 0 ? (
            <RecordRow
              label="MORE"
              text={"+" + hiddenRowCount + " 条附加记录"}
            />
          ) : null}
          {content.metrics.length === 0 &&
          !content.relation &&
          rows.length === 0 ? (
            <RecordRow label="STATUS" text="PUBLIC RECORD / 已公开记录" />
          ) : null}
        </div>
      </div>
    </div>
  );
}

function RecordRow({
  label,
  meta,
  text,
}: {
  readonly label: string;
  readonly meta?: string;
  readonly text: string;
}) {
  return (
    <div className="grid min-h-0 grid-cols-[96px_1fr] items-center border border-[#7d6c4e]/23 bg-[#111410]/78 px-5 py-3">
      <div className="font-mono text-[15px] font-black tracking-[0.14em] text-[#a88d5d]">
        {label}
      </div>
      <div className="min-w-0">
        <div className="line-clamp-2 text-[24px] font-bold leading-[1.25] text-[#ddd4c4]">
          {text}
        </div>
        {meta ? (
          <div className="mt-1 truncate text-[13px] tracking-[0.12em] text-[#827967]">
            {meta}
          </div>
        ) : null}
      </div>
    </div>
  );
}

function SubtitleBand({ shot }: { readonly shot: ShotFrame }) {
  const subtitle = shot.subtitle;
  return (
    <footer className="col-span-3 grid grid-cols-[250px_1fr_180px] items-center border-y border-[#8f7650]/36 bg-[#080b09]/92 px-7">
      <div>
        <div className="text-[12px] font-black uppercase tracking-[0.35em] text-[#8f7d5d]">
          Transcript
        </div>
        <div className="mt-2 truncate text-[20px] font-black tracking-[0.06em] text-[#d3c4a7]">
          {subtitle?.speaker ?? shot.scene.title}
        </div>
      </div>
      <div className="border-x border-[#8f7650]/25 px-10">
        <div className="line-clamp-3 whitespace-pre-line text-center text-[31px] font-bold leading-[1.38] tracking-[0.03em] text-[#f0e9dc]">
          {subtitle?.lines.join("\n") || shot.scene.text || shot.scene.title}
        </div>
      </div>
      <div className="text-right font-mono text-[14px] tracking-[0.15em] text-[#82745c]">
        {subtitle
          ? String(subtitle.windowIndex + 1) + " / " + subtitle.windowCount
          : formatClock(shot.clock.sceneMs)}
      </div>
    </footer>
  );
}

function nameSizeClass(name: string): string {
  const length = Array.from(name).length;
  if (length <= 4) {
    return "text-[28px]";
  }
  if (length <= 7) {
    return "text-[23px]";
  }
  return "text-[19px]";
}

function formatClock(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  return Math.floor(seconds / 60) + ":" + String(seconds % 60).padStart(2, "0");
}
