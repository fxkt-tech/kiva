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

    expect(definitions.map((definition) => definition.name)).toEqual([
      "守夜人",
      "法官",
    ]);
    expect(definitions[0]?.lines["phase.night"].template).not.toBe(
      definitions[1]?.lines["phase.night"].template,
    );
  });

  it("rejects duplicate IDs and incomplete line catalogs", () => {
    const duplicate = structuredClone(presenterDefinitionsJson);
    duplicate[1]!.id = duplicate[0]!.id;
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

  it("rejects placeholder drift and unsafe voice files", () => {
    const placeholderDrift = structuredClone(presenterDefinitionsJson);
    placeholderDrift[0]!.lines["action.wolf_kill"].template =
      "狼人选择了{player}。";
    expect(() => validatePresenterDefinitions(placeholderDrift)).toThrow(
      "template placeholders must equal [target]",
    );

    const unsafeVoice = structuredClone(presenterDefinitionsJson);
    unsafeVoice[0]!.lines["phase.night"].voice!.file = "../night.mp3";
    expect(() => validatePresenterDefinitions(unsafeVoice)).toThrow(
      "safe mp3 basename",
    );
  });

  it("requires an explicit voice mapping decision for seats 1 through 12", () => {
    const missingSeat = structuredClone(presenterDefinitionsJson) as unknown as {
      0: {
        lines: {
          "prompt.speech": {
            voiceBySeat: Record<string, unknown>;
          };
        };
      };
    };
    delete missingSeat[0].lines["prompt.speech"].voiceBySeat["12"];

    expect(() => validatePresenterDefinitions(missingSeat)).toThrow(
      "exactly seats 1 through 12",
    );
  });

  it("creates an isolated game snapshot and validates it strictly", () => {
    const [definition] = validatePresenterDefinitions(presenterDefinitionsJson);
    const snapshot = createGamePresenterSnapshot(definition!);

    expect(snapshot).toMatchObject({
      presenterSourceId: "night_watch",
      name: "守夜人",
      avatar: null,
    });
    expect(snapshot.lines).not.toBe(definition!.lines);
    expect(validateGamePresenterSnapshot(snapshot)).toBe(snapshot);
    expect(() =>
      validateGamePresenterSnapshot({ ...snapshot, lines: {} }),
    ).toThrow("keys are invalid");
  });
});
