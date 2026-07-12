import { getActiveEvents } from "./event-log";
import { formatEventForHost, formatEventForPublic } from "./event-presenter";
import type { GameEndReason, GameEvent, NightDeathReason } from "./events";
import type { PlayerSnapshot } from "./player";
import type { GamePresenterSnapshot } from "./presenter-definition";
import { resolvePresenter } from "./presenter";
import type { Phase, PlayerId } from "./types";
import type { PlayerVoiceArtifact, SpeechCue } from "./voice";
import {
  resolvePresenterVoiceClips,
  type PresenterVoiceManifest,
  type ResolvedPresenterVoiceClip,
} from "./presenter-voice";

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
  readonly avatar: string | null;
  readonly roleName: string;
  readonly status: "alive" | "dead";
  readonly highlighted: boolean;
};

export type StageAction =
  | "protect"
  | "attack"
  | "inspect"
  | "antidote"
  | "poison";

export type StageActionResult =
  | "selected"
  | "good"
  | "wolves"
  | "used"
  | "skipped"
  | "unresolved";

export type StagePresentation =
  | {
      readonly kind: "action";
      readonly action: StageAction;
      readonly actor:
        | { readonly kind: "player"; readonly playerId: PlayerId }
        | { readonly kind: "wolves" };
      readonly targetPlayerId: PlayerId | null;
      readonly result: StageActionResult;
    }
  | {
      readonly kind: "night_result";
      readonly deaths: readonly {
        readonly playerId: PlayerId;
        readonly reason: NightDeathReason | null;
      }[];
    }
  | {
      readonly kind: "vote_result";
      readonly voteType: "exile" | "pk";
      readonly outcome: "exiled" | "tied" | "no_exile";
      readonly exiledPlayerId: PlayerId | null;
      readonly candidates: readonly {
        readonly playerId: PlayerId;
        readonly votes: number;
      }[];
      readonly abstentions: number;
    }
  | {
      readonly kind: "game_result";
      readonly winner: "wolves" | "good";
      readonly reason: GameEndReason;
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
  readonly presenter: GamePresenterSnapshot;
  readonly rhythm?: Partial<PlaybackRhythmConfig>;
  readonly audience?: "public" | "director";
  readonly durationForScene?: (scene: PlaybackItem, event: GameEvent) => number | null;
  readonly voiceArtifactsByEventId?: Readonly<Record<string, PlayerVoiceArtifact>>;
  readonly presenterVoiceManifest?: PresenterVoiceManifest;
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
  readonly presenterName: string;
  readonly presenterAvatar: string | null;
  readonly transcriptSpeaker: "presenter" | "player";
  readonly presenterCue: {
    readonly copyKey: string;
    readonly text: string;
  };
  readonly playerVoice?: {
    readonly eventId: string;
    readonly playerId: PlayerId;
    readonly file: string;
    readonly durationMs: number;
    readonly startsAtOffsetMs: number;
    readonly cues: readonly SpeechCue[];
  } | null;
  readonly presenterVoiceClips?: readonly ResolvedPresenterVoiceClip[];
  readonly presenterSourceId?: string;
  readonly stage?: StagePresentation | null;
};

export const VOICE_TIMING = {
  presenterToPlayerMs: 250,
  playerTailMs: 350,
  presenterClipGapMs: 80,
} as const;

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
  options: CompilePublicPlaybackOptions,
): readonly PlaybackItem[] {
  const publiclyDeadPlayerIds = new Set<PlayerId>();
  const rhythm = { ...defaultRhythm, ...options.rhythm };
  const audience = options.audience ?? "public";

  const scenes = getActiveEvents(events)
    .filter((event) => shouldIncludeEvent(event, audience))
    .flatMap((event) => {
      const presented = audience === "director"
        ? formatEventForHost(event, players)
        : formatEventForPublic(event, players);
      if (!presented) {
        return [];
      }
      const presenter = resolvePresenter(options.presenter, event, players);
      const artifact = options.voiceArtifactsByEventId?.[event.id] ?? null;
      const presenterVoiceClips = options.presenterVoiceManifest
        ? resolvePresenterVoiceClips(options.presenterVoiceManifest, presenter.cue)
        : [];
      const presenterPlanDurationMs = presenterVoiceClips.reduce(
        (total, clip, index) =>
          total + clip.durationMs + (index === 0 ? 0 : VOICE_TIMING.presenterClipGapMs),
        0,
      );
      const playerVoice =
        presenter.transcriptSpeaker === "player" && artifact
          ? {
              eventId: event.id,
              playerId: artifact.playerId,
              file: artifact.audio.file,
              durationMs: artifact.audio.durationMs,
              startsAtOffsetMs:
                presenterPlanDurationMs +
                VOICE_TIMING.presenterToPlayerMs,
              cues: artifact.cues,
            }
          : null;

      updatePublicDeaths(publiclyDeadPlayerIds, event);

      const scene = {
        index: event.index,
        phase: event.phase,
        kind: kindForEvent(event),
        title: presented.title,
        text: presenter.transcriptText,
        details: presented.details ?? [],
        durationMs: durationForEvent(event, presenter.transcriptText, rhythm),
        startsAtMs: 0,
        players: playersForScene(players, publiclyDeadPlayerIds, event),
        presenterName: presenter.presenterName,
        presenterAvatar: presenter.presenterAvatar,
        transcriptSpeaker: presenter.transcriptSpeaker,
        presenterCue: presenter.cue,
        playerVoice,
        presenterVoiceClips,
        presenterSourceId: options.presenter.presenterSourceId,
        stage: stagePresentationForEvent(event),
      };
      const durationMs = playerVoice
        ? playerVoice.startsAtOffsetMs +
          playerVoice.durationMs +
          VOICE_TIMING.playerTailMs
        : options.durationForScene?.(scene, event) ??
          (presenterPlanDurationMs || scene.durationMs);

      return [{ ...scene, durationMs }];
    });

  return withTimelineStarts(scenes);
}

