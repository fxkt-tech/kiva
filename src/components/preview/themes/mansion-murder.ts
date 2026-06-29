import type { PlaybackScenePlayer } from "@/core/playback";
import {
  clearCanvas,
  drawAvatar,
  drawGlowText,
  drawPanel,
  drawSubtitleBar,
  drawTextBlock,
  drawVignette,
  drawVoteResultTable,
  type Rect,
} from "../canvas-renderer";
import { fadeIn, shake, slideIn, zoom } from "../shot-engine/animation";
import type {
  PreviewThemeSkin,
  SkinRenderInput,
  SkinSeatInput,
} from "../show-theme";

const WIDTH = 1920;
const HEIGHT = 1080;

export const mansionMurderTheme: PreviewThemeSkin = {
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
  draw: {
    background: drawBackground,
    topBar: drawTopCaseBar,
    seat: drawSeatTrackItem,
    speechShot: drawSpeechShot,
    voteShot: drawVoteShot,
    phaseShot: drawPhaseShot,
    announcementShot: drawAnnouncementShot,
    resolutionShot: drawResolutionShot,
    subtitle: drawSubtitle,
    effects: drawEffects,
  },
};

function drawBackground(input: SkinRenderInput): void {
  const ctx = input.ctx;
  const backgroundImage = input.frame.scene.phase === "night"
    ? input.frame.backgroundImages.night
    : input.frame.backgroundImages.day;

  if (backgroundImage) {
    drawCoverImage(ctx, backgroundImage, WIDTH, HEIGHT);
  } else {
    clearCanvas(ctx, mansionMurderTheme.tokens.background);
    drawMansionBackdrop(ctx);
  }

  drawVignette(ctx);
  drawPanel(ctx, { x: 0, y: 0, width: WIDTH, height: HEIGHT }, {
    fill: "rgba(0, 0, 0, 0.18)",
  });
}

function drawTopCaseBar(input: SkinRenderInput): void {
  const { ctx, frame } = input;
  drawPanel(ctx, frame.layout.topBar, {
    fill: "rgba(8, 7, 7, 0.88)",
    stroke: "rgba(244, 241, 234, 0.12)",
  });
  drawTextBlock(ctx, "MANSION MURDER", 72, 58, {
    color: mansionMurderTheme.tokens.muted,
    font: "800 24px sans-serif",
    maxWidth: 420,
    lineHeight: 32,
  });
  drawTextBlock(ctx, frame.scene.title, 620, 58, {
    color: mansionMurderTheme.tokens.text,
    font: "900 32px sans-serif",
    maxWidth: 720,
    lineHeight: 38,
  });
  drawTextBlock(ctx, `#${frame.scene.index}   ${frame.scene.phase}`, 1570, 58, {
    color: "#756b62",
    font: "800 22px sans-serif",
    maxWidth: 270,
    lineHeight: 30,
  });
}

function drawSeatTrackItem(input: SkinSeatInput): void {
  const { ctx, frame, slot } = input;
  const player = slot.player;
  const dead = player.status === "dead";
  const active = player.playerId === frame.activePlayer?.playerId;
  const highlighted = player.highlighted || active;
  const offsetX = active
    ? (slot.side === "left" ? -slideIn(frame.clock.enterProgress, 18) : slideIn(frame.clock.enterProgress, 18))
    : 0;
  const rect = { ...slot.rect, x: slot.rect.x + offsetX };
  const avatarImage = player.avatar ? frame.avatarImages[player.avatar] : null;

  drawPanel(ctx, rect, {
    fill: dead ? "rgba(15, 13, 12, 0.58)" : "rgba(16, 13, 11, 0.74)",
    stroke: highlighted
      ? "rgba(143, 211, 255, 0.72)"
      : "rgba(244, 241, 234, 0.14)",
    lineWidth: highlighted ? 3 : 1,
  });

  drawAvatar(ctx, player.name, {
    x: rect.x + 48,
    y: rect.y + 56,
    radius: 34,
  }, {
    fill: dead ? "rgba(41, 37, 36, 0.92)" : "rgba(143, 211, 255, 0.14)",
    stroke: dead ? "rgba(120, 113, 108, 0.36)" : "rgba(143, 211, 255, 0.58)",
    text: dead ? mansionMurderTheme.tokens.muted : mansionMurderTheme.tokens.text,
    image: avatarImage,
  });

  ctx.fillStyle = mansionMurderTheme.tokens.muted;
  ctx.font = "800 17px sans-serif";
  ctx.fillText(`Seat ${player.seatNo}`, rect.x + 96, rect.y + 36);

  ctx.fillStyle = dead ? mansionMurderTheme.tokens.muted : mansionMurderTheme.tokens.text;
  ctx.font = "900 30px sans-serif";
  ctx.fillText(player.name, rect.x + 96, rect.y + 72);

  ctx.fillStyle = dead ? mansionMurderTheme.tokens.muted : roleColor(player.roleName);
  ctx.font = "800 18px sans-serif";
  ctx.fillText(`身份：${player.roleName}`, rect.x + 96, rect.y + 100);

  ctx.fillStyle = dead ? mansionMurderTheme.tokens.danger : mansionMurderTheme.tokens.good;
  ctx.font = "900 15px sans-serif";
  ctx.fillText(dead ? "DEAD" : "ALIVE", rect.x + rect.width - 68, rect.y + 36);
}

