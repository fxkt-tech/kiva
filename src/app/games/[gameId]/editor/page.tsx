import Link from "next/link";
import { notFound } from "next/navigation";
import { continueGameAction } from "@/app/actions";
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
    <main className="min-h-screen bg-zinc-950 px-4 py-5 text-zinc-100 sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-7xl flex-col gap-5">
        <header className="flex flex-col gap-4 border-b border-zinc-800 pb-5 lg:flex-row lg:items-end lg:justify-between">
          <div className="min-w-0">
            <Link href="/" className="text-xs text-zinc-500 hover:text-zinc-300">
              Local games
            </Link>
            <h1 className="mt-2 truncate text-2xl font-semibold text-zinc-50">
              {record.game.title}
            </h1>
            <p className="mt-1 truncate text-xs text-zinc-500">
              {record.game.id}
            </p>
          </div>
          <div className="flex flex-col gap-2 sm:flex-row">
            <form action={continueGameAction.bind(null, typedGameId)}>
              <button
                type="submit"
                className="w-full rounded-md bg-zinc-100 px-4 py-2 text-sm font-medium text-zinc-950 transition hover:bg-white sm:w-auto"
              >
                Continue
              </button>
            </form>
            <Link
              href={`/games/${record.game.id}/preview`}
              className="rounded-md border border-zinc-700 px-4 py-2 text-center text-sm font-medium text-zinc-200 transition hover:border-zinc-500 hover:text-white"
              target="_blank"
            >
              Open preview
            </Link>
          </div>
        </header>

        <div className="grid grid-cols-1 gap-5 lg:grid-cols-[minmax(0,1fr)_390px]">
          <div className="flex min-w-0 flex-col gap-5">
            <GameBoard game={record.game} events={record.events} />
            <EventTimeline
              gameId={typedGameId}
              events={record.events}
              players={record.game.players}
            />
          </div>

          <aside className="flex min-w-0 flex-col gap-5">
            <DraftPanel
              gameId={typedGameId}
              draft={record.draft}
              players={record.game.players}
              generations={record.generations}
            />
            <section className="rounded-lg border border-zinc-800 bg-zinc-900/45">
              <div className="border-b border-zinc-800 px-4 py-3">
                <h2 className="text-sm font-semibold text-zinc-100">
                  Embedded preview
                </h2>
              </div>
              <div className="p-3">
                <iframe
                  title="Public playback preview"
                  src={`/games/${record.game.id}/preview`}
                  className="aspect-video w-full rounded-md border border-zinc-800 bg-black"
                />
              </div>
            </section>
          </aside>
        </div>
      </div>
    </main>
  );
}
