import type { PlaybackSceneKind } from "@/core/playback";
import type { RenderablePlayer, ShotFrame } from "../shot-engine/types";

export type CaseBoardRowTone =
  | "detail"
  | "alive-player"
  | "dead-player"
  | "neutral"
  | "result";

export type CaseBoardRow = {
  readonly tone: CaseBoardRowTone;
  readonly label: string;
  readonly text: string;
  readonly meta?: string;
  readonly count?: number;
};

export type CaseBoardContentVariant = "standard" | "vote" | "ending";

export type CaseBoardContent = {
  readonly variant: CaseBoardContentVariant;
  readonly kindLabel: string;
  readonly title: string;
  readonly body: string;
  readonly rows: readonly CaseBoardRow[];
};

export function caseBoardContentForFrame(frame: ShotFrame): CaseBoardContent {
  if (isEndingScene(frame)) {
    return {
      variant: "ending",
      kindLabel: "Final / Reveal",
      title: winnerTitle(frame.scene.title),
      body: "",
      rows: [],
    };
  }

  if (isVoteResolutionScene(frame)) {
    return {
      variant: "vote",
      kindLabel: "Resolution / Vote",
      title: frame.scene.title,
      body: "",
      rows: voteRows(frame.scene.details),
    };
  }

  const detailRows = frame.scene.details.map<CaseBoardRow>((detail, index) => ({
    tone: "detail",
    label: formatNumberLabel(index + 1),
    text: detail,
  }));
  const playerRows = frame.highlightedPlayers.map(playerRow);
  const rows =
    detailRows.length > 0
      ? detailRows
      : playerRows.length > 0
        ? playerRows
        : [
            {
              tone: "neutral",
              label: "FILE",
              text: "PUBLIC RECORD",
            } satisfies CaseBoardRow,
          ];

  return {
    variant: "standard",
    kindLabel: kindLabelForScene(frame.scene.kind),
    title: frame.scene.title,
    body: frame.scene.text,
    rows,
  };
}

function isEndingScene(frame: ShotFrame): boolean {
  return frame.scene.title.startsWith("游戏结束：");
}

function isVoteResolutionScene(frame: ShotFrame): boolean {
  return (
    frame.scene.kind === "resolution" &&
    (frame.scene.title === "投票结算" || frame.scene.title === "PK 结算") &&
    frame.scene.details.some((detail) => detail.includes("->"))
  );
}

function winnerTitle(title: string): string {
  return title.replace(/^游戏结束：/, "");
}

function voteRows(details: readonly string[]): readonly CaseBoardRow[] {
  const buckets = new Map<string, string[]>();

  details.forEach((detail) => {
    const [voter, target] = detail.split("->").map((part) => part.trim());
    if (!voter || !target) {
      return;
    }

    const voters = buckets.get(target) ?? [];
    voters.push(shortSeatLabel(voter));
    buckets.set(target, voters);
  });

  return [...buckets.entries()]
    .sort(([, left], [, right]) => right.length - left.length)
    .map(([target, voters], index) => ({
      tone: index === 0 && target !== "弃票" ? "result" : "detail",
      label: target === "弃票" ? "--" : formatNumberLabel(seatNoFromLabel(target)),
      text: target,
      meta: voters.join("  "),
      count: voters.length,
    }));
}

function shortSeatLabel(label: string): string {
  return formatNumberLabel(seatNoFromLabel(label));
}

function seatNoFromLabel(label: string): number {
  const match = /^(\d+)\s*号/.exec(label);
  return match ? Number(match[1]) : 0;
}

function playerRow(player: RenderablePlayer): CaseBoardRow {
  return {
    tone: player.status === "dead" ? "dead-player" : "alive-player",
    label: formatNumberLabel(player.seatNo),
    text: player.name,
    meta: player.roleName,
  };
}

function kindLabelForScene(kind: PlaybackSceneKind): string {
  switch (kind) {
    case "phase":
      return "PHASE";
    case "announcement":
      return "ANNOUNCEMENT";
    case "vote":
      return "VOTE";
    case "resolution":
      return "RESOLUTION";
    case "speech":
      return "SPEECH";
  }
}

function formatNumberLabel(value: number): string {
  return String(value).padStart(2, "0");
}
