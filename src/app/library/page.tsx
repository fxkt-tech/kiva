import {
  LibraryWorkspace,
  type LibraryTab,
} from "@/components/library/library-workspace";
import { createGameRepository } from "@/server/game-repository";
import { createLibraryActions } from "@/server/library-actions";
import { createLibraryRepository } from "@/server/library-repository";

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

  return (
    <LibraryWorkspace
      activeTab={activeTab}
      selectedId={params.id ?? null}
      library={library}
    />
  );
}

function parseTab(value: string | undefined): LibraryTab {
  if (value === "characters" || value === "presets") {
    return value;
  }

  return "roles";
}
