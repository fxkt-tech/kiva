"use client";

import { useEffect, useRef, useState } from "react";
import {
  playbackIndexAtMs,
  playbackTotalDurationMs,
  type PlaybackItem,
} from "@/core/playback";
import {
  DEFAULT_SHOW_THEME_ID,
  getShowTheme,
  renderThemeFrame,
} from "./show-theme";
import { createSixPlayerStageLayout } from "./stage-layout";

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

  if (!scene) {
    context.fillStyle = "#050506";
    context.fillRect(0, 0, context.canvas.width, context.canvas.height);
    context.fillStyle = "#d4d4d8";
    context.font = "600 64px sans-serif";
    context.fillText("No playable scenes", 120, 520);
    return;
  }

  renderThemeFrame(context, {
    theme: getShowTheme(DEFAULT_SHOW_THEME_ID),
    scene,
    items,
    layout: createSixPlayerStageLayout(scene.players),
    timeMs,
  });
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
