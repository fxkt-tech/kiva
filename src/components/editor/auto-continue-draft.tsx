"use client";

import { useEffect, useRef } from "react";
import { continueGameAction } from "@/app/actions";
import type { GameId } from "@/core/types";

type AutoContinueDraftProps = {
  readonly gameId: GameId;
};

export function AutoContinueDraft({ gameId }: AutoContinueDraftProps) {
  const formRef = useRef<HTMLFormElement>(null);
  const submittedRef = useRef(false);

  useEffect(() => {
    if (submittedRef.current) {
      return;
    }

    submittedRef.current = true;
    formRef.current?.requestSubmit();
  }, []);

  return (
    <div className="px-3 py-6 text-sm text-zinc-500">
      <p>Generating next draft...</p>
      <form
        ref={formRef}
        action={continueGameAction.bind(null, gameId)}
        aria-hidden="true"
        className="hidden"
      />
    </div>
  );
}
