import type { PlaybackScenePlayer } from "@/core/playback";

export type Rect = {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
};

export type Point = {
  readonly x: number;
  readonly y: number;
};

export type Circle = {
  readonly x: number;
  readonly y: number;
  readonly radius: number;
};

export function clearCanvas(
  ctx: CanvasRenderingContext2D,
  color: string,
): void {
  ctx.fillStyle = color;
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
}

export function drawTextBlock(
  ctx: CanvasRenderingContext2D,
  text: string,
  x: number,
  y: number,
  options: {
    readonly color: string;
    readonly font: string;
    readonly maxWidth: number;
    readonly lineHeight: number;
  },
): void {
  ctx.fillStyle = options.color;
  ctx.font = options.font;
  ctx.textBaseline = "alphabetic";

  let line = "";
  let currentY = y;
  for (const character of Array.from(text)) {
    const nextLine = `${line}${character}`;
    if (line && ctx.measureText(nextLine).width > options.maxWidth) {
      ctx.fillText(line, x, currentY);
      line = character;
      currentY += options.lineHeight;
    } else {
      line = nextLine;
    }
  }

  if (line) {
    ctx.fillText(line, x, currentY);
  }
}

export function drawPanel(
  ctx: CanvasRenderingContext2D,
  rect: Rect,
  options: {
    readonly fill: string;
    readonly stroke?: string;
    readonly lineWidth?: number;
  },
): void {
  ctx.fillStyle = options.fill;
  ctx.fillRect(rect.x, rect.y, rect.width, rect.height);

  if (options.stroke) {
    ctx.strokeStyle = options.stroke;
    ctx.lineWidth = options.lineWidth ?? 2;
    ctx.strokeRect(rect.x, rect.y, rect.width, rect.height);
  }
}

