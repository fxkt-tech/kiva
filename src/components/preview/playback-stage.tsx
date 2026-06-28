"use client";

import { useEffect, useRef, useState } from "react";
import {
  playbackIndexAtMs,
  playbackTotalDurationMs,
  type PlaybackItem,
} from "@/core/playback";

const CANVAS_WIDTH = 1920;
const CANVAS_HEIGHT = 1080;
const FRAME_RATE = 30;
const TICK_MS = 1000 / FRAME_RATE;

type PlaybackStageProps = {
  readonly items: readonly PlaybackItem[];
  readonly controls?: "visible" | "hidden";
};

type RecordingStatus = "idle" | "recording" | "ready" | "failed";

export function PlaybackStage({
  items,
  controls = "visible",
}: PlaybackStageProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const downloadUrlRef = useRef<string | null>(null);
  const [currentTimeMs, setCurrentTimeMs] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [recordingStatus, setRecordingStatus] =
    useState<RecordingStatus>("idle");
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const hasItems = items.length > 0;
  const totalDurationMs = playbackTotalDurationMs(items);
  const safeTimeMs = clamp(currentTimeMs, 0, Math.max(0, totalDurationMs));
  const safeIndex = hasItems ? playbackIndexAtMs(items, safeTimeMs) : 0;
  const progress = hasItems ? `${safeIndex + 1} / ${items.length}` : "0 / 0";
  const canPlay = hasItems && safeTimeMs < totalDurationMs;
  const canRecord = hasItems && recordingStatus !== "recording";

  useEffect(() => {
    setCurrentTimeMs((timeMs) => clamp(timeMs, 0, Math.max(0, totalDurationMs)));
  }, [totalDurationMs]);

  useEffect(() => {
    drawPlaybackFrame(canvasRef.current, items, safeTimeMs);
  }, [items, safeTimeMs]);

  useEffect(() => {
    if (!playing || !hasItems) {
      return;
    }

    const intervalId = window.setInterval(() => {
      setCurrentTimeMs((timeMs) => {
        const nextTimeMs = Math.min(totalDurationMs, timeMs + TICK_MS);
        if (nextTimeMs >= totalDurationMs) {
          window.clearInterval(intervalId);
          setPlaying(false);
          stopRecording();
        }

        return nextTimeMs;
      });
    }, TICK_MS);

    return () => window.clearInterval(intervalId);
  }, [hasItems, playing, totalDurationMs]);

  useEffect(() => {
    return () => {
      if (downloadUrlRef.current) {
        URL.revokeObjectURL(downloadUrlRef.current);
      }
    };
  }, []);

  function play() {
    if (!hasItems) {
      return;
    }

    if (safeTimeMs >= totalDurationMs) {
      setCurrentTimeMs(0);
    }
    setPlaying(true);
  }

  function pause() {
    setPlaying(false);
  }

  function reset() {
    setCurrentTimeMs(0);
    setPlaying(false);
  }

  function seekTo(timeMs: number) {
    setCurrentTimeMs(clamp(timeMs, 0, Math.max(0, totalDurationMs)));
    setPlaying(false);
  }

  function startRecording() {
    const canvas = canvasRef.current;
    setError(null);

    if (!canvas || !hasItems) {
      return;
    }

    if (!canvas.captureStream || !window.MediaRecorder) {
      setRecordingStatus("failed");
      setError("Canvas recording is not supported.");
      return;
    }

    if (downloadUrlRef.current) {
      URL.revokeObjectURL(downloadUrlRef.current);
      downloadUrlRef.current = null;
      setDownloadUrl(null);
    }

    try {
      const stream = canvas.captureStream(FRAME_RATE);
      const mimeType = preferredMimeType();
      const recorder = new MediaRecorder(
        stream,
        mimeType ? { mimeType } : undefined,
      );

      chunksRef.current = [];
      recorderRef.current = recorder;
      recorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          chunksRef.current.push(event.data);
        }
      };
      recorder.onstop = () => {
        stream.getTracks().forEach((track) => {
          track.stop();
        });
        const blob = new Blob(chunksRef.current, {
          type: recorder.mimeType || "video/webm",
        });
        const nextDownloadUrl = URL.createObjectURL(blob);
        downloadUrlRef.current = nextDownloadUrl;
        recorderRef.current = null;
        setDownloadUrl(nextDownloadUrl);
        setRecordingStatus("ready");
      };

      drawPlaybackFrame(canvas, items, 0);
      setCurrentTimeMs(0);
      recorder.start();
      setRecordingStatus("recording");
      setPlaying(true);
    } catch (caught) {
      recorderRef.current = null;
      setPlaying(false);
      setRecordingStatus("failed");
      setError(caught instanceof Error ? caught.message : String(caught));
    }
  }

  function stopRecording() {
    const recorder = recorderRef.current;
    if (recorder && recorder.state !== "inactive") {
      recorder.stop();
    }
  }

  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4 bg-black p-4 text-white">
      <section className="aspect-video w-full max-w-6xl overflow-hidden bg-black shadow-2xl shadow-black">
        <canvas
          aria-label="Playback canvas"
          className="h-full w-full bg-black"
          height={CANVAS_HEIGHT}
          ref={canvasRef}
          width={CANVAS_WIDTH}
        />
        {!hasItems ? <span className="sr-only">No playable scenes</span> : null}
      </section>
      {controls === "visible" ? (
        <section
          aria-label="Playback controls"
          className="grid w-full max-w-6xl gap-3 border border-zinc-800 bg-zinc-950 px-4 py-3 text-sm text-zinc-200 shadow-lg shadow-black/40"
        >
          <div className="grid gap-2">
            <div className="flex items-center justify-between gap-4 text-xs text-zinc-500">
              <span>Timeline</span>
              <span className="font-mono">
                {formatTime(safeTimeMs)} / {formatTime(totalDurationMs)}
              </span>
            </div>
            <input
              aria-label="Timeline"
              className="h-2 w-full accent-cyan-400"
              disabled={!hasItems || recordingStatus === "recording"}
              max={Math.max(0, totalDurationMs)}
              min={0}
              onChange={(event) => seekTo(Number(event.currentTarget.value))}
              step={100}
              type="range"
              value={safeTimeMs}
            />
          </div>
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div className="font-mono text-zinc-400">{progress}</div>
            <div className="flex flex-wrap items-center gap-2">
              <button
                className="border border-red-900/70 px-3 py-2 text-red-200 transition hover:border-red-500 hover:bg-red-950/40 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
                disabled={!canRecord}
                onClick={startRecording}
                type="button"
              >
                Record
              </button>
              <button
                className="border border-zinc-700 px-3 py-2 text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
                disabled={!canPlay}
                onClick={playing ? pause : play}
                type="button"
              >
                {playing ? "Pause" : "Play"}
              </button>
              <button
                className="border border-zinc-700 px-3 py-2 text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
                disabled={!hasItems || (safeTimeMs === 0 && !playing)}
                onClick={reset}
                type="button"
              >
                Reset
              </button>
              {downloadUrl ? (
                <a
                  className="border border-emerald-800 px-3 py-2 text-emerald-200 transition hover:border-emerald-500 hover:bg-emerald-950/40"
                  download="kiva-playback.webm"
                  href={downloadUrl}
                >
                  Download WebM
                </a>
              ) : null}
              {recordingStatus === "recording" ? (
                <span className="text-xs text-red-300">Recording</span>
              ) : null}
              {recordingStatus === "ready" ? (
                <span className="text-xs text-emerald-300">Ready</span>
              ) : null}
              {error ? <span className="text-xs text-red-300">{error}</span> : null}
            </div>
          </div>
        </section>
      ) : null}
    </main>
  );
}