function drawSpeechShot(input: SkinRenderInput): void {
  const { ctx, frame } = input;
  const rect = frame.layout.mainStage;
  const alpha = fadeIn(frame.clock.enterProgress);
  const y = rect.y + slideIn(frame.clock.enterProgress, 26);

  ctx.save();
  ctx.globalAlpha = alpha;
  drawPanel(ctx, { ...rect, y }, {
    fill: "rgba(5, 5, 6, 0.28)",
    stroke: "rgba(143, 211, 255, 0.22)",
  });
  drawTextBlock(ctx, "SUSPECT STATEMENT", rect.x + 46, y + 68, {
    color: mansionMurderTheme.tokens.accent,
    font: "900 24px sans-serif",
    maxWidth: 520,
    lineHeight: 34,
  });

  if (frame.activePlayer) {
    drawSpeakerPortrait(input, frame.activePlayer, {
      ...frame.layout.portrait,
      y: frame.layout.portrait.y + slideIn(frame.clock.enterProgress, 18),
    });
  } else {
    drawTextBlock(ctx, frame.scene.title, rect.x + 56, y + 270, {
      color: mansionMurderTheme.tokens.text,
      font: "900 56px sans-serif",
      maxWidth: rect.width - 112,
      lineHeight: 68,
    });
  }

  drawTextBlock(ctx, frame.scene.title, rect.x + 48, y + rect.height - 54, {
    color: "#c7bba8",
    font: "900 34px sans-serif",
    maxWidth: rect.width - 96,
    lineHeight: 44,
  });
  ctx.restore();
}

function drawSpeakerPortrait(
  input: SkinRenderInput,
  player: PlaybackScenePlayer,
  rect: Rect,
): void {
  const { ctx, frame } = input;
  const avatarImage = player.avatar ? frame.avatarImages[player.avatar] : null;
  const portraitScale = zoom(frame.clock.progress, 1, 1.035);
  const portraitRect = scaleRect(rect, portraitScale);

  drawPanel(ctx, portraitRect, {
    fill: "rgba(8, 10, 12, 0.62)",
    stroke: "rgba(143, 211, 255, 0.36)",
  });

  if (avatarImage) {
    drawCoverImage(ctx, avatarImage, portraitRect.width, portraitRect.height, {
      x: portraitRect.x,
      y: portraitRect.y,
    });
    drawPanel(ctx, portraitRect, { fill: "rgba(0, 0, 0, 0.34)" });
  } else {
    drawFallbackPortrait(ctx, player, portraitRect);
  }

  drawPanel(ctx, {
    x: portraitRect.x,
    y: portraitRect.y + portraitRect.height - 138,
    width: portraitRect.width,
    height: 138,
  }, {
    fill: "rgba(5, 5, 6, 0.70)",
    stroke: "rgba(244, 241, 234, 0.08)",
  });

  ctx.fillStyle = mansionMurderTheme.tokens.muted;
  ctx.font = "900 28px sans-serif";
  ctx.fillText(`${player.seatNo} 号`, portraitRect.x + 48, portraitRect.y + portraitRect.height - 80);
  ctx.fillStyle = mansionMurderTheme.tokens.text;
  ctx.font = "900 72px sans-serif";
  ctx.fillText(player.name, portraitRect.x + 154, portraitRect.y + portraitRect.height - 58);
  ctx.fillStyle = roleColor(player.roleName);
  ctx.font = "900 24px sans-serif";
  ctx.fillText(`身份：${player.roleName}`, portraitRect.x + 48, portraitRect.y + portraitRect.height - 28);
}

