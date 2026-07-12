import type { GameEvent } from "./events";
import { formatEventForHost, type PresentedEvent } from "./event-presenter";
import type { Game } from "./game";
import type { GameScriptSnapshot } from "./game-script";
import type { ModelBindingSnapshot, PlayerSnapshot } from "./player";
import { deriveGameState } from "./state";
import type { Faction, GameRole, PlayerId, Ruleset } from "./types";
import { projectVisibleEvents, type VisibilityContext } from "./visibility";

export type PlayerContextRosterEntry = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly isSelf: boolean;
  readonly role?: GameRole;
  readonly faction?: Faction;
};

export type PlayerContextTimelineItem = PresentedEvent & {
  readonly index: number;
  readonly type: GameEvent["type"];
  readonly phase: GameEvent["phase"];
  readonly dayNumber: number | null;
};

export type PlayerPromptKnowledge = {
  readonly publicFacts: readonly PlayerContextTimelineItem[];
  readonly publicClaims: readonly PlayerContextTimelineItem[];
  readonly privateFacts: readonly PlayerContextTimelineItem[];
  readonly factionDiscussion: readonly PlayerContextTimelineItem[];
};

export type PlayerPromptState = {
  readonly currentPhase: GameEvent["phase"];
  readonly dayNumber: number;
  readonly alivePlayerIds: readonly PlayerId[];
  readonly deadPlayerIds: readonly PlayerId[];
  readonly pkPlayerIds: readonly PlayerId[];
  readonly witchResources: {
    readonly antidoteAvailable: boolean;
    readonly poisonAvailable: boolean;
  } | null;
};

export type PlayerLlmContext = {
  readonly gameId: Game["id"];
  readonly gameTitle: string;
  readonly script: GameScriptSnapshot;
  readonly ruleset: Ruleset;
  readonly viewer: {
    readonly playerId: PlayerId;
    readonly seatNo: number;
    readonly name: string;
    readonly role: GameRole;
    readonly roleName: string;
    readonly faction: Faction;
    readonly team: PlayerSnapshot["team"];
    readonly mechanicKey: PlayerSnapshot["mechanicKey"];
    readonly persona: string;
    readonly speakingStyle: string;
    readonly reasoningStyle: string;
    readonly characterSystemPromptSnapshot: string;
    readonly roleSystemPromptSnapshot: string;
    readonly roleActionPromptSnapshot: string | null;
    readonly systemPrompt: string;
    readonly modelBindingSnapshot: ModelBindingSnapshot;
  };
  readonly roster: readonly PlayerContextRosterEntry[];
  readonly visibleEvents: readonly GameEvent[];
  readonly timeline: readonly PlayerContextTimelineItem[];
  readonly knowledge: PlayerPromptKnowledge;
  readonly state: PlayerPromptState;
};

export type BuildPlayerLlmContextInput = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly viewerPlayerId: PlayerId;
};

