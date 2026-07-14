"use client";

import { CheckCircle2, Circle } from "lucide-react";
import { useEffect, useState, useTransition } from "react";
import { generateEpisodeScriptAction } from "@/app/actions";
import { Button } from "@/components/ui/button";
import { iconButtonClassName } from "@/components/ui/button-styles";
import type { GameId } from "@/core/types";

export const autoContinueEpisodeAuthorPreferenceKey =
  "kiva:auto-continue-episode-author";
export const autoContinueEpisodeAuthorDelayMs = 1_000;

export function EpisodeAutoContinueButton({
  gameId,
  expectedJobId,
  ready,
}: {
  readonly gameId: GameId;
  readonly expectedJobId: string | null;
  readonly ready: boolean;
}) {
  const [enabled, setEnabled] = useState(false);
  const [, startTransition] = useTransition();

  useEffect(() => {
    setEnabled(
      isAutoContinueEpisodeAuthorEnabled(
        window.localStorage.getItem(autoContinueEpisodeAuthorPreferenceKey),
      ),
    );
  }, []);

  useEffect(() => {
    if (
      !shouldScheduleEpisodeAuthorAdvance(enabled, ready, expectedJobId) ||
      expectedJobId === null
    ) {
      return;
    }

    const timeoutId = window.setTimeout(() => {
      startTransition(async () => {
        await generateEpisodeScriptAction(gameId, expectedJobId);
      });
    }, autoContinueEpisodeAuthorDelayMs);

    return () => window.clearTimeout(timeoutId);
  }, [enabled, expectedJobId, gameId, ready, startTransition]);

  const label = enabled
    ? "Disable auto-continue Script Author"
    : "Enable auto-continue Script Author";

  return (
    <Button
      type="button"
      aria-label={label}
      aria-pressed={enabled}
      title={label}
      className={iconButtonClassName({
        variant: enabled ? "success" : "default",
      })}
      onClick={() => {
        const nextEnabled = !enabled;
        window.localStorage.setItem(
          autoContinueEpisodeAuthorPreferenceKey,
          String(nextEnabled),
        );
        setEnabled(nextEnabled);
      }}
      unstyled
    >
      {enabled ? (
        <CheckCircle2 aria-hidden="true" className="h-4 w-4" />
      ) : (
        <Circle aria-hidden="true" className="h-4 w-4" />
      )}
    </Button>
  );
}

export function isAutoContinueEpisodeAuthorEnabled(
  storedValue: string | null,
): boolean {
  return storedValue === "true";
}

export function shouldScheduleEpisodeAuthorAdvance(
  enabled: boolean,
  ready: boolean,
  expectedJobId: string | null,
): boolean {
  return enabled && ready && expectedJobId !== null;
}
