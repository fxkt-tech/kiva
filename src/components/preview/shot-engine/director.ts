import type { PlaybackItem } from "@/core/playback";
import { createCinematicShotLayout } from "./shot-layout";
import { subtitleCueForScene } from "./subtitles";
import type { RenderablePlayer, ShotClock, ShotFrame } from "./types";

export function createShotFrame(input: {
  readonly gameTitle?: string;
  readonly scene: PlaybackItem;
  readonly items: readonly PlaybackItem[];
  readonly timeMs: number;
  readonly backgroundImages: ShotFrame["backgroundImages"];
  readonly avatarImages: ShotFrame["avatarImages"];
}): ShotFrame {
  const clock = createShotClock(input.scene, input.timeMs);
  const activePlayerId = activePlayerIdForScene(input.scene);
  const players = input.scene.players.map<RenderablePlayer>((player) => ({
    ...player,
    emphasis: player.playerId === activePlayerId
      ? "active"
      : player.highlighted ? "highlighted" : "normal",
  }));
  const activePlayer =
    players.find((player) => player.emphasis === "active") ?? null;
  const highlightedPlayers = players.filter(
    (player) => player.emphasis === "active" || player.emphasis === "highlighted",
  );
  const subtitle = input.scene.kind === "speech"
    ? subtitleCueForScene(input.scene, clock.progress, activePlayer)
    : null;

  return {
    gameTitle: input.gameTitle ?? "",
    scene: input.scene,
    items: input.items,
    layout: createCinematicShotLayout(players),
    clock,
    players,
    activePlayer,
    highlightedPlayers,
    subtitle,
    backgroundImages: input.backgroundImages,
    avatarImages: input.avatarImages,
  };
}

export function createShotClock(
  scene: PlaybackItem,
  timeMs: number,
): ShotClock {
  const durationMs = Math.max(1, scene.durationMs);
  const sceneMs = Math.min(durationMs, Math.max(0, timeMs - scene.startsAtMs));
  const progress = sceneMs / durationMs;

  return {
    absoluteMs: timeMs,
    sceneMs,
    durationMs,
    progress,
    enterProgress: Math.min(1, sceneMs / 450),
    exitProgress: Math.min(1, Math.max(0, durationMs - sceneMs) / 450),
  };
}

function activePlayerIdForScene(scene: PlaybackItem): string | null {
  if (scene.kind !== "speech") {
    return null;
  }

  return scene.players.find((player) => player.highlighted)?.playerId ?? null;
}
