import { Img } from "remotion";
import { easeOutCubic } from "@/components/preview/shot-engine/animation";
import { subtitleWindows } from "@/components/preview/shot-engine/subtitles";
import type {
  RenderablePlayer,
  ShotFrame,
} from "@/components/preview/shot-engine/types";
import type { HtmlFrameViewModel } from "../composition/frame-view-model";
import { transcriptPresentationForScene } from "./stage-copy";

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
      <div className="absolute inset-0 grid grid-cols-[430px_minmax(0,1fr)_430px] grid-rows-[88px_minmax(0,1fr)_196px] gap-x-6 gap-y-4 px-10 pb-8 pt-7">
        <StageHeader shot={shot} />
        <SeatTrack
          avatarUrls={assets.avatarUrls}
          players={leftPlayers}
          side="left"
        />
        <section
          aria-label="视觉舞台"
          className="relative col-start-2 row-start-2 min-h-0 overflow-hidden border border-[#9d8052]/35 bg-[#0b0e0c]/28 shadow-[0_26px_80px_rgba(0,0,0,0.38)]"
          style={{ translate: "0 " + (1 - enter) * 14 + "px" }}
        />
        <SeatTrack
          avatarUrls={assets.avatarUrls}
          players={rightPlayers}
          side="right"
        />
        <SubtitleBand avatarUrls={assets.avatarUrls} shot={shot} />
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
    <header className="col-start-2 row-start-1 flex min-w-0 items-center border border-[#9d8052]/35 bg-[#090c0a]/78 px-8 shadow-[0_18px_50px_rgba(0,0,0,0.24)]">
      <h1 className="truncate text-[44px] font-black leading-none tracking-[0.04em] text-[#f0e7d4]">
        {shot.scene.title || "等待下一环节"}
      </h1>
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
    <aside
      className={[
        "row-span-3 grid min-h-0 grid-rows-6 gap-3",
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
        "relative grid min-h-0 grid-cols-[116px_1fr_72px] items-center gap-4 overflow-hidden border px-3 py-3",
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
    <div className="grid h-[112px] w-[112px] place-items-center overflow-hidden border border-[#aa9165]/42 bg-[#171b17]">
      {avatarUrl ? (
        <Img className="h-full w-full object-cover" src={avatarUrl} />
      ) : (
        <span className="text-[42px] font-black text-[#c9b48c]">
          {Array.from(name.trim())[0] ?? "?"}
        </span>
      )}
    </div>
  );
}

function SubtitleBand({
  avatarUrls,
  shot,
}: {
  readonly avatarUrls: Readonly<Record<string, string>>;
  readonly shot: ShotFrame;
}) {
  const presentation = transcriptPresentationForScene(
    shot.scene,
    shot.activePlayer,
  );
  const windows = subtitleWindows(presentation.content, 22);
  const windowIndex = Math.min(
    Math.max(0, windows.length - 1),
    Math.floor(shot.clock.progress * windows.length),
  );
  const lines = windows[windowIndex] ?? [];
  const avatarUrl = presentation.speaker.avatar
    ? avatarUrls[presentation.speaker.avatar] ?? presentation.speaker.avatar
    : null;
  const speakerName = presentation.speaker.seatNo
    ? `${String(presentation.speaker.seatNo).padStart(2, "0")}号 ${presentation.speaker.name}`
    : presentation.speaker.name;

  return (
    <footer className="col-start-2 row-start-3 grid min-w-0 grid-cols-[132px_minmax(0,1fr)] items-center gap-7 border border-[#9d8052]/36 bg-[#080b09]/92 p-5 shadow-[0_18px_50px_rgba(0,0,0,0.28)]">
      <div className="grid h-[132px] w-[132px] place-items-center overflow-hidden border border-[#aa9165]/42 bg-[#151914]">
        {avatarUrl ? (
          <Img className="h-full w-full object-cover" src={avatarUrl} />
        ) : (
          <span className="text-[48px] font-black text-[#c9b48c]">
            {presentation.speaker.kind === "presenter"
              ? "主"
              : Array.from(presentation.speaker.name.trim())[0] ?? "?"}
          </span>
        )}
      </div>
      <div className="min-w-0 self-stretch py-1">
        <div className="truncate text-[25px] font-black tracking-[0.08em] text-[#c7aa74]">
          {speakerName}
        </div>
        <div className="mt-2 line-clamp-3 whitespace-pre-line text-[30px] font-bold leading-[1.2] tracking-[0.01em] text-[#f0e9dc]">
          {lines.join("\n")}
        </div>
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
