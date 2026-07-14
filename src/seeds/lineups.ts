import lineupDefinitionsJson from "../../kivdb/lineups.json";
import { validateLineups } from "@/core/lineup";
import { seedActors } from "./actors";

export const seedLineups = validateLineups(lineupDefinitionsJson, seedActors);
