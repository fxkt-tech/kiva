import type { GameEvent } from "./events";
import type { PlaybackItem } from "./playback";
import {
  estimateSpeechDurationMs,
  speechBudgetForKey,
  spokenCharacterCount,
  type SpeechBudgetKey,
} from "./speech-budget";

export type DirectorDurationProjection = {
  readonly committedDurationMs: number;
  readonly projectedMinDurationMs: number;
  readonly projectedMaxDurationMs: number;
  readonly speechCount: number;
  readonly speechCharacters: number;
  readonly actualSpeechDurationMs: number;
  readonly missingSpeechVoiceCount: number;
};

export function projectDirectorDuration(input: {
  readonly events: readonly GameEvent[];
  readonly playback: readonly PlaybackItem[];
}): DirectorDurationProjection {
  const eventByIndex = new Map(input.events.map((event) => [event.index, event]));
  let projectedMinDurationMs = 0;
  let projectedMaxDurationMs = 0;
  let speechCount = 0;
  let speechCharacters = 0;
  let actualSpeechDurationMs = 0;
  let missingSpeechVoiceCount = 0;

  for (const item of input.playback) {
    const event = eventByIndex.get(item.index);
    const budgetKey = event ? speechBudgetKeyForEvent(event, input.events) : null;
    if (!event || !budgetKey || !("text" in event.payload)) {
      projectedMinDurationMs += item.durationMs;
      projectedMaxDurationMs += item.durationMs;
      continue;
    }

    const budget = speechBudgetForKey(budgetKey);
    const actualVoiceMs = item.playerVoice?.durationMs ?? 0;
    const sceneWithoutPlayerVoiceMs = item.playerVoice
      ? item.durationMs - actualVoiceMs
      : 600;

    speechCount += 1;
    speechCharacters += spokenCharacterCount(event.payload.text);
    actualSpeechDurationMs += actualVoiceMs;
    if (!item.playerVoice) missingSpeechVoiceCount += 1;
    projectedMinDurationMs +=
      sceneWithoutPlayerVoiceMs +
      estimateSpeechDurationMs(budget.targetMinCharacters);
    projectedMaxDurationMs +=
      sceneWithoutPlayerVoiceMs +
      estimateSpeechDurationMs(budget.targetMaxCharacters);
  }

  return {
    committedDurationMs: playbackDurationMs(input.playback),
    projectedMinDurationMs,
    projectedMaxDurationMs,
    speechCount,
    speechCharacters,
    actualSpeechDurationMs,
    missingSpeechVoiceCount,
  };
}

function speechBudgetKeyForEvent(
  event: GameEvent,
  events: readonly GameEvent[],
): SpeechBudgetKey | null {
  switch (event.type) {
    case "wolf_strategy_given":
      return "wolf_strategy";
    case "wolf_opinion_given":
      return "wolf_opinion";
    case "last_words_given":
      return "last_words";
    case "pk_speech_given":
      return "pk";
    case "day_speech_given":
      return events.some(
        (candidate) =>
          candidate.index < event.index &&
          candidate.type === "day_speech_given" &&
          candidate.payload.dayNumber === event.payload.dayNumber,
      )
        ? "day_response"
        : "day_first";
    default:
      return null;
  }
}

function playbackDurationMs(playback: readonly PlaybackItem[]): number {
  const lastItem = playback.at(-1);
  return lastItem ? lastItem.startsAtMs + lastItem.durationMs : 0;
}
