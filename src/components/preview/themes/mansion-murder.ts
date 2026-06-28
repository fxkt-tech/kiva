import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import {
  clearCanvas,
  drawGlowText,
  drawPanel,
  drawPlayerFile,
  drawTextBlock,
  drawVignette,
  drawVoteLine,
  type Point,
  type Rect,
} from "../canvas-renderer";
import type { ShowThemePack, ThemeRenderInput } from "../show-theme";

const WIDTH = 1920;
const HEIGHT = 1080;

export const mansionMurderTheme: ShowThemePack = {
  id: "mansion_murder",
  name: "Mansion Murder",
  tokens: {
    background: "#050506",
    panel: "#17120f",
    text: "#f4f1ea",
    muted: "#8b8178",
    accent: "#8fd3ff",
    danger: "#b4232a",
    good: "#86efac",
    wolf: "#f87171",
  },
  render: {
    renderPhase,
    renderAnnouncement,
    renderSpeech,
    renderVote,
    renderResolution,
    renderEnd: renderResolution,
  },
};

function renderPhase(input: ThemeRenderInput): void {
  baseFrame(input, "CASE FILE");
  drawChapterCard(input.ctx, input.scene, {
    label: "CASE FILE",
    titleY: 500,
    titleSize: 92,
  });
}

function renderAnnouncement(input: ThemeRenderInput): void {
  baseFrame(input, "CASE BULLETIN");
  drawChapterCard(input.ctx, input.scene, {
    label: "CASE BULLETIN",
    titleY: 360,
    titleSize: 70,
  });
  drawTextBlock(input.ctx, input.scene.text, 138, 520, {
    color: "#d8d0c4",
    font: "400 42px sans-serif",
    maxWidth: 1120,
    lineHeight: 62,
  });
  drawPlayerFiles(input.ctx, input.scene, { x: 1370, y: 210 });
}

function renderSpeech(input: ThemeRenderInput): void {
  baseFrame(input, "SUSPECT STATEMENT");
  const speaker = activePlayer(input.scene);
  drawPlayerFiles(input.ctx, input.scene, { x: 1370, y: 210 });
  drawTextBlock(input.ctx, "SUSPECT STATEMENT", 104, 210, {
    color: mansionMurderTheme.tokens.accent,
    font: "700 28px sans-serif",
    maxWidth: 760,
    lineHeight: 36,
  });
  if (speaker) {
    drawSuspectPortrait(input.ctx, speaker);
  }
  drawPanel(
    input.ctx,
    { x: 100, y: 430, width: 1160, height: 395 },
    {
      fill: "rgba(244, 241, 234, 0.08)",
      stroke: "rgba(244, 241, 234, 0.22)",
    },
  );
  drawTextBlock(input.ctx, input.scene.text, 142, 510, {
    color: "#ede7dc",
    font: "400 44px sans-serif",
    maxWidth: 1065,
    lineHeight: 66,
  });
}

function renderVote(input: ThemeRenderInput): void {
  baseFrame(input, "EVIDENCE VOTE");
  drawTextBlock(input.ctx, "EVIDENCE VOTE", 104, 190, {
    color: "#fca5a5",
    font: "700 28px sans-serif",
    maxWidth: 760,
    lineHeight: 36,
  });
  drawTextBlock(input.ctx, input.scene.title, 104, 280, {
    color: mansionMurderTheme.tokens.text,
    font: "800 70px sans-serif",
    maxWidth: 1100,
    lineHeight: 84,
  });
  drawCaseBoard(input.ctx, input.scene);
  drawTextBlock(input.ctx, input.scene.text, 104, 875, {
    color: "#e7dccd",
    font: "500 36px sans-serif",
    maxWidth: 1220,
    lineHeight: 52,
  });
}

