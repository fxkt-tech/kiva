import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { MockLlmClient } from "../llm";
import { generateActionDraft } from "../action-generation";
import type { DraftId, EventId, Faction, GameId, GameRole, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const wolf = playerByRole("werewolf");
const secondWolf = game.players.find(
  (player) => player.gameRole === "werewolf" && player.playerId !== wolf.playerId,
)!;
const seer = playerByRole("seer");
const witch = playerByRole("witch");
const villager = playerByRole("villager");

describe("action generation", () => {
  it("applies legal seer target suggestions", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: seerDraft(villager.playerId),
      llmClient: new MockLlmClient([
        {
          targetPlayerId: wolf.playerId,
          reasoning: "优先查验发言和站边最可疑的人。",
        },
      ]),
      generationId: "generation_1",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "seer_check_selected",
      targetPlayerIds: [wolf.playerId],
      payload: { targetPlayerId: wolf.playerId },
    });
    expect(result.generation).toMatchObject({
      status: "success",
      purpose: "action",
      request: {
        schemaName: "werewolf_target_action_v2",
        systemPrompt: expect.stringContaining("【执行优先级】"),
        messages: expect.arrayContaining([
          expect.objectContaining({
            role: "user",
            content: expect.stringContaining("【合法候选】"),
          }),
        ]),
      },
      parsedOutput: {
        targetPlayerId: wolf.playerId,
        reasoning: "优先查验发言和站边最可疑的人。",
      },
    });
    expect(result.generation?.request?.systemPrompt).toContain(
      seer.persona,
    );
    expect(result.generation?.request?.systemPrompt).toContain(
      seer.roleSystemPromptSnapshot,
    );
    expect(result.generation?.request?.systemPrompt).toContain(
      seer.roleActionPromptSnapshot,
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      `你的身份是 ${seer.roleName}`,
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "预言家选择本夜查验目标",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "【仅与本轮有关的规则】",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "预言家每夜查验一名其他存活玩家",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "【合法候选】",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "【已确认的公开事实】",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "decisionSummary",
    );
  });

  it("includes visible event text and details in action prompts", async () => {
    const result = await generateActionDraft({
      game,
      events: [
        ...setupEvents(),
        {
          ...baseEvent(6),
          type: "day_speech_given",
          phase: "speech",
          actorPlayerId: villager.playerId,
          targetPlayerIds: [villager.playerId],
          visibility: { kind: "public" },
          payload: {
            playerId: villager.playerId,
            text: "我觉得 1 号发言像狼人。",
            dayNumber: 1,
            round: 1,
          },
        },
        {
          ...baseEvent(7),
          type: "exile_resolved",
          phase: "vote",
          targetPlayerIds: [],
          visibility: { kind: "public" },
          payload: {
            voteType: "exile",
            dayNumber: 1,
            round: 1,
            exiledPlayerId: null,
            tiedPlayerIds: [wolf.playerId, villager.playerId],
            voteTable: [
              {
                voterPlayerId: villager.playerId,
                targetPlayerId: wolf.playerId,
              },
              {
                voterPlayerId: wolf.playerId,
                targetPlayerId: villager.playerId,
              },
            ],
            revealedRoles: [],
          },
        },
      ],
      draft: voteDraft(villager.playerId),
      llmClient: new MockLlmClient([{ targetPlayerId: null }]),
      generationId: "generation_5",
      createdAt,
    });

    const content = result.generation?.request?.messages[0]?.content ?? "";

    expect(content).toContain(
      `${villager.seatNo} 号 ${villager.name}发言：我觉得 1 号发言像狼人。`,
    );
    expect(content).toContain(
      `投票结算：平票：${wolf.seatNo} 号 ${wolf.name}、${villager.seatNo} 号 ${villager.name}。`,
    );
    expect(content).toContain(
      `  - ${villager.seatNo} 号 ${villager.name} -> ${wolf.seatNo} 号 ${wolf.name}`,
    );
    expect(content).toContain(
      `  - ${wolf.seatNo} 号 ${wolf.name} -> ${villager.seatNo} 号 ${villager.name}`,
    );
    expect(content).not.toContain("#6");
    expect(content).not.toContain("#7");
    expect(content).not.toContain("reasoning 是给主理人看的");
  });

  it("keeps safe visible role information in action prompts", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: wolfDraft(villager.playerId),
      llmClient: new MockLlmClient([{ targetPlayerId: seer.playerId }]),
      generationId: "generation_roster",
      createdAt,
    });

    const content = result.generation?.request?.messages[0]?.content ?? "";

    expect(content).toContain(
      `${wolf.playerId} | ${wolf.seatNo} 号 | ${wolf.name} | 你；狼人，狼人阵营`,
    );
    expect(content).toContain(
      `${secondWolf.playerId} | ${secondWolf.seatNo} 号 | ${secondWolf.name} | 已知狼人队友；狼人，狼人阵营`,
    );
    expect(content).toContain(`${seer.playerId} | ${seer.seatNo} 号 | ${seer.name}`);
    expect(content).not.toContain(
      `${seer.playerId} | ${seer.seatNo} 号 | ${seer.name} | 预言家`,
    );
  });

  it("rejects action drafts whose actor cannot use the draft mechanic", async () => {
    const draft = {
      ...wolfDraft(villager.playerId),
      actorPlayerId: seer.playerId,
    } satisfies DraftEvent;

    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft,
      llmClient: new MockLlmClient([{ targetPlayerId: villager.playerId }]),
      generationId: "generation_bad_actor",
      createdAt,
    });

    expect(result.draft).toEqual(draft);
    expect(result.generation).toMatchObject({
      status: "failed",
      purpose: "action",
      error: "Player cannot perform wolf_vote_cast",
    });
  });

  it("rejects illegal wolf kill targets and keeps draft unchanged", async () => {
    const draft = wolfDraft(villager.playerId);
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft,
      llmClient: new MockLlmClient([{ targetPlayerId: "missing" }]),
      generationId: "generation_2",
      createdAt,
    });

    expect(result.draft).toEqual(draft);
    expect(result.generation).toMatchObject({
      status: "failed",
      purpose: "action",
      request: {
        schemaName: "werewolf_target_action_v2",
        messages: expect.any(Array),
      },
      error: "Illegal targetPlayerId for wolf_vote_cast",
      attempts: [
        { error: "Illegal targetPlayerId for wolf_vote_cast" },
        { error: "Illegal targetPlayerId for wolf_vote_cast" },
      ],
    });
  });

  it("repairs one illegal action target using the shared legal options", async () => {
    const draft = wolfDraft(villager.playerId);
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft,
      llmClient: new MockLlmClient([
        { targetPlayerId: "missing" },
        {
          targetPlayerId: villager.playerId,
          decisionSummary: "改为合法候选。",
        },
      ]),
      generationId: "generation_repaired",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "wolf_vote_cast",
      payload: { targetPlayerId: villager.playerId },
    });
    expect(result.generation).toMatchObject({
      status: "success",
      attempts: [
        { error: "Illegal targetPlayerId for wolf_vote_cast" },
        {
          request: {
            messages: [
              {
                content: expect.stringContaining(villager.playerId),
              },
            ],
          },
          error: null,
        },
      ],
    });
  });

  it("applies vote abstain suggestions", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: voteDraft(villager.playerId),
      llmClient: new MockLlmClient([{ targetPlayerId: null }]),
      generationId: "generation_3",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "vote_cast",
      targetPlayerIds: [],
      payload: { targetPlayerId: null },
    });
  });

  it("repairs a vote response that omits the required target field", async () => {
    const draft = voteDraft(villager.playerId);
    const invalidOutput = { decisionSummary: "想弃票但漏掉了字段。" };
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft,
      llmClient: new MockLlmClient([invalidOutput, invalidOutput]),
      generationId: "generation_missing_vote_target",
      createdAt,
    });

    expect(result.draft).toEqual(draft);
    expect(result.generation).toMatchObject({
      status: "failed",
      error: "targetPlayerId is required for vote_cast",
      attempts: [
        { error: "targetPlayerId is required for vote_cast" },
        { error: "targetPlayerId is required for vote_cast" },
      ],
    });
  });

  it("applies witch medicine suggestions", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: witchPoisonDraft(),
      llmClient: new MockLlmClient([
        { used: true, targetPlayerId: wolf.playerId },
      ]),
      generationId: "generation_4",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "witch_poison_decided",
      targetPlayerIds: [wolf.playerId],
      payload: { used: true, targetPlayerId: wolf.playerId },
    });
  });

  it("accepts an antidote summary that describes the attacked player as good", async () => {
    const rescuedPlayerId = villager.playerId;
    const events = [
      ...setupEvents(),
      {
        ...baseEvent(7),
        type: "wolf_vote_resolved",
        phase: "night",
        targetPlayerIds: [rescuedPlayerId],
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: {
          votes: [],
          tallies: [],
          tiedTargetPlayerIds: [rescuedPlayerId],
          targetPlayerId: rescuedPlayerId,
          resolution: "majority",
          dayNumber: 1,
        },
      },
      {
        ...baseEvent(8),
        type: "witch_death_info_shown",
        phase: "night",
        actorPlayerId: witch.playerId,
        targetPlayerIds: [rescuedPlayerId],
        visibility: { kind: "player_private", playerIds: [witch.playerId] },
        payload: { killedPlayerId: rescuedPlayerId },
      },
    ] satisfies readonly GameEvent[];
    const result = await generateActionDraft({
      game,
      events,
      draft: witchAntidoteDraft(rescuedPlayerId),
      llmClient: new MockLlmClient([
        {
          used: true,
          targetPlayerId: rescuedPlayerId,
          decisionSummary:
            "首夜确认8号唐棠被袭击，为避免好人阵营潜在损失，决定使用解药救其性命。",
        },
      ]),
      generationId: "generation_antidote_fact_repair",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      type: "witch_antidote_decided",
      payload: { used: true, targetPlayerId: rescuedPlayerId },
    });
    expect(result.generation).toMatchObject({
      status: "success",
    });
    expect(result.generation).not.toHaveProperty("attempts");
  });

  it("requires explicit used output for witch medicine suggestions", async () => {
    const draft = witchPoisonDraft();
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft,
      llmClient: new MockLlmClient([
        { targetPlayerId: wolf.playerId, reasoning: "想使用毒药" },
      ]),
      generationId: "generation_missing_used",
      createdAt,
    });

    expect(result.draft).toEqual(draft);
    expect(result.generation).toMatchObject({
      status: "failed",
      purpose: "action",
      error: "used is required for witch_poison_decided",
    });
  });

  it("rejects a target when witch output says the medicine was not used", async () => {
    const draft = witchPoisonDraft();
    const invalidOutput = {
      used: false,
      targetPlayerId: wolf.playerId,
      decisionSummary: "字段互相矛盾。",
    };
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft,
      llmClient: new MockLlmClient([invalidOutput, invalidOutput]),
      generationId: "generation_conflicting_witch_output",
      createdAt,
    });

    expect(result.draft).toEqual(draft);
    expect(result.generation).toMatchObject({
      status: "failed",
      error:
        "targetPlayerId must be null when used=false for witch_poison_decided",
    });
  });

  it("tells witch medicine drafts to output used explicitly", async () => {
    const result = await generateActionDraft({
      game,
      events: setupEvents(),
      draft: witchPoisonDraft(),
      llmClient: new MockLlmClient([
        { used: true, targetPlayerId: wolf.playerId },
      ]),
      generationId: "generation_witch_prompt",
      createdAt,
    });

    const content = result.generation?.request?.messages[0]?.content ?? "";

    expect(content).toContain("used");
    expect(content).toContain('"used":true');
    expect(content).toContain('"used":false');
    expect(content).toContain("decisionSummary");
  });

  it("generates a sealed wolf ballot from discussion without exposing earlier ballots", async () => {
    const events = [
      ...setupEvents(),
      {
        ...baseEvent(7),
        type: "wolf_strategy_given",
        phase: "night",
        actorPlayerId: wolf.playerId,
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: { playerId: wolf.playerId, text: "优先寻找神职。", dayNumber: 1 },
      },
      {
        ...baseEvent(8),
        type: "wolf_vote_cast",
        phase: "night",
        actorPlayerId: wolf.playerId,
        visibility: { kind: "host_only" },
        payload: { voterPlayerId: wolf.playerId, targetPlayerId: seer.playerId, dayNumber: 1 },
      },
    ] satisfies readonly GameEvent[];
    const result = await generateActionDraft({
      game,
      events,
      draft: wolfVoteDraft(villager.playerId),
      llmClient: new MockLlmClient([{ targetPlayerId: seer.playerId }]),
      generationId: "generation_wolf_vote",
      createdAt,
    });
    const content = result.generation?.request?.messages[0]?.content ?? "";

    expect(result.draft).toMatchObject({
      type: "wolf_vote_cast",
      payload: { targetPlayerId: seer.playerId },
    });
    expect(content).toContain("优先寻找神职");
    expect(content).not.toContain("狼人密票");
  });

  it("accepts wolf ballot wording without semantic keyword validation", async () => {
    const events = [
      ...setupEvents(),
      {
        ...baseEvent(7),
        type: "wolf_strategy_given",
        phase: "night",
        actorPlayerId: wolf.playerId,
        visibility: { kind: "faction_private", faction: "wolves" },
        payload: {
          playerId: wolf.playerId,
          text: "先统一投 8 号，但关于其身份的理由没有证据。",
          dayNumber: 1,
        },
      },
    ] satisfies readonly GameEvent[];
    const result = await generateActionDraft({
      game,
      events,
      draft: wolfVoteDraft(villager.playerId),
      llmClient: new MockLlmClient([
        {
          targetPlayerId: villager.playerId,
          decisionSummary: "8 号一直沉默，是疑似神职。",
        },
      ]),
      generationId: "generation_wolf_ballot_fact_repair",
      createdAt,
    });

    expect(result.draft).toMatchObject({
      payload: { targetPlayerId: villager.playerId },
    });
    expect(result.generation).toMatchObject({
      status: "success",
    });
    expect(result.generation).not.toHaveProperty("attempts");
  });
});

