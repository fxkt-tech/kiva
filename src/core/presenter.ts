import type { DraftEvent } from "./drafts";
import type { GameEvent } from "./events";
import type { PlayerSnapshot } from "./player";
import type {
  GamePresenterSnapshot,
  PresenterCopyKey,
  PresenterDefinition,
  PresenterLine,
  PresenterSeatLine,
} from "./presenter-definition";
import type { GameRole, PlayerId } from "./types";

export type PresenterCue = {
  readonly copyKey: PresenterCopyKey;
  readonly text: string;
  readonly voiceFile: string | null;
};

export type PresenterResolution = {
  readonly presenterName: string;
  readonly presenterAvatar: string | null;
  readonly transcriptSpeaker: "presenter" | "player";
  readonly transcriptText: string;
  readonly cue: PresenterCue;
};

type EventLike = GameEvent | DraftEvent;

export function resolvePresenter(
  presenter: GamePresenterSnapshot,
  event: EventLike,
  players: readonly PlayerSnapshot[],
): PresenterResolution {
  if (
    event.type === "last_words_given" ||
    event.type === "day_speech_given" ||
    event.type === "pk_speech_given"
  ) {
    return resolveSpeech(presenter, event, players);
  }

  const request = presenterRequest(event, players);
  const cue = resolveCue(presenter, request.key, request.values);
  return {
    presenterName: presenter.name,
    presenterAvatar: presenter.avatar,
    transcriptSpeaker: "presenter",
    transcriptText: cue.text,
    cue,
  };
}

export function declaredPresenterVoiceFiles(
  definitions: readonly PresenterDefinition[],
): ReadonlySet<string> {
  const files = new Set<string>();
  for (const definition of definitions) {
    for (const line of Object.values(definition.lines)) {
      if ("voice" in line) {
        if (line.voice) {
          files.add(line.voice.file);
        }
        continue;
      }
      Object.values(line.voiceBySeat).forEach((voice) => {
        if (voice) {
          files.add(voice.file);
        }
      });
    }
  }
  return files;
}

function resolveSpeech(
  presenter: GamePresenterSnapshot,
  event: Extract<
    EventLike,
    { readonly type: "last_words_given" | "day_speech_given" | "pk_speech_given" }
  >,
  players: readonly PlayerSnapshot[],
): PresenterResolution {
  const player = players.find(
    (candidate) => candidate.playerId === event.payload.playerId,
  );
  const key = event.type === "last_words_given"
    ? "prompt.last_words"
    : event.type === "pk_speech_given"
      ? "prompt.pk"
      : "prompt.speech";
  const cue = resolveCue(
    presenter,
    key,
    { seatNo: String(player?.seatNo ?? "?") },
    player?.seatNo,
  );
  const authoredText = event.payload.text;
  if (authoredText.trim()) {
    return {
      presenterName: presenter.name,
      presenterAvatar: presenter.avatar,
      transcriptSpeaker: "player",
      transcriptText: authoredText,
      cue,
    };
  }

  const fallback = resolveCue(presenter, "fallback.speech", {});
  return {
    presenterName: presenter.name,
    presenterAvatar: presenter.avatar,
    transcriptSpeaker: "presenter",
    transcriptText: fallback.text,
    cue,
  };
}

