import { notFound } from "next/navigation";
import { eventsWithDraftPreview } from "@/components/preview/preview-events";
import { createCompositionInput } from "@/components/preview-v2/composition/create-composition-input";
import { PreviewV2Studio } from "@/components/preview-v2/preview-v2-studio";
import {
  compilePublicPlayback,
  type CompilePublicPlaybackOptions,
} from "@/core/playback";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import { loadPresenterVoiceManifest } from "@/server/presenter-voice-manifest";

type PreviewPageProps = {
  readonly params: Promise<{ readonly gameId: string }>;
  readonly searchParams?: Promise<{ readonly focus?: string }>;
};

export const dynamic = "force-dynamic";

export default async function PreviewPage({
  params,
  searchParams,
}: PreviewPageProps) {
  const { gameId } = await params;
  const { focus } = (await searchParams) ?? {};
  const dataDir = process.env.KIVA_DATA_DIR;
  const record = await createGameActions(createGameRepository(dataDir)).getGame(
    gameId as GameId,
  );
  if (!record) {
    notFound();
  }

  const includesDraft = focus === "current" && record.draft !== null;
  const presenterVoiceManifest = await loadPresenterVoiceManifest(
    dataDir ?? "kivdb",
    record.game.presenter.presenterSourceId,
  ).catch(() => undefined);
  const playbackOptions = {
    presenter: record.game.presenter,
    audience: "director",
    voiceArtifactsByEventId: record.voiceArtifactsByEventId,
    presenterVoiceManifest,
  } satisfies CompilePublicPlaybackOptions;
  const confirmedItems = compilePublicPlayback(
    record.events,
    record.game.players,
    playbackOptions,
  );
  const items = includesDraft
    ? compilePublicPlayback(
        eventsWithDraftPreview(record.events, record.draft),
        record.game.players,
        playbackOptions,
      )
    : confirmedItems;
  const missingVoiceCount = confirmedItems.filter(
    (item) => item.transcriptSpeaker === "player" && item.playerVoice === null,
  ).length;
  const exportBlocker = presenterVoiceManifest === undefined
    ? "主理人预制配音库缺失，请先运行 pnpm voice:build-presenter。"
    : missingVoiceCount > 0
      ? `仍有 ${missingVoiceCount} 条玩家配音未完成。`
      : confirmedItems.length === 0
        ? "没有已确认的播放记录，暂时无法生成视频。"
        : null;

  return (
    <PreviewV2Studio
      composition={createCompositionInput({
        gameId: record.game.id,
        gameTitle: record.game.title,
        script: record.game.script,
        items,
      })}
      canExport={
        confirmedItems.length > 0 &&
        missingVoiceCount === 0 &&
        presenterVoiceManifest !== undefined
      }
      missingVoiceCount={missingVoiceCount}
      exportBlocker={exportBlocker}
    />
  );
}