function drawPlaybackFrame(
  canvas: HTMLCanvasElement | null,
  items: readonly PlaybackItem[],
  timeMs: number,
): void {
  if (!canvas) {
    return;
  }

  const context = canvas.getContext("2d");
  if (!context) {
    return;
  }

  const scene = items[playbackIndexAtMs(items, timeMs)];
  drawBackground(context);

  if (!scene) {
    drawText(context, "No playable scenes", 120, 520, {
      color: "#d4d4d8",
      font: "600 64px sans-serif",
      maxWidth: 1680,
      lineHeight: 78,
    });
    return;
  }

  drawSceneMeta(context, scene);
  drawPlayers(context, scene);
  drawSceneContent(context, scene);
  drawSceneDetails(context, scene);
}

function drawBackground(context: CanvasRenderingContext2D): void {
  context.fillStyle = "#050506";
  context.fillRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);
  context.fillStyle = "#0b0f12";
  context.fillRect(0, 0, CANVAS_WIDTH, 108);
  context.strokeStyle = "#18181b";
  context.lineWidth = 2;
  context.beginPath();
  context.moveTo(0, 108);
  context.lineTo(CANVAS_WIDTH, 108);
  context.stroke();
}

function drawSceneMeta(
  context: CanvasRenderingContext2D,
  scene: PlaybackItem,
): void {
  drawText(context, `#${scene.index}   ${scene.phase}   ${scene.kind}`, 96, 66, {
    color: "#71717a",
    font: "600 24px sans-serif",
    maxWidth: 1100,
    lineHeight: 32,
  });
}