function renderResolution(input: ThemeRenderInput): void {
  baseFrame(input, "CASE STATUS");
  drawTextBlock(input.ctx, "CASE STATUS", 104, 210, {
    color: mansionMurderTheme.tokens.danger,
    font: "700 28px sans-serif",
    maxWidth: 760,
    lineHeight: 36,
  });
  drawGlowText(input.ctx, input.scene.title, 104, 350, {
    color: mansionMurderTheme.tokens.text,
    glow: "rgba(180, 35, 42, 0.65)",
    font: "800 74px sans-serif",
    maxWidth: 1180,
    lineHeight: 88,
  });
  drawTextBlock(input.ctx, input.scene.text, 104, 505, {
    color: "#d8d0c4",
    font: "400 42px sans-serif",
    maxWidth: 1180,
    lineHeight: 62,
  });
  drawPlayerFiles(input.ctx, input.scene, { x: 1370, y: 210 });
  drawDetails(input.ctx, input.scene);
}

function baseFrame(input: ThemeRenderInput, label: string): void {
  const ctx = input.ctx;
  clearCanvas(ctx, mansionMurderTheme.tokens.background);
  drawMansionBackdrop(ctx);
  drawVignette(ctx);
  drawTopCaseBar(ctx, input.scene, label);
}

function drawTopCaseBar(
  ctx: CanvasRenderingContext2D,
  scene: PlaybackItem,
  label: string,
): void {
  drawPanel(ctx, { x: 0, y: 0, width: WIDTH, height: 108 }, {
    fill: "rgba(12, 10, 9, 0.92)",
    stroke: "rgba(120, 113, 108, 0.18)",
  });
  drawTextBlock(ctx, label, 96, 66, {
    color: mansionMurderTheme.tokens.muted,
    font: "700 24px sans-serif",
    maxWidth: 460,
    lineHeight: 32,
  });
  drawTextBlock(ctx, `#${scene.index}   ${scene.phase}   ${scene.kind}`, 1370, 66, {
    color: "#6f655d",
    font: "600 22px sans-serif",
    maxWidth: 450,
    lineHeight: 30,
  });
}

function drawMansionBackdrop(ctx: CanvasRenderingContext2D): void {
  const gradient = ctx.createLinearGradient(0, 0, WIDTH, HEIGHT);
  gradient.addColorStop(0, "#050506");
  gradient.addColorStop(0.48, "#10100f");
  gradient.addColorStop(1, "#160b0c");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  ctx.fillStyle = "rgba(143, 211, 255, 0.06)";
  for (let index = 0; index < 6; index += 1) {
    ctx.fillRect(130 + index * 225, 120, 72, 600);
  }
  ctx.fillStyle = "rgba(180, 35, 42, 0.09)";
  ctx.fillRect(0, 910, WIDTH, 170);
}

function drawChapterCard(
  ctx: CanvasRenderingContext2D,
  scene: PlaybackItem,
  options: {
    readonly label: string;
    readonly titleY: number;
    readonly titleSize: number;
  },
): void {
  drawPanel(ctx, { x: 96, y: 220, width: 1170, height: 570 }, {
    fill: "rgba(244, 241, 234, 0.08)",
    stroke: "rgba(244, 241, 234, 0.25)",
  });
  drawTextBlock(ctx, options.label, 138, 300, {
    color: mansionMurderTheme.tokens.danger,
    font: "700 28px sans-serif",
    maxWidth: 700,
    lineHeight: 36,
  });
  drawTextBlock(ctx, scene.title, 138, options.titleY, {
    color: mansionMurderTheme.tokens.text,
    font: `800 ${options.titleSize}px sans-serif`,
    maxWidth: 1030,
    lineHeight: options.titleSize + 14,
  });
  if (scene.text) {
    drawTextBlock(ctx, scene.text, 142, 660, {
      color: "#bdb4a8",
      font: "400 38px sans-serif",
      maxWidth: 1000,
      lineHeight: 54,
    });
  }
  drawPlayerFiles(ctx, scene, { x: 1370, y: 210 });
}

