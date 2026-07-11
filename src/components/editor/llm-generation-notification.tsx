"use client";

import { Bell, BellOff, BellRing, CheckCircle2, Circle } from "lucide-react";
import { useEffect, useState, useTransition } from "react";
import { confirmDraftAction } from "@/app/actions";
import { Button } from "@/components/ui/button";
import { iconButtonClassName } from "@/components/ui/button-styles";
import type { DraftId, GameId } from "@/core/types";

const pendingRequestPrefix = "kiva:llm-generation-pending:";
const autoConfirmDraftPreferenceKey = "kiva:auto-confirm-draft";
export const autoConfirmDraftDelayMs = 1_000;

type PendingGenerationRequest = {
  readonly draftId: string;
  readonly previousGenerationId: string | null;
};

export function markLlmGenerationPending(
  gameId: string,
  draftId: string,
  previousGenerationId: string | null,
): void {
  if (typeof window === "undefined") return;

  window.sessionStorage.setItem(
    pendingRequestKey(gameId),
    JSON.stringify({ draftId, previousGenerationId }),
  );
}

export function requestLlmNotificationPermission(): void {
  if (
    typeof Notification === "undefined" ||
    Notification.permission !== "default"
  ) {
    return;
  }

  void Notification.requestPermission();
}

type NotificationPermissionState = NotificationPermission | "unsupported";

export function LlmNotificationPermissionButton() {
  const [permission, setPermission] =
    useState<NotificationPermissionState>("unsupported");

  useEffect(() => {
    setPermission(
      typeof Notification === "undefined"
        ? "unsupported"
        : Notification.permission,
    );
  }, []);

  if (permission === "unsupported") return null;

  const enabled = permission === "granted";
  const denied = permission === "denied";
  const label = enabled
    ? "Browser notifications enabled"
    : denied
      ? "Browser notifications blocked"
      : "Enable browser notifications";

  return (
    <Button
      type="button"
      aria-label={label}
      title={label}
      className={iconButtonClassName()}
      disabled={enabled || denied}
      onClick={() => {
        void Notification.requestPermission().then(setPermission);
      }}
      unstyled
    >
      {enabled ? (
        <BellRing aria-hidden="true" className="h-4 w-4" />
      ) : denied ? (
        <BellOff aria-hidden="true" className="h-4 w-4" />
      ) : (
        <Bell aria-hidden="true" className="h-4 w-4" />
      )}
    </Button>
  );
}

export function AutoConfirmDraftButton({
  gameId,
  draftId,
  ready,
}: {
  readonly gameId: GameId;
  readonly draftId: DraftId;
  readonly ready: boolean;
}) {
  const [enabled, setEnabled] = useState(false);
  const [, startTransition] = useTransition();

  useEffect(() => {
    setEnabled(
      isAutoConfirmDraftEnabled(
        window.localStorage.getItem(autoConfirmDraftPreferenceKey),
      ),
    );
  }, []);

  useEffect(() => {
    if (!shouldScheduleAutoConfirmDraft(enabled, ready)) return;

    const timeoutId = window.setTimeout(() => {
      startTransition(async () => {
        await confirmDraftAction(gameId, draftId);
      });
    }, autoConfirmDraftDelayMs);

    return () => window.clearTimeout(timeoutId);
  }, [draftId, enabled, gameId, ready, startTransition]);

  const label = enabled
    ? "Disable auto-confirm draft"
    : "Enable auto-confirm draft";

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
          autoConfirmDraftPreferenceKey,
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

type LlmGenerationNotificationProps = {
  readonly gameId: GameId;
  readonly draftId: DraftId;
  readonly generationId: string | null;
  readonly generationStatus: "success" | "failed" | null;
};

export function LlmGenerationNotification({
  gameId,
  draftId,
  generationId,
  generationStatus,
}: LlmGenerationNotificationProps) {
  useEffect(() => {
    if (!generationId || !generationStatus) return;

    const key = pendingRequestKey(gameId);
    const pendingValue = window.sessionStorage.getItem(key);
    if (!isCompletedLlmGeneration(pendingValue, draftId, generationId)) {
      return;
    }

    window.sessionStorage.removeItem(key);

    if (
      typeof Notification !== "undefined" &&
      Notification.permission === "granted"
    ) {
      const succeeded = generationStatus === "success";
      new Notification(succeeded ? "LLM 请求已完成" : "LLM 请求失败", {
        body: succeeded
          ? "Editor 中的草稿已经生成完成。"
          : "Editor 中的草稿生成失败，请返回页面查看详情。",
        tag: `kiva-llm-${gameId}`,
      });
    }

  }, [draftId, gameId, generationId, generationStatus]);

  return null;
}

export function isAutoConfirmDraftEnabled(value: string | null): boolean {
  return value === "true";
}

export function shouldScheduleAutoConfirmDraft(
  enabled: boolean,
  ready: boolean,
): boolean {
  return enabled && ready;
}

function pendingRequestKey(gameId: string): string {
  return `${pendingRequestPrefix}${gameId}`;
}

function readPendingRequest(value: string | null): PendingGenerationRequest | null {
  if (!value) return null;

  try {
    const parsed = JSON.parse(value) as Partial<PendingGenerationRequest>;
    const previousGenerationId = parsed.previousGenerationId;
    return typeof parsed.draftId === "string" &&
      (typeof previousGenerationId === "string" || previousGenerationId === null)
      ? { draftId: parsed.draftId, previousGenerationId }
      : null;
  } catch {
    return null;
  }
}

export function isCompletedLlmGeneration(
  pendingValue: string | null,
  draftId: string,
  generationId: string,
): boolean {
  const pending = readPendingRequest(pendingValue);
  return (
    pending !== null &&
    pending.draftId === draftId &&
    pending.previousGenerationId !== generationId
  );
}
