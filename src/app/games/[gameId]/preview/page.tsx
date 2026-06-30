import { notFound } from "next/navigation";
import { PlaybackStage } from "@/components/preview/playback-stage";
import { systemVoiceSourceForScene } from "@/components/preview/preview-audio";
import { confirmDraftEvent, type DraftEvent } from "@/core/drafts";
import { getActiveEvents } from "@/core/event-log";
import type { GameEvent } from "@/core/events";
import { compilePublicPlayback } from "@/core/playback";
import type { EventId, GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import { loadSystemVoiceDurations } from "@/server/preview-voice-assets";

type PreviewPageProps = {
  readonly params: Promise<{
    readonly gameId: string;
  }>;
  readonly searchParams?: Promise<{
    readonly focus?: string;
  }>;
};

export default async function PreviewPage({
  params,
  searchParams,
}: PreviewPageProps) {
  const { gameId } = await params;
  const resolvedSearchParams = await searchParams;
  const focusParam = resolvedSearchParams?.focus;
  const dataDir = process.env.KIVA_DATA_DIR;
  const record = await createGameActions(createGameRepository(dataDir)).getGame(
    gameId as GameId,
  );

  if (!record) {
    notFound();
  }

  const focusCurrent = focusParam === "current";
  const playbackEvents = focusCurrent
    ? eventsWithDraftPreview(record.events, record.draft)
    : record.events;
  const voiceDurations = await loadSystemVoiceDurations(dataDir);

  return (
    <PlaybackStage
      initialPosition={focusCurrent ? "end" : "start"}
      items={compilePublicPlayback(playbackEvents, record.game.players, {
        audience: "director",
        durationForScene: (scene) => {
          const source = systemVoiceSourceForScene(scene);
          return source ? voiceDurations.get(source) ?? null : null;
        },
      })}
    />
  );
}

function eventsWithDraftPreview(
  events: readonly GameEvent[],
  draft: DraftEvent | null,
): readonly GameEvent[] {
  if (!draft) {
    return events;
  }

  const nextIndex = getActiveEvents(events).length + 1;
  return [
    ...events,
    confirmDraftEvent({
      draft,
      eventId: `preview_${draft.id}` as EventId,
      index: nextIndex,
      createdAt: draft.createdAt,
    }),
  ];
}
