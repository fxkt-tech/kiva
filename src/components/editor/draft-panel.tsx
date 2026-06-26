import {
  confirmDraftAction,
  deleteDraftAction,
  editDraftDisplayAction,
} from "@/app/actions";
import type { DraftEvent } from "@/core/drafts";
import type { EventVisibility } from "@/core/events";
import type { GameId } from "@/core/types";

type DraftPanelProps = {
  readonly gameId: GameId;
  readonly draft: DraftEvent | null;
};

export function DraftPanel({ gameId, draft }: DraftPanelProps) {
  if (!draft) {
    return (
      <section className="rounded-lg border border-zinc-800 bg-zinc-900/45">
        <div className="border-b border-zinc-800 px-4 py-3">
          <h2 className="text-sm font-semibold text-zinc-100">Draft</h2>
        </div>
        <div className="px-4 py-8 text-sm text-zinc-500">
          No draft. Continue the game to plan the next event.
        </div>
      </section>
    );
  }

  return (
    <section className="rounded-lg border border-zinc-800 bg-zinc-900/45">
      <div className="flex items-start justify-between gap-3 border-b border-zinc-800 px-4 py-3">
        <div>
          <h2 className="text-sm font-semibold text-zinc-100">Draft</h2>
          <div className="mt-1 text-xs text-zinc-500">
            {draft.phase} · {formatVisibility(draft.visibility)}
          </div>
        </div>
        <span className="rounded-full border border-sky-900/80 px-2 py-1 text-xs text-sky-300">
          {draft.type}
        </span>
      </div>
      <div className="space-y-4 p-4">
        <div>
          <div className="text-xs font-medium uppercase tracking-[0.14em] text-zinc-500">
            Reason
          </div>
          <p className="mt-2 whitespace-pre-wrap break-words text-sm leading-6 text-zinc-300">
            {draft.reason || "No reason provided."}
          </p>
        </div>

        <form
          action={editDraftDisplayAction.bind(null, gameId)}
          className="space-y-3"
        >
          <label className="block">
            <span className="text-xs font-medium text-zinc-400">Title</span>
            <input
              name="title"
              defaultValue={draft.display?.title ?? draft.type}
              className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition placeholder:text-zinc-600 focus:border-zinc-400"
            />
          </label>
          <label className="block">
            <span className="text-xs font-medium text-zinc-400">Text</span>
            <textarea
              name="text"
              defaultValue={draft.display?.text ?? ""}
              rows={5}
              className="mt-1 w-full resize-y rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm leading-6 text-zinc-100 outline-none transition placeholder:text-zinc-600 focus:border-zinc-400"
            />
          </label>
          <button
            type="submit"
            className="rounded-md border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-200 transition hover:border-zinc-500 hover:text-white"
          >
            Save display
          </button>
        </form>

        <div className="flex flex-col gap-2 sm:flex-row">
          <form action={confirmDraftAction.bind(null, gameId)}>
            <button
              type="submit"
              className="w-full rounded-md bg-zinc-100 px-3 py-2 text-xs font-medium text-zinc-950 transition hover:bg-white sm:w-auto"
            >
              Confirm draft
            </button>
          </form>
          <form action={deleteDraftAction.bind(null, gameId)}>
            <button
              type="submit"
              className="w-full rounded-md border border-red-900/80 px-3 py-2 text-xs font-medium text-red-300 transition hover:border-red-600 hover:text-red-200 sm:w-auto"
            >
              Delete draft
            </button>
          </form>
        </div>
      </div>
    </section>
  );
}

function formatVisibility(visibility: EventVisibility): string {
  switch (visibility.kind) {
    case "public":
      return "public";
    case "host_only":
      return "host only";
    case "player_private":
      return `private ${visibility.playerIds.length}`;
    case "faction_private":
      return `${visibility.faction} private`;
    case "custom":
      return `custom ${visibility.playerIds.length}`;
  }
}
