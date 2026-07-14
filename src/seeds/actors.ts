import actorDefinitionsJson from "../../kivdb/actors.json";
import {
  assertActorLibraryInvariants,
  validateActorDefinitions,
} from "@/core/actor-definition";

export const seedActors = validateActorDefinitions(actorDefinitionsJson);
assertActorLibraryInvariants(seedActors);
