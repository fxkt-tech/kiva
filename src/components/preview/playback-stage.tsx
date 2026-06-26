"use client";

import { useEffect, useState } from "react";
import type { PlaybackItem } from "@/core/playback";

const SPEED_OPTIONS = [0.5, 1, 1.5, 2] as const;
type PlaybackSpeed = (typeof SPEED_OPTIONS)[number];

type PlaybackStageProps = {
  readonly items: readonly PlaybackItem[];
};

export function PlaybackStage({ items }: PlaybackStageProps) {
  const hasItems = items.length > 0;
  const [index, setIndex] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState<PlaybackSpeed>(1);
  const maxIndex = Math.max(0, items.length - 1);
  const safeIndex = hasItems ? Math.min(index, maxIndex) : 0;
  const current = hasItems ? items[safeIndex] : undefined;
  const progress = hasItems ? `${safeIndex + 1} / ${items.length}` : "0 / 0";

  useEffect(() => {
    if (!hasItems) {
      setIndex(0);
      setPlaying(false);
      return;
    }

    setIndex((currentIndex) => {
      const nextIndex = Math.min(currentIndex, maxIndex);
      if (nextIndex >= maxIndex) {
        setPlaying(false);
      }

      return nextIndex;
    });
  }, [hasItems, maxIndex]);

  useEffect(() => {
    if (!playing || !hasItems) {
      return;
    }

    const timeoutId = window.setTimeout(() => {
      setIndex((currentIndex) => {
        if (currentIndex >= maxIndex) {
          setPlaying(false);
          return currentIndex;
        }

        const nextIndex = currentIndex + 1;
        if (nextIndex >= maxIndex) {
          setPlaying(false);
        }

        return nextIndex;
      });
    }, Math.max(250, (current?.durationMs ?? 2200) / speed));

    return () => window.clearTimeout(timeoutId);
  }, [current?.durationMs, hasItems, maxIndex, playing, speed]);

  function goToPrevious() {
    setIndex((currentIndex) => Math.max(0, currentIndex - 1));
  }

  function goToNext() {
    setIndex((currentIndex) => {
      const nextIndex = Math.min(maxIndex, currentIndex + 1);

      if (nextIndex >= maxIndex) {
        setPlaying(false);
      }

      return nextIndex;
    });
  }

  function reset() {
    setIndex(0);
    setPlaying(false);
  }

  const canAdvance = hasItems && safeIndex < maxIndex;
  const activelyPlaying = playing && canAdvance;

  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-4 bg-black p-4 text-white">
      <section className="aspect-video w-full max-w-6xl overflow-hidden bg-zinc-950 shadow-2xl shadow-black">
        <div className="flex h-full flex-col justify-center px-[7%] py-[6%]">
          {current ? (
            <article className="max-w-4xl">
              <div className="mb-5 flex flex-wrap items-center gap-3 text-sm uppercase tracking-[0.2em] text-zinc-500">
                <span>#{current.index}</span>
                <span>{current.phase}</span>
              </div>
              <h1 className="break-words text-4xl font-semibold leading-tight text-zinc-50 sm:text-5xl lg:text-6xl">
                {current.title}
              </h1>
              {current.text ? (
                <p className="mt-6 whitespace-pre-wrap break-words text-xl leading-8 text-zinc-300 sm:text-2xl sm:leading-10">
                  {current.text}
                </p>
              ) : null}
            </article>
          ) : (
            <div className="text-center">
              <div className="text-sm uppercase tracking-[0.2em] text-zinc-600">
                Playback
              </div>
              <p className="mt-4 text-3xl font-semibold text-zinc-300">
                Waiting for public event
              </p>
            </div>
          )}
        </div>
      </section>
      <section
        aria-label="Playback controls"
        className="flex w-full max-w-6xl flex-wrap items-center justify-between gap-3 border border-zinc-800 bg-zinc-950 px-4 py-3 text-sm text-zinc-200 shadow-lg shadow-black/40"
      >
        <div className="font-mono text-zinc-400">{progress}</div>
        <div className="flex flex-wrap items-center gap-2">
          <label className="flex items-center gap-2 text-xs text-zinc-400">
            <span>Speed</span>
            <select
              className="border border-zinc-700 bg-zinc-950 px-2 py-2 text-zinc-200 outline-none transition focus:border-zinc-500"
              onChange={(event) =>
                setSpeed(Number(event.currentTarget.value) as PlaybackSpeed)
              }
              value={speed}
            >
              {SPEED_OPTIONS.map((option) => (
                <option key={option} value={option}>
                  {option}x
                </option>
              ))}
            </select>
          </label>
          <button
            className="border border-zinc-700 px-3 py-2 text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
            disabled={!hasItems || index === 0}
            onClick={goToPrevious}
            type="button"
          >
            Prev
          </button>
          <button
            className="border border-zinc-700 px-3 py-2 text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
            disabled={!canAdvance}
            onClick={() => setPlaying((currentPlaying) => !currentPlaying)}
            type="button"
          >
            {activelyPlaying ? "Pause" : "Play"}
          </button>
          <button
            className="border border-zinc-700 px-3 py-2 text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
            disabled={!canAdvance}
            onClick={goToNext}
            type="button"
          >
            Next
          </button>
          <button
            className="border border-zinc-700 px-3 py-2 text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-900 disabled:cursor-not-allowed disabled:border-zinc-900 disabled:text-zinc-700"
            disabled={!hasItems || (safeIndex === 0 && !playing)}
            onClick={reset}
            type="button"
          >
            Reset
          </button>
        </div>
      </section>
    </main>
  );
}