function drawSceneContent(
  context: CanvasRenderingContext2D,
  scene: PlaybackItem,
): void {
  const label = scene.kind === "speech"
    ? "Speaker"
    : scene.kind === "vote"
      ? "Vote card"
      : scene.kind === "resolution"
        ? "Resolution"
        : scene.kind === "phase"
          ? "Phase"
          : "Announcement";
  const labelColor = scene.kind === "vote"
    ? "#fbbf24"
    : scene.kind === "resolution"
      ? "#fca5a5"
      : scene.kind === "speech"
        ? "#67e8f9"
        : "#71717a";

  drawText(context, label, 104, 232, {
    color: labelColor,
    font: "700 28px sans-serif",
    maxWidth: 1000,
    lineHeight: 36,
  });

  if (scene.kind === "speech") {
    const speaker = scene.players.find((player) => player.highlighted);
    if (speaker) {
      drawText(context, `Seat ${speaker.seatNo}  ${speaker.name}`, 104, 315, {
        color: "#f4f4f5",
        font: "700 68px sans-serif",
        maxWidth: 1180,
        lineHeight: 78,
      });
    }
    drawText(context, scene.text, 128, 450, {
      color: "#e4e4e7",
      font: "400 46px sans-serif",
      maxWidth: 1160,
      lineHeight: 68,
    });
    context.strokeStyle = "#22d3ee";
    context.lineWidth = 5;
    context.beginPath();
    context.moveTo(104, 448);
    context.lineTo(104, 850);
    context.stroke();
    return;
  }

  const titleFont = scene.kind === "phase"
    ? "800 92px sans-serif"
    : "800 72px sans-serif";
  drawText(context, scene.title, 104, scene.kind === "phase" ? 420 : 330, {
    color: "#fafafa",
    font: titleFont,
    maxWidth: 1220,
    lineHeight: scene.kind === "phase" ? 106 : 86,
  });

  if (scene.text) {
    if (scene.kind === "vote") {
      context.fillStyle = "rgba(251, 191, 36, 0.10)";
      context.strokeStyle = "rgba(251, 191, 36, 0.35)";
      context.lineWidth = 2;
      context.fillRect(104, 500, 1040, 150);
      context.strokeRect(104, 500, 1040, 150);
      drawText(context, scene.text, 138, 585, {
        color: "#fef3c7",
        font: "500 46px sans-serif",
        maxWidth: 970,
        lineHeight: 60,
      });
      return;
    }

    drawText(context, scene.text, 104, scene.kind === "phase" ? 575 : 490, {
      color: "#d4d4d8",
      font: "400 46px sans-serif",
      maxWidth: 1220,
      lineHeight: 66,
    });
  }
}

function drawPlayers(
  context: CanvasRenderingContext2D,
  scene: PlaybackItem,
): void {
  const x = 1400;
  drawText(context, "Players", x, 170, {
    color: "#71717a",
    font: "700 24px sans-serif",
    maxWidth: 360,
    lineHeight: 32,
  });

  scene.players.forEach((player, index) => {
    const y = 210 + index * 92;
    context.fillStyle = player.highlighted
      ? "rgba(34, 211, 238, 0.12)"
      : "rgba(24, 24, 27, 0.72)";
    context.strokeStyle = player.highlighted
      ? "rgba(34, 211, 238, 0.55)"
      : "#27272a";
    context.lineWidth = 2;
    context.fillRect(x, y, 400, 68);
    context.strokeRect(x, y, 400, 68);
    drawText(context, `Seat ${player.seatNo}`, x + 20, y + 42, {
      color: "#71717a",
      font: "500 20px sans-serif",
      maxWidth: 92,
      lineHeight: 24,
    });
    drawText(context, player.name, x + 122, y + 43, {
      color: player.status === "dead" ? "#a1a1aa" : "#f4f4f5",
      font: "700 28px sans-serif",
      maxWidth: 150,
      lineHeight: 32,
    });
    drawText(context, player.status, x + 310, y + 42, {
      color: player.status === "dead" ? "#fca5a5" : "#6ee7b7",
      font: "600 20px sans-serif",
      maxWidth: 74,
      lineHeight: 24,
    });
  });
}

function drawSceneDetails(
  context: CanvasRenderingContext2D,
  scene: PlaybackItem,
): void {
  context.strokeStyle = "#18181b";
  context.lineWidth = 2;
  context.beginPath();
  context.moveTo(0, 932);
  context.lineTo(CANVAS_WIDTH, 932);
  context.stroke();
  drawText(context, "Details", 96, 984, {
    color: "#52525b",
    font: "700 22px sans-serif",
    maxWidth: 160,
    lineHeight: 28,
  });
  const detailText = scene.details.length > 0 ? scene.details.join("    ") : "";
  if (detailText) {
    drawText(context, detailText, 230, 984, {
      color: "#a1a1aa",
      font: "400 24px sans-serif",
      maxWidth: 1500,
      lineHeight: 34,
    });
  }
}

function drawText(
  context: CanvasRenderingContext2D,
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
  context.fillStyle = options.color;
  context.font = options.font;
  context.textBaseline = "alphabetic";

  const words = Array.from(text);
  let line = "";
  let currentY = y;
  for (const word of words) {
    const nextLine = `${line}${word}`;
    if (line && context.measureText(nextLine).width > options.maxWidth) {
      context.fillText(line, x, currentY);
      line = word;
      currentY += options.lineHeight;
    } else {
      line = nextLine;
    }
  }

  if (line) {
    context.fillText(line, x, currentY);
  }
}

function preferredMimeType(): string | undefined {
  for (const mimeType of [
    "video/webm;codecs=vp9",
    "video/webm;codecs=vp8",
    "video/webm",
  ]) {
    if (MediaRecorder.isTypeSupported(mimeType)) {
      return mimeType;
    }
  }

  return undefined;
}

function formatTime(milliseconds: number): string {
  const totalSeconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
