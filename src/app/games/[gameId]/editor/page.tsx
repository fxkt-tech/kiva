import Link from "next/link";
import { notFound } from "next/navigation";
import { DraftPanel } from "@/components/editor/draft-panel";
import { EventTimeline } from "@/components/editor/event-timeline";
import { GameBoard } from "@/components/editor/game-board";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";

type EditorPageProps = {
  readonly params: Promise<{
    readonly gameId: string;
  }>;
};

export default async function EditorPage({ params }: EditorPageProps) {
  const { gameId } = await params;
  const typedGameId = gameId as GameId;
  const record = await createGameActions(createGameRepository()).getGame(
    typedGameId,
  );

  if (!record) {
    notFound();
  }

  return (
    <main className="h-screen overflow-hidden bg-zinc-950 p-3 text-zinc-100 sm:p-4">
      <div className="flex h-full min-h-0 flex-col gap-3">
        <header className="flex h-12 shrink-0 items-center justify-between gap-3 border-b border-zinc-800 pb-3">
          <div className="flex min-w-0 items-center gap-3">
            <Link
              href="/"
              className="shrink-0 rounded-md border border-zinc-800 px-3 py-1.5 text-xs font-medium text-zinc-400 transition hover:border-zinc-600 hover:text-zinc-100"
            >
              Back
            </Link>
            <div className="flex min-w-0 items-baseline gap-3">
              <h1 className="shrink-0 text-lg font-semibold text-zinc-50">
                {record.game.title}
              </h1>
              <p className="truncate text-xs text-zinc-500">
                {record.game.id}
              </p>
            </div>
          </div>
          <Link
            href={`/games/${record.game.id}/preview`}
            className="shrink-0 rounded-md border border-zinc-700 px-3 py-1.5 text-center text-xs font-medium text-zinc-200 transition hover:border-zinc-500 hover:text-white"
            target="_blank"
          >
            Open preview
          </Link>
        </header>

        <div className="grid min-h-0 flex-1 grid-cols-1 gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(520px,0.82fr)]">
          <div className="grid min-h-0 gap-3 lg:grid-rows-[auto_minmax(0,1fr)]">
            <GameBoard game={record.game} events={record.events} />
            <DraftPanel
              gameId={typedGameId}
              draft={record.draft}
              players={record.game.players}
              generations={record.generations}
            />
          </div>

          <div className="grid min-h-0 gap-3 lg:grid-rows-[minmax(0,1fr)_auto]">
            <EventTimeline
              gameId={typedGameId}
              events={record.events}
              players={record.game.players}
            />
            <section className="flex min-h-0 flex-col rounded-lg border border-zinc-800 bg-zinc-900/45">
              <div className="shrink-0 border-b border-zinc-800 px-3 py-2">
                <h2 className="text-sm font-semibold text-zinc-100">
                  Embedded preview
                </h2>
              </div>
              <div className="p-2">
                <iframe
                  title="Public playback preview"
                  src={`/games/${record.game.id}/preview`}
                  className="aspect-video w-full rounded-md border border-zinc-800 bg-black"
                />
              </div>
            </section>
          </div>
        </div>
      </div>
    </main>
  );
}
