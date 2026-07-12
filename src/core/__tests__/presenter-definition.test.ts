import { describe, expect, it } from "vitest";
import presenterDefinitionsJson from "../../../kivdb/presenters.json";
import {
  createGamePresenterSnapshot,
  validateGamePresenterSnapshot,
  validatePresenterDefinitions,
} from "../presenter-definition";

describe("presenter definitions", () => {
  it("validates the complete KivDB presenter collection", () => {
    const definitions = validatePresenterDefinitions(presenterDefinitionsJson);

    expect(definitions.map((definition) => definition.name)).toEqual(["闻舟"]);
    expect(definitions[0]?.lines["phase.night"].template).toContain("灯灭了");
  });

  it("rejects duplicate IDs and incomplete line catalogs", () => {
    const duplicate = [
      structuredClone(presenterDefinitionsJson[0]!),
      structuredClone(presenterDefinitionsJson[0]!),
    ];
    expect(() => validatePresenterDefinitions(duplicate)).toThrow(
      "Duplicate presenter definition id",
    );

    const incomplete = structuredClone(presenterDefinitionsJson) as unknown as {
      0: { lines: Record<string, unknown> };
    };
    delete incomplete[0].lines["phase.night"];
    expect(() => validatePresenterDefinitions(incomplete)).toThrow(
      "missing: phase.night",
    );
  });

  it("rejects placeholder drift", () => {
    const placeholderDrift = structuredClone(presenterDefinitionsJson);
    placeholderDrift[0]!.lines["action.wolf_kill"].template =
      "狼人选择了{player}。";
    expect(() => validatePresenterDefinitions(placeholderDrift)).toThrow(
      "template placeholders must equal [target]",
    );
  });

  it("creates an isolated game snapshot and validates it strictly", () => {
    const [definition] = validatePresenterDefinitions(presenterDefinitionsJson);
    const snapshot = createGamePresenterSnapshot(definition!);

    expect(snapshot).toMatchObject({
      presenterSourceId: "wen_zhou",
      name: "闻舟",
      avatar: "/kivdb-assets/presenters/presenter_wen_zhou_v1.png",
    });
    expect(snapshot.lines).not.toBe(definition!.lines);
    expect(validateGamePresenterSnapshot(snapshot)).toBe(snapshot);
    expect(() =>
      validateGamePresenterSnapshot({ ...snapshot, lines: {} }),
    ).toThrow("keys are invalid");
  });
});
