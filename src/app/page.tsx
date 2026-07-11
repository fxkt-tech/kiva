import Link from "next/link";
import { Eye, SquarePen, Trash2 } from "lucide-react";
import { deleteGameAction } from "@/app/actions";
import { GameTokenUsageButton } from "@/components/home/game-token-usage-button";
import { GameTitleEditor } from "@/components/home/game-title-editor";
import { NewGameDialog } from "@/components/home/new-game-dialog";
import { ThemeToggle } from "@/components/theme-toggle";
import { Button } from "@/components/ui/button";
import { iconButtonClassName, textButtonClassName } from "@/components/ui/button-styles";
import type { GenerationRecord } from "@/core/generation-record";
import { summarizeGenerationTokenUsage } from "@/core/token-usage";
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
    <main className="min-h-screen bg-background px-4 py-6 text-foreground sm:px-6 lg:px-8">
      <div className="mx-auto flex max-w-6xl flex-col gap-6">
        <header className="flex flex-col gap-4 border-b border-border pb-5 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <p className="text-xs font-medium uppercase tracking-[0.18em] text-subtle">
              Kiva Director
            </p>
            <h1 className="mt-2 text-2xl font-semibold text-foreground">
              Local games
            </h1>
          </div>
          <div className="flex flex-col gap-2 sm:flex-row">
            <ThemeToggle />
            <Link
              href="/library"
              className={textButtonClassName()}
            >
              Library
            </Link>
            <NewGameDialog
              presets={creatablePresets}
              roles={library.roles.filter((role) => role.enabled)}
              characters={library.characters.filter((character) => character.enabled)}
              presenters={library.presenters.filter((presenter) => presenter.enabled)}
            />
          </div>
        </header>

        <section className="overflow-hidden rounded-lg border border-border bg-surface/45">
          {records.length === 0 ? (
            <div className="px-4 py-10 text-sm text-muted">
              No local games yet.
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full min-w-[760px] table-fixed text-left text-sm">
                <colgroup>
                  <col className="w-auto" />
                  <col className="w-28" />
                  <col className="w-36" />
                  <col className="w-52" />
                </colgroup>
                <thead>
                  <tr className="border-b border-border text-xs font-medium uppercase tracking-[0.14em] text-subtle">
                    <th className="px-4 py-3 font-medium">Game</th>
                    <th className="px-4 py-3 font-medium">Events</th>
                    <th className="px-4 py-3 font-medium">Updated</th>
                    <th className="px-4 py-3 text-right font-medium">Operate</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {records.map((record) => (
                    <tr key={record.game.id} className="align-middle">
                      <td className="px-4 py-4">
                        <div className="min-w-0">
                          <GameTitleEditor
                            gameId={record.game.id}
                            title={record.game.title}
                          />
                          <div className="mt-1 truncate text-xs text-subtle">
                            {record.game.id}
                          </div>
                        </div>
                      </td>
                      <td className="px-4 py-4 text-muted">
                        {record.events.filter((event) => event.status === "active")
                          .length}
                        {record.draft ? " + draft" : ""}
                      </td>
                      <td className="px-4 py-4">
                        <time dateTime={record.game.updatedAt} className="text-subtle">
                          {formatDate(record.game.updatedAt)}
                        </time>
                      </td>
                      <td className="px-4 py-4">
                        <div className="flex justify-end gap-1.5">
                          <GameTokenUsageButton
                            gameTitle={record.game.title}
                            total={summarizeGenerationTokenUsage(record.generations)}
                            speech={summarizeGenerationTokenUsage(
                              generationsByPurpose(record.generations, "speech"),
                            )}
                            action={summarizeGenerationTokenUsage(
                              generationsByPurpose(record.generations, "action"),
                            )}
                          />
                          <Link
                            href={`/games/${record.game.id}/preview`}
                            className={iconButtonClassName({ size: "md" })}
                            target="_blank"
                            aria-label="Preview game"
                            title="Preview game"
                          >
                            <Eye aria-hidden="true" className="h-4 w-4" />
                          </Link>
                          <Link
                            href={`/games/${record.game.id}/editor`}
                            className={iconButtonClassName({ size: "md" })}
                            aria-label="Open editor"
                            title="Open editor"
                          >
                            <SquarePen aria-hidden="true" className="h-4 w-4" />
                          </Link>
                          <form action={deleteGameAction.bind(null, record.game.id)}>
                            <Button
                              type="submit"
                              buttonStyle="icon"
                              iconSize="md"
                              variant="danger"
                              aria-label="Delete game"
                              title="Delete game"
                            >
                              <Trash2 aria-hidden="true" className="h-4 w-4" />
                            </Button>
                          </form>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

function generationsByPurpose(
  generations: readonly GenerationRecord[],
  purpose: GenerationRecord["purpose"],
): readonly GenerationRecord[] {
  return generations.filter((generation) => generation.purpose === purpose);
}

function formatDate(value: string): string {
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}
