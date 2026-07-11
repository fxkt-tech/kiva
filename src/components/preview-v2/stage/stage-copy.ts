import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";

export type TranscriptPresentation = {
  readonly speaker: {
    readonly kind: "player" | "presenter";
    readonly name: string;
    readonly seatNo: number | null;
    readonly avatar: string | null;
    readonly roleName: string | null;
  };
  readonly content: string;
};

export type TranscriptSpeakerIdentity = {
  readonly kind: "presenter" | "role";
  readonly label: string;
};

export type TranscriptTextSegment = {
  readonly text: string;
  readonly roleName: string | null;
};

export function transcriptPresentationForScene(
  scene: PlaybackItem,
  activePlayer: PlaybackScenePlayer | null,
): TranscriptPresentation {
  const player = scene.transcriptSpeaker === "player" ? activePlayer : null;

  return {
    speaker: player
      ? {
          kind: "player",
          name: player.name,
          seatNo: player.seatNo,
          avatar: player.avatar,
          roleName: player.roleName,
        }
      : {
          kind: "presenter",
          name: scene.presenterName,
          seatNo: null,
          avatar: scene.presenterAvatar,
          roleName: null,
        },
    content: scene.text,
  };
}

export function transcriptSpeakerIdentity(
  presentation: TranscriptPresentation,
): TranscriptSpeakerIdentity {
  if (presentation.speaker.kind === "player") {
    return {
      kind: "role",
      label: presentation.speaker.roleName ?? "未知身份",
    };
  }

  return { kind: "presenter", label: "主理人" };
}

export function transcriptTextSegments(
  text: string,
  speakerKind: TranscriptPresentation["speaker"]["kind"],
  players: readonly PlaybackScenePlayer[],
): readonly TranscriptTextSegment[] {
  if (speakerKind === "player") {
    return [{ text, roleName: null }];
  }

  const mentions = playerMentions(players);
  if (!text || mentions.size === 0) {
    return [{ text, roleName: null }];
  }

  const pattern = [...mentions.keys()]
    .sort((left, right) => right.length - left.length)
    .map(escapeRegExp)
    .join("|");
  const matcher = new RegExp(pattern, "gu");
  const segments: TranscriptTextSegment[] = [];
  let cursor = 0;

  for (const match of text.matchAll(matcher)) {
    const mention = match[0];
    const index = match.index;
    if (/^\d/u.test(mention) && index > 0 && /\d/u.test(text[index - 1]!)) {
      continue;
    }
    if (index > cursor) {
      segments.push({ text: text.slice(cursor, index), roleName: null });
    }
    segments.push({ text: mention, roleName: mentions.get(mention) ?? null });
    cursor = index + mention.length;
  }

  if (cursor < text.length) {
    segments.push({ text: text.slice(cursor), roleName: null });
  }

  return segments.length > 0 ? segments : [{ text, roleName: null }];
}

function playerMentions(
  players: readonly PlaybackScenePlayer[],
): ReadonlyMap<string, string> {
  const mentions = new Map<string, string>();
  for (const player of players) {
    const seat = String(player.seatNo);
    for (const mention of [
      `${seat}号${player.name}`,
      `${seat}号 ${player.name}`,
      `${seat} 号${player.name}`,
      `${seat} 号 ${player.name}`,
      player.name,
      `${seat}号`,
      `${seat} 号`,
    ]) {
      if (mention.trim()) {
        mentions.set(mention, player.roleName);
      }
    }
  }
  return mentions;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function narrationTextForScene(scene: PlaybackItem): string {
  return scene.text;
}
