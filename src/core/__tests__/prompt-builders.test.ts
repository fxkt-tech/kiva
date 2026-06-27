import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import type { GameEvent } from "../events";
import { createSeedGame } from "../game";
import { buildPlayerLlmContext } from "../player-context";
import { buildSpeechPrompt, SPEECH_PROMPT_VERSION } from "../prompt-builders";
import type { DraftId, EventId, GameId } from "../types";

const gameId = "game_1" as GameId;
const createdAt = "2026-06-26T00:00:00.000Z";
const game = createSeedGame({ gameId, createdAt });
const [wolf, , seer, , villager] = game.players;

describe("prompt builders", () => {
  it("builds speech prompts from visibility-safe player context", () => {
    const context = buildPlayerLlmContext({
      game,
      events: [
        roleAssigned(1, seer),
        {
          ...baseEvent(2),
          type: "phase_started",
          phase: "night",
          visibility: { kind: "public" },
          payload: { phase: "night", dayNumber: 1 },
        },
        {
          ...baseEvent(3),
          type: "seer_check_result",
          phase: "night",
          actorPlayerId: seer.playerId,
          targetPlayerIds: [wolf.playerId],
          visibility: { kind: "player_private", playerIds: [seer.playerId] },
          payload: { targetPlayerId: wolf.playerId, result: "wolves" },
        },
      ] satisfies readonly GameEvent[],
      viewerPlayerId: seer.playerId,
    });

    const prompt = buildSpeechPrompt({
      context,
      draft: speechDraft(),
    });
    const combined = [
      prompt.systemPrompt,
      ...prompt.messages.map((message) => message.content),
    ].join("\n");

    expect(prompt.promptVersion).toBe(SPEECH_PROMPT_VERSION);
    expect(prompt.schemaName).toBe("werewolf_speech_v1");
    expect(prompt.systemPrompt).toContain(seer.characterSystemPromptSnapshot);
    expect(prompt.systemPrompt).toContain(seer.roleSystemPromptSnapshot);
    expect(prompt.systemPrompt).toContain("你只能依据用户消息中列出的可见信息发言");
    expect(combined).toContain("短句、直接、有推进感");
    expect(combined).toContain("3 号 周知");
    expect(combined).toContain(`你的身份：${seer.roleName}`);
    expect(combined).toContain("查验结果");
    expect(combined).toContain("speech");
    expect(combined).toContain("必须输出 JSON 对象");
    expect(combined).toContain('"text"');
    expect(combined).not.toContain("1 号 秦川：狼人");
    expect(combined).not.toContain("5 号 陈墨：平民");
  });

  it("falls back to legacy viewer system prompt when prompt snapshots are empty", () => {
    const legacySystemPrompt = "旧局人设 prompt 仍要保留";
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
    const context = buildPlayerLlmContext({
      game: legacyGame,
      events: [roleAssigned(1, seer)],
      viewerPlayerId: seer.playerId,
    });

    const prompt = buildSpeechPrompt({
      context,
      draft: speechDraft(),
    });

    expect(prompt.systemPrompt).toContain(legacySystemPrompt);
    expect(prompt.systemPrompt).toContain("你只能依据用户消息中列出的可见信息发言");
  });
});

function speechDraft(): Extract<DraftEvent, { type: "day_speech_given" }> {
  return {
    id: "draft_1" as DraftId,
    gameId,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: seer.playerId,
    visibility: { kind: "public" },
    payload: {
      playerId: seer.playerId,
      text: "默认发言",
      dayNumber: 1,
      round: 1,
    },
    createdAt,
  };
}

function roleAssigned(
  index: number,
  player: typeof game.players[number],
): Extract<GameEvent, { type: "role_assigned" }> {
  return {
    ...baseEvent(index),
    type: "role_assigned",
    phase: "setup",
    targetPlayerIds: [player.playerId],
    visibility: { kind: "player_private", playerIds: [player.playerId] },
    payload: {
      playerId: player.playerId,
      role: player.gameRole,
      faction: player.faction,
    },
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