export function drawAvatar(
  ctx: CanvasRenderingContext2D,
  name: string,
  circle: Circle,
  options: {
    readonly fill: string;
    readonly stroke: string;
    readonly text: string;
    readonly image?: HTMLImageElement | null;
  },
): void {
  if (options.image) {
    ctx.save();
    ctx.beginPath();
    ctx.arc(circle.x, circle.y, circle.radius, 0, Math.PI * 2);
    ctx.clip();
    drawCoverImageInCircle(ctx, options.image, circle);
    ctx.restore();

    ctx.strokeStyle = options.stroke;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.arc(circle.x, circle.y, circle.radius, 0, Math.PI * 2);
    ctx.stroke();
    return;
  }

  const initial = Array.from(name)[0] ?? "";
  ctx.fillStyle = options.fill;
  ctx.strokeStyle = options.stroke;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.arc(circle.x, circle.y, circle.radius, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();

  ctx.fillStyle = options.text;
  ctx.font = "700 28px sans-serif";
  ctx.textBaseline = "alphabetic";
  ctx.fillText(initial, circle.x - 12, circle.y + 10);
}

function drawCoverImageInCircle(
  ctx: CanvasRenderingContext2D,
  image: HTMLImageElement,
  circle: Circle,
): void {
  const size = circle.radius * 2;
  const imageRatio = image.width / image.height;
  const sourceWidth = imageRatio > 1 ? image.height : image.width;
  const sourceHeight = imageRatio > 1 ? image.height : image.width;
  const sourceX = (image.width - sourceWidth) / 2;
  const sourceY = (image.height - sourceHeight) / 2;

  ctx.drawImage(
    image,
    sourceX,
    sourceY,
    sourceWidth,
    sourceHeight,
    circle.x - circle.radius,
    circle.y - circle.radius,
    size,
    size,
  );
}

export function drawStatusStamp(
  ctx: CanvasRenderingContext2D,
  label: string,
  rect: Rect,
  options: {
    readonly fill: string;
    readonly stroke: string;
    readonly text: string;
  },
): void {
  drawPanel(ctx, rect, {
    fill: options.fill,
    stroke: options.stroke,
    lineWidth: 2,
  });
  ctx.fillStyle = options.text;
  ctx.font = "800 18px sans-serif";
  ctx.textBaseline = "alphabetic";
  ctx.fillText(label, rect.x + 22, rect.y + 29);
}

export function drawGlowText(
  ctx: CanvasRenderingContext2D,
  text: string,
  x: number,
  y: number,
  options: {
    readonly color: string;
    readonly glow: string;
    readonly font: string;
    readonly maxWidth: number;
    readonly lineHeight: number;
  },
): void {
  ctx.save();
  ctx.shadowColor = options.glow;
  ctx.shadowBlur = 18;
  drawTextBlock(ctx, text, x, y, options);
  ctx.restore();
}

export function drawSubtitleBar(
  ctx: CanvasRenderingContext2D,
  options: {
    readonly speaker: string;
    readonly text: string;
    readonly rect: Rect;
    readonly colors: {
      readonly fill: string;
      readonly stroke: string;
      readonly speaker: string;
      readonly text: string;
    };
  },
): void {
  drawPanel(ctx, options.rect, {
    fill: options.colors.fill,
    stroke: options.colors.stroke,
    lineWidth: 2,
  });
  ctx.fillStyle = options.colors.speaker;
  ctx.font = "800 28px sans-serif";
  ctx.textBaseline = "alphabetic";
  ctx.fillText(options.speaker, options.rect.x + 36, options.rect.y + 56);

  ctx.fillStyle = options.colors.text;
  ctx.font = "700 34px sans-serif";
  drawTextBlock(ctx, options.text, options.rect.x + 36, options.rect.y + 110, {
    color: options.colors.text,
    font: "700 34px sans-serif",
    maxWidth: options.rect.width - 72,
    lineHeight: 42,
  });
}

export function drawPlayerFile(
  ctx: CanvasRenderingContext2D,
  player: PlaybackScenePlayer,
  rect: Rect,
  options: {
    readonly fill: string;
    readonly stroke: string;
    readonly text: string;
    readonly muted: string;
    readonly statusAlive: string;
    readonly statusDead: string;
  },
): void {
  drawPanel(ctx, rect, {
    fill: options.fill,
    stroke: player.highlighted ? options.stroke : "rgba(82, 82, 91, 0.75)",
  });
  ctx.font = "500 20px sans-serif";
  ctx.fillStyle = options.muted;
  ctx.fillText(`Seat ${player.seatNo}`, rect.x + 20, rect.y + 46);
  ctx.font = "700 28px sans-serif";
  ctx.fillStyle = player.status === "dead" ? options.muted : options.text;
  ctx.fillText(player.name, rect.x + 122, rect.y + 47);
  ctx.font = "600 20px sans-serif";
  ctx.fillStyle =
    player.status === "dead" ? options.statusDead : options.statusAlive;
  ctx.fillText(player.status, rect.x + rect.width - 90, rect.y + 46);
}

export function drawVoteResultTable(
  ctx: CanvasRenderingContext2D,
  options: {
    readonly title: string;
    readonly rows: readonly string[];
    readonly result: readonly string[];
    readonly rect: Rect;
    readonly colors: {
      readonly fill: string;
      readonly stroke: string;
      readonly title: string;
      readonly text: string;
      readonly accent: string;
    };
  },
): void {
  drawPanel(ctx, options.rect, {
    fill: options.colors.fill,
    stroke: options.colors.stroke,
    lineWidth: 2,
  });

  const x = options.rect.x + 32;
  ctx.fillStyle = options.colors.title;
  ctx.font = "800 34px sans-serif";
  ctx.textBaseline = "alphabetic";
  ctx.fillText(options.title, x, options.rect.y + 54);

  ctx.fillStyle = options.colors.text;
  ctx.font = "600 24px sans-serif";
  options.rows.forEach((row, index) => {
    ctx.fillText(row, x, options.rect.y + 112 + index * 38);
  });

  ctx.fillStyle = options.colors.accent;
  ctx.font = "800 26px sans-serif";
  ctx.fillText("本轮结果", x, options.rect.y + 242);

  ctx.fillStyle = options.colors.text;
  ctx.font = "700 24px sans-serif";
  options.result.forEach((row, index) => {
    ctx.fillText(row, x, options.rect.y + 288 + index * 36);
  });
}

export function drawVoteLine(
  ctx: CanvasRenderingContext2D,
  from: Point,
  to: Point,
  color: string,
  lineWidth = 4,
): void {
  ctx.strokeStyle = color;
  ctx.lineWidth = lineWidth;
  ctx.beginPath();
  ctx.moveTo(from.x, from.y);
  ctx.lineTo(to.x, to.y);
  ctx.stroke();
}

export function drawVignette(ctx: CanvasRenderingContext2D): void {
  const gradient = ctx.createLinearGradient(0, 0, 0, ctx.canvas.height);
  gradient.addColorStop(0, "rgba(0, 0, 0, 0.55)");
  gradient.addColorStop(0.5, "rgba(0, 0, 0, 0)");
  gradient.addColorStop(1, "rgba(0, 0, 0, 0.68)");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
}
