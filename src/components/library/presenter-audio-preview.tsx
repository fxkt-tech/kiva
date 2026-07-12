"use client";

import { Play, Square } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";

export type PresenterAudioPreviewOption = {
  readonly label: string;
  readonly sources: readonly string[];
};

let stopActivePreview: (() => void) | null = null;

export function PresenterAudioPreview({
  options,
}: {
  readonly options: readonly PresenterAudioPreviewOption[];
}) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [error, setError] = useState(false);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const canceledRef = useRef(false);

  const stop = useCallback(() => {
    canceledRef.current = true;
    audioRef.current?.pause();
    audioRef.current = null;
    setPlaying(false);
    if (stopActivePreview === stop) stopActivePreview = null;
  }, []);

  useEffect(() => stop, [stop]);

  async function play() {
    stopActivePreview?.();
    canceledRef.current = false;
    stopActivePreview = stop;
    setError(false);
    setPlaying(true);
    try {
      const sources = options[selectedIndex]?.sources ?? [];
      for (let index = 0; index < sources.length; index += 1) {
        if (canceledRef.current) return;
        await playSource(sources[index]!);
        if (index < sources.length - 1) await delay(80);
      }
    } catch {
      setError(true);
    } finally {
      if (!canceledRef.current) stop();
    }
  }

  function playSource(source: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const audio = new Audio(source);
      audioRef.current = audio;
      audio.addEventListener("ended", () => resolve(), { once: true });
      audio.addEventListener("error", () => reject(new Error("audio failed")), {
        once: true,
      });
      void audio.play().catch(reject);
    });
  }

  const unavailable = options.length === 0;

  return (
    <div className="flex items-center gap-1.5" data-ignore-dirty>
      {options.length > 1 ? (
        <select
          aria-label="试听座位"
          className="h-7 rounded border border-interactive-border bg-background px-1.5 font-mono text-[11px] text-muted outline-none"
          disabled={playing}
          onChange={(event) => setSelectedIndex(Number(event.currentTarget.value))}
          value={selectedIndex}
        >
          {options.map((option, index) => (
            <option key={option.label} value={index}>
              {option.label}
            </option>
          ))}
        </select>
      ) : null}
      <Button
        aria-label={playing ? "停止试听" : "播放主持音频"}
        buttonStyle="icon"
        disabled={unavailable}
        iconSize="sm"
        onClick={playing ? stop : play}
        title={unavailable ? "尚未构建该文案的配音" : error ? "播放失败，点击重试" : playing ? "停止" : "试听配音"}
        type="button"
      >
        {playing ? <Square size={12} /> : <Play size={13} />}
      </Button>
      {error ? <span className="text-[10px] text-danger-badge-foreground">失败</span> : null}
    </div>
  );
}

function delay(milliseconds: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, milliseconds));
}
