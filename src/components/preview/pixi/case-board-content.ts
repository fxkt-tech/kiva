import type { PlaybackSceneKind } from "@/core/playback";
import type { RenderablePlayer, ShotFrame } from "../shot-engine/types";

export type CaseBoardRowTone =
  | "detail"
  | "alive-player"
  | "dead-player"
  | "neutral";

export type CaseBoardRow = {
  readonly tone: CaseBoardRowTone;
  readonly label: string;
  readonly text: string;
  readonly meta?: string;
};

export type CaseBoardContent = {
  readonly kindLabel: string;
  readonly title: string;
  readonly body: string;
  readonly rows: readonly CaseBoardRow[];
};

export function caseBoardContentForFrame(frame: ShotFrame): CaseBoardContent {
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
    kindLabel: kindLabelForScene(frame.scene.kind),
    title: frame.scene.title,
    body: frame.scene.text,
    rows,
  };
}

function playerRow(player: RenderablePlayer): CaseBoardRow {
  return {
    tone: player.status === "dead" ? "dead-player" : "alive-player",
    label: formatNumberLabel(player.seatNo),
    text: player.name,
    meta: `身份：${player.roleName}`,
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