function drawFallbackPortrait(
  ctx: CanvasRenderingContext2D,
  player: PlaybackScenePlayer,
  rect: Rect,
): void {
  const gradient = ctx.createLinearGradient(rect.x, rect.y, rect.x + rect.width, rect.y + rect.height);
  gradient.addColorStop(0, "rgba(143, 211, 255, 0.16)");
  gradient.addColorStop(0.55, "rgba(5, 5, 6, 0.58)");
  gradient.addColorStop(1, "rgba(180, 35, 42, 0.18)");
  ctx.fillStyle = gradient;
  ctx.fillRect(rect.x, rect.y, rect.width, rect.height);

  ctx.fillStyle = "rgba(244, 241, 234, 0.08)";
  ctx.beginPath();
  ctx.arc(rect.x + rect.width / 2, rect.y + 160, 86, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillRect(rect.x + rect.width / 2 - 138, rect.y + 258, 276, 180);

  ctx.fillStyle = "rgba(244, 241, 234, 0.16)";
  ctx.font = "900 148px sans-serif";
  ctx.fillText(Array.from(player.name)[0] ?? "?", rect.x + rect.width / 2 - 58, rect.y + 235);
}

function drawVoteShot(input: SkinRenderInput): void {
  drawVoteResultTable(input.ctx, {
    title: input.frame.scene.title,
    rows: input.frame.scene.details.length > 0
      ? input.frame.scene.details
      : [input.frame.scene.text],
    result: input.frame.scene.details.length > 0
      ? [input.frame.scene.text]
      : ["等待本轮投票结算"],
    rect: input.frame.layout.eventPanel,
    colors: tableColors(),
  });
}

function drawResolutionShot(input: SkinRenderInput): void {
  if (input.frame.scene.details.length > 0 && input.frame.scene.phase === "vote") {
    drawVoteResultTable(input.ctx, {
      title: input.frame.scene.title,
      rows: input.frame.scene.details,
      result: [input.frame.scene.text],
      rect: input.frame.layout.eventPanel,
      colors: tableColors(),
    });
    return;
  }

  drawAnnouncementCard(input, "CASE STATUS");
}

function drawPhaseShot(input: SkinRenderInput): void {
  drawAnnouncementCard(input, "CASE FILE");
}

function drawAnnouncementShot(input: SkinRenderInput): void {
  drawAnnouncementCard(input, "CASE BULLETIN");
}

function drawAnnouncementCard(input: SkinRenderInput, label: string): void {
  const { ctx, frame } = input;
  const rect = frame.layout.eventPanel;
  const impactOffset = frame.scene.kind === "announcement"
    ? shake(frame.clock.enterProgress, 8)
    : 0;

  drawPanel(ctx, { ...rect, x: rect.x + impactOffset }, {
    fill: "rgba(9, 8, 8, 0.68)",
    stroke: "rgba(244, 241, 234, 0.18)",
  });
  drawTextBlock(ctx, label, rect.x + 44, rect.y + 76, {
    color: mansionMurderTheme.tokens.danger,
    font: "900 26px sans-serif",
    maxWidth: rect.width - 88,
    lineHeight: 34,
  });
  drawGlowText(ctx, frame.scene.title, rect.x + 44, rect.y + 246, {
    color: mansionMurderTheme.tokens.text,
    glow: "rgba(180, 35, 42, 0.45)",
    font: "900 64px sans-serif",
    maxWidth: rect.width - 88,
    lineHeight: 78,
  });
  drawTextBlock(ctx, frame.scene.text, rect.x + 48, rect.y + 384, {
    color: "#d8d0c4",
    font: "700 34px sans-serif",
    maxWidth: rect.width - 96,
    lineHeight: 48,
  });
}

function drawSubtitle(input: SkinRenderInput): void {
  const cue = input.frame.subtitle;
  if (!cue) {
    return;
  }

  drawSubtitleBar(input.ctx, {
    speaker: cue.speaker,
    text: cue.lines.join("\n"),
    rect: input.frame.layout.subtitle,
    colors: {
      fill: "rgba(5, 5, 6, 0.76)",
      stroke: "rgba(143, 211, 255, 0.22)",
      speaker: mansionMurderTheme.tokens.accent,
      text: mansionMurderTheme.tokens.text,
    },
  });
}

function drawEffects(input: SkinRenderInput): void {
  const { ctx, frame } = input;
  const activeSlot = frame.layout.seatSlots.find(
    (slot) => slot.player.playerId === frame.activePlayer?.playerId,
  );
  if (!activeSlot) {
    return;
  }

  ctx.strokeStyle = "rgba(143, 211, 255, 0.46)";
  ctx.lineWidth = 4;
  ctx.strokeRect(
    activeSlot.rect.x - 6,
    activeSlot.rect.y - 6,
    activeSlot.rect.width + 12,
    activeSlot.rect.height + 12,
  );
}

function tableColors() {
  return {
    fill: "rgba(23, 18, 15, 0.88)",
    stroke: "rgba(180, 35, 42, 0.48)",
    title: mansionMurderTheme.tokens.text,
    text: "#d8d0c4",
    accent: mansionMurderTheme.tokens.danger,
  };
}

function roleColor(roleName: string): string {
  return roleName === "狼人"
    ? mansionMurderTheme.tokens.wolf
    : mansionMurderTheme.tokens.good;
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
  target: { readonly x: number; readonly y: number } = { x: 0, y: 0 },
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
    target.x,
    target.y,
    width,
    height,
  );
}

function scaleRect(rect: Rect, scale: number): Rect {
  const width = rect.width * scale;
  const height = rect.height * scale;

  return {
    x: rect.x + (rect.width - width) / 2,
    y: rect.y + (rect.height - height) / 2,
    width,
    height,
  };
}