export function buildPlayerLlmContext(
  input: BuildPlayerLlmContextInput,
): PlayerLlmContext {
  const viewer = input.game.players.find(
    (player) => player.playerId === input.viewerPlayerId,
  );

  if (!viewer) {
    throw new Error(`Viewer player not found: ${input.viewerPlayerId}`);
  }

  const visibilityContext = createVisibilityContext(input.game.players);
  const visibleEvents = projectVisibleEvents(
    input.events,
    viewer.playerId,
    visibilityContext,
  );
  const state = deriveGameState(input.game.players, input.events);
  const timeline = visibleEvents.map((event) => ({
    index: event.index,
    type: event.type,
    phase: event.phase,
    dayNumber: eventDayNumber(event),
    ...formatEventForHost(event, input.game.players),
  }));

  return {
    gameId: input.game.id,
    gameTitle: input.game.title,
    script: input.game.script,
    ruleset: input.game.ruleset,
    viewer: {
      playerId: viewer.playerId,
      seatNo: viewer.seatNo,
      name: viewer.name,
      role: viewer.gameRole,
      roleName: viewer.roleName,
      faction: viewer.faction,
      team: viewer.team,
      mechanicKey: viewer.mechanicKey,
      persona: viewer.persona,
      speakingStyle: viewer.speakingStyle,
      reasoningStyle: viewer.reasoningStyle,
      characterSystemPromptSnapshot: viewer.characterSystemPromptSnapshot,
      roleSystemPromptSnapshot: viewer.roleSystemPromptSnapshot,
      roleActionPromptSnapshot: viewer.roleActionPromptSnapshot,
      systemPrompt: viewer.systemPrompt,
      modelBindingSnapshot: viewer.modelBindingSnapshot,
    },
    roster: input.game.players.map((player) => rosterEntryForViewer(player, viewer)),
    visibleEvents,
    timeline,
    knowledge: classifyPromptKnowledge(visibleEvents, timeline),
    state: {
      currentPhase: state.currentPhase,
      dayNumber: state.dayNumber,
      alivePlayerIds: state.alivePlayerIds,
      deadPlayerIds: state.deadPlayerIds,
      pkPlayerIds: state.pk.status === "pending" ? state.pk.tiedPlayerIds : [],
      witchResources:
        viewer.gameRole === "witch"
          ? {
              antidoteAvailable: state.witch.antidoteAvailable,
              poisonAvailable: state.witch.poisonAvailable,
            }
          : null,
    },
  };
}

function eventDayNumber(event: GameEvent): number | null {
  if ("dayNumber" in event.payload) {
    return event.payload.dayNumber;
  }

  return null;
}

function classifyPromptKnowledge(
  events: readonly GameEvent[],
  timeline: readonly PlayerContextTimelineItem[],
): PlayerPromptKnowledge {
  const publicFacts: PlayerContextTimelineItem[] = [];
  const publicClaims: PlayerContextTimelineItem[] = [];
  const privateFacts: PlayerContextTimelineItem[] = [];
  const factionDiscussion: PlayerContextTimelineItem[] = [];

  for (let index = 0; index < events.length; index += 1) {
    const event = events[index];
    const item = timeline[index];
    if (!event || !item) continue;

    if (event.type === "role_assigned" || event.type === "phase_started") {
      continue;
    }

    if (
      event.type === "wolf_strategy_given" ||
      event.type === "wolf_opinion_given"
    ) {
      factionDiscussion.push(item);
      continue;
    }

    if (
      event.type === "day_speech_given" ||
      event.type === "pk_speech_given" ||
      event.type === "last_words_given"
    ) {
      publicClaims.push(item);
      continue;
    }

    if (event.visibility.kind === "public") {
      publicFacts.push(item);
      continue;
    }

    privateFacts.push(item);
  }

  return { publicFacts, publicClaims, privateFacts, factionDiscussion };
}

function createVisibilityContext(
  players: readonly PlayerSnapshot[],
): VisibilityContext {
  return {
    wolfPlayerIds: players
      .filter((player) => player.gameRole === "werewolf")
      .map((player) => player.playerId),
  };
}

function rosterEntryForViewer(
  player: PlayerSnapshot,
  viewer: PlayerSnapshot,
): PlayerContextRosterEntry {
  const entry = {
    playerId: player.playerId,
    seatNo: player.seatNo,
    name: player.name,
    isSelf: player.playerId === viewer.playerId,
  };

  if (canViewerKnowRole(player, viewer)) {
    return {
      ...entry,
      role: player.gameRole,
      faction: player.faction,
    };
  }

  return entry;
}

function canViewerKnowRole(
  player: PlayerSnapshot,
  viewer: PlayerSnapshot,
): boolean {
  if (player.playerId === viewer.playerId) {
    return true;
  }

  return viewer.gameRole === "werewolf" && player.gameRole === "werewolf";
}
