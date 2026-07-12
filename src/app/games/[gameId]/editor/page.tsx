import { ArrowLeft, ExternalLink } from "lucide-react";
import Link from "next/link";
import { notFound, redirect } from "next/navigation";
import { DraftPanel } from "@/components/editor/draft-panel";
import { EventTimeline } from "@/components/editor/event-timeline";
import { iconButtonClassName } from "@/components/ui/button-styles";
import { getActiveEvents } from "@/core/event-log";
import { actorBriefForStep } from "@/core/episode-script";
import { isLlmSpeechDraft } from "@/core/llm-task-specs";
import { speechBudgetForDraft } from "@/core/speech-budget";
import type { GameId } from "@/core/types";
import { deriveGameState } from "@/core/state";
import { createGameActions } from "@/server/game-actions";
import {
  createGameRepository,
  type GameRecord,
} from "@/server/game-repository";

type EditorPageProps = {
  readonly params: Promise<{
    readonly gameId: string;
  }>;
  readonly searchParams: Promise<{
    readonly timeline?: string;
  }>;
};

export default async function EditorPage({
  params,
  searchParams,
}: EditorPageProps) {
  const { gameId } = await params;
  const { timeline } = await searchParams;
  const typedGameId = gameId as GameId;
  const dataDir = process.env.KIVA_DATA_DIR;
  const gameActions = createGameActions(createGameRepository(dataDir));
  const loadedRecord = await gameActions.getGame(typedGameId);

  if (!loadedRecord) {
    notFound();
  }
  if (
    loadedRecord.game.runMode === "scripted" &&
    loadedRecord.episodeScript?.status !== "approved"
  ) {
    redirect(`/games/${loadedRecord.game.id}/script`);
  }

  const record = loadedRecord.draft
    ? loadedRecord
    : await gameActions.continueGame(typedGameId);
  const previewHref = currentPreviewHref(record);
  const previewHidden = timeline === "full";
  const editorHref = `/games/${record.game.id}/editor`;
  const togglePreviewHref = previewHidden
    ? editorHref
    : `${editorHref}?timeline=full`;

  return (
    <main className="h-screen overflow-hidden bg-background p-3 text-foreground sm:p-4">
      <div className="grid h-full min-h-0 grid-cols-1 gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(520px,0.9fr)]">
        <div
          className={
            previewHidden
              ? "grid min-h-0 grid-rows-[minmax(0,1fr)] gap-3"
              : "grid min-h-0 gap-3 lg:grid-rows-[minmax(0,0.56fr)_minmax(0,0.44fr)]"
          }
        >
          {previewHidden ? null : (
            <section className="flex min-h-0 flex-col overflow-hidden rounded-lg border border-border bg-surface/45">
              <div className="flex shrink-0 items-center justify-between gap-3 border-b border-border px-3 py-2">
                <h2 className="text-sm font-semibold text-foreground">
                  Preview
                </h2>
                <div className="flex gap-1.5">
                  <Link
                    href={previewHref}
                    aria-label="Open preview in new tab"
                    title="Open preview in new tab"
                    className={iconButtonClassName()}
                    target="_blank"
                  >
                    <ExternalLink aria-hidden="true" className="h-4 w-4" />
                  </Link>
                </div>
              </div>
              <div className="grid min-h-0 flex-1 place-items-center p-2 [container-type:size]">
                <iframe
                  key={previewHref}
                  title="Public playback preview"
                  src={previewHref}
                  className="aspect-video rounded-md border border-border bg-black [width:min(100cqw,calc(100cqh*16/9))]"
                />
              </div>
            </section>
          )}
          <EventTimeline
            gameId={typedGameId}
            events={record.events}
            players={record.game.players}
            generations={record.generations}
            previewHidden={previewHidden}
            togglePreviewHref={togglePreviewHref}
          />
        </div>

        <div className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)] gap-3">
          <header className="flex h-9 shrink-0 items-center gap-3 border-b border-border pb-2">
            <Link
              href="/"
              aria-label="Back"
              title="Back"
              className={iconButtonClassName()}
            >
              <ArrowLeft aria-hidden="true" className="h-4 w-4" />
            </Link>
            <h1 className="min-w-0 truncate text-base font-semibold text-foreground">
              {record.game.title}
            </h1>
          </header>
          <DraftPanel
            gameId={typedGameId}
            draft={record.draft}
            players={record.game.players}
            alivePlayerIds={
              deriveGameState(record.game.players, record.events).alivePlayerIds
            }
            generations={record.generations}
            speechBudget={speechBudgetForRecord(record)}
            actorBrief={actorBriefForRecord(record)}
            scriptedStructureLocked={record.episodeScript?.status === "approved"}
          />
        </div>
      </div>
    </main>
  );
}

function actorBriefForRecord(record: GameRecord) {
  if (record.episodeScript?.status !== "approved") return null;
  return actorBriefForStep(
    record.episodeScript.script,
    getActiveEvents(record.events).length + 1,
  );
}

function speechBudgetForRecord(record: GameRecord) {
  const draft = record.draft;
  if (!draft || !isLlmSpeechDraft(draft)) return undefined;
  if (record.episodeScript?.status === "approved") {
    return actorBriefForStep(
      record.episodeScript.script,
      getActiveEvents(record.events).length + 1,
    )?.budget;
  }
  const hasPriorDaySpeech =
    draft.type === "day_speech_given" &&
    getActiveEvents(record.events).some(
      (event) =>
        event.type === "day_speech_given" &&
        event.payload.dayNumber === draft.payload.dayNumber,
    );
  return speechBudgetForDraft({ draft, hasPriorDaySpeech });
}

function currentPreviewHref(record: GameRecord): string {
  const params = new URLSearchParams({
    focus: "current",
    rev: [
      record.game.updatedAt,
      record.events.length,
      record.draft?.id ?? "none",
      record.draft?.createdAt ?? "none",
      record.generations.at(-1)?.createdAt ?? "none",
    ].join("|"),
  });

  return `/games/${record.game.id}/preview?${params.toString()}`;
}
