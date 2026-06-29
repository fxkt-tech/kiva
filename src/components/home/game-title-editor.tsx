"use client";

import { Check, Pencil, X } from "lucide-react";
import { useEffect, useRef, useState, useTransition } from "react";
import { renameGameAction } from "@/app/actions";
import type { GameId } from "@/core/types";

type GameTitleEditorProps = {
  readonly gameId: GameId;
  readonly title: string;
};

export function GameTitleEditor({ gameId, title }: GameTitleEditorProps) {
  const [editing, setEditing] = useState(false);
  const [currentTitle, setCurrentTitle] = useState(title);
  const [draftTitle, setDraftTitle] = useState(title);
  const [error, setError] = useState<string | null>(null);
  const [pending, startTransition] = useTransition();
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!editing) {
      setCurrentTitle(title);
      setDraftTitle(title);
    }
  }, [editing, title]);

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [editing]);

  function beginEditing() {
    setDraftTitle(currentTitle);
    setError(null);
    setEditing(true);
  }

  function cancelEditing() {
    setDraftTitle(currentTitle);
    setError(null);
    setEditing(false);
  }

  function saveTitle() {
    const nextTitle = draftTitle.trim();
    if (nextTitle.length === 0) {
      setError("Name cannot be blank.");
      return;
    }

    const formData = new FormData();
    formData.set("title", nextTitle);
    setError(null);
    startTransition(() => {
      void renameGameAction(gameId, formData)
        .then(() => {
          setCurrentTitle(nextTitle);
          setDraftTitle(nextTitle);
          setEditing(false);
        })
        .catch((reason: unknown) => {
          setError(reason instanceof Error ? reason.message : "Save failed.");
        });
    });
  }

  if (editing) {
    return (
      <div className="min-w-0">
        <div className="flex items-center gap-1.5">
          <input
            ref={inputRef}
            value={draftTitle}
            onChange={(event) => setDraftTitle(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                saveTitle();
              }
              if (event.key === "Escape") {
                cancelEditing();
              }
            }}
            aria-label="Game name"
            disabled={pending}
            className="h-8 min-w-0 flex-1 rounded-md border border-sky-800/80 bg-zinc-950 px-2.5 text-sm font-medium text-zinc-50 outline-none transition focus:border-sky-500 focus:ring-2 focus:ring-sky-950"
          />
          <button
            type="button"
            onClick={saveTitle}
            disabled={pending}
            aria-label="Save game name"
            title="Save game name"
            className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-emerald-900/80 text-emerald-300 transition hover:border-emerald-600 hover:text-emerald-200 disabled:opacity-50"
          >
            <Check aria-hidden="true" className="h-4 w-4" />
          </button>
          <button
            type="button"
            onClick={cancelEditing}
            disabled={pending}
            aria-label="Cancel rename"
            title="Cancel rename"
            className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md border border-zinc-700 text-zinc-400 transition hover:border-zinc-500 hover:text-zinc-100 disabled:opacity-50"
          >
            <X aria-hidden="true" className="h-4 w-4" />
          </button>
        </div>
        {error ? <div className="mt-1 text-xs text-red-300">{error}</div> : null}
      </div>
    );
  }

  return (
    <div className="group flex min-w-0 items-center gap-2">
      <div className="min-w-0 truncate text-sm font-medium text-zinc-100">
        {currentTitle}
      </div>
      <button
        type="button"
        onClick={beginEditing}
        aria-label="Rename game"
        title="Rename game"
        className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-md border border-transparent text-zinc-500 opacity-0 transition hover:border-zinc-700 hover:text-zinc-200 group-hover:opacity-100 focus:opacity-100"
      >
        <Pencil aria-hidden="true" className="h-3.5 w-3.5" />
      </button>
    </div>
  );
}
