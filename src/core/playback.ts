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

export function compilePublicPlayback(
  events: readonly GameEvent[],
  players: readonly PlayerSnapshot[],
): readonly PlaybackItem[] {
  let startsAtMs = 0;
  const publiclyDeadPlayerIds = new Set<PlayerId>();

  return getActiveEvents(events)
    .filter((event) => event.visibility.kind === "public")
    .flatMap((event) => {
      const presented = formatEventForPublic(event, players);
      if (!presented) {
        return [];
      }

      updatePublicDeaths(publiclyDeadPlayerIds, event);

      const durationMs = durationForEvent(event);
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

function durationForEvent(event: GameEvent): number {
  switch (event.type) {
    case "phase_started":
      return 1600;
    case "death_announced":
      return 2200;
    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      return 4200;
    case "vote_cast":
      return 1200;
    case "exile_resolved":
      return 2400;
    case "game_ended":
      return 5000;
    default:
      return 2200;
  }
}
