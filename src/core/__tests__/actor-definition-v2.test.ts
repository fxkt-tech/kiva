import { describe, expect, it } from "vitest";
import {
  assertActorLibraryInvariants,
  compileActorAuthorCard,
  compileActorRuntimeCard,
  createActorSnapshot,
  validateActorDefinitions,
  validateActorSnapshot,
  type ActorDefinition,
} from "../actor-definition";

function actor(
  id = "qin_chuan",
  name = "秦川",
  overrides: Partial<ActorDefinition> = {},
): ActorDefinition {
  return {
    id,
    identity: {
      name,
      portrait: `/kivdb-assets/characters/${id}.png`,
      tags: ["全局建模"],
      visualAnchor: "冷青轮廓光和细框眼镜",
    },
    core: {
      stableCore: "先拼接信息结构，再给出有边界的判断。",
      drive: "让讨论回到真正影响局势的分歧。",
      blindSpot: "证据稀疏时容易过度建模。",
      changeBoundary: "可以修正模型，但不会伪装成凭直觉下结论的人。",
    },
    cognition: {
      attention: "关注信息流、口径变化和行动收益。",
      evidencePolicy: "区分事实、主张与推断，并标出置信边界。",
      decisionPolicy: "比较多个解释后选择可继续验证的方案。",
      correctionTrigger: "新事实能排除当前模型时立即重建。",
    },
    interaction: {
      tableFunction: "信息整合",
      socialStrategy: "用关键问题重定义争论。",
      pressureResponse: "被质疑时先公开模型前提。",
      conflictAxes: ["模型-反例"],
    },
    expression: {
      cadence: "语速偏慢，先条件后结论。",
      diction: "压缩、精确、少形容词。",
      rhetoricalMoves: ["列出关键前提", "提出验证问题"],
      avoid: ["装作全知", "重复整段时间线"],
    },
    production: {
      modelBinding: {
        provider: "volcengine",
        model: "doubao-seed-2-1-turbo-260628",
        responseFormat: "json",
      },
      voice: {
        provider: "edge",
        adapterVersion: 1,
        voice: "zh-CN-YunxiNeural",
        lang: "zh-CN",
        pitch: "-4Hz",
        rate: "+0%",
        volume: "+0%",
      },
    },
    enabled: true,
    revision: 1,
    ...overrides,
  };
}

describe("Actor Definition v2", () => {
  it("validates exact structured Actors and preserves 秦川", () => {
    const actors = [actor(), actor("qiao_ke", "乔可")];
    expect(validateActorDefinitions(actors)).toBe(actors);
    expect(() =>
      validateActorDefinitions([
        actor("qin_chuan", "秦川二号"),
      ]),
    ).toThrow('Stable Actor must remain exactly "秦川" / qin_chuan');
    expect(() =>
      validateActorDefinitions([actor(), actor("impostor", "秦川")]),
    ).toThrow('Stable Actor must remain exactly "秦川" / qin_chuan');
    expect(() => assertActorLibraryInvariants([actor("qiao_ke", "乔可")])).toThrow(
      'Actor library must contain exactly one "秦川" / qin_chuan',
    );
  });

  it("rejects legacy prose fields and named Actor dependencies", () => {
    expect(() =>
      validateActorDefinitions([
        { ...actor(), persona: "legacy" },
      ]),
    ).toThrow("unknown: persona");

    expect(() =>
      validateActorDefinitions([
        actor(),
        actor("qiao_ke", "乔可", {
          interaction: {
            ...actor().interaction,
            socialStrategy: "专门追问秦川",
          },
        }),
      ]),
    ).toThrow("must not reference Actor qin_chuan");
  });

  it("snapshots production configuration and compiles bounded projections", () => {
    const definition = actor();
    const snapshot = createActorSnapshot(definition);
    (definition.identity.tags as string[])[0] = "mutated";

    expect(validateActorSnapshot(snapshot)).toEqual(snapshot);
    expect(snapshot.identity.tags).toEqual(["全局建模"]);
    expect(compileActorRuntimeCard(snapshot)).not.toHaveProperty("production");
    expect(compileActorRuntimeCard(snapshot)).not.toHaveProperty("blindSpot");
    expect(compileActorAuthorCard(snapshot)).not.toHaveProperty("production");
    expect(compileActorAuthorCard(snapshot)).not.toHaveProperty("expression");
    expect(compileActorAuthorCard(snapshot).tableFunction).toBe("信息整合");
  });
});
