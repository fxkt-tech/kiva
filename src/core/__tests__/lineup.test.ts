import { describe, expect, it } from "vitest";
import type { ActorDefinition } from "../actor-definition";
import { validateLineups, type Lineup } from "../lineup";

const roleIds = [
  "villager",
  "seer",
  "werewolf",
  "witch",
  "villager",
  "werewolf",
  "guard",
  "hunter",
  "werewolf",
  "villager",
  "werewolf",
  "villager",
] as const;

function actors(): readonly ActorDefinition[] {
  return Array.from({ length: 12 }, (_, index) => ({
    id: index === 0 ? "qin_chuan" : `actor_${index + 1}`,
    identity: {
      name: index === 0 ? "秦川" : `演员${index + 1}`,
      portrait: `/actor-${index + 1}.png`,
      tags: ["测试"],
      visualAnchor: "轮廓",
    },
    core: {
      stableCore: "稳定核心",
      drive: "驱动力",
      blindSpot: "盲点",
      changeBoundary: "变化边界",
    },
    cognition: {
      attention: "注意入口",
      evidencePolicy: "证据策略",
      decisionPolicy: "决策策略",
      correctionTrigger: "修正条件",
    },
    interaction: {
      tableFunction: "群像功能",
      socialStrategy: "互动策略",
      pressureResponse: "压力反应",
      conflictAxes: ["测试轴"],
    },
    expression: {
      cadence: "节奏",
      diction: "用词",
      rhetoricalMoves: ["动作"],
      avoid: ["禁项"],
    },
    production: {
      modelBinding: {
        provider: "local",
        model: "test",
        responseFormat: "json" as const,
      },
      voice: {
        provider: "edge" as const,
        adapterVersion: 1 as const,
        voice: "zh-CN-XiaoxiaoNeural",
        lang: "zh-CN",
        pitch: "+0Hz",
        rate: "+0%",
        volume: "+0%",
      },
    },
    enabled: true,
    revision: 1,
  }));
}

function lineup(overrides: Partial<Lineup> = {}): Lineup {
  return {
    id: "standard",
    name: "标准阵容",
    rulesetId: "classic_twelve",
    seats: roleIds.map((ruleRoleId, index) => ({
      seatNo: index + 1,
      ruleRoleId,
      actorId: actors()[index]!.id,
    })),
    enabled: true,
    revision: 1,
    ...overrides,
  };
}

describe("Lineup", () => {
  it("owns one exact 12-seat truth source", () => {
    const value = [lineup()];
    expect(validateLineups(value, actors())).toBe(value);
    expect(value[0]).not.toHaveProperty("roleIds");
    expect(value[0]).not.toHaveProperty("characterIds");
    expect(value[0]?.seats[0]).not.toHaveProperty("modelBindingOverride");
  });

  it("rejects duplicate Actors and invalid Rule Role composition", () => {
    const duplicateActor = lineup({
      seats: lineup().seats.map((seat, index) =>
        index === 1 ? { ...seat, actorId: lineup().seats[0]!.actorId } : seat,
      ),
    });
    expect(() => validateLineups([duplicateActor], actors())).toThrow(
      "duplicates Actor",
    );

    const wrongRoles = lineup({
      seats: lineup().seats.map((seat, index) =>
        index === 0 ? { ...seat, ruleRoleId: "werewolf" } : seat,
      ),
    });
    expect(() => validateLineups([wrongRoles], actors())).toThrow(
      "must contain 4 werewolf",
    );
  });
});
