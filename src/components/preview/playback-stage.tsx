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
      <section className="aspect-video w-full max-w-6xl overflow-hidden bg-[#070708] shadow-2xl shadow-black">
        <div className="grid h-full grid-rows-[1fr_auto]">
          {current ? (
            <div className="grid min-h-0 grid-cols-[minmax(0,1fr)_260px] gap-6 px-[5%] pt-[4%]">
              <SceneBody scene={current} />
              <PlayerRail scene={current} />
            </div>
          ) : (
            <div className="flex h-full flex-col items-center justify-center text-center">
              <div className="text-sm uppercase tracking-[0.2em] text-zinc-600">
                Playback
              </div>
              <p className="mt-4 text-3xl font-semibold text-zinc-300">
                Waiting for public event
              </p>
            </div>
          )}
          {current ? (
            <div className="border-t border-zinc-900 px-[5%] py-4">
              <div className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                  <div className="mb-2 text-xs uppercase tracking-[0.18em] text-zinc-600">
                    Details
                  </div>
                  {current.details.length > 0 ? (
                    <div className="flex flex-wrap gap-2">
                      {current.details.map((detail) => (
                        <span
                          className="max-w-full break-words border border-zinc-800 bg-zinc-950 px-2.5 py-1 text-xs text-zinc-400"
                          key={detail}
                        >
                          {detail}
                        </span>
                      ))}
                    </div>
                  ) : (
                    <p className="text-xs text-zinc-700">No extra details</p>
                  )}
                </div>
                <div className="shrink-0 text-right font-mono text-xs text-zinc-600">
                  <div>{formatTime(current.startsAtMs)}</div>
                  <div>{formatTime(current.startsAtMs + current.durationMs)}</div>
                </div>
              </div>
            </div>
          ) : null}
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

function SceneBody({ scene }: { readonly scene: PlaybackItem }) {
  const highlightedPlayers = scene.players.filter((player) => player.highlighted);
  const primaryPlayer = highlightedPlayers[0];

  if (scene.kind === "speech") {
    return (
      <article className="flex min-w-0 flex-col justify-center">
        <SceneMeta scene={scene} />
        <div className="mb-4 text-xs uppercase tracking-[0.18em] text-cyan-300">
          Speaker
        </div>
        {primaryPlayer ? (
          <div className="mb-6 flex items-baseline gap-3">
            <span className="text-lg text-zinc-500">
              Seat {primaryPlayer.seatNo}
            </span>
            <span className="text-4xl font-semibold text-zinc-50">
              {primaryPlayer.name}
            </span>
          </div>
        ) : null}
        <blockquote className="max-w-4xl border-l-2 border-cyan-400 pl-6 text-2xl leading-10 text-zinc-200">
          {scene.text}
        </blockquote>
      </article>
    );
  }

  if (scene.kind === "vote") {
    return (
      <article className="flex min-w-0 flex-col justify-center">
        <SceneMeta scene={scene} />
        <div className="mb-4 text-xs uppercase tracking-[0.18em] text-amber-300">
          Vote card
        </div>
        <h1 className="break-words text-5xl font-semibold leading-tight text-zinc-50">
          {scene.title}
        </h1>
        <div className="mt-6 max-w-3xl border border-amber-400/30 bg-amber-400/10 px-5 py-4 text-2xl leading-9 text-amber-50">
          {scene.text}
        </div>
      </article>
    );
  }

  if (scene.kind === "resolution") {
    return (
      <article className="flex min-w-0 flex-col justify-center">
        <SceneMeta scene={scene} />
        <div className="mb-4 text-xs uppercase tracking-[0.18em] text-red-300">
          Resolution
        </div>
        <h1 className="break-words text-5xl font-semibold leading-tight text-zinc-50">
          {scene.title}
        </h1>
        <p className="mt-6 max-w-4xl whitespace-pre-wrap break-words text-2xl leading-9 text-zinc-300">
          {scene.text}
        </p>
      </article>
    );
  }

  if (scene.kind === "phase") {
    return (
      <article className="flex min-w-0 flex-col justify-center">
        <SceneMeta scene={scene} />
        <div className="text-xs uppercase tracking-[0.22em] text-zinc-600">
          Phase
        </div>
        <h1 className="mt-4 break-words text-6xl font-semibold leading-none text-zinc-50">
          {scene.title}
        </h1>
        <p className="mt-6 whitespace-pre-wrap break-words text-2xl leading-9 text-zinc-400">
          {scene.text}
        </p>
      </article>
    );
  }

  return (
    <article className="flex min-w-0 flex-col justify-center">
      <SceneMeta scene={scene} />
      <div className="mb-4 text-xs uppercase tracking-[0.18em] text-zinc-500">
        Announcement
      </div>
      <h1 className="break-words text-5xl font-semibold leading-tight text-zinc-50">
        {scene.title}
      </h1>
      {scene.text ? (
        <p className="mt-6 whitespace-pre-wrap break-words text-2xl leading-9 text-zinc-300">
          {scene.text}
        </p>
      ) : null}
    </article>
  );
}

function SceneMeta({ scene }: { readonly scene: PlaybackItem }) {
  return (
    <div className="mb-5 flex flex-wrap items-center gap-3 text-sm uppercase tracking-[0.18em] text-zinc-500">
      <span>#{scene.index}</span>
      <span>{scene.phase}</span>
      <span>{scene.kind}</span>
    </div>
  );
}

function PlayerRail({ scene }: { readonly scene: PlaybackItem }) {
  return (
    <aside className="min-h-0 py-2">
      <div className="mb-3 text-xs uppercase tracking-[0.18em] text-zinc-600">
        Players
      </div>
      <div className="grid gap-2">
        {scene.players.map((player) => (
          <div
            className={[
              "grid grid-cols-[42px_minmax(0,1fr)_auto] items-center gap-2 border px-3 py-2 text-sm",
              player.highlighted
                ? "border-cyan-500/60 bg-cyan-500/10 text-zinc-100"
                : "border-zinc-800 bg-zinc-950/70 text-zinc-400",
              player.status === "dead" ? "opacity-65" : "",
            ].join(" ")}
            key={player.playerId}
          >
            <span className="text-xs text-zinc-600">Seat {player.seatNo}</span>
            <span className="truncate font-medium">{player.name}</span>
            <span
              className={
                player.status === "dead"
                  ? "text-xs text-red-300"
                  : "text-xs text-emerald-300"
              }
            >
              {player.status}
            </span>
          </div>
        ))}
      </div>
    </aside>
  );
}

function formatTime(milliseconds: number): string {
  const totalSeconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
