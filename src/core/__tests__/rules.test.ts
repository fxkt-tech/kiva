import { describe, expect, it } from "vitest";
import { createPlayerSnapshot } from "../player";
import {
  checkWinCondition,
  createEndgameReveal,
  getEligibleVoters,
  getLegalNightTargets,
  resolveNightDeaths,
  resolveVote,
  validateWitchDecision,
} from "../rules";
import { createDefaultRuleset, type PlayerId } from "../types";

const p1 = "p1" as PlayerId;
const p2 = "p2" as PlayerId;
const p3 = "p3" as PlayerId;
const p4 = "p4" as PlayerId;
const p5 = "p5" as PlayerId;
const p6 = "p6" as PlayerId;
const p7 = "p7" as PlayerId;
const p8 = "p8" as PlayerId;
const unknownPlayerId = "unknown" as PlayerId;

const players = [
  createPlayerSnapshot({
    playerId: p1,
    seatNo: 1,
    name: "P1",
    gameRole: "werewolf",
  }),
  createPlayerSnapshot({
    playerId: p2,
    seatNo: 2,
    name: "P2",
    gameRole: "werewolf",
  }),
  createPlayerSnapshot({
    playerId: p3,
    seatNo: 3,
    name: "P3",
    gameRole: "seer",
  }),
  createPlayerSnapshot({
    playerId: p4,
    seatNo: 4,
    name: "P4",
    gameRole: "witch",
  }),
  createPlayerSnapshot({
    playerId: p5,
    seatNo: 5,
    name: "P5",
    gameRole: "villager",
  }),
  createPlayerSnapshot({
    playerId: p6,
    seatNo: 6,
    name: "P6",
    gameRole: "villager",
  }),
  createPlayerSnapshot({
    playerId: p7,
    seatNo: 7,
    name: "P7",
    gameRole: "guard",
  }),
  createPlayerSnapshot({
    playerId: p8,
    seatNo: 8,
    name: "P8",
    gameRole: "hunter",
  }),
];

