import { createLibraryRepository } from "../server/library-repository";
import { seedCharacters } from "../seeds/characters";
import { seedPresets } from "../seeds/presets";
import { seedRoles } from "../seeds/roles";
import { seedPresenters } from "../seeds/presenters";

type SeedTarget = "all" | "roles" | "characters" | "presets" | "presenters";

const VALID_TARGETS = new Set<SeedTarget>([
  "all",
  "roles",
  "characters",
  "presets",
  "presenters",
]);

async function main(): Promise<void> {
  const target = parseTarget(process.argv.slice(2));
  const repository = createLibraryRepository(process.env.KIVA_DATA_DIR);

  if (target === "all") {
    await repository.saveAll({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
      presenters: seedPresenters,
    });
    console.log(
      `Seeded library: ${seedRoles.length} roles, ${seedCharacters.length} characters, ${seedPresets.length} presets, ${seedPresenters.length} presenters`,
    );
    return;
  }

  if (target === "roles") {
    await repository.saveRoles(seedRoles);
    console.log(`Seeded role library: ${seedRoles.length} roles`);
    return;
  }

  if (target === "characters") {
    await repository.saveCharacters(seedCharacters);
    console.log(`Seeded role library: ${seedCharacters.length} characters`);
    return;
  }

  if (target === "presenters") {
    await repository.savePresenters(seedPresenters);
    console.log(`Seeded presenter library: ${seedPresenters.length} presenters`);
    return;
  }

  await repository.savePresets(seedPresets);
  console.log(`Seeded role library: ${seedPresets.length} presets`);
}

function parseTarget(args: readonly string[]): SeedTarget {
  if (args.length === 0) {
    return "all";
  }

  if (args.length !== 1 || !VALID_TARGETS.has(args[0] as SeedTarget)) {
    throw new Error(
      "Usage: seed-library.ts [all|roles|characters|presets|presenters]",
    );
  }

  return args[0] as SeedTarget;
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
