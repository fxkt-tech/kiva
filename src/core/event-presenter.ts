import type { DraftEvent } from "./drafts";
import type {
  EventVisibility,
  GameEndReason,
  GameEvent,
  VoteType,
} from "./events";
import type { PlayerSnapshot } from "./player";
import type { Faction, GameRole, Phase, PlayerId } from "./types";

export type PresentedEvent = {
  readonly title: string;
  readonly text: string;
  readonly details?: readonly string[];
};

export function formatEventForHost(
  event: GameEvent,
  players: readonly PlayerSnapshot[],
): PresentedEvent {
  return formatEvent(event, players);
}

export function formatDraftForHost(
  draft: DraftEvent,
  players: readonly PlayerSnapshot[],
): PresentedEvent {
  const presented = formatEvent(draft, players);
  return {
    ...presented,
    title: `待确认：${presented.title}`,
  };
}

export function formatEventForPublic(
  event: GameEvent,
  players: readonly PlayerSnapshot[],
): PresentedEvent | null {
  if (event.visibility.kind !== "public") {
    return null;
  }

  return formatEvent(event, players);
}

function formatEvent(
  event: GameEvent | DraftEvent,
  players: readonly PlayerSnapshot[],
): PresentedEvent {
  switch (event.type) {
    case "phase_started":
      return {
        title: phaseStartedTitle(event.payload.phase, event.payload.dayNumber),
        text: phaseStartedText(event.payload.phase),
      };

    case "role_assigned":
      return {
        title: "身份牌",
        text: `${playerLabel(players, event.payload.playerId)} 获得身份：${roleLabel(event.payload.role)}。`,
      };

    case "wolf_kill_selected":
      return {
        title: "狼人刀人",
        text: `${actorPrefix(players, event.actorPlayerId)}选择击杀 ${playerLabel(players, event.payload.targetPlayerId)}。`,
      };

    case "seer_check_selected":
      return {
        title: "预言家查验",
        text: `${actorPrefix(players, event.actorPlayerId)}选择查验 ${playerLabel(players, event.payload.targetPlayerId)}。`,
      };

    case "seer_check_result":
      return {
        title: "查验结果",
        text: `${actorPrefix(players, event.actorPlayerId)}查验 ${playerLabel(players, event.payload.targetPlayerId)}，结果：${factionLabel(event.payload.result)}。`,
      };

    case "witch_death_info_shown":
      return {
        title: "女巫死亡信息",
        text: event.payload.killedPlayerId
          ? `${actorPrefix(players, event.actorPlayerId)}得知夜间被袭击：${playerLabel(players, event.payload.killedPlayerId)}。`
          : `${actorPrefix(players, event.actorPlayerId)}得知夜间无人被袭击。`,
      };

    case "witch_antidote_decided":
      return {
        title: "女巫解药",
        text:
          event.payload.used && event.payload.targetPlayerId
            ? `${actorPrefix(players, event.actorPlayerId)}使用解药救 ${playerLabel(players, event.payload.targetPlayerId)}。`
            : `${actorPrefix(players, event.actorPlayerId)}不使用解药。`,
      };

    case "witch_poison_decided":
      return {
        title: "女巫毒药",
        text:
          event.payload.used && event.payload.targetPlayerId
            ? `${actorPrefix(players, event.actorPlayerId)}使用毒药毒 ${playerLabel(players, event.payload.targetPlayerId)}。`
            : `${actorPrefix(players, event.actorPlayerId)}不使用毒药。`,
      };

    case "night_resolved":
      return {
        title: "夜间结算",
        text:
          event.payload.deadPlayerIds.length > 0
            ? `夜间死亡：${playerListLabel(players, event.payload.deadPlayerIds)}。`
            : "夜间无人死亡。",
      };

    case "death_announced":
      return {
        title: "昨夜死讯",
        text:
          event.payload.deadPlayerIds.length > 0
            ? `昨夜死亡：${playerListLabel(players, event.payload.deadPlayerIds)}。`
            : "昨夜平安夜，没有玩家死亡。",
      };

    case "last_words_given":
      return {
        title: `${playerLabel(players, event.payload.playerId)}遗言`,
        text: event.payload.text,
      };

    case "day_speech_given":
      return {
        title: `${playerLabel(players, event.payload.playerId)}发言`,
        text: event.payload.text,
      };

    case "pk_speech_given":
      return {
        title: `${playerLabel(players, event.payload.playerId)}PK 发言`,
        text: event.payload.text,
      };

    case "vote_cast":
      return {
        title: voteTitle(event.payload.voteType),
        text: `${playerLabel(players, event.payload.voterPlayerId)} 投给 ${event.payload.targetPlayerId ? playerLabel(players, event.payload.targetPlayerId) : "弃票"}。`,
      };

    case "exile_resolved":
      return {
        title: event.payload.voteType === "pk" ? "PK 结算" : "投票结算",
        text: exileResolutionText(
          players,
          event.payload.exiledPlayerId,
          event.payload.tiedPlayerIds,
        ),
        details: event.payload.voteTable.map(
          (vote) =>
            `${playerLabel(players, vote.voterPlayerId)} -> ${
              vote.targetPlayerId
                ? playerLabel(players, vote.targetPlayerId)
                : "弃票"
            }`,
        ),
      };

    case "game_ended":
      return {
        title: `游戏结束：${winnerLabel(event.payload.winner)}胜利`,
        text: `${winnerLabel(event.payload.winner)}胜利，原因：${winReasonLabel(event.payload.reason)}。`,
        details: event.payload.revealedRoles.map(
          (role) =>
            `${playerLabel(players, role.playerId)}：${role.roleName}（${factionLabel(role.faction)}）`,
        ),
      };
  }
}

