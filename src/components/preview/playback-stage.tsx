"use client";

import {
  CircleDot,
  Download,
  Pause,
  Play,
  RotateCcw,
  SkipBack,
  SkipForward,
  Square,
} from "lucide-react";
import type { ReactNode } from "react";
import { useEffect, useRef, useState } from "react";
import {
  playbackIndexAtMs,
  playbackTotalDurationMs,
  type PlaybackItem,
} from "@/core/playback";
import {
  type PreviewAvatarImages,
  type PreviewBackgroundImages,
  type PreviewRenderFrameInput,
  type PreviewRendererHandle,
} from "./preview-renderer";
import { systemVoiceSourceForScene } from "./preview-audio";

const FRAME_RATE = 30;
const TICK_MS = 1000 / FRAME_RATE;
const backgroundImageSources = {
  day: "/kivdb-assets/preview/day-background.png",
  night: "/kivdb-assets/preview/night-background.png",
} as const;

type PlaybackStageProps = {
  readonly items: readonly PlaybackItem[];
  readonly initialPosition?: "start" | "end";
};

type RecordingStatus = "idle" | "recording" | "ready" | "failed";

export function PlaybackStage({
  items,
  initialPosition = "start",
}: PlaybackStageProps) {
  const pixiHostRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<PreviewRendererHandle | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const audioSceneKeyRef = useRef<string | null>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const downloadUrlRef = useRef<string | null>(null);
  const latestRenderInputRef = useRef<PreviewRenderFrameInput | null>(null);
  const [currentTimeMs, setCurrentTimeMs] = useState(() =>
    initialTimeForItems(items, initialPosition),
  );
  const [playing, setPlaying] = useState(false);
  const [recordingStatus, setRecordingStatus] =
    useState<RecordingStatus>("idle");
  const [downloadUrl, setDownloadUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [rendererHandle, setRendererHandle] =
    useState<PreviewRendererHandle | null>(null);
  const [backgroundImages, setBackgroundImages] =
    useState<PreviewBackgroundImages>({ day: null, night: null });
  const [avatarImages, setAvatarImages] = useState<PreviewAvatarImages>({});
  const hasItems = items.length > 0;
  const totalDurationMs = playbackTotalDurationMs(items);
  const safeTimeMs = clamp(currentTimeMs, 0, Math.max(0, totalDurationMs));
  const safeIndex = hasItems ? playbackIndexAtMs(items, safeTimeMs) : 0;
  const progress = hasItems ? `${safeIndex + 1} / ${items.length}` : "0 / 0";
  const canPlay = hasItems && safeTimeMs < totalDurationMs;
  const canNavigate =
    hasItems && recordingStatus !== "recording" && items.length > 1;
  const canGoPrevious = canNavigate && safeIndex > 0;
  const canGoNext = canNavigate && safeIndex < items.length - 1;
  const canRecord = hasItems && recordingStatus !== "recording";
  const isRecording = recordingStatus === "recording";
  const recordControlLabel = isRecording ? "Stop recording" : "Start recording";
  latestRenderInputRef.current = {
    items,
    timeMs: safeTimeMs,
    backgroundImages,
    avatarImages,
  };

  useEffect(() => {
    setCurrentTimeMs(initialTimeForItems(items, initialPosition));
  }, [initialPosition, items]);

  useEffect(() => {
    let cancelled = false;
    let currentRenderer: PreviewRendererHandle | null = null;

    async function createRenderer() {
      rendererRef.current?.destroy();
      rendererRef.current = null;
      setRendererHandle(null);

      if (!pixiHostRef.current) {
        return;
      }

      const { createPixiPreviewRenderer } = await import(
        "./pixi/pixi-preview-renderer"
      );
      if (cancelled || !pixiHostRef.current) {
        return;
      }

      currentRenderer = await createPixiPreviewRenderer(pixiHostRef.current);
      if (cancelled) {
        currentRenderer.destroy();
        return;
      }

      rendererRef.current = currentRenderer;
      renderLatestFrame(currentRenderer, latestRenderInputRef.current);
      setRendererHandle(currentRenderer);
    }

    createRenderer().catch((caught) => {
      if (!cancelled) {
        setError(caught instanceof Error ? caught.message : String(caught));
      }
    });

    return () => {
      cancelled = true;
      currentRenderer?.destroy();
      if (rendererRef.current === currentRenderer) {
        rendererRef.current = null;
        setRendererHandle(null);
      }
    };
  }, []);

  useEffect(() => {
    renderLatestFrame(rendererHandle, latestRenderInputRef.current);
  }, [
    avatarImages,
    backgroundImages,
    items,
    rendererHandle,
    safeTimeMs,
  ]);

  useEffect(() => {
    let cancelled = false;

    loadBackgroundImages().then((images) => {
      if (!cancelled) {
        setBackgroundImages(images);
      }
    });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    loadAvatarImages(items).then((images) => {
      if (!cancelled) {
        setAvatarImages(images);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [items]);

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
    if (!playing || !hasItems) {
      return;
    }

    playSystemVoiceForScene(items[safeIndex]);
  }, [hasItems, items, playing, safeIndex]);

  useEffect(() => {
    return () => {
      stopSystemVoice();
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
    stopSystemVoice();
  }

  function reset() {
    setCurrentTimeMs(0);
    setPlaying(false);
    stopSystemVoice();
  }

  function seekTo(timeMs: number) {
    setCurrentTimeMs(clamp(timeMs, 0, Math.max(0, totalDurationMs)));
    setPlaying(false);
    stopSystemVoice();
  }

  function seekToItem(index: number) {
    const item = items[index];
    if (!item) {
      return;
    }

    seekTo(item.startsAtMs);
  }

  function startRecording() {
    const canvas = rendererRef.current?.canvas ?? null;
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

      rendererRef.current?.renderFrame({
        items,
        timeMs: 0,
        backgroundImages,
        avatarImages,
      });
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
    setPlaying(false);
    stopSystemVoice();
  }

  function playSystemVoiceForScene(scene: PlaybackItem | undefined) {
    if (!scene) {
      return;
    }

    const source = systemVoiceSourceForScene(scene);
    const sceneKey = `${scene.index}:${source ?? ""}`;
    if (audioSceneKeyRef.current === sceneKey) {
      return;
    }

    stopSystemVoice();
    audioSceneKeyRef.current = sceneKey;

    if (!source) {
      return;
    }

    const audio = new Audio(source);
    audioRef.current = audio;
    audio.play().catch(() => {
      if (audioRef.current === audio) {
        audioRef.current = null;
      }
    });
  }

  function stopSystemVoice() {
    const audio = audioRef.current;
    audioSceneKeyRef.current = null;
    audioRef.current = null;
    if (!audio) {
      return;
    }

    audio.pause();
    audio.currentTime = 0;
  }

  function renderLatestFrame(
    handle: PreviewRendererHandle | null,
    input: PreviewRenderFrameInput | null,
  ) {
    if (!handle || !input) {
      return;
    }

    handle.renderFrame(input);
  }

  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4 bg-black p-4 text-white">
      <section className="aspect-video w-full max-w-6xl overflow-hidden bg-black shadow-2xl shadow-black">
        <div
          aria-label="Playback canvas"
          className="h-full w-full bg-black [&_canvas]:h-full [&_canvas]:w-full"
          ref={pixiHostRef}
        />
        {!hasItems ? <span className="sr-only">No playable scenes</span> : null}
      </section>
      <section
        aria-label="Playback controls"
        className="grid w-full max-w-6xl gap-3 border border-zinc-800 bg-zinc-950/95 px-4 py-3 text-sm text-zinc-200 shadow-lg shadow-black/40"
      >
        <div className="grid gap-2">
          <div className="flex items-center justify-between gap-4 text-xs text-zinc-500">
            <span className="font-mono">{progress}</span>
            <span className="font-mono">
              {formatTime(safeTimeMs)} / {formatTime(totalDurationMs)}
            </span>
          </div>
          <input
            aria-label="Timeline"
            className="h-2 w-full accent-cyan-400"
            disabled={!hasItems || isRecording}
            max={Math.max(0, totalDurationMs)}
            min={0}
            onChange={(event) => seekTo(Number(event.currentTarget.value))}
            step={100}
            type="range"
            value={safeTimeMs}
          />
        </div>
        <div className="grid grid-cols-[minmax(8rem,1fr)_auto_minmax(8rem,1fr)] items-center gap-3">
          <div className="min-w-0 font-mono text-xs text-zinc-500">
            {error ? <span className="text-red-300">{error}</span> : null}
          </div>
          <div className="flex items-center justify-center gap-2">
            <IconButton
              disabled={!canGoPrevious}
              label="Previous"
              onClick={() => seekToItem(safeIndex - 1)}
            >
              <SkipBack aria-hidden="true" size={20} strokeWidth={2.4} />
            </IconButton>
            <IconButton
              disabled={!canPlay}
              label={playing ? "Pause" : "Play"}
              onClick={playing ? pause : play}
              variant="primary"
            >
              {playing ? (
                <Pause aria-hidden="true" size={22} strokeWidth={2.5} />
              ) : (
                <Play aria-hidden="true" size={22} strokeWidth={2.5} />
              )}
            </IconButton>
            <IconButton
              disabled={!canGoNext}
              label="Next"
              onClick={() => seekToItem(safeIndex + 1)}
            >
              <SkipForward aria-hidden="true" size={20} strokeWidth={2.4} />
            </IconButton>
            <IconButton
              disabled={!hasItems || (safeTimeMs === 0 && !playing)}
              label="Reset"
              onClick={reset}
            >
              <RotateCcw aria-hidden="true" size={20} strokeWidth={2.4} />
            </IconButton>
          </div>
          <div className="flex items-center justify-end gap-2">
            {isRecording ? (
              <span
                aria-label="Recording"
                className="h-2 w-2 rounded-full bg-red-400 shadow-[0_0_12px_rgba(248,113,113,0.9)]"
              />
            ) : null}
            <IconButton
              disabled={!isRecording && !canRecord}
              label={recordControlLabel}
              onClick={isRecording ? stopRecording : startRecording}
              variant="record"
            >
              {isRecording ? (
                <Square aria-hidden="true" size={19} strokeWidth={2.7} />
              ) : (
                <CircleDot aria-hidden="true" size={21} strokeWidth={2.5} />
              )}
            </IconButton>
            <a
              aria-disabled={!downloadUrl}
              aria-label="Download WebM"
              className="inline-flex h-10 w-10 items-center justify-center border border-emerald-900/70 text-emerald-200 transition hover:border-emerald-500 hover:bg-emerald-950/40 aria-disabled:pointer-events-none aria-disabled:cursor-not-allowed aria-disabled:border-zinc-900 aria-disabled:text-zinc-700"
              download={downloadUrl ? "kiva-playback.webm" : undefined}
              href={downloadUrl ?? undefined}
              title="Download WebM"
            >
              <Download aria-hidden="true" size={20} strokeWidth={2.4} />
            </a>
          </div>
        </div>
      </section>
    </main>
  );
}

function IconButton({
  children,
  disabled,
  label,
  onClick,
  variant = "default",
}: {
  readonly children: ReactNode;
  readonly disabled: boolean;
  readonly label: string;
  readonly onClick: () => void;
  readonly variant?: "default" | "primary" | "record";
}) {
  const className = {
    default:
      "border-zinc-700 text-zinc-200 hover:border-zinc-500 hover:bg-zinc-900 disabled:border-zinc-900 disabled:text-zinc-700",
    primary:
      "border-cyan-700 text-cyan-100 hover:border-cyan-400 hover:bg-cyan-950/40 disabled:border-zinc-900 disabled:text-zinc-700",
    record:
      "border-red-900/70 text-red-200 hover:border-red-500 hover:bg-red-950/40 disabled:border-zinc-900 disabled:text-zinc-700",
  }[variant];

  return (
    <button
      aria-label={label}
      className={`inline-flex h-10 w-10 items-center justify-center border transition disabled:cursor-not-allowed ${className}`}
      disabled={disabled}
      onClick={onClick}
      title={label}
      type="button"
    >
      {children}
    </button>
  );
}

function initialTimeForItems(
  items: readonly PlaybackItem[],
  initialPosition: NonNullable<PlaybackStageProps["initialPosition"]>,
): number {
  if (initialPosition === "start") {
    return 0;
  }

  return playbackTotalDurationMs(items);
}

async function loadBackgroundImages(): Promise<PreviewBackgroundImages> {
  const [day, night] = await Promise.all([
    loadImage(backgroundImageSources.day),
    loadImage(backgroundImageSources.night),
  ]);

  return { day, night };
}

function loadImage(src: string): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const image = new Image();
    image.crossOrigin = "anonymous";
    image.onload = () => resolve(image);
    image.onerror = () => resolve(null);
    image.src = src;
  });
}

async function loadAvatarImages(
  items: readonly PlaybackItem[],
): Promise<PreviewAvatarImages> {
  const sources = [
    ...new Set(
      items.flatMap((item) =>
        item.players.flatMap((player) => (player.avatar ? [player.avatar] : [])),
      ),
    ),
  ];
  const entries = await Promise.all(
    sources.map(async (source) => [source, await loadImage(source)] as const),
  );

  return Object.fromEntries(entries);
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