describe("rules", () => {
  it("allows wolves to target living non-wolves", () => {
    expect(
      getLegalNightTargets("wolf_kill", players, [p1, p2, p3, p4, p5, p6], p1),
    ).toEqual([p3, p4, p5, p6]);
  });

  it("returns no wolf kill targets when actor is omitted or not a wolf", () => {
    expect(
      getLegalNightTargets("wolf_kill", players, [p1, p2, p3, p4], undefined),
    ).toEqual([]);
    expect(
      getLegalNightTargets("wolf_kill", players, [p1, p2, p3, p4], p3),
    ).toEqual([]);
  });

  it("allows seer to check living players except self", () => {
    expect(
      getLegalNightTargets("seer_check", players, [p1, p2, p3, p4], p3),
    ).toEqual([p1, p2, p4]);
  });

  it("allows witch to poison living players except self", () => {
    expect(
      getLegalNightTargets("witch_poison", players, [p1, p3, p4, p5], p4),
    ).toEqual([p1, p3, p5]);
  });

  it("allows guard to protect any living player including self", () => {
    expect(
      getLegalNightTargets("guard_protect", players, [p1, p3, p7], p7),
    ).toEqual([p1, p3, p7]);
  });

  it("allows a dead hunter to shoot any living player", () => {
    expect(
      getLegalNightTargets("hunter_shot", players, [p1, p3, p7], p8),
    ).toEqual([p1, p3, p7]);
  });

  it("returns no seer targets when actor is omitted", () => {
    expect(getLegalNightTargets("seer_check", players, [p1, p2, p3, p4])).toEqual(
      [],
    );
  });

  it("returns no seer targets when actor is unknown", () => {
    expect(
      getLegalNightTargets("seer_check", players, [p1, p2, p3, p4], unknownPlayerId),
    ).toEqual([]);
  });

  it("returns no seer targets when actor is dead", () => {
    expect(
      getLegalNightTargets("seer_check", players, [p1, p2, p4], p3),
    ).toEqual([]);
  });

  it("returns no seer targets when actor is not seer", () => {
    expect(
      getLegalNightTargets("seer_check", players, [p1, p2, p3, p4], p4),
    ).toEqual([]);
  });

  it("returns no witch poison targets when actor is invalid", () => {
    expect(
      getLegalNightTargets("witch_poison", players, [p1, p2, p3, p4], unknownPlayerId),
    ).toEqual([]);
  });

  it("returns no witch poison targets when actor is not witch", () => {
    expect(
      getLegalNightTargets("witch_poison", players, [p1, p2, p3, p4], p3),
    ).toEqual([]);
  });

  it("ignores unknown alive player ids when building targets", () => {
    expect(
      getLegalNightTargets("wolf_kill", players, [
        p1,
        p2,
        p3,
        unknownPlayerId,
      ], p1),
    ).toEqual([p3]);
  });

  it("kills wolf target without antidote", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        guardTargetId: null,
        antidoteTargetId: null,
        poisonTargetId: null,
      }),
    ).toEqual([p3]);
  });

  it("rescues wolf kill with antidote", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        guardTargetId: null,
        antidoteTargetId: p3,
        poisonTargetId: null,
      }),
    ).toEqual([]);
  });

  it("settles allowed same-night antidote and poison by rescuing the kill and applying poison", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        guardTargetId: null,
        antidoteTargetId: p3,
        poisonTargetId: p5,
      }),
    ).toEqual([p5]);
  });

  it("deduplicates a target killed by both wolves and poison", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        guardTargetId: null,
        antidoteTargetId: null,
        poisonTargetId: p3,
      }),
    ).toEqual([p3]);
  });

  it("rescues wolf kill with guard protection", () => {
    expect(
      resolveNightDeaths({
        wolfKillTargetId: p3,
        guardTargetId: p3,
        antidoteTargetId: null,
        poisonTargetId: null,
      }),
    ).toEqual([]);
  });

  it("rejects same-night antidote and poison by default", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 2,
          witchPlayerId: p4,
          killedPlayerId: p3,
          antidoteTargetId: p3,
          poisonTargetId: p5,
        },
        createDefaultRuleset(),
      ),
    ).toEqual({
      ok: false,
      reason: "same_night_antidote_and_poison_forbidden",
    });
  });

  it("accepts same-night antidote and poison when ruleset allows it", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 2,
          witchPlayerId: p4,
          killedPlayerId: p3,
          antidoteTargetId: p3,
          poisonTargetId: p5,
        },
        {
          ...createDefaultRuleset(),
          witchAllowSameNightAntidoteAndPoison: true,
        },
      ),
    ).toEqual({ ok: true });
  });

  it("rejects first-night self-save when ruleset forbids it", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 1,
          witchPlayerId: p4,
          killedPlayerId: p4,
          antidoteTargetId: p4,
          poisonTargetId: null,
        },
        {
          ...createDefaultRuleset(),
          witchFirstNightSelfSave: false,
        },
      ),
    ).toEqual({
      ok: false,
      reason: "first_night_self_save_forbidden",
    });
  });

  it("rejects antidote without death", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 2,
          witchPlayerId: p4,
          killedPlayerId: null,
          antidoteTargetId: p3,
          poisonTargetId: null,
        },
        createDefaultRuleset(),
      ),
    ).toEqual({
      ok: false,
      reason: "antidote_without_death",
    });
  });

  it("rejects antidote target mismatch", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 2,
          witchPlayerId: p4,
          killedPlayerId: p3,
          antidoteTargetId: p5,
          poisonTargetId: null,
        },
        createDefaultRuleset(),
      ),
    ).toEqual({
      ok: false,
      reason: "antidote_target_mismatch",
    });
  });

  it("treats an empty-string antidote target as present", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 2,
          witchPlayerId: p4,
          killedPlayerId: p3,
          antidoteTargetId: "" as PlayerId,
          poisonTargetId: null,
        },
        createDefaultRuleset(),
      ),
    ).toEqual({
      ok: false,
      reason: "antidote_target_mismatch",
    });
  });

  it("accepts valid antidote only", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 2,
          witchPlayerId: p4,
          killedPlayerId: p3,
          antidoteTargetId: p3,
          poisonTargetId: null,
        },
        createDefaultRuleset(),
      ),
    ).toEqual({ ok: true });
  });

  it("accepts valid poison only", () => {
    expect(
      validateWitchDecision(
        {
          nightNumber: 2,
          witchPlayerId: p4,
          killedPlayerId: p3,
          antidoteTargetId: null,
          poisonTargetId: p5,
        },
        createDefaultRuleset(),
      ),
    ).toEqual({ ok: true });
  });

  it("good team wins when all wolves are dead", () => {
    expect(checkWinCondition(players, [p1, p2], createDefaultRuleset())).toEqual({
      ended: true,
      winner: "good",
      reason: "all_wolves_dead",
    });
  });

  it("wolves win by slaughter all when all good players are dead", () => {
    expect(
      checkWinCondition(players, [p3, p4, p5, p6, p7, p8], {
        ...createDefaultRuleset(),
        winCondition: "slaughter_all",
      }),
    ).toEqual({
      ended: true,
      winner: "wolves",
      reason: "all_good_dead",
    });
  });

  it("wolves win by slaughter side when all gods are dead", () => {
    expect(checkWinCondition(players, [p3, p4, p7, p8], createDefaultRuleset())).toEqual({
      ended: true,
      winner: "wolves",
      reason: "all_gods_dead",
    });
  });

  it("wolves win by slaughter side when all villagers are dead", () => {
    expect(checkWinCondition(players, [p5, p6], createDefaultRuleset())).toEqual({
      ended: true,
      winner: "wolves",
      reason: "all_villagers_dead",
    });
  });

  it("does not end when both sides still have live win groups", () => {
    expect(checkWinCondition(players, [], createDefaultRuleset())).toEqual({
      ended: false,
    });
  });

  it("exiles the single highest vote target", () => {
    expect(
      resolveVote({
        votes: [
          { voterPlayerId: p1, targetPlayerId: p3 },
          { voterPlayerId: p2, targetPlayerId: p3 },
          { voterPlayerId: p3, targetPlayerId: p1 },
          { voterPlayerId: p4, targetPlayerId: null },
        ],
        allowAbstainVote: true,
      }),
    ).toEqual({
      exiledPlayerId: p3,
      tiedPlayerIds: [],
      voteTable: [
        { voterPlayerId: p1, targetPlayerId: p3 },
        { voterPlayerId: p2, targetPlayerId: p3 },
        { voterPlayerId: p3, targetPlayerId: p1 },
        { voterPlayerId: p4, targetPlayerId: null },
      ],
    });
  });

  it("returns tied highest targets in first-voted order", () => {
    expect(
      resolveVote({
        votes: [
          { voterPlayerId: p1, targetPlayerId: p5 },
          { voterPlayerId: p2, targetPlayerId: p6 },
          { voterPlayerId: p3, targetPlayerId: p6 },
          { voterPlayerId: p4, targetPlayerId: p5 },
        ],
        allowAbstainVote: true,
      }),
    ).toEqual({
      exiledPlayerId: null,
      tiedPlayerIds: [p5, p6],
      voteTable: [
        { voterPlayerId: p1, targetPlayerId: p5 },
        { voterPlayerId: p2, targetPlayerId: p6 },
        { voterPlayerId: p3, targetPlayerId: p6 },
        { voterPlayerId: p4, targetPlayerId: p5 },
      ],
    });
  });

  it("returns no exile and no tie when every vote abstains", () => {
    expect(
      resolveVote({
        votes: [
          { voterPlayerId: p1, targetPlayerId: null },
          { voterPlayerId: p2, targetPlayerId: null },
        ],
        allowAbstainVote: true,
      }),
    ).toEqual({
      exiledPlayerId: null,
      tiedPlayerIds: [],
      voteTable: [
        { voterPlayerId: p1, targetPlayerId: null },
        { voterPlayerId: p2, targetPlayerId: null },
      ],
    });
  });

  it("throws when abstain votes are forbidden", () => {
    expect(() =>
      resolveVote({
        votes: [
          { voterPlayerId: p1, targetPlayerId: p3 },
          { voterPlayerId: p2, targetPlayerId: null },
        ],
        allowAbstainVote: false,
      }),
    ).toThrow("Abstain votes are not allowed");
  });

  it("excludes pk players from eligible voters when pk voting is non-pk only", () => {
    expect(
      getEligibleVoters({
        alivePlayerIds: [p1, p2, p3, p4, p5],
        voteType: "pk",
        pkPlayerIds: [p2, p4],
        pkVoters: "non_pk_only",
      }),
    ).toEqual([p1, p3, p5]);
  });

  it("keeps all living players eligible for pk voting when self filtering is done by the candidate UI", () => {
    expect(
      getEligibleVoters({
        alivePlayerIds: [p1, p2, p3, p4, p5],
        voteType: "pk",
        pkPlayerIds: [p2, p4],
        pkVoters: "all_living_non_self",
      }),
    ).toEqual([p1, p2, p3, p4, p5]);
  });

  it("keeps every living player eligible for exile and sheriff votes", () => {
    const input = {
      alivePlayerIds: [p1, p2, p3],
      pkPlayerIds: [p2],
      pkVoters: "non_pk_only" as const,
    };

    expect(getEligibleVoters({ ...input, voteType: "exile" })).toEqual([
      p1,
      p2,
      p3,
    ]);
    expect(getEligibleVoters({ ...input, voteType: "sheriff" })).toEqual([
      p1,
      p2,
      p3,
    ]);
  });

  it("reveals every endgame role with localized role names", () => {
    expect(createEndgameReveal(players)).toEqual([
      { playerId: p1, roleId: "werewolf", roleName: "狼人", faction: "wolves" },
      { playerId: p2, roleId: "werewolf", roleName: "狼人", faction: "wolves" },
      { playerId: p3, roleId: "seer", roleName: "预言家", faction: "good" },
      { playerId: p4, roleId: "witch", roleName: "女巫", faction: "good" },
      { playerId: p5, roleId: "villager", roleName: "平民", faction: "good" },
      { playerId: p6, roleId: "villager", roleName: "平民", faction: "good" },
      { playerId: p7, roleId: "guard", roleName: "守卫", faction: "good" },
      { playerId: p8, roleId: "hunter", roleName: "猎人", faction: "good" },
    ]);
  });
});
