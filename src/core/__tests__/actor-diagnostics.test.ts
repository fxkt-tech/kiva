import { describe, expect, it } from "vitest";
import { seedActors } from "@/seeds/actors";
import {
  compileActorComparisonMatrix,
  diagnoseActorPool,
  diagnoseActorSelection,
} from "../actor-diagnostics";

describe("Actor diagnostics", () => {
  it("compares the complete extensible pool across production dimensions", () => {
    const matrix = compileActorComparisonMatrix(seedActors);
    const diagnostic = diagnoseActorPool(seedActors);

    expect(matrix).toHaveLength(16);
    expect(matrix[0]).toMatchObject({
      actorId: "qin_chuan",
      tableFunction: expect.any(String),
      decisionPolicy: expect.any(String),
      voice: expect.any(String),
    });
    expect(diagnostic).toMatchObject({
      enabledActorCount: 16,
      distinctVoiceCount: 6,
      hardIssues: [],
    });
  });

  it("flags behavioral clones without making soft diversity a game rule", () => {
    const source = seedActors.find((actor) => actor.id === "qiao_ke")!;
    const clone = {
      ...structuredClone(source),
      id: "behavior_clone",
      identity: { ...source.identity, name: "行为克隆" },
    };
    const diagnostic = diagnoseActorPool([...seedActors, clone]);

    expect(diagnostic.nearDuplicatePairs).toContainEqual({
      actorIds: ["qiao_ke", "behavior_clone"],
      score: 1,
    });
    expect(diagnostic.hardIssues).toEqual([]);
  });

  it("checks exact selected-12 constraints separately from pool diversity", () => {
    const selected = seedActors.slice(0, 12).map((actor) => actor.id);
    expect(
      diagnoseActorSelection({ actors: seedActors, actorIds: selected }),
    ).toMatchObject({
      selectedActorCount: 12,
      uniqueActorCount: 12,
      hardIssues: [],
    });
    expect(
      diagnoseActorSelection({
        actors: seedActors,
        actorIds: [...selected.slice(0, 11), selected[0]!],
      }).hardIssues,
    ).toContain("A Lineup cannot select the same Actor more than once");
  });
});
