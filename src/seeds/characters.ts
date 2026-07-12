import characterDefinitionsJson from "../../kivdb/characters.json";
import { validateCharacterDefinitions } from "@/core/character-definition";

export const seedCharacters = validateCharacterDefinitions(
  characterDefinitionsJson,
);
