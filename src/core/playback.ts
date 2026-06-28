import { getActiveEvents } from "./event-log";
import { formatEventForPublic } from "./event-presenter";
import type { GameEvent } from "./events";
import type { PlayerSnapshot } from "./player";
import type { Phase, PlayerId } from "./types";

export type PlaybackSceneKind =
  | "phase"
  | "announcement"
  | "speech"
  | "vote"
  | "resolution";

export type PlaybackScenePlayer = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly status: "alive" | "dead";
  readonly highlighted: boolean;
};

export type PlaybackRhythmConfig = {
  readonly phaseMs: number;
  readonly announcementMs: number;
  readonly deathAnnouncementMs: number;
  readonly speechBaseMs: number;
  readonly speechMsPerCharacter: number;
  readonly speechMinMs: number;
  readonly speechMaxMs: number;
  readonly voteMs: number;
  readonly resolutionMs: number;
  readonly gameEndMs: number;
};

export type CompilePublicPlaybackOptions = {
  readonly rhythm?: Partial<PlaybackRhythmConfig>;
};

export type PlaybackItem = {
  readonly index: number;
  readonly phase: Phase;
  readonly kind: PlaybackSceneKind;
  readonly title: string;
  readonly text: string;
  readonly details: readonly string[];
  readonly durationMs: number;
  readonly startsAtMs: number;
  readonly players: readonly PlaybackScenePlayer[];
};

const defaultRhythm: PlaybackRhythmConfig = {
  phaseMs: 1600,
  announcementMs: 2200,
  deathAnnouncementMs: 2200,
  speechBaseMs: 3600,
  speechMsPerCharacter: 60,
  speechMinMs: 4200,
  speechMaxMs: 12000,
  voteMs: 1200,
  resolutionMs: 2400,
  gameEndMs: 5000,
};

export function compilePublicPlayback(
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
  options: CompilePublicPlaybackOptions = {},
): readonly PlaybackItem[] {
  let startsAtMs = 0;
  const publiclyDeadPlayerIds = new Set<PlayerId>();
  const rhythm = { ...defaultRhythm, ...options.rhythm };

  return getActiveEvents(events)
    .filter((event) => event.visibility.kind === "public")
    .filter((event) => event.type !== "vote_cast")
    .flatMap((event) => {
      const presented = formatEventForPublic(event, players);
      if (!presented) {
        return [];
      }

      updatePublicDeaths(publiclyDeadPlayerIds, event);

      const durationMs = durationForEvent(event, presented.text, rhythm);
      const scene = {
        index: event.index,
        phase: event.phase,
        kind: kindForEvent(event),
        title: presented.title,
        text: presented.text,
        details: presented.details ?? [],
        durationMs,
        startsAtMs,
        players: playersForScene(players, publiclyDeadPlayerIds, event),
      };
      startsAtMs += durationMs;

      return [scene];
    });
}

export function playbackTotalDurationMs(
  items: readonly PlaybackItem[],
): number {
  const lastItem = items.at(-1);
  return lastItem ? lastItem.startsAtMs + lastItem.durationMs : 0;
}

export function playbackIndexAtMs(
  items: readonly PlaybackItem[],
  timeMs: number,
): number {
  if (items.length === 0) {
    return 0;
  }

  const clampedTimeMs = Math.max(0, timeMs);
  const index = items.findIndex(
    (item) =>
      clampedTimeMs >= item.startsAtMs &&
      clampedTimeMs < item.startsAtMs + item.durationMs,
  );

  return index === -1 ? items.length - 1 : index;
}

function kindForEvent(event: GameEvent): PlaybackSceneKind {
  switch (event.type) {
    case "phase_started":
      return "phase";
    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      return "speech";
    case "vote_cast":
      return "vote";
    case "exile_resolved":
    case "game_ended":
    case "night_resolved":
      return "resolution";
    case "death_announced":
      return "announcement";
    default:
      return "announcement";
  }
}

function updatePublicDeaths(
  deadPlayerIds: Set<PlayerId>,
  event: GameEvent,
): void {
  switch (event.type) {
    case "death_announced":
    case "night_resolved":
      event.payload.deadPlayerIds.forEach((playerId) => {
        deadPlayerIds.add(playerId);
      });
      return;
    case "exile_resolved":
      if (event.payload.exiledPlayerId) {
        deadPlayerIds.add(event.payload.exiledPlayerId);
      }
      return;
    default:
      return;
  }
}

function playersForScene(
  players: readonly PlayerSnapshot[],
  deadPlayerIds: ReadonlySet<PlayerId>,
  event: GameEvent,
): readonly PlaybackScenePlayer[] {
  const highlightedPlayerIds = highlightedPlayersForEvent(event);

  return players.map((player) => ({
    playerId: player.playerId,
    seatNo: player.seatNo,
    name: player.name,
    status: deadPlayerIds.has(player.playerId) ? "dead" : "alive",
    highlighted: highlightedPlayerIds.has(player.playerId),
  }));
}

function highlightedPlayersForEvent(event: GameEvent): ReadonlySet<PlayerId> {
  const playerIds = new Set<PlayerId>();

  if (event.actorPlayerId) {
    playerIds.add(event.actorPlayerId);
  }

  event.targetPlayerIds?.forEach((playerId) => {
    playerIds.add(playerId);
  });

  switch (event.type) {
    case "death_announced":
    case "night_resolved":
      event.payload.deadPlayerIds.forEach((playerId) => {
        playerIds.add(playerId);
      });
      break;
    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      playerIds.add(event.payload.playerId);
      break;
    case "vote_cast":
      playerIds.add(event.payload.voterPlayerId);
      if (event.payload.targetPlayerId) {
        playerIds.add(event.payload.targetPlayerId);
      }
      break;
    case "exile_resolved":
      if (event.payload.exiledPlayerId) {
        playerIds.add(event.payload.exiledPlayerId);
      }
      event.payload.tiedPlayerIds.forEach((playerId) => {
        playerIds.add(playerId);
      });
      break;
  }

  return playerIds;
}

function durationForEvent(
  event: GameEvent,
  text: string,
  rhythm: PlaybackRhythmConfig,
): number {
  switch (event.type) {
    case "phase_started":
      return rhythm.phaseMs;
    case "death_announced":
      return rhythm.deathAnnouncementMs;
    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      return clamp(
        rhythm.speechBaseMs + text.length * rhythm.speechMsPerCharacter,
        rhythm.speechMinMs,
        rhythm.speechMaxMs,
      );
    case "vote_cast":
      return rhythm.voteMs;
    case "exile_resolved":
      return rhythm.resolutionMs;
    case "game_ended":
      return rhythm.gameEndMs;
    default:
      return rhythm.announcementMs;
  }
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