function drawSuspectPortrait(
  ctx: CanvasRenderingContext2D,
  player: PlaybackScenePlayer,
): void {
  drawPanel(ctx, { x: 104, y: 255, width: 740, height: 120 }, {
    fill: "rgba(15, 23, 42, 0.42)",
    stroke: "rgba(143, 211, 255, 0.45)",
  });
  drawTextBlock(ctx, `Seat ${player.seatNo}`, 134, 302, {
    color: mansionMurderTheme.tokens.muted,
    font: "500 26px sans-serif",
    maxWidth: 160,
    lineHeight: 34,
  });
  drawTextBlock(ctx, player.name, 285, 323, {
    color: mansionMurderTheme.tokens.text,
    font: "800 58px sans-serif",
    maxWidth: 460,
    lineHeight: 68,
  });
}

function drawPlayerFiles(
  ctx: CanvasRenderingContext2D,
  scene: PlaybackItem,
  origin: Point,
): void {
  drawTextBlock(ctx, "SUSPECT FILES", origin.x, origin.y - 40, {
    color: mansionMurderTheme.tokens.muted,
    font: "700 22px sans-serif",
    maxWidth: 380,
    lineHeight: 30,
  });

  scene.players.forEach((player, index) => {
    drawPlayerFile(ctx, player, {
      x: origin.x,
      y: origin.y + index * 86,
      width: 410,
      height: 66,
    }, {
      fill: player.status === "dead"
        ? "rgba(24, 18, 18, 0.78)"
        : "rgba(23, 18, 15, 0.88)",
      stroke: mansionMurderTheme.tokens.accent,
      text: mansionMurderTheme.tokens.text,
      muted: mansionMurderTheme.tokens.muted,
      statusAlive: mansionMurderTheme.tokens.good,
      statusDead: mansionMurderTheme.tokens.danger,
    });
  });
}

function drawCaseBoard(ctx: CanvasRenderingContext2D, scene: PlaybackItem): void {
  const rects = boardRects(scene.players);
  rects.forEach((rect, index) => {
    const player = scene.players[index];
    if (player) {
      drawPlayerFile(ctx, player, rect, {
        fill: "rgba(23, 18, 15, 0.90)",
        stroke: mansionMurderTheme.tokens.danger,
        text: mansionMurderTheme.tokens.text,
        muted: mansionMurderTheme.tokens.muted,
        statusAlive: mansionMurderTheme.tokens.good,
        statusDead: mansionMurderTheme.tokens.danger,
      });
    }
  });

  const highlighted = rects.filter((_, index) => scene.players[index]?.highlighted);
  if (highlighted.length >= 2) {
    for (let index = 0; index < highlighted.length - 1; index += 1) {
      drawVoteLine(ctx, center(highlighted[index]!), center(highlighted[index + 1]!), mansionMurderTheme.tokens.danger, 5);
    }
  } else if (rects.length >= 2) {
    drawVoteLine(ctx, center(rects[0]!), center(rects[1]!), mansionMurderTheme.tokens.danger, 4);
  }
}

function boardRects(players: readonly PlaybackScenePlayer[]): Rect[] {
  return players.map((_, index) => ({
    x: 145 + (index % 3) * 385,
    y: 410 + Math.floor(index / 3) * 150,
    width: 335,
    height: 92,
  }));
}

function center(rect: Rect): Point {
  return {
    x: rect.x + rect.width / 2,
    y: rect.y + rect.height / 2,
  };
}

function drawDetails(ctx: CanvasRenderingContext2D, scene: PlaybackItem): void {
  if (scene.details.length === 0) {
    return;
  }

  drawPanel(ctx, { x: 104, y: 760, width: 1180, height: 150 }, {
    fill: "rgba(244, 241, 234, 0.06)",
    stroke: "rgba(244, 241, 234, 0.18)",
  });
  drawTextBlock(ctx, scene.details.join("    "), 136, 825, {
    color: "#bdb4a8",
    font: "400 28px sans-serif",
    maxWidth: 1110,
    lineHeight: 42,
  });
}

function activePlayer(scene: PlaybackItem): PlaybackScenePlayer | undefined {
  return scene.players.find((player) => player.highlighted);
}
