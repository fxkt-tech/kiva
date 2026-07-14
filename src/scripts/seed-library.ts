import { createContentCatalog } from "../server/content-catalog";
import { seedActors } from "../seeds/actors";
import { seedLineups } from "../seeds/lineups";
import { seedPresenters } from "../seeds/presenters";
import { seedScripts } from "../seeds/scripts";

async function main(): Promise<void> {
  if (process.argv.length > 2) {
    throw new Error("Usage: seed-library.ts");
  }
  const catalog = createContentCatalog(process.env.KIVA_DATA_DIR);
  await catalog.save({
    actors: seedActors,
    lineups: seedLineups,
    presenters: seedPresenters,
    scripts: seedScripts,
  });
  console.log(
    `Seeded Content Catalog: ${seedActors.length} actors, ${seedLineups.length} lineups, ${seedPresenters.length} presenters, ${seedScripts.length} scripts`,
  );
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
