"use client";

import { Check, Pencil, X } from "lucide-react";
import { useEffect, useRef, useState, useTransition } from "react";
import { renameGameAction } from "@/app/actions";
import { Button } from "@/components/ui/button";
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
            className="h-8 min-w-0 flex-1 rounded-md border border-interactive-border bg-background px-2.5 text-sm font-medium text-foreground outline-none transition"
          />
          <Button
            onClick={saveTitle}
            disabled={pending}
            aria-label="Save game name"
            title="Save game name"
            buttonStyle="icon"
            variant="success"
          >
            <Check aria-hidden="true" className="h-4 w-4" />
          </Button>
          <Button
            onClick={cancelEditing}
            disabled={pending}
            aria-label="Cancel rename"
            title="Cancel rename"
            buttonStyle="icon"
          >
            <X aria-hidden="true" className="h-4 w-4" />
          </Button>
        </div>
        {error ? <div className="mt-1 text-xs text-red-300">{error}</div> : null}
      </div>
    );
  }

  return (
    <div className="group flex min-w-0 items-center gap-2">
      <div className="min-w-0 truncate text-sm font-medium text-foreground">
        {currentTitle}
      </div>
      <Button
        onClick={beginEditing}
        aria-label="Rename game"
        title="Rename game"
        buttonStyle="icon"
        className="opacity-0 group-hover:opacity-100"
        iconSize="xs"
      >
        <Pencil aria-hidden="true" className="h-3.5 w-3.5" />
      </Button>
    </div>
  );
}
