import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { MockLlmClient } from "../llm";
import { generateActionDraft } from "../action-generation";
import type { DraftId, EventId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const [wolf, , seer, witch, villager] = game.players;

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
        schemaName: "werewolf_action_v1",
        systemPrompt: expect.stringContaining("行动建议"),
        messages: expect.arrayContaining([
          expect.objectContaining({
            role: "user",
            content: expect.stringContaining("可选目标"),
          }),
        ]),
      },
      parsedOutput: {
        targetPlayerId: wolf.playerId,
        reasoning: "优先查验发言和站边最可疑的人。",
      },
    });
    expect(result.generation?.request?.systemPrompt).toContain(
      seer.characterSystemPromptSnapshot,
    );
    expect(result.generation?.request?.systemPrompt).toContain(
      seer.roleSystemPromptSnapshot,
    );
    expect(result.generation?.request?.systemPrompt).toContain(
      seer.roleActionPromptSnapshot,
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      `你的身份：${seer.roleName}`,
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "mechanic=seer_check",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "可选目标",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "可见事件",
    );
    expect(result.generation?.request?.messages[0]?.content).toContain(
      "reasoning",
    );
  });

  it("falls back to legacy system prompt for action prompts when snapshots are empty", async () => {
    const legacySystemPrompt = "旧局行动人设 prompt";
    const legacyGame = {
      ...game,
      players: game.players.map((player) =>
        player.playerId === seer.playerId
          ? {
              ...player,
              characterSystemPromptSnapshot: "",
              roleSystemPromptSnapshot: "",
              roleActionPromptSnapshot: null,
              systemPrompt: legacySystemPrompt,
            }
          : player,
      ),
    };

    const result = await generateActionDraft({
      game: legacyGame,
      events: setupEvents(),
      draft: seerDraft(villager.playerId),
      llmClient: new MockLlmClient([{ targetPlayerId: wolf.playerId }]),
      generationId: "generation_legacy",
      createdAt,
    });

    expect(result.generation?.request?.systemPrompt).toContain(legacySystemPrompt);
    expect(result.generation?.request?.systemPrompt).toContain("行动建议");
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

    expect(content).toContain("#6 5 号 陈墨发言：我觉得 1 号发言像狼人。");
    expect(content).toContain("#7 投票结算：平票：1 号 秦川、5 号 陈墨。");
    expect(content).toContain("  - 5 号 陈墨 -> 1 号 秦川");
    expect(content).toContain("  - 1 号 秦川 -> 5 号 陈墨");
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

    expect(content).toContain(`1 秦川 playerId=${wolf.playerId} role=werewolf faction=wolves`);
    expect(content).toContain(`2 林夏 playerId=${game.players[1]?.playerId} role=werewolf faction=wolves`);
    expect(content).toContain(`3 周知 playerId=${seer.playerId}`);
    expect(content).not.toContain(`3 周知 playerId=${seer.playerId} role=seer`);
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
      error: "Player cannot perform wolf_kill_selected",
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
        schemaName: "werewolf_action_v1",
        messages: expect.any(Array),
      },
      error: "Illegal targetPlayerId for wolf_kill_selected",
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
});

function setupEvents(): readonly GameEvent[] {
  return [
    roleAssigned(1, wolf.playerId, "werewolf", "wolves"),
    roleAssigned(2, seer.playerId, "seer", "good"),
    roleAssigned(3, witch.playerId, "witch", "good"),
    roleAssigned(4, villager.playerId, "villager", "good"),
    {
      ...baseEvent(5),
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
    ...draftBase("wolf_kill_selected"),
    phase: "night",
    actorPlayerId: wolf.playerId,
    targetPlayerIds: [targetPlayerId],
    visibility: { kind: "faction_private", faction: "wolves" },
    payload: { targetPlayerId },
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
  role: "werewolf" | "seer" | "witch" | "villager",
  faction: "wolves" | "good",
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

function baseEvent(index: number) {
  return {
    id: `event_${index}` as EventId,
    gameId,
    index,
    status: "active",
    createdAt: `2026-06-26T00:0${index}:00.000Z`,
  } as const;
}