function presenterRequest(
  event: Exclude<
    EventLike,
    { readonly type: "last_words_given" | "day_speech_given" | "pk_speech_given" }
  >,
  players: readonly PlayerSnapshot[],
): {
  readonly key: PresenterCopyKey;
  readonly values: Readonly<Record<string, string>>;
} {
  switch (event.type) {
    case "phase_started":
      return { key: phaseKey(event.payload.phase), values: {} };
    case "role_assigned":
      return {
        key: "action.role_assigned",
        values: {
          player: playerLabel(players, event.payload.playerId),
          role: roleLabel(event.payload.role),
        },
      };
    case "wolf_kill_selected":
      return targetRequest("action.wolf_kill", players, event.payload.targetPlayerId);
    case "seer_check_selected":
      return targetRequest("action.seer_check", players, event.payload.targetPlayerId);
    case "seer_check_result":
      return targetRequest(
        event.payload.result === "wolves"
          ? "action.seer_result.wolves"
          : "action.seer_result.good",
        players,
        event.payload.targetPlayerId,
      );
    case "witch_death_info_shown":
      return event.payload.killedPlayerId
        ? targetRequest(
            "action.witch_death_info.killed",
            players,
            event.payload.killedPlayerId,
          )
        : { key: "action.witch_death_info.safe", values: {} };
    case "witch_antidote_decided":
      return event.payload.used && event.payload.targetPlayerId
        ? targetRequest(
            "action.witch_antidote.used",
            players,
            event.payload.targetPlayerId,
          )
        : { key: "action.witch_antidote.skipped", values: {} };
    case "witch_poison_decided":
      return event.payload.used && event.payload.targetPlayerId
        ? targetRequest(
            "action.witch_poison.used",
            players,
            event.payload.targetPlayerId,
          )
        : { key: "action.witch_poison.skipped", values: {} };
    case "guard_protect_selected":
      return targetRequest(
        "action.guard_protect",
        players,
        event.payload.targetPlayerId,
      );
    case "hunter_shot_decided":
      return targetRequest(
        "action.hunter_shot",
        players,
        event.payload.targetPlayerId,
      );
    case "night_resolved":
      return event.payload.deadPlayerIds.length
        ? {
            key: "resolution.night.deaths",
            values: {
              players: playerListLabel(players, event.payload.deadPlayerIds),
            },
          }
        : { key: "resolution.night.safe", values: {} };
    case "death_announced":
      return event.payload.deadPlayerIds.length
        ? {
            key: "announcement.deaths",
            values: {
              players: playerListLabel(players, event.payload.deadPlayerIds),
            },
          }
        : { key: "announcement.safe", values: {} };
    case "vote_cast":
      return event.payload.targetPlayerId
        ? {
            key: "vote.cast",
            values: {
              voter: playerLabel(players, event.payload.voterPlayerId),
              target: playerLabel(players, event.payload.targetPlayerId),
            },
          }
        : {
            key: "vote.abstain",
            values: {
              voter: playerLabel(players, event.payload.voterPlayerId),
            },
          };
    case "exile_resolved": {
      const prefix = event.payload.voteType === "pk" ? "resolution.pk" : "resolution.exile";
      if (event.payload.exiledPlayerId) {
        return {
          key: `${prefix}.exiled` as PresenterCopyKey,
          values: {
            player: playerLabel(players, event.payload.exiledPlayerId),
          },
        };
      }
      if (event.payload.tiedPlayerIds.length) {
        return {
          key: `${prefix}.tied` as PresenterCopyKey,
          values: {
            players: playerListLabel(players, event.payload.tiedPlayerIds),
          },
        };
      }
      return { key: `${prefix}.none` as PresenterCopyKey, values: {} };
    }
    case "game_ended":
      return { key: `game_end.${event.payload.reason}`, values: {} };
  }
}

function phaseKey(phase: EventLike["phase"]): PresenterCopyKey {
  return `phase.${phase}` as PresenterCopyKey;
}

function targetRequest(
  key: PresenterCopyKey,
  players: readonly PlayerSnapshot[],
  targetPlayerId: PlayerId,
) {
  return {
    key,
    values: { target: playerLabel(players, targetPlayerId) },
  };
}

function resolveCue(
  presenter: GamePresenterSnapshot,
  key: PresenterCopyKey,
  values: Readonly<Record<string, string>>,
  seatNo?: number,
): PresenterCue {
  const line = presenter.lines[key];
  const voice = voiceForLine(line, seatNo);
  return {
    copyKey: key,
    text: renderTemplate(line.template, values),
    voiceFile: voice?.status === "ready" ? voice.file : null,
  };
}

function voiceForLine(line: PresenterLine, seatNo?: number) {
  if ("voice" in line) {
    return line.voice;
  }
  return seatNo === undefined
    ? null
    : (line as PresenterSeatLine).voiceBySeat[String(seatNo)] ?? null;
}

function renderTemplate(
  template: string,
  values: Readonly<Record<string, string>>,
): string {
  return template.replace(/\{([a-zA-Z][a-zA-Z0-9]*)\}/g, (_, key: string) => {
    const value = values[key];
    if (value === undefined) {
      throw new Error(`Missing presenter template value: ${key}`);
    }
    return value;
  });
}

function playerLabel(
  players: readonly PlayerSnapshot[],
  playerId: PlayerId,
): string {
  const player = players.find((candidate) => candidate.playerId === playerId);
  return player ? `${player.seatNo}号${player.name}` : `未知玩家${playerId}`;
}

function playerListLabel(
  players: readonly PlayerSnapshot[],
  playerIds: readonly PlayerId[],
): string {
  return playerIds.map((playerId) => playerLabel(players, playerId)).join("、");
}

function roleLabel(role: GameRole): string {
  switch (role) {
    case "werewolf":
      return "狼人";
    case "seer":
      return "预言家";
    case "witch":
      return "女巫";
    case "hunter":
      return "猎人";
    case "guard":
      return "守卫";
    case "villager":
      return "平民";
  }
}
