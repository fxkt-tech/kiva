"use client";

import { Clapperboard } from "lucide-react";
import {
  createContext,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export type EpisodeDetailView = "request" | "director-review";

type EpisodeDetailSelection = {
  readonly view: EpisodeDetailView;
  readonly showRequest: () => void;
  readonly showDirectorReview: () => void;
};

const EpisodeDetailSelectionContext =
  createContext<EpisodeDetailSelection | null>(null);

export function EpisodeDetailSelectionProvider({
  children,
  initialView = "request",
}: {
  readonly children?: ReactNode;
  readonly initialView?: EpisodeDetailView;
}) {
  const [view, setView] = useState<EpisodeDetailView>(initialView);
  const selection = useMemo<EpisodeDetailSelection>(
    () => ({
      view,
      showRequest: () => setView("request"),
      showDirectorReview: () => setView("director-review"),
    }),
    [view],
  );

  return (
    <EpisodeDetailSelectionContext.Provider value={selection}>
      {children}
    </EpisodeDetailSelectionContext.Provider>
  );
}

export function useEpisodeDetailSelection(): EpisodeDetailSelection | null {
  return useContext(EpisodeDetailSelectionContext);
}

export function DirectorReviewButton() {
  const selection = useEpisodeDetailSelection();
  const selected = selection?.view === "director-review";

  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={selection?.showDirectorReview}
      className={`inline-flex w-full items-center justify-center gap-2 rounded-md border px-4 py-2 text-sm font-semibold transition-colors ${
        selected
          ? "border-accent bg-accent text-accent-foreground"
          : "border-border bg-background/45 text-foreground hover:border-accent/70 hover:bg-accent/10"
      }`}
    >
      <Clapperboard aria-hidden="true" className="h-4 w-4" />
      Director review
    </button>
  );
}
