import type { PlaybackScenePlayer } from "@/core/playback";
import {
  clearCanvas,
  drawGlowText,
  drawPanel,
  drawTextBlock,
  drawVignette,
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
    fill: "rgba(0, 0, 0, 0.34)",
  });
  drawSceneFocus(ctx);
}

function drawTopCaseBar(input: SkinRenderInput): void {
  const { ctx, frame } = input;
  drawPanel(ctx, frame.layout.topBar, {
    fill: "rgba(6, 6, 6, 0.76)",
    stroke: "rgba(244, 241, 234, 0.07)",
  });
  drawTextBlock(ctx, "MANSION MURDER", 72, 58, {
    color: "rgba(244, 241, 234, 0.56)",
    font: "800 24px sans-serif",
    maxWidth: 420,
    lineHeight: 32,
  });
  drawTextBlock(ctx, frame.scene.title, 620, 58, {
    color: mansionMurderTheme.tokens.text,
    font: "900 31px sans-serif",
    maxWidth: 720,
    lineHeight: 38,
  });
  drawTextBlock(ctx, `#${frame.scene.index}   ${frame.scene.phase}`, 1570, 58, {
    color: "rgba(199, 187, 168, 0.50)",
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
  const rect = seatDisplayRect(slot, offsetX);
  const avatarImage = player.avatar ? frame.avatarImages[player.avatar] : null;
  const avatarRect = seatAvatarRect(rect, slot.side);
  const statusRect = seatStatusRect(rect, slot.side);
  const textRect = seatTextRect(rect, avatarRect, slot.side);
  const textAlign = slot.side === "left" ? "right" : "left";
  const eventStatus = statusForPlayer(input, player);

  drawPanel(ctx, rect, {
    fill: dead ? "rgba(8, 8, 8, 0.42)" : "rgba(8, 8, 8, 0.58)",
    stroke: highlighted
      ? "rgba(143, 211, 255, 0.64)"
      : "rgba(244, 241, 234, 0.08)",
    lineWidth: highlighted ? 2 : 1,
  });

  drawSeatAvatar(ctx, player, avatarRect, avatarImage, dead);

  drawFittedText(ctx, `Seat ${player.seatNo}`, textRect.x, rect.y + 46, {
    color: "rgba(199, 187, 168, 0.62)",
    weight: 800,
    maxSize: 18,
    minSize: 14,
    maxWidth: textRect.width,
    align: textAlign,
  });

  drawFittedText(ctx, player.name, textRect.x, rect.y + 88, {
    color: dead ? mansionMurderTheme.tokens.muted : mansionMurderTheme.tokens.text,
    weight: active ? 900 : 800,
    maxSize: active ? 36 : 32,
    minSize: 22,
    maxWidth: textRect.width,
    align: textAlign,
  });

  drawFittedText(ctx, `身份：${player.roleName}`, textRect.x, rect.y + 124, {
    color: dead ? mansionMurderTheme.tokens.muted : roleColor(player.roleName),
    weight: 800,
    maxSize: 21,
    minSize: 16,
    maxWidth: textRect.width,
    align: textAlign,
  });

  if (eventStatus) {
    drawPanel(ctx, {
      x: textRect.x,
      y: rect.y + rect.height - 48,
      width: textRect.width,
      height: 30,
    }, {
      fill: eventStatus.tone === "danger"
        ? "rgba(69, 10, 10, 0.36)"
        : "rgba(8, 47, 73, 0.30)",
      stroke: eventStatus.tone === "danger"
        ? "rgba(248, 113, 113, 0.18)"
        : "rgba(143, 211, 255, 0.18)",
      lineWidth: 1,
    });
    drawFittedText(ctx, eventStatus.label, textRect.x + 12, rect.y + rect.height - 27, {
      color: eventStatus.tone === "danger" ? "#fecaca" : mansionMurderTheme.tokens.accent,
      weight: 900,
      maxSize: 17,
      minSize: 13,
      maxWidth: textRect.width - 24,
      align: textAlign,
    });
  }

  drawStatusBadge(ctx, dead ? "DEAD" : "ALIVE", statusRect, {
    fill: dead ? "rgba(69, 10, 10, 0.42)" : "rgba(6, 78, 59, 0.32)",
    stroke: dead ? "rgba(248, 113, 113, 0.28)" : "rgba(134, 239, 172, 0.22)",
    text: dead ? mansionMurderTheme.tokens.danger : mansionMurderTheme.tokens.good,
  });

  if (highlighted) {
    ctx.fillStyle = "rgba(143, 211, 255, 0.58)";
    ctx.fillRect(
      slot.side === "left" ? rect.x : rect.x + rect.width - 4,
      rect.y + 14,
      4,
      rect.height - 28,
    );
  }
}

function drawSeatAvatar(
  ctx: CanvasRenderingContext2D,
  player: PlaybackScenePlayer,
  rect: Rect,
  image: HTMLImageElement | null,
  dead: boolean,
): void {
  ctx.save();
  roundedRectPath(ctx, rect, 22);
  ctx.clip();

  if (image) {
    drawCoverImage(ctx, image, rect.width, rect.height, { x: rect.x, y: rect.y });
    if (dead) {
      drawPanel(ctx, rect, { fill: "rgba(0, 0, 0, 0.44)" });
    }
  } else {
    const gradient = ctx.createLinearGradient(rect.x, rect.y, rect.x + rect.width, rect.y + rect.height);
    gradient.addColorStop(0, dead ? "rgba(41, 37, 36, 0.88)" : "rgba(143, 211, 255, 0.16)");
    gradient.addColorStop(1, "rgba(5, 5, 6, 0.72)");
    ctx.fillStyle = gradient;
    ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
    ctx.fillStyle = dead ? "rgba(139, 129, 120, 0.36)" : "rgba(244, 241, 234, 0.22)";
    ctx.font = "900 88px sans-serif";
    ctx.fillText(Array.from(player.name)[0] ?? "?", rect.x + 42, rect.y + 106);
  }

  ctx.restore();
  ctx.strokeStyle = dead ? "rgba(120, 113, 108, 0.32)" : "rgba(143, 211, 255, 0.42)";
  ctx.lineWidth = 2;
  roundedRectPath(ctx, rect, 22);
  ctx.stroke();
}

function drawSpeechShot(input: SkinRenderInput): void {
  const { ctx, frame } = input;
  const rect = frame.layout.mainStage;
  const alpha = fadeIn(frame.clock.enterProgress);
  const y = rect.y + slideIn(frame.clock.enterProgress, 26);

  ctx.save();
  ctx.globalAlpha = alpha;
  drawPanel(ctx, { ...rect, y }, {
    fill: "rgba(5, 5, 6, 0.08)",
  });
  drawTextBlock(ctx, "SUSPECT STATEMENT", rect.x + 44, y + 46, {
    color: "rgba(143, 211, 255, 0.82)",
    font: "900 22px sans-serif",
    maxWidth: 520,
    lineHeight: 34,
  });

  if (frame.activePlayer) {
    drawSpeakerPortrait(input, frame.activePlayer, {
      x: frame.layout.portrait.x - 44,
      y: frame.layout.portrait.y - 30 + slideIn(frame.clock.enterProgress, 18),
      width: frame.layout.portrait.width + 88,
      height: frame.layout.portrait.height + 74,
    });
  } else {
    drawTextBlock(ctx, frame.scene.title, rect.x + 56, y + 270, {
      color: mansionMurderTheme.tokens.text,
      font: "900 56px sans-serif",
      maxWidth: rect.width - 112,
      lineHeight: 68,
    });
  }

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
    fill: "rgba(8, 10, 12, 0.38)",
    stroke: "rgba(143, 211, 255, 0.18)",
  });

  if (avatarImage) {
    drawCoverImage(ctx, avatarImage, portraitRect.width, portraitRect.height, {
      x: portraitRect.x,
      y: portraitRect.y,
    });
    drawPanel(ctx, portraitRect, { fill: "rgba(0, 0, 0, 0.18)" });
  } else {
    drawFallbackPortrait(ctx, player, portraitRect);
  }

  const lowerThird = {
    x: portraitRect.x,
    y: portraitRect.y + portraitRect.height - 154,
    width: portraitRect.width,
    height: 154,
  };
  drawLowerThirdGradient(ctx, lowerThird);

  ctx.fillStyle = "rgba(199, 187, 168, 0.72)";
  ctx.font = "900 26px sans-serif";
  ctx.fillText(`${player.seatNo} 号`, portraitRect.x + 50, portraitRect.y + portraitRect.height - 84);
  ctx.fillStyle = mansionMurderTheme.tokens.text;
  ctx.font = "900 68px sans-serif";
  ctx.fillText(player.name, portraitRect.x + 156, portraitRect.y + portraitRect.height - 62);
  ctx.fillStyle = roleColor(player.roleName);
  ctx.font = "900 23px sans-serif";
  ctx.fillText(`身份：${player.roleName}`, portraitRect.x + 50, portraitRect.y + portraitRect.height - 30);
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
  drawVoteBoard(input, {
    title: input.frame.scene.title,
    rows: input.frame.scene.details,
    result: input.frame.scene.details.length > 0
      ? input.frame.scene.text
      : "等待所有玩家完成投票",
    mode: "pending",
  });
}

