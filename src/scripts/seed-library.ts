import { createLibraryRepository } from "../server/library-repository";
import { seedCharacters } from "../seeds/characters";
import { seedPresets } from "../seeds/presets";
import { seedRoles } from "../seeds/roles";

type SeedTarget = "all" | "roles" | "characters" | "presets";

const VALID_TARGETS = new Set<SeedTarget>([
  "all",
  "roles",
  "characters",
  "presets",
]);

async function main(): Promise<void> {
  const target = parseTarget(process.argv.slice(2));
  const repository = createLibraryRepository(process.env.KIVA_DATA_DIR);

  if (target === "all") {
    await repository.saveAll({
      roles: seedRoles,
      characters: seedCharacters,
      presets: seedPresets,
    });
    console.log(
      `Seeded role library: ${seedRoles.length} roles, ${seedCharacters.length} characters, ${seedPresets.length} presets`,
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

  await repository.savePresets(seedPresets);
  console.log(`Seeded role library: ${seedPresets.length} presets`);
}

function parseTarget(args: readonly string[]): SeedTarget {
  if (args.length === 0) {
    return "all";
  }

  if (args.length !== 1 || !VALID_TARGETS.has(args[0] as SeedTarget)) {
    throw new Error("Usage: seed-library.ts [all|roles|characters|presets]");
  }

  return args[0] as SeedTarget;
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
