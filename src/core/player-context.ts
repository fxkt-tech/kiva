import type { GameEvent } from "./events";
import { formatEventForHost, type PresentedEvent } from "./event-presenter";
import type { Game } from "./game";
import type { ModelBindingSnapshot, PlayerSnapshot } from "./player";
import type { Faction, GameRole, PlayerId } from "./types";
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
};

export type PlayerLlmContext = {
  readonly gameId: Game["id"];
  readonly gameTitle: string;
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

  return {
    gameId: input.game.id,
    gameTitle: input.game.title,
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
    timeline: visibleEvents.map((event) => ({
      index: event.index,
      type: event.type,
      phase: event.phase,
      ...formatEventForHost(event, input.game.players),
    })),
  };
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