function setupEvents(): readonly GameEvent[] {
  return [
    roleAssigned(1, wolf.playerId, "werewolf", "wolves"),
    roleAssigned(2, secondWolf.playerId, "werewolf", "wolves"),
    roleAssigned(3, seer.playerId, "seer", "good"),
    roleAssigned(4, witch.playerId, "witch", "good"),
    roleAssigned(5, villager.playerId, "villager", "good"),
    {
      ...baseEvent(6),
      type: "phase_started",
      phase: "night",
      visibility: { kind: "public" },
      payload: { phase: "night", dayNumber: 1 },
    },
  ] satisfies readonly GameEvent[];
}

function seerDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("seer_check_selected"),
    phase: "night",
    actorPlayerId: seer.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "player_private", playerIds: [seer.playerId] },
    payload: { targetPlayerId },
  } as DraftEvent;
}

function wolfDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("wolf_vote_cast"),
    phase: "night",
    actorPlayerId: wolf.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "host_only" },
    payload: { voterPlayerId: wolf.playerId, targetPlayerId, dayNumber: 1 },
  } as DraftEvent;
}

function wolfVoteDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("wolf_vote_cast"),
    phase: "night",
    actorPlayerId: secondWolf.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "host_only" },
    payload: { voterPlayerId: secondWolf.playerId, targetPlayerId, dayNumber: 1 },
  } as DraftEvent;
}

function voteDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("vote_cast"),
    phase: "vote",
    actorPlayerId: villager.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "public" },
    payload: {
      voterPlayerId: villager.playerId,
      targetPlayerId,
      dayNumber: 1,
      round: 1,
      voteType: "exile",
    },
  } as DraftEvent;
}

function witchPoisonDraft(): DraftEvent {
  return {
    ...draftBase("witch_poison_decided"),
    phase: "night",
    actorPlayerId: witch.playerId,
    targetPlayerIds: [],
    visibility: { kind: "player_private", playerIds: [witch.playerId] },
    payload: { used: false, targetPlayerId: null },
  } as DraftEvent;
}

function witchAntidoteDraft(targetPlayerId: PlayerId): DraftEvent {
  return {
    ...draftBase("witch_antidote_decided"),
    phase: "night",
    actorPlayerId: witch.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "player_private", playerIds: [witch.playerId] },
    payload: { used: false, targetPlayerId: null },
  } as DraftEvent;
}

function draftBase(type: DraftEvent["type"]) {
  return {
    id: "draft_1" as DraftId,
    gameId,
    status: "draft",
    type,
    createdAt,
  } as const;
}

function roleAssigned(
  index: number,
  playerId: PlayerId,
  role: GameRole,
  faction: Faction,
): Extract<GameEvent, { type: "role_assigned" }> {
  return {
    ...baseEvent(index),
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [playerId],
    visibility: { kind: "player_private", playerIds: [playerId] },
    payload: { playerId, role, faction },
  };
}

function playerByRole(role: GameRole) {
  const player = game.players.find((candidate) => candidate.gameRole === role);
  if (!player) {
    throw new Error(`Missing player for role ${role}`);
  }

  return player;
}

function baseEvent(index: number) {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active",
    createdAt: `2026-06-26T00:0${index}:00.000Z`,
  } as const;
}
