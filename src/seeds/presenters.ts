import presenterDefinitionsJson from "../../kivdb/presenters.json";
import { validatePresenterDefinitions } from "@/core/presenter-definition";

export const seedPresenters = validatePresenterDefinitions(
  presenterDefinitionsJson,
);