export function formatVisibility(visibility: EventVisibility): string {
  switch (visibility.kind) {
    case "public":
      return "public";
    case "host_only":
      return "host only";
    case "player_private":
      return `private ${visibility.playerIds.length}`;
    case "faction_private":
      return `${visibility.faction} private`;
    case "custom":
      return `custom ${visibility.playerIds.length}`;
  }
}

function phaseStartedTitle(phase: Phase, dayNumber: number): string {
  switch (phase) {
    case "night":
      return `第 ${dayNumber} 夜开始`;
    case "day":
      return `第 ${dayNumber} 天开始`;
    case "last_words":
      return "遗言阶段";
    case "speech":
      return "发言阶段";
    case "vote":
      return "放逐投票";
    case "pk":
      return "PK 阶段";
    case "setup":
      return "准备阶段";
    case "ended":
      return "游戏结束";
  }
}

function phaseStartedText(phase: Phase): string {
  switch (phase) {
    case "night":
      return "夜晚开始。";
    case "day":
      return "天亮了。";
    case "last_words":
      return "死亡玩家发表遗言。";
    case "speech":
      return "存活玩家依次发言。";
    case "vote":
      return "进入本日放逐投票。";
    case "pk":
      return "平票玩家进入 PK。";
    case "setup":
      return "准备身份和座位。";
    case "ended":
      return "本局结束。";
  }
}

function playerLabel(
  players: readonly PlayerSnapshot[],
  playerId: PlayerId,
): string {
  const player = players.find((candidate) => candidate.playerId === playerId);
  if (!player) {
    return `未知玩家 ${playerId}`;
  }

  return `${player.seatNo} 号 ${player.name}`;
}

function playerListLabel(
  players: readonly PlayerSnapshot[],
  playerIds: readonly PlayerId[],
): string {
  return playerIds.map((playerId) => playerLabel(players, playerId)).join("、");
}

function actorPrefix(
  players: readonly PlayerSnapshot[],
  actorPlayerId: PlayerId | undefined,
): string {
  return actorPlayerId ? `${playerLabel(players, actorPlayerId)} ` : "";
}

function voteTitle(voteType: VoteType): string {
  return voteType === "pk" ? "PK 投票" : "放逐投票";
}

function exileResolutionText(
  players: readonly PlayerSnapshot[],
  exiledPlayerId: PlayerId | null,
  tiedPlayerIds: readonly PlayerId[],
): string {
  if (exiledPlayerId) {
    return `放逐出局：${playerLabel(players, exiledPlayerId)}。`;
  }

  if (tiedPlayerIds.length > 0) {
    return `平票：${playerListLabel(players, tiedPlayerIds)}。`;
  }

  return "本轮无人被放逐。";
}

function roleLabel(role: GameRole): string {
  switch (role) {
    case "werewolf":
      return "狼人";
    case "seer":
      return "预言家";
    case "witch":
      return "女巫";
    case "villager":
      return "平民";
  }
}

function factionLabel(faction: Faction): string {
  return faction === "wolves" ? "狼人阵营" : "好人阵营";
}

function winnerLabel(winner: "wolves" | "good"): string {
  return winner === "wolves" ? "狼人阵营" : "好人阵营";
}

function winReasonLabel(reason: GameEndReason): string {
  switch (reason) {
    case "all_wolves_dead":
      return "所有狼人出局";
    case "all_gods_dead":
      return "所有神职出局";
    case "all_villagers_dead":
      return "所有平民出局";
    case "all_good_dead":
      return "所有好人出局";
  }
}
