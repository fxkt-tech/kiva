import {
  LibraryWorkspace,
  type LibraryTab,
} from "@/components/library/library-workspace";
import { createContentActions } from "@/server/content-actions";
import { createContentCatalog } from "@/server/content-catalog";
import { createGameRepository } from "@/server/game-repository";
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
  const catalog = await createContentActions({
    catalog: createContentCatalog(dataDir),
    gameRepository: createGameRepository(dataDir),
  }).getCatalog();
  const presenterVoiceManifests = Object.fromEntries(
    (await Promise.all(
      catalog.presenters.map(async (presenter) => {
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
      catalog={catalog}
      presenterVoiceManifests={presenterVoiceManifests}
    />
  );
}

function parseTab(value: string | undefined): LibraryTab {
  if (
    value === "actors" ||
    value === "lineups" ||
    value === "scripts" ||
    value === "presenters" ||
    value === "rules"
  ) {
    return value;
  }

  return "actors";
}