function drawResolutionShot(input: SkinRenderInput): void {
  if (input.frame.scene.details.length > 0 && input.frame.scene.phase === "vote") {
    drawVoteBoard(input, {
      title: input.frame.scene.title,
      rows: input.frame.scene.details,
      result: input.frame.scene.text,
      mode: "resolved",
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

function drawVoteBoard(
  input: SkinRenderInput,
  options: {
    readonly title: string;
    readonly rows: readonly string[];
    readonly result: string;
    readonly mode: "pending" | "resolved";
  },
): void {
  const { ctx, frame } = input;
  const rect = {
    x: frame.layout.eventPanel.x - 40,
    y: frame.layout.eventPanel.y - 28,
    width: frame.layout.eventPanel.width + 80,
    height: frame.layout.eventPanel.height + 20,
  };
  const votes = options.rows.map(parseVoteRow);
  const tallies = voteTallies(votes);
  const winner = tallies[0];

  drawPanel(ctx, rect, {
    fill: "rgba(8, 8, 8, 0.58)",
    stroke: options.mode === "resolved"
      ? "rgba(180, 35, 42, 0.28)"
      : "rgba(143, 211, 255, 0.22)",
    lineWidth: 1,
  });

  drawPanel(ctx, {
    x: rect.x,
    y: rect.y,
    width: rect.width,
    height: 82,
  }, {
    fill: "rgba(0, 0, 0, 0.26)",
  });

  ctx.fillStyle = mansionMurderTheme.tokens.text;
  ctx.font = "900 38px sans-serif";
  ctx.fillText(options.title, rect.x + 36, rect.y + 54);

  ctx.fillStyle = options.mode === "resolved"
    ? mansionMurderTheme.tokens.danger
    : mansionMurderTheme.tokens.accent;
  ctx.font = "900 20px sans-serif";
  ctx.fillText(options.mode === "resolved" ? "EXILE RESULT" : "VOTE IN PROGRESS", rect.x + rect.width - 250, rect.y + 52);

  const left = {
    x: rect.x + 36,
    y: rect.y + 120,
    width: 480,
    height: rect.height - 166,
  };
  const right = {
    x: rect.x + 544,
    y: rect.y + 120,
    width: rect.width - 580,
    height: rect.height - 166,
  };

  drawSectionLabel(ctx, "投票明细", left.x, left.y);
  if (votes.length > 0) {
    votes.slice(0, 8).forEach((vote, index) => {
      const y = left.y + 46 + index * 42;
      ctx.fillStyle = "rgba(244, 241, 234, 0.72)";
      ctx.font = "800 22px sans-serif";
      drawFittedText(ctx, vote.voter, left.x, y, {
        color: "rgba(244, 241, 234, 0.72)",
        weight: 800,
        maxSize: 22,
        minSize: 17,
        maxWidth: 170,
      });
      ctx.fillStyle = "rgba(143, 211, 255, 0.54)";
      ctx.font = "900 20px sans-serif";
      ctx.fillText("->", left.x + 174, y);
      drawFittedText(ctx, vote.target, left.x + 214, y, {
        color: vote.target === winner?.target
          ? mansionMurderTheme.tokens.danger
          : mansionMurderTheme.tokens.text,
        weight: 900,
        maxSize: 24,
        minSize: 18,
        maxWidth: left.width - 224,
      });
    });
  } else {
    ctx.fillStyle = "rgba(244, 241, 234, 0.62)";
    ctx.font = "800 26px sans-serif";
    ctx.fillText("等待投票记录生成", left.x, left.y + 54);
  }

  drawPanel(ctx, {
    x: right.x,
    y: right.y - 22,
    width: right.width,
    height: right.height + 36,
  }, {
    fill: "rgba(0, 0, 0, 0.24)",
    stroke: "rgba(244, 241, 234, 0.08)",
    lineWidth: 1,
  });

  drawSectionLabel(ctx, "票型统计", right.x + 28, right.y + 2);
  if (tallies.length > 0) {
    tallies.slice(0, 4).forEach((tally, index) => {
      const y = right.y + 54 + index * 46;
      const barWidth = Math.max(18, (right.width - 184) * (tally.count / votes.length));
      ctx.fillStyle = tally.target === winner?.target
        ? "rgba(180, 35, 42, 0.48)"
        : "rgba(143, 211, 255, 0.22)";
      ctx.fillRect(right.x + 28, y - 24, barWidth, 28);
      ctx.fillStyle = mansionMurderTheme.tokens.text;
      ctx.font = "900 23px sans-serif";
      drawFittedText(ctx, tally.target, right.x + 36, y, {
        color: mansionMurderTheme.tokens.text,
        weight: 900,
        maxSize: 23,
        minSize: 18,
        maxWidth: right.width - 150,
      });
      drawFittedText(ctx, `${tally.count} 票`, right.x + right.width - 96, y, {
        color: tally.target === winner?.target
          ? "#fecaca"
          : "rgba(244, 241, 234, 0.72)",
        weight: 900,
        maxSize: 22,
        minSize: 18,
        maxWidth: 72,
      });
    });
  } else {
    ctx.fillStyle = "rgba(244, 241, 234, 0.58)";
    ctx.font = "800 24px sans-serif";
    ctx.fillText("尚无票型统计", right.x + 28, right.y + 58);
  }

  const resultBox = {
    x: right.x + 28,
    y: right.y + right.height - 134,
    width: right.width - 56,
    height: 112,
  };
  const resultSummary = summarizeVoteResult(options.result);
  drawPanel(ctx, resultBox, {
    fill: options.mode === "resolved"
      ? "rgba(69, 10, 10, 0.36)"
      : "rgba(8, 47, 73, 0.30)",
    stroke: options.mode === "resolved"
      ? "rgba(248, 113, 113, 0.22)"
      : "rgba(143, 211, 255, 0.18)",
    lineWidth: 1,
  });
  ctx.fillStyle = options.mode === "resolved"
    ? "#fecaca"
    : mansionMurderTheme.tokens.accent;
  ctx.font = "900 19px sans-serif";
  ctx.fillText(
    resultSummary.label,
    resultBox.x + 24,
    resultBox.y + 36,
  );
  drawFittedText(ctx, resultSummary.value, resultBox.x + 24, resultBox.y + 82, {
    color: mansionMurderTheme.tokens.text,
    weight: 900,
    maxSize: options.mode === "resolved" ? 34 : 26,
    minSize: 18,
    maxWidth: resultBox.width - 48,
  });
}

function drawSectionLabel(
  ctx: CanvasRenderingContext2D,
  label: string,
  x: number,
  y: number,
): void {
  ctx.fillStyle = "rgba(143, 211, 255, 0.78)";
  ctx.font = "900 18px sans-serif";
  ctx.fillText(label, x, y);
}

function drawSubtitle(input: SkinRenderInput): void {
  const cue = input.frame.subtitle;
  if (!cue) {
    return;
  }

  const { ctx, frame } = input;
  drawSubtitleGradient(ctx, {
    x: 0,
    y: frame.layout.subtitle.y - 54,
    width: WIDTH,
    height: HEIGHT - frame.layout.subtitle.y + 54,
  });

  ctx.fillStyle = mansionMurderTheme.tokens.accent;
  ctx.font = "900 27px sans-serif";
  ctx.fillText(cue.speaker, frame.layout.subtitle.x + 36, frame.layout.subtitle.y + 22);

  drawTextBlock(ctx, cue.lines.join("\n"), frame.layout.subtitle.x + 36, frame.layout.subtitle.y + 74, {
    color: mansionMurderTheme.tokens.text,
    font: "800 34px sans-serif",
    maxWidth: frame.layout.subtitle.width - 72,
    lineHeight: 42,
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

  ctx.strokeStyle = "rgba(143, 211, 255, 0.34)";
  ctx.lineWidth = 2;
  const rect = seatDisplayRect(activeSlot, 0);
  ctx.strokeRect(rect.x - 4, rect.y - 4, rect.width + 8, rect.height + 8);
}

function roleColor(roleName: string): string {
  return roleName === "狼人"
    ? mansionMurderTheme.tokens.wolf
    : mansionMurderTheme.tokens.good;
}

function parseVoteRow(row: string): {
  readonly voter: string;
  readonly target: string;
} {
  const [voter, target] = row.split("->").map((part) => part.trim());

  return {
    voter: voter || row,
    target: target || "弃票",
  };
}

function seatDisplayRect(
  slot: { readonly side: "left" | "right"; readonly rect: Rect },
  offsetX: number,
): Rect {
  return {
    x: slot.rect.x + offsetX,
    y: slot.rect.y + 10,
    width: slot.rect.width,
    height: slot.rect.height - 22,
  };
}

function seatAvatarRect(rect: Rect, side: "left" | "right"): Rect {
  const size = Math.min(168, rect.height - 20);

  return {
    x: side === "left"
      ? rect.x + rect.width - size - 18
      : rect.x + 18,
    y: rect.y + (rect.height - size) / 2,
    width: size,
    height: size,
  };
}

function seatStatusRect(rect: Rect, side: "left" | "right"): Rect {
  return {
    x: side === "left" ? rect.x + 16 : rect.x + rect.width - 74,
    y: rect.y + 16,
    width: 58,
    height: 26,
  };
}

function seatTextRect(
  rect: Rect,
  avatarRect: Rect,
  side: "left" | "right",
): Rect {
  if (side === "left") {
    const x = rect.x + 38;
    const right = avatarRect.x - 18;
    return {
      x,
      y: rect.y + 52,
      width: Math.max(120, right - x),
      height: rect.height - 70,
    };
  }

  const x = avatarRect.x + avatarRect.width + 18;
  const right = rect.x + rect.width - 38;
  return {
    x,
    y: rect.y + 52,
    width: Math.max(120, right - x),
    height: rect.height - 70,
  };
}

function drawStatusBadge(
  ctx: CanvasRenderingContext2D,
  label: string,
  rect: Rect,
  colors: {
    readonly fill: string;
    readonly stroke: string;
    readonly text: string;
  },
): void {
  drawPanel(ctx, rect, {
    fill: colors.fill,
    stroke: colors.stroke,
    lineWidth: 1,
  });
  drawFittedText(ctx, label, rect.x + 8, rect.y + 18, {
    color: colors.text,
    weight: 900,
    maxSize: 13,
    minSize: 10,
    maxWidth: rect.width - 16,
    align: "center",
  });
}

function statusForPlayer(
  input: SkinRenderInput,
  player: PlaybackScenePlayer,
): { readonly label: string; readonly tone: "accent" | "danger" } | null {
  if (player.status === "dead") {
    return { label: "已死亡", tone: "danger" };
  }

  if (input.frame.activePlayer?.playerId === player.playerId) {
    return { label: "正在发言", tone: "accent" };
  }

  if (input.frame.scene.kind === "resolution") {
    const result = summarizeVoteResult(input.frame.scene.text);
    if (result.label === "放逐出局" && result.value.includes(player.name)) {
      return { label: "放逐出局", tone: "danger" };
    }
  }

  return null;
}

function roundedRectPath(
  ctx: CanvasRenderingContext2D,
  rect: Rect,
  radius: number,
): void {
  const safeRadius = Math.min(radius, rect.width / 2, rect.height / 2);
  ctx.beginPath();
  ctx.moveTo(rect.x + safeRadius, rect.y);
  ctx.lineTo(rect.x + rect.width - safeRadius, rect.y);
  ctx.quadraticCurveTo(rect.x + rect.width, rect.y, rect.x + rect.width, rect.y + safeRadius);
  ctx.lineTo(rect.x + rect.width, rect.y + rect.height - safeRadius);
  ctx.quadraticCurveTo(rect.x + rect.width, rect.y + rect.height, rect.x + rect.width - safeRadius, rect.y + rect.height);
  ctx.lineTo(rect.x + safeRadius, rect.y + rect.height);
  ctx.quadraticCurveTo(rect.x, rect.y + rect.height, rect.x, rect.y + rect.height - safeRadius);
  ctx.lineTo(rect.x, rect.y + safeRadius);
  ctx.quadraticCurveTo(rect.x, rect.y, rect.x + safeRadius, rect.y);
  ctx.closePath();
}

function voteTallies(
  votes: readonly { readonly target: string }[],
): readonly { readonly target: string; readonly count: number }[] {
  const counts = new Map<string, number>();
  votes.forEach((vote) => {
    counts.set(vote.target, (counts.get(vote.target) ?? 0) + 1);
  });

  return [...counts.entries()]
    .map(([target, count]) => ({ target, count }))
    .sort((left, right) => right.count - left.count || left.target.localeCompare(right.target));
}

function summarizeVoteResult(result: string): {
  readonly label: string;
  readonly value: string;
} {
  const normalized = result.replace(/[。.]$/u, "").trim();
  const [label, value] = normalized.split(/[:：]/u).map((part) => part.trim());

  if (label && value) {
    return { label, value };
  }

  if (normalized.endsWith("出局") && normalized !== "无人出局") {
    return {
      label: "放逐出局",
      value: normalized.slice(0, -"出局".length).trim(),
    };
  }

  return {
    label: "当前状态",
    value: normalized || "等待投票记录",
  };
}

function drawFittedText(
  ctx: CanvasRenderingContext2D,
  text: string,
  x: number,
  y: number,
  options: {
    readonly color: string;
    readonly weight: number;
    readonly maxSize: number;
    readonly minSize: number;
    readonly maxWidth: number;
    readonly align?: "left" | "center" | "right";
  },
): void {
  let size = options.maxSize;
  ctx.fillStyle = options.color;
  ctx.font = `${options.weight} ${size}px sans-serif`;

  while (
    size > options.minSize &&
    ctx.measureText(text).width > options.maxWidth
  ) {
    size -= 1;
    ctx.font = `${options.weight} ${size}px sans-serif`;
  }

  const width = ctx.measureText(text).width;
  const drawX = options.align === "right"
    ? x + options.maxWidth - width
    : options.align === "center"
      ? x + (options.maxWidth - width) / 2
      : x;
  ctx.fillText(text, drawX, y);
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

function drawSceneFocus(ctx: CanvasRenderingContext2D): void {
  let gradient = ctx.createLinearGradient(0, 0, 440, 0);
  gradient.addColorStop(0, "rgba(0, 0, 0, 0.54)");
  gradient.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 96, 480, HEIGHT - 96);

  gradient = ctx.createLinearGradient(WIDTH, 0, WIDTH - 440, 0);
  gradient.addColorStop(0, "rgba(0, 0, 0, 0.54)");
  gradient.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = gradient;
  ctx.fillRect(WIDTH - 480, 96, 480, HEIGHT - 96);

  gradient = ctx.createLinearGradient(0, 120, 0, 380);
  gradient.addColorStop(0, "rgba(0, 0, 0, 0.34)");
  gradient.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 96, WIDTH, 320);
}

function drawLowerThirdGradient(ctx: CanvasRenderingContext2D, rect: Rect): void {
  const gradient = ctx.createLinearGradient(0, rect.y, 0, rect.y + rect.height);
  gradient.addColorStop(0, "rgba(0, 0, 0, 0)");
  gradient.addColorStop(0.28, "rgba(0, 0, 0, 0.62)");
  gradient.addColorStop(1, "rgba(0, 0, 0, 0.88)");
  ctx.fillStyle = gradient;
  ctx.fillRect(rect.x, rect.y, rect.width, rect.height);

  ctx.strokeStyle = "rgba(244, 241, 234, 0.09)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(rect.x, rect.y + 1);
  ctx.lineTo(rect.x + rect.width, rect.y + 1);
  ctx.stroke();
}

function drawSubtitleGradient(ctx: CanvasRenderingContext2D, rect: Rect): void {
  const gradient = ctx.createLinearGradient(0, rect.y, 0, rect.y + rect.height);
  gradient.addColorStop(0, "rgba(0, 0, 0, 0)");
  gradient.addColorStop(0.32, "rgba(0, 0, 0, 0.62)");
  gradient.addColorStop(1, "rgba(0, 0, 0, 0.90)");
  ctx.fillStyle = gradient;
  ctx.fillRect(rect.x, rect.y, rect.width, rect.height);
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
