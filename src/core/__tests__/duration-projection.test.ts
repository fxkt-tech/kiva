import { describe, expect, it } from "vitest";
import type { GameEvent } from "../events";
import { projectDirectorDuration } from "../duration-projection";
import type { PlaybackItem } from "../playback";
import type { EventId, GameId, PlayerId } from "../types";

const gameId = "duration_game" as GameId;
const playerId = "duration_player" as PlayerId;

describe("director duration projection", () => {
  it("replaces actual speech audio with the shared target interval", () => {
    const event = daySpeechEvent();
    const projection = projectDirectorDuration({
      events: [event],
      playback: [playbackItem()],
    });

    expect(projection).toMatchObject({
      committedDurationMs: 21_000,
      speechCount: 1,
      speechCharacters: 4,
      actualSpeechDurationMs: 20_000,
      missingSpeechVoiceCount: 0,
      projectedMinDurationMs: 17_394,
      projectedMaxDurationMs: 23_951,
    });
  });
});

function daySpeechEvent(): Extract<GameEvent, { type: "day_speech_given" }> {
  return {
    id: "duration_event" as EventId,
    gameId,
    index: 1,
    status: "active",
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: playerId,
    visibility: { kind: "public" },
    payload: { playerId, text: "测试发言", dayNumber: 1, round: 1 },
    createdAt: "2026-07-12T00:00:00.000Z",
  };
}

function playbackItem(): PlaybackItem {
  return {
    index: 1,
    phase: "speech",
    kind: "speech",
    title: "发言",
    text: "测试发言",
    details: [],
    durationMs: 21_000,
    startsAtMs: 0,
    players: [],
    presenterName: "主理人",
    presenterAvatar: null,
    transcriptSpeaker: "player",
    presenterCue: { copyKey: "speech", text: "请发言", values: {} },
    playerVoice: {
      eventId: "duration_event",
      playerId,
      file: "voice.mp3",
      durationMs: 20_000,
      startsAtOffsetMs: 650,
      cues: [],
    },
    presenterVoiceClips: [],
    presenterSourceId: "host",
    stage: null,
  };
}
