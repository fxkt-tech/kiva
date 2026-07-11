"use client";

import type { ReactNode } from "react";
import { useFormStatus } from "react-dom";
import { Button } from "@/components/ui/button";
import {
  markLlmGenerationPending,
  requestLlmNotificationPermission,
} from "./llm-generation-notification";

type FormSubmitButtonProps = {
  readonly label: ReactNode;
  readonly pendingLabel: ReactNode;
  readonly className: string;
  readonly ariaLabel?: string;
  readonly title?: string;
  readonly forcePending?: boolean;
  readonly llmGenerationNotification?: {
    readonly gameId: string;
    readonly draftId: string;
    readonly previousGenerationId: string | null;
  };
};

export function FormSubmitButton({
  label,
  pendingLabel,
  className,
  ariaLabel,
  title,
  forcePending = false,
  llmGenerationNotification,
}: FormSubmitButtonProps) {
  const { pending: formPending } = useFormStatus();
  const pending = forcePending || formPending;

  return (
    <Button
      type="submit"
      disabled={pending}
      className={className}
      aria-label={ariaLabel}
      title={title}
      onClick={() => {
        if (!llmGenerationNotification) return;
        markLlmGenerationPending(
          llmGenerationNotification.gameId,
          llmGenerationNotification.draftId,
          llmGenerationNotification.previousGenerationId,
        );
        requestLlmNotificationPermission();
      }}
      unstyled
    >
      {pending ? pendingLabel : label}
    </Button>
  );
}
