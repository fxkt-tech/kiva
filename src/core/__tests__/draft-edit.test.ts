import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import { applyDraftPayloadEdit } from "../draft-edit";
import type { DraftId, GameId, PlayerId } from "../types";

const gameId = "game_1" as GameId;
const draftId = "draft_1" as DraftId;
const player1 = "player_1" as PlayerId;
const player2 = "player_2" as PlayerId;
const player3 = "player_3" as PlayerId;
const createdAt = "2026-06-26T00:00:00.000Z";

describe("applyDraftPayloadEdit", () => {
  it("edits target draft payloads and targetPlayerIds", () => {
    const edited = applyDraftPayloadEdit(wolfKillDraft(player1), {
      targetPlayerId: player2,
    });

    expect(edited.payload).toEqual({ targetPlayerId: player2 });
    expect(edited.targetPlayerIds).toEqual([player2]);
    expect(edited.actorPlayerId).toBe(player1);
  });

  it("rejects unsupported fields for target-only drafts", () => {
    expect(() =>
      applyDraftPayloadEdit(seerCheckDraft(player1), {
        targetPlayerId: player2,
        text: "ignored?",
      }),
    ).toThrow("Unsupported draft edit for seer_check_selected");
  });

  it("edits witch medicine use and normalizes unused target", () => {
    const edited = applyDraftPayloadEdit(witchAntidoteDraft(player1), {
      used: false,
      targetPlayerId: player2,
    });

    expect(edited.payload).toEqual({ used: false, targetPlayerId: null });
    expect(edited.targetPlayerIds).toEqual([]);
  });

  it("requires a target when witch medicine is used", () => {
    expect(() =>
      applyDraftPayloadEdit(witchPoisonDraft(player1), {
        used: true,
        targetPlayerId: null,
      }),
    ).toThrow("targetPlayerId is required when witch_poison_decided is used");
  });

  it("edits text drafts while preserving contextual payload fields", () => {
    const edited = applyDraftPayloadEdit(daySpeechDraft(player1), {
      text: "  我听后置位发言。  ",
    });

    expect(edited.payload).toEqual({
      playerId: player1,
      text: "我听后置位发言。",
      dayNumber: 2,
      round: 1,
    });
  });

  it("allows empty trimmed text", () => {
    const edited = applyDraftPayloadEdit(lastWordsDraft(player1), {
      text: "   ",
    });

    expect(edited.payload).toMatchObject({ text: "" });
    expect(edited.targetPlayerIds).toEqual([player1]);
  });

  it("edits vote target and treats null as abstain", () => {
    const edited = applyDraftPayloadEdit(voteCastDraft(player1, player2), {
      targetPlayerId: null,
    });

    expect(edited.payload).toEqual({
      voterPlayerId: player1,
      targetPlayerId: null,
      dayNumber: 1,
      round: 1,
      voteType: "exile",
    });
    expect(edited.targetPlayerIds).toEqual([]);
  });

  it("rejects unsupported draft types", () => {
    expect(() =>
      applyDraftPayloadEdit(roleAssignedDraft(player1), {
        targetPlayerId: player2,
      }),
    ).toThrow("Unsupported draft edit for role_assigned");
  });
});

function baseDraft(overrides: Partial<DraftEvent>): DraftEvent {
  return {
    id: draftId,
    gameId,
    status: "draft",
    phase: "night",
    visibility: { kind: "public" },
    payload: {},
    createdAt,
    ...overrides,
  } as DraftEvent;
}

function wolfKillDraft(actorPlayerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "wolf_kill_selected",
    actorPlayerId,
    targetPlayerIds: [player2],
    payload: { targetPlayerId: player2 },
  });
}

function seerCheckDraft(actorPlayerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "seer_check_selected",
    actorPlayerId,
    targetPlayerIds: [player2],
    payload: { targetPlayerId: player2 },
  });
}

function witchAntidoteDraft(actorPlayerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "witch_antidote_decided",
    actorPlayerId,
    targetPlayerIds: [],
    payload: { used: false, targetPlayerId: null },
  });
}

function witchPoisonDraft(actorPlayerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "witch_poison_decided",
    actorPlayerId,
    targetPlayerIds: [],
    payload: { used: false, targetPlayerId: null },
  });
}

function lastWordsDraft(playerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "last_words_given",
    phase: "last_words",
    actorPlayerId: playerId,
    targetPlayerIds: [playerId],
    payload: {
      playerId,
      text: "我的遗言先到这里。",
      dayNumber: 1,
      reason: "night_death",
    },
  });
}

function daySpeechDraft(playerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: playerId,
    payload: {
      playerId,
      text: "我先给出自己的判断。",
      dayNumber: 2,
      round: 1,
    },
  });
}

function voteCastDraft(voterPlayerId: PlayerId, targetPlayerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "vote_cast",
    phase: "vote",
    actorPlayerId: voterPlayerId,
    targetPlayerIds: [targetPlayerId],
    payload: {
      voterPlayerId,
      targetPlayerId,
      dayNumber: 1,
      round: 1,
      voteType: "exile",
    },
  });
}

function roleAssignedDraft(playerId: PlayerId): DraftEvent {
  return baseDraft({
    type: "role_assigned",
    phase: "setup",
    actorPlayerId: playerId,
    targetPlayerIds: [playerId],
    payload: { playerId, role: "werewolf", faction: "wolves" },
  });
}
