import type { GameEvent } from "./events";
import {
  compileActorRuntimeCard,
  type ActorRuntimeCard,
} from "./actor-definition";
import { formatEventForHost, type PresentedEvent } from "./event-presenter";
import type { Game } from "./game";
import type { GameScriptSnapshot } from "./game-script";
import type { ModelBindingSnapshot } from "./model-binding";
import type { PlayerSnapshot } from "./player";
import type { RuleRole } from "./rule-role";
import { deriveGameState } from "./state";
import type { Faction, RuleRoleId, PlayerId, Ruleset } from "./types";
import { projectVisibleEvents, type VisibilityContext } from "./visibility";

export type PlayerContextRosterEntry = {
  readonly playerId: PlayerId;
  readonly seatNo: number;
  readonly name: string;
  readonly isSelf: boolean;
  readonly role?: RuleRoleId;
  readonly faction?: Faction;
};

export type PlayerContextTimelineItem = PresentedEvent & {
  readonly index: number;
  readonly type: GameEvent["type"];
  readonly phase: GameEvent["phase"];
  readonly dayNumber: number | null;
};

const MAX_PROMPT_KNOWLEDGE_ITEMS_PER_SECTION = 40;

export function selectPromptKnowledgeItems(
  items: readonly PlayerContextTimelineItem[],
): readonly PlayerContextTimelineItem[] {
  return items.slice(-MAX_PROMPT_KNOWLEDGE_ITEMS_PER_SECTION);
}

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
    readonly actor: ActorRuntimeCard;
    readonly ruleRole: RuleRole;
    readonly modelBinding: ModelBindingSnapshot;
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
      actor: compileActorRuntimeCard(viewer.actor),
      ruleRole: structuredClone(viewer.ruleRole),
      modelBinding: structuredClone(viewer.actor.production.modelBinding),
    },
    roster: input.game.players.map((player) => rosterEntryForViewer(player, viewer)),
    visibleEvents,
    timeline,
    knowledge: classifyPromptKnowledge(visibleEvents, timeline, state.dayNumber),
    state: {
      currentPhase: state.currentPhase,
      dayNumber: state.dayNumber,
      alivePlayerIds: state.alivePlayerIds,
      deadPlayerIds: state.deadPlayerIds,
      pkPlayerIds: state.pk.status === "pending" ? state.pk.tiedPlayerIds : [],
      witchResources:
        viewer.ruleRole.id === "witch"
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
  currentDayNumber: number,
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

  return {
    publicFacts: selectPublicFacts(publicFacts, currentDayNumber),
    publicClaims: selectPublicClaims(events, publicClaims, currentDayNumber),
    privateFacts,
    factionDiscussion: selectCurrentFactionDiscussion(
      factionDiscussion,
      currentDayNumber,
    ),
  };
}

function selectPublicFacts(
  facts: readonly PlayerContextTimelineItem[],
  currentDayNumber: number,
): readonly PlayerContextTimelineItem[] {
  return facts.filter(
    (item) =>
      item.dayNumber === null ||
      item.dayNumber >= currentDayNumber - 1 ||
      item.type === "death_announced" ||
      item.type === "exile_resolved" ||
      item.type === "game_ended",
  );
}

function selectPublicClaims(
  events: readonly GameEvent[],
  claims: readonly PlayerContextTimelineItem[],
  currentDayNumber: number,
): readonly PlayerContextTimelineItem[] {
  const currentDay = claims.filter(
    (item) => item.dayNumber === currentDayNumber,
  );
  const lastWords = claims.filter((item) => item.type === "last_words_given");
  const latestEarlierClaimByPlayer = new Map<PlayerId, PlayerContextTimelineItem>();
  const claimByIndex = new Map(claims.map((item) => [item.index, item]));

  for (const event of events) {
    if (
      (event.type !== "day_speech_given" &&
        event.type !== "pk_speech_given") ||
      event.payload.dayNumber >= currentDayNumber
    ) {
      continue;
    }
    const item = claimByIndex.get(event.index);
    if (item) latestEarlierClaimByPlayer.set(event.payload.playerId, item);
  }

  return uniqueTimelineItems([
    ...latestEarlierClaimByPlayer.values(),
    ...lastWords,
    ...currentDay,
  ]);
}

function selectCurrentFactionDiscussion(
  items: readonly PlayerContextTimelineItem[],
  currentDayNumber: number,
): readonly PlayerContextTimelineItem[] {
  const current = items.filter((item) => item.dayNumber === currentDayNumber);
  if (current.length > 0) return current;
  const latestDay = items.reduce(
    (day, item) => Math.max(day, item.dayNumber ?? 0),
    0,
  );
  return items.filter((item) => item.dayNumber === latestDay);
}

function uniqueTimelineItems(
  items: readonly PlayerContextTimelineItem[],
): readonly PlayerContextTimelineItem[] {
  return [...new Map(items.map((item) => [item.index, item])).values()].sort(
    (left, right) => left.index - right.index,
  );
}

function createVisibilityContext(
  players: readonly PlayerSnapshot[],
): VisibilityContext {
  return {
    wolfPlayerIds: players
      .filter((player) => player.ruleRole.id === "werewolf")
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
    name: player.actor.identity.name,
    isSelf: player.playerId === viewer.playerId,
  };

  if (canViewerKnowRole(player, viewer)) {
    return {
      ...entry,
      role: player.ruleRole.id,
      faction: player.ruleRole.faction,
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

  return (
    viewer.ruleRole.id === "werewolf" && player.ruleRole.id === "werewolf"
  );
}
