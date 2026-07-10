"use client";

import {
  ArrowLeft,
  ChevronLeft,
  ChevronRight,
  Pause,
  Play,
  RotateCcw,
} from "lucide-react";
import { Player, type PlayerRef } from "@remotion/player";
import Link from "next/link";
import { useEffect, useRef, useState } from "react";
import { playbackIndexAtMs } from "@/core/playback";
import { Button } from "@/components/ui/button";
import { iconButtonClassName } from "@/components/ui/button-styles";
import { KivaVideoComposition } from "./composition/kiva-video-composition";
import {
  compositionDurationInFrames,
  frameToMilliseconds,
  millisecondsToFrame,
} from "./composition/timing";
import type { VideoCompositionInput } from "./composition/types";
import { VIDEO_SPEC } from "./composition/video-spec";
import { ExportPanel } from "./export-panel";

export function PreviewV2Studio({
  composition,
  includesDraft,
  canExport,
}: {
  readonly composition: VideoCompositionInput;
  readonly includesDraft: boolean;
  readonly canExport: boolean;
}) {
  const playerRef = useRef<PlayerRef>(null);
  const durationInFrames = compositionDurationInFrames(
    composition.items,
    VIDEO_SPEC.fps,
  );
  const [frame, setFrame] = useState(0);
  const [playing, setPlaying] = useState(false);
  const timeMs = frameToMilliseconds(frame, VIDEO_SPEC.fps);
  const sceneIndex =
    composition.items.length > 0
      ? playbackIndexAtMs(composition.items, timeMs)
      : 0;

  useEffect(() => {
    const player = playerRef.current;
    if (!player) {
      return;
    }
    const onFrame = (event: { detail: { frame: number } }) =>
      setFrame(event.detail.frame);
    const onPlay = () => setPlaying(true);
    const onPause = () => setPlaying(false);
    player.addEventListener("frameupdate", onFrame);
    player.addEventListener("play", onPlay);
    player.addEventListener("pause", onPause);
    player.addEventListener("ended", onPause);
    return () => {
      player.removeEventListener("frameupdate", onFrame);
      player.removeEventListener("play", onPlay);
      player.removeEventListener("pause", onPause);
      player.removeEventListener("ended", onPause);
    };
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if ((event.target as HTMLElement | null)?.closest("input,button,a")) {
        return;
      }
      if (event.key === " ") {
        event.preventDefault();
        playerRef.current?.toggle();
      } else if (event.key === "ArrowLeft") {
        event.preventDefault();
        seekScene(sceneIndex - 1);
      } else if (event.key === "ArrowRight") {
        event.preventDefault();
        seekScene(sceneIndex + 1);
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });

  function seekScene(index: number) {
    const scene = composition.items[index];
    if (!scene) {
      return;
    }
    playerRef.current?.seekTo(
      millisecondsToFrame(scene.startsAtMs, VIDEO_SPEC.fps),
    );
  }

  return (
    <main className="min-h-dvh overflow-auto bg-background p-3 text-foreground lg:h-dvh lg:overflow-hidden">
      <div className="mx-auto flex min-h-[calc(100dvh-1.5rem)] max-w-[1800px] flex-col gap-3 lg:h-full lg:min-h-0">
        <header className="-mx-3 flex min-h-12 flex-wrap items-center justify-between gap-x-6 gap-y-2 border-b border-border px-3 pb-3">
          <div className="flex min-w-0 items-center gap-3">
            <Link
              aria-label="Back to editor"
              className={iconButtonClassName()}
              href={`/games/${composition.gameId}/editor`}
              title="Back to editor"
            >
              <ArrowLeft aria-hidden="true" className="h-4 w-4" />
            </Link>
            <h1 className="truncate text-base font-semibold text-foreground">
              {composition.gameTitle}
            </h1>
            <span className="hidden shrink-0 rounded-full border border-border bg-badge px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-badge-foreground sm:inline-flex">
              HTML video master
            </span>
          </div>
          <div className="font-mono text-[11px] tracking-[0.1em] text-subtle">
            1920×1080 · 30 FPS · {composition.items.length} records
          </div>
        </header>

        {includesDraft ? (
          <div className="shrink-0 rounded-md border border-warning-badge-foreground/30 bg-warning-badge px-3 py-2 text-xs text-warning-badge-foreground">
            当前预览包含未确认草稿；正式导出只使用已确认事件。
          </div>
        ) : null}

        <div className="grid min-h-0 flex-1 gap-3 lg:grid-cols-[minmax(0,1fr)_360px]">
          <div className="flex min-h-0 flex-col gap-3">
            <div className="grid min-h-[280px] flex-1 place-items-center [container-type:size] lg:min-h-0">
              <section className="aspect-video overflow-hidden rounded-lg border border-border bg-black shadow-xl shadow-black/20 [width:min(100cqw,calc(100cqh*16/9))]">
                <Player
                  acknowledgeRemotionLicense
                  component={KivaVideoComposition}
                  compositionHeight={VIDEO_SPEC.height}
                  compositionWidth={VIDEO_SPEC.width}
                  controls={false}
                  durationInFrames={durationInFrames}
                  fps={VIDEO_SPEC.fps}
                  inputProps={composition}
                  numberOfSharedAudioTags={8}
                  ref={playerRef}
                  spaceKeyToPlayOrPause={false}
                  style={{ width: "100%", height: "100%" }}
                />
              </section>
            </div>

            <section className="grid shrink-0 gap-3 rounded-lg border border-border bg-surface/45 px-4 py-3">
              <div className="grid grid-cols-[auto_1fr_auto] items-center gap-4">
                <span className="font-mono text-xs text-muted">
                  {composition.items.length > 0
                    ? String(sceneIndex + 1) + " / " + composition.items.length
                    : "0 / 0"}
                </span>
                <input
                  aria-label="Timeline"
                  className="accent-accent"
                  max={Math.max(0, durationInFrames - 1)}
                  min={0}
                  onChange={(event) =>
                    playerRef.current?.seekTo(Number(event.currentTarget.value))
                  }
                  type="range"
                  value={frame}
                />
                <span className="font-mono text-xs text-subtle">
                  {formatTime(timeMs)}
                </span>
              </div>
              <div className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-4">
                <div className="truncate text-xs tracking-[0.08em] text-muted">
                  {composition.items[sceneIndex]?.title ?? "No playback records"}
                </div>
                <div className="flex items-center gap-2">
                  <ControlButton
                    disabled={sceneIndex <= 0}
                    label="Previous scene"
                    onClick={() => seekScene(sceneIndex - 1)}
                  >
                    <ChevronLeft size={18} />
                  </ControlButton>
                  <ControlButton
                    disabled={composition.items.length === 0}
                    label={playing ? "Pause" : "Play"}
                    onClick={() => playerRef.current?.toggle()}
                  >
                    {playing ? <Pause size={18} /> : <Play size={18} />}
                  </ControlButton>
                  <ControlButton
                    disabled={sceneIndex >= composition.items.length - 1}
                    label="Next scene"
                    onClick={() => seekScene(sceneIndex + 1)}
                  >
                    <ChevronRight size={18} />
                  </ControlButton>
                  <ControlButton
                    disabled={frame === 0}
                    label="Reset"
                    onClick={() => playerRef.current?.seekTo(0)}
                  >
                    <RotateCcw size={17} />
                  </ControlButton>
                </div>
              </div>
            </section>
          </div>
          <div className="min-h-0">
            <ExportPanel
              canExport={canExport}
              gameId={composition.gameId}
            />
          </div>
        </div>
      </div>
    </main>
  );
}

function ControlButton({
  children,
  disabled,
  label,
  onClick,
}: {
  readonly children: React.ReactNode;
  readonly disabled: boolean;
  readonly label: string;
  readonly onClick: () => void;
}) {
  return (
    <Button
      aria-label={label}
      buttonStyle="icon"
      disabled={disabled}
      iconSize="md"
      onClick={onClick}
      title={label}
    >
      {children}
    </Button>
  );
}

function formatTime(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  return Math.floor(seconds / 60) + ":" + String(seconds % 60).padStart(2, "0");
}
