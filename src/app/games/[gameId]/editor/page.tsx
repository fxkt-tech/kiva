import { ArrowLeft, ExternalLink } from "lucide-react";
import Link from "next/link";
import { notFound } from "next/navigation";
import { DraftPanel } from "@/components/editor/draft-panel";
import { EventTimeline } from "@/components/editor/event-timeline";
import type { GameId } from "@/core/types";
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
    <main className="h-screen overflow-hidden bg-zinc-950 p-3 text-zinc-100 sm:p-4">
      <div className="flex h-full min-h-0 flex-col gap-3">
        <header className="flex h-9 shrink-0 items-center gap-3 border-b border-zinc-800 pb-2">
          <div className="flex min-w-0 items-center gap-3">
            <Link
              href="/"
              aria-label="Back"
              title="Back"
              className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-zinc-800 text-zinc-400 transition hover:border-zinc-600 hover:text-zinc-100"
            >
              <ArrowLeft aria-hidden="true" className="h-4 w-4" />
            </Link>
            <div className="min-w-0">
              <h1 className="truncate text-base font-semibold text-zinc-50">
                {record.game.title}
              </h1>
            </div>
          </div>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-1 gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(520px,0.9fr)]">
          <div
            className={
              previewHidden
                ? "grid min-h-0 grid-rows-[minmax(0,1fr)] gap-3"
                : "grid min-h-0 gap-3 lg:grid-rows-[auto_minmax(0,1fr)]"
            }
          >
            {previewHidden ? null : (
              <section className="flex min-h-0 flex-col rounded-lg border border-zinc-800 bg-zinc-900/45">
                <div className="flex shrink-0 items-center justify-between gap-3 border-b border-zinc-800 px-3 py-2">
                  <h2 className="text-sm font-semibold text-zinc-100">
                    Preview
                  </h2>
                  <Link
                    href={previewHref}
                    aria-label="Open preview in new tab"
                    title="Open preview in new tab"
                    className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-zinc-700 text-zinc-300 transition hover:border-zinc-500 hover:text-white"
                    target="_blank"
                  >
                    <ExternalLink aria-hidden="true" className="h-4 w-4" />
                  </Link>
                </div>
                <div className="p-2">
                  <iframe
                    key={previewHref}
                    title="Public playback preview"
                    src={previewHref}
                    className="aspect-video w-full rounded-md border border-zinc-800 bg-black"
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

          <div className="min-h-0">
            <DraftPanel
              gameId={typedGameId}
              draft={record.draft}
              players={record.game.players}
              generations={record.generations}
            />
          </div>
        </div>
      </div>
    </main>
  );
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
