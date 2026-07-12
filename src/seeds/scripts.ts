import gameScriptDefinitionsJson from "../../kivdb/scripts.json";
import { validateGameScriptDefinitions } from "@/core/game-script";

export const seedScripts = validateGameScriptDefinitions(
  gameScriptDefinitionsJson,
);
