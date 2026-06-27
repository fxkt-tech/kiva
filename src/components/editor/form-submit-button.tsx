"use client";

import { useFormStatus } from "react-dom";

type FormSubmitButtonProps = {
  readonly label: string;
  readonly pendingLabel: string;
  readonly className: string;
};

export function FormSubmitButton({
  label,
  pendingLabel,
  className,
}: FormSubmitButtonProps) {
  const { pending } = useFormStatus();

  return (
    <button type="submit" disabled={pending} className={className}>
      {pending ? pendingLabel : label}
    </button>
  );
}
