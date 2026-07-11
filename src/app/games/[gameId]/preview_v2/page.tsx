import { notFound } from "next/navigation";
import { eventsWithDraftPreview } from "@/components/preview/preview-events";
import { systemVoiceSourceForScene } from "@/components/preview/preview-audio";
import { createCompositionInput } from "@/components/preview-v2/composition/create-composition-input";
import { PreviewV2Studio } from "@/components/preview-v2/preview-v2-studio";
import {
  compilePublicPlayback,
  type CompilePublicPlaybackOptions,
} from "@/core/playback";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import { loadSystemVoiceDurations } from "@/server/preview-voice-assets";

type PreviewV2PageProps = {
  readonly params: Promise<{ readonly gameId: string }>;
  readonly searchParams?: Promise<{ readonly focus?: string }>;
};

export const dynamic = "force-dynamic";

export default async function PreviewV2Page({
  params,
  searchParams,
}: PreviewV2PageProps) {
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
  const voiceDurations = await loadSystemVoiceDurations(dataDir);
  const playbackOptions = {
    presenter: record.game.presenter,
    audience: "director",
    durationForScene: (scene) => {
      const source = systemVoiceSourceForScene(scene);
      return source ? voiceDurations.get(source) ?? null : null;
    },
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

  return (
    <PreviewV2Studio
      composition={createCompositionInput({
        gameId: record.game.id,
        gameTitle: record.game.title,
        items,
      })}
      includesDraft={includesDraft}
      canExport={confirmedItems.length > 0}
    />
  );
}
