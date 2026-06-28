import type { PlaybackScenePlayer } from "@/core/playback";
import {
  clearCanvas,
  drawAvatar,
  drawGlowText,
  drawPanel,
  drawStatusStamp,
  drawSubtitleBar,
  drawTextBlock,
  drawVignette,
  drawVoteResultTable,
  type Rect,
} from "../canvas-renderer";
import type { PlayerSlot } from "../stage-layout";
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
    renderBackground,
    renderPlayerCard,
    renderCenterStage,
    renderSubtitle,
    renderEffect,
  },
};

function renderBackground(input: ThemeRenderInput): void {
  const ctx = input.ctx;
  const backgroundImage = input.scene.phase === "night"
    ? input.backgroundImages.night
    : input.backgroundImages.day;

  if (backgroundImage) {
    drawCoverImage(ctx, backgroundImage, WIDTH, HEIGHT);
  } else {
    clearCanvas(ctx, mansionMurderTheme.tokens.background);
    drawMansionBackdrop(ctx);
  }

  drawVignette(ctx);
  drawTopCaseBar(ctx, input);
}

function renderPlayerCard(input: ThemeRenderInput): void {
  input.layout.playerSlots.forEach((slot) => {
    drawSuspectCard(input.ctx, slot);
  });
}

function renderCenterStage(input: ThemeRenderInput): void {
  switch (input.scene.kind) {
    case "speech":
      drawSpeechFocus(input);
      return;
    case "vote":
      drawVoteMoment(input);
      return;
    case "resolution":
      drawResolutionMoment(input);
      return;
    case "phase":
    case "announcement":
      drawAnnouncementMoment(input);
      return;
  }
}

function renderSubtitle(input: ThemeRenderInput): void {
  const speaker = activePlayer(input.scene.players);
  drawSubtitleBar(input.ctx, {
    speaker: speaker
      ? `${speaker.seatNo} 号 ${speaker.name}`
      : labelForScene(input),
    text: input.scene.text || input.scene.title,
    rect: input.layout.subtitle,
    colors: {
      fill: "rgba(5, 5, 6, 0.86)",
      stroke: "rgba(143, 211, 255, 0.30)",
      speaker: speaker ? mansionMurderTheme.tokens.accent : "#c7bba8",
      text: mansionMurderTheme.tokens.text,
    },
  });
}

function renderEffect(input: ThemeRenderInput): void {
  const speaker = activePlayer(input.scene.players);
  if (!speaker || input.scene.kind !== "speech") {
    return;
  }

  const slot = input.layout.playerSlots.find(
    (candidate) => candidate.player.playerId === speaker.playerId,
  );
  if (!slot) {
    return;
  }

  input.ctx.strokeStyle = "rgba(143, 211, 255, 0.55)";
  input.ctx.lineWidth = 6;
  input.ctx.strokeRect(
    slot.rect.x - 8,
    slot.rect.y - 8,
    slot.rect.width + 16,
    slot.rect.height + 16,
  );
}

function drawTopCaseBar(ctx: CanvasRenderingContext2D, input: ThemeRenderInput): void {
  drawPanel(ctx, { x: 0, y: 0, width: WIDTH, height: 98 }, {
    fill: "rgba(12, 10, 9, 0.92)",
    stroke: "rgba(120, 113, 108, 0.18)",
  });
  drawTextBlock(ctx, "MANSION MURDER", 88, 60, {
    color: mansionMurderTheme.tokens.muted,
    font: "800 24px sans-serif",
    maxWidth: 420,
    lineHeight: 32,
  });
  drawTextBlock(ctx, input.scene.title, 690, 60, {
    color: mansionMurderTheme.tokens.text,
    font: "800 32px sans-serif",
    maxWidth: 620,
    lineHeight: 38,
  });
  drawTextBlock(ctx, `#${input.scene.index}   ${input.scene.phase}`, 1570, 60, {
    color: "#6f655d",
    font: "700 22px sans-serif",
    maxWidth: 270,
    lineHeight: 30,
  });
}

function drawMansionBackdrop(ctx: CanvasRenderingContext2D): void {
  const gradient = ctx.createLinearGradient(0, 0, WIDTH, HEIGHT);
  gradient.addColorStop(0, "#050506");
  gradient.addColorStop(0.5, "#12100d");
  gradient.addColorStop(1, "#190a0c");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, WIDTH, HEIGHT);

  ctx.fillStyle = "rgba(143, 211, 255, 0.055)";
  for (let index = 0; index < 7; index += 1) {
    ctx.fillRect(548 + index * 116, 132, 34, 650);
  }
  ctx.fillStyle = "rgba(180, 35, 42, 0.10)";
  ctx.fillRect(0, 826, WIDTH, 254);
}

