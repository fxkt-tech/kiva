import {
  LibraryWorkspace,
  type LibraryTab,
} from "@/components/library/library-workspace";
import { createGameRepository } from "@/server/game-repository";
import { createLibraryActions } from "@/server/library-actions";
import { createLibraryRepository } from "@/server/library-repository";
import { loadPresenterVoiceManifest } from "@/server/presenter-voice-manifest";

export const dynamic = "force-dynamic";

type LibraryPageProps = {
  readonly searchParams: Promise<{
    readonly tab?: string;
    readonly id?: string;
  }>;
};

export default async function LibraryPage({ searchParams }: LibraryPageProps) {
  const params = await searchParams;
  const activeTab = parseTab(params.tab);
  const dataDir = process.env.KIVA_DATA_DIR;
  const library = await createLibraryActions({
    libraryRepository: createLibraryRepository(dataDir),
    gameRepository: createGameRepository(dataDir),
  }).getLibrary();
  const presenterVoiceManifests = Object.fromEntries(
    (await Promise.all(
      library.presenters.map(async (presenter) => {
        const manifest = await loadPresenterVoiceManifest(
          dataDir ?? "kivdb",
          presenter.id,
        ).catch(() => null);
        return manifest ? [presenter.id, manifest] as const : null;
      }),
    )).filter((entry) => entry !== null),
  );

  return (
    <LibraryWorkspace
      activeTab={activeTab}
      selectedId={params.id ?? null}
      library={library}
      presenterVoiceManifests={presenterVoiceManifests}
    />
  );
}

function parseTab(value: string | undefined): LibraryTab {
  if (
    value === "characters" ||
    value === "presenters" ||
    value === "presets"
  ) {
    return value;
  }

  return "roles";
}