function withTimelineStarts(
  scenes: readonly PlaybackItem[],
): readonly PlaybackItem[] {
  let startsAtMs = 0;

  return scenes.map((scene) => {
    const item = { ...scene, startsAtMs };
    startsAtMs += scene.durationMs;
    return item;
  });
}

function shouldIncludeEvent(
  event: GameEvent,
  audience: NonNullable<CompilePublicPlaybackOptions["audience"]>,
): boolean {
  if (
    event.type === "vote_cast" ||
    event.type === "wolf_leader_selected" ||
    event.type === "wolf_vote_cast" ||
    event.type === "role_assigned"
  ) {
    return false;
  }

  if (event.type === "night_resolved") {
    return audience === "director";
  }

  return audience === "director" || event.visibility.kind === "public";
}

export function stagePresentationForEvent(
  event: GameEvent,
): StagePresentation | null {
  switch (event.type) {
    case "guard_protect_selected":
      return playerAction(event, "protect", event.payload.targetPlayerId, "selected");
    case "wolf_vote_resolved":
      return {
        kind: "action",
        action: "attack",
        actor: { kind: "wolves" },
        targetPlayerId: event.payload.targetPlayerId,
        result: event.payload.targetPlayerId ? "selected" : "unresolved",
      };
    case "seer_check_selected":
      return playerAction(event, "inspect", event.payload.targetPlayerId, "selected");
    case "seer_check_result":
      return playerAction(
        event,
        "inspect",
        event.payload.targetPlayerId,
        event.payload.result,
      );
    case "witch_antidote_decided":
      return playerAction(
        event,
        "antidote",
        event.payload.targetPlayerId,
        event.payload.used ? "used" : "skipped",
      );
    case "witch_poison_decided":
      return playerAction(
        event,
        "poison",
        event.payload.targetPlayerId,
        event.payload.used ? "used" : "skipped",
      );
    case "night_resolved": {
      const reasons = new Map(
        event.payload.deaths?.map((death) => [death.playerId, death.reason]),
      );
      return {
        kind: "night_result",
        deaths: event.payload.deadPlayerIds.map((playerId) => ({
          playerId,
          reason: reasons.get(playerId) ?? null,
        })),
      };
    }
    case "death_announced":
      return {
        kind: "night_result",
        deaths: event.payload.deadPlayerIds.map((playerId) => ({
          playerId,
          reason: null,
        })),
      };
    case "exile_resolved": {
      const tallies = new Map<PlayerId, number>();
      let abstentions = 0;
      for (const vote of event.payload.voteTable) {
        if (vote.targetPlayerId) {
          tallies.set(
            vote.targetPlayerId,
            (tallies.get(vote.targetPlayerId) ?? 0) + 1,
          );
        } else {
          abstentions += 1;
        }
      }
      const candidates = [...tallies.entries()]
        .map(([playerId, votes]) => ({ playerId, votes }))
        .sort((left, right) =>
          right.votes - left.votes || left.playerId.localeCompare(right.playerId),
        );
      return {
        kind: "vote_result",
        voteType: event.payload.voteType === "pk" ? "pk" : "exile",
        outcome: event.payload.exiledPlayerId
          ? "exiled"
          : event.payload.tiedPlayerIds.length > 0 ? "tied" : "no_exile",
        exiledPlayerId: event.payload.exiledPlayerId,
        candidates,
        abstentions,
      };
    }
    case "game_ended":
      return {
        kind: "game_result",
        winner: event.payload.winner,
        reason: event.payload.reason,
      };
    default:
      return null;
  }
}

function playerAction(
  event: GameEvent,
  action: StageAction,
  targetPlayerId: PlayerId | null,
  result: StageActionResult,
): StagePresentation | null {
  return event.actorPlayerId
    ? {
        kind: "action",
        action,
        actor: { kind: "player", playerId: event.actorPlayerId },
        targetPlayerId,
        result,
      }
    : null;
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
    case "wolf_strategy_given":
    case "wolf_opinion_given":
      return "speech";
    case "vote_cast":
      return "vote";
    case "exile_resolved":
    case "game_ended":
    case "night_resolved":
    case "wolf_vote_resolved":
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
    case "hunter_shot_decided":
      deadPlayerIds.add(event.payload.targetPlayerId);
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
    avatar: player.avatar,
    roleName: player.roleName,
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
    case "hunter_shot_decided":
      playerIds.add(event.payload.targetPlayerId);
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