function drawCoverImage(
  ctx: CanvasRenderingContext2D,
  image: HTMLImageElement,
  width: number,
  height: number,
): void {
  const imageRatio = image.width / image.height;
  const canvasRatio = width / height;
  const sourceWidth = imageRatio > canvasRatio
    ? image.height * canvasRatio
    : image.width;
  const sourceHeight = imageRatio > canvasRatio
    ? image.height
    : image.width / canvasRatio;
  const sourceX = (image.width - sourceWidth) / 2;
  const sourceY = (image.height - sourceHeight) / 2;

  ctx.drawImage(
    image,
    sourceX,
    sourceY,
    sourceWidth,
    sourceHeight,
    0,
    0,
    width,
    height,
  );
}

function drawSuspectCard(ctx: CanvasRenderingContext2D, slot: PlayerSlot): void {
  const player = slot.player;
  const dead = player.status === "dead";
  drawPanel(ctx, slot.rect, {
    fill: dead ? "rgba(24, 18, 18, 0.68)" : "rgba(23, 18, 15, 0.92)",
    stroke: player.highlighted
      ? "rgba(143, 211, 255, 0.75)"
      : "rgba(244, 241, 234, 0.18)",
    lineWidth: player.highlighted ? 4 : 2,
  });

  drawAvatar(ctx, player.name, {
    x: slot.rect.x + 74,
    y: slot.rect.y + 76,
    radius: 42,
  }, {
    fill: dead ? "rgba(41, 37, 36, 0.96)" : "rgba(143, 211, 255, 0.16)",
    stroke: dead ? "rgba(120, 113, 108, 0.38)" : "rgba(143, 211, 255, 0.62)",
    text: dead ? mansionMurderTheme.tokens.muted : mansionMurderTheme.tokens.text,
  });

  ctx.fillStyle = mansionMurderTheme.tokens.muted;
  ctx.font = "800 20px sans-serif";
  ctx.fillText(`Seat ${player.seatNo}`, slot.rect.x + 138, slot.rect.y + 56);

  ctx.fillStyle = dead ? mansionMurderTheme.tokens.muted : mansionMurderTheme.tokens.text;
  ctx.font = "900 38px sans-serif";
  ctx.fillText(player.name, slot.rect.x + 138, slot.rect.y + 105);

  ctx.fillStyle = dead ? mansionMurderTheme.tokens.muted : roleColor(player.roleName);
  ctx.font = "800 22px sans-serif";
  ctx.fillText(`身份：${player.roleName}`, slot.rect.x + 138, slot.rect.y + 142);

  drawStatusStamp(ctx, dead ? "DEAD" : "ALIVE", {
    x: slot.rect.x + slot.rect.width - 128,
    y: slot.rect.y + 126,
    width: 96,
    height: 42,
  }, {
    fill: dead ? "rgba(69, 10, 10, 0.45)" : "rgba(6, 78, 59, 0.28)",
    stroke: dead ? "rgba(180, 35, 42, 0.72)" : "rgba(134, 239, 172, 0.42)",
    text: dead ? "#fecaca" : "#bbf7d0",
  });
}

function roleColor(roleName: string): string {
  return roleName === "狼人"
    ? mansionMurderTheme.tokens.wolf
    : mansionMurderTheme.tokens.good;
}

function drawAnnouncementMoment(input: ThemeRenderInput): void {
  const rect = input.layout.center;
  drawPanel(input.ctx, rect, {
    fill: "rgba(244, 241, 234, 0.07)",
    stroke: "rgba(244, 241, 234, 0.24)",
  });
  drawTextBlock(input.ctx, labelForScene(input), rect.x + 42, rect.y + 82, {
    color: mansionMurderTheme.tokens.danger,
    font: "800 28px sans-serif",
    maxWidth: rect.width - 84,
    lineHeight: 36,
  });
  drawGlowText(input.ctx, input.scene.title, rect.x + 42, rect.y + 260, {
    color: mansionMurderTheme.tokens.text,
    glow: "rgba(180, 35, 42, 0.45)",
    font: "900 70px sans-serif",
    maxWidth: rect.width - 84,
    lineHeight: 84,
  });
  drawTextBlock(input.ctx, input.scene.text, rect.x + 46, rect.y + 410, {
    color: "#d8d0c4",
    font: "500 38px sans-serif",
    maxWidth: rect.width - 92,
    lineHeight: 56,
  });
}

