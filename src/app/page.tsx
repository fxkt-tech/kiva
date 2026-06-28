import Link from "next/link";
import { deleteGameAction } from "@/app/actions";
import { NewGameDialog } from "@/components/home/new-game-dialog";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import { createLibraryActions } from "@/server/library-actions";
import { createLibraryRepository } from "@/server/library-repository";

export const dynamic = "force-dynamic";

export default async function HomePage() {
  const dataDir = process.env.KIVA_DATA_DIR;
  const records = await createGameActions(
    createGameRepository(dataDir),
  ).listGames();
  const library = await createLibraryActions({
    libraryRepository: createLibraryRepository(dataDir),
    gameRepository: createGameRepository(dataDir),
  }).getLibrary();
  const creatablePresets = library.presets.filter(
    (preset) => library.diagnostics.presets[preset.id]?.canCreateGame === true,
  );

  return (
    <main className="min-h-screen bg-zinc-950 px-4 py-6 text-zinc-100 sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="flex flex-col gap-4 border-b border-zinc-800 pb-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-xs font-medium uppercase tracking-[0.18em] text-zinc-500">
              Kiva Director
            </p>
            <h1 className="mt-2 text-2xl font-semibold text-zinc-50">
              Local games
            </h1>
          </div>
          <div className="flex flex-col gap-2 sm:flex-row">
            <Link
              href="/library"
              className="rounded-md border border-zinc-700 px-4 py-2 text-center text-sm font-medium text-zinc-200 transition hover:border-zinc-500 hover:text-white"
            >
              Library
            </Link>
            <NewGameDialog
              presets={creatablePresets}
              roles={library.roles.filter((role) => role.enabled)}
              characters={library.characters.filter((character) => character.enabled)}
            />
          </div>
        </header>

        <section className="overflow-hidden rounded-lg border border-zinc-800 bg-zinc-900/45">
          <div className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 border-b border-zinc-800 px-4 py-3 text-xs font-medium uppercase tracking-[0.14em] text-zinc-500 sm:grid-cols-[minmax(0,1fr)_160px_160px_auto]">
            <span>Game</span>
            <span className="hidden sm:block">Events</span>
            <span className="hidden sm:block">Updated</span>
            <span>Operate</span>
          </div>
          {records.length === 0 ? (
            <div className="px-4 py-10 text-sm text-zinc-400">
              No local games yet.
            </div>
          ) : (
            <div className="divide-y divide-zinc-800">
              {records.map((record) => (
                <div
                  key={record.game.id}
                  className="grid grid-cols-[minmax(0,1fr)_auto] gap-3 px-4 py-4 text-sm sm:grid-cols-[minmax(0,1fr)_160px_160px_auto] sm:items-center"
                >
                  <div className="min-w-0">
                    <div className="truncate font-medium text-zinc-100">
                      {record.game.title}
                    </div>
                    <div className="mt-1 truncate text-xs text-zinc-500">
                      {record.game.id}
                    </div>
                  </div>
                  <div className="hidden text-zinc-400 sm:block">
                    {record.events.filter((event) => event.status === "active")
                      .length}
                    {record.draft ? " + draft" : ""}
                  </div>
                  <time
                    dateTime={record.game.updatedAt}
                    className="hidden text-zinc-500 sm:block"
                  >
                    {formatDate(record.game.updatedAt)}
                  </time>
                  <div className="flex flex-wrap justify-end gap-2">
                    <Link
                      href={`/games/${record.game.id}/preview`}
                      className="rounded-md border border-sky-900/80 px-3 py-2 text-xs font-medium text-sky-300 transition hover:border-sky-600 hover:text-sky-200"
                      target="_blank"
                    >
                      Preview
                    </Link>
                    <Link
                      href={`/games/${record.game.id}/editor`}
                      className="rounded-md border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-200 transition hover:border-zinc-500 hover:text-white"
                    >
                      Editor
                    </Link>
                    <form action={deleteGameAction.bind(null, record.game.id)}>
                      <button
                        type="submit"
                        className="rounded-md border border-red-900/80 px-3 py-2 text-xs font-medium text-red-300 transition hover:border-red-600 hover:text-red-200"
                      >
                        Delete
                      </button>
                    </form>
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

function formatDate(value: string): string {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}
