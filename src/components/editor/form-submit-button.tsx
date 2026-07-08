"use client";

import type { ReactNode } from "react";
import { useFormStatus } from "react-dom";
import { Button } from "@/components/ui/button";

type FormSubmitButtonProps = {
  readonly label: ReactNode;
  readonly pendingLabel: ReactNode;
  readonly className: string;
  readonly ariaLabel?: string;
  readonly title?: string;
};

export function FormSubmitButton({
  label,
  pendingLabel,
  className,
  ariaLabel,
  title,
}: FormSubmitButtonProps) {
  const { pending } = useFormStatus();

  return (
    <Button
      type="submit"
      disabled={pending}
      className={className}
      aria-label={ariaLabel}
      title={title}
      unstyled
    >
      {pending ? pendingLabel : label}
    </Button>
  );
}
