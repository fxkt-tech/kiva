"use client";

import type { ReactNode } from "react";
import { useFormStatus } from "react-dom";

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
    <button
      type="submit"
      disabled={pending}
      className={className}
      aria-label={ariaLabel}
      title={title}
    >
      {pending ? pendingLabel : label}
    </button>
  );
}