function drawSpeechFocus(input: ThemeRenderInput): void {
  const rect = input.layout.center;
  const speaker = activePlayer(input.scene.players);
  drawPanel(input.ctx, rect, {
    fill: "rgba(15, 23, 42, 0.28)",
    stroke: "rgba(143, 211, 255, 0.26)",
  });
  drawTextBlock(input.ctx, "SUSPECT STATEMENT", rect.x + 44, rect.y + 82, {
    color: mansionMurderTheme.tokens.accent,
    font: "800 28px sans-serif",
    maxWidth: rect.width - 88,
    lineHeight: 36,
  });

  if (speaker) {
    drawFeaturedSpeaker(input.ctx, speaker, {
      x: rect.x + 78,
      y: rect.y + 155,
      width: rect.width - 156,
      height: 260,
    });
  }

  drawTextBlock(input.ctx, input.scene.title, rect.x + 44, rect.y + 510, {
    color: "#c7bba8",
    font: "800 40px sans-serif",
    maxWidth: rect.width - 88,
    lineHeight: 52,
  });
}

function drawFeaturedSpeaker(
  ctx: CanvasRenderingContext2D,
  player: PlaybackScenePlayer,
  rect: Rect,
): void {
  drawPanel(ctx, rect, {
    fill: "rgba(5, 5, 6, 0.48)",
    stroke: "rgba(143, 211, 255, 0.38)",
  });
  drawAvatar(ctx, player.name, {
    x: rect.x + 106,
    y: rect.y + 130,
    radius: 74,
  }, {
    fill: "rgba(143, 211, 255, 0.18)",
    stroke: "rgba(143, 211, 255, 0.68)",
    text: mansionMurderTheme.tokens.text,
  });
  ctx.fillStyle = mansionMurderTheme.tokens.muted;
  ctx.font = "800 26px sans-serif";
  ctx.fillText(`${player.seatNo} 号`, rect.x + 225, rect.y + 105);
  ctx.fillStyle = mansionMurderTheme.tokens.text;
  ctx.font = "900 66px sans-serif";
  ctx.fillText(player.name, rect.x + 225, rect.y + 178);
}

function drawVoteMoment(input: ThemeRenderInput): void {
  drawVoteResultTable(input.ctx, {
    title: input.scene.title,
    rows: input.scene.details.length > 0 ? input.scene.details : [input.scene.text],
    result: input.scene.details.length > 0 ? [input.scene.text] : ["等待本轮投票结算"],
    rect: input.layout.center,
    colors: {
      fill: "rgba(23, 18, 15, 0.92)",
      stroke: "rgba(180, 35, 42, 0.54)",
      title: mansionMurderTheme.tokens.text,
      text: "#d8d0c4",
      accent: mansionMurderTheme.tokens.danger,
    },
  });
}

function drawResolutionMoment(input: ThemeRenderInput): void {
  if (input.scene.details.length > 0 && input.scene.phase === "vote") {
    drawVoteResultTable(input.ctx, {
      title: input.scene.title,
      rows: input.scene.details,
      result: [input.scene.text],
      rect: input.layout.center,
      colors: {
        fill: "rgba(23, 18, 15, 0.92)",
        stroke: "rgba(180, 35, 42, 0.54)",
        title: mansionMurderTheme.tokens.text,
        text: "#d8d0c4",
        accent: mansionMurderTheme.tokens.danger,
      },
    });
    return;
  }

  drawAnnouncementMoment(input);
}

function labelForScene(input: ThemeRenderInput): string {
  switch (input.scene.kind) {
    case "phase":
      return "CASE FILE";
    case "speech":
      return "SUSPECT STATEMENT";
    case "vote":
      return "EVIDENCE VOTE";
    case "resolution":
      return "CASE STATUS";
    case "announcement":
      return "CASE BULLETIN";
  }
}

function activePlayer(
  players: readonly PlaybackScenePlayer[],
): PlaybackScenePlayer | undefined {
  return players.find((player) => player.highlighted);
}
