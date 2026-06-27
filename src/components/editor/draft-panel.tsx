import {
  confirmDraftAction,
  deleteDraftAction,
  editDraftPayloadAction,
} from "@/app/actions";
import type { DraftEvent } from "@/core/drafts";
import { formatDraftForHost, formatVisibility } from "@/core/event-presenter";
import type { PlayerSnapshot } from "@/core/player";
import type { GameId } from "@/core/types";

type DraftPanelProps = {
  readonly gameId: GameId;
  readonly draft: DraftEvent | null;
  readonly players: readonly PlayerSnapshot[];
};

export function DraftPanel({ gameId, draft, players }: DraftPanelProps) {
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

  const presented = formatDraftForHost(draft, players);

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
            Summary
          </div>
          <h3 className="mt-2 break-words text-sm font-medium text-zinc-100">
            {presented.title}
          </h3>
          <p className="mt-2 whitespace-pre-wrap break-words text-sm leading-6 text-zinc-300">
            {presented.text}
          </p>
          {presented.details && presented.details.length > 0 ? (
            <ul className="mt-2 space-y-1 text-xs text-zinc-500">
              {presented.details.map((detail) => (
                <li key={detail}>{detail}</li>
              ))}
            </ul>
          ) : null}
        </div>

        {draft.reason ? (
          <div>
            <div className="text-xs font-medium uppercase tracking-[0.14em] text-zinc-500">
              Reason
            </div>
            <p className="mt-2 whitespace-pre-wrap break-words text-sm leading-6 text-zinc-500">
              {draft.reason}
            </p>
          </div>
        ) : null}

        <DraftPayloadForm gameId={gameId} draft={draft} players={players} />

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

function DraftPayloadForm({
  gameId,
  draft,
  players,
}: {
  readonly gameId: GameId;
  readonly draft: DraftEvent;
  readonly players: readonly PlayerSnapshot[];
}) {
  const controls = renderDraftPayloadControls(draft, players);

  if (!controls) {
    return null;
  }

  return (
    <form
      action={editDraftPayloadAction.bind(null, gameId)}
      className="space-y-3 border-t border-zinc-800 pt-4"
    >
      <div className="text-xs font-medium uppercase tracking-[0.14em] text-zinc-500">
        Action payload
      </div>
      {controls}
      <button
        type="submit"
        className="rounded-md border border-zinc-700 px-3 py-2 text-xs font-medium text-zinc-200 transition hover:border-zinc-500 hover:text-white"
      >
        Save action
      </button>
    </form>
  );
}

function renderDraftPayloadControls(
  draft: DraftEvent,
  players: readonly PlayerSnapshot[],
) {
  switch (draft.type) {
    case "wolf_kill_selected":
    case "seer_check_selected":
      return (
        <PlayerTargetField
          label="Target player"
          players={players}
          defaultValue={draft.payload.targetPlayerId}
        />
      );

    case "witch_antidote_decided":
    case "witch_poison_decided":
      return (
        <div className="space-y-3">
          <label className="flex items-center gap-2 text-xs font-medium text-zinc-400">
            <input
              type="checkbox"
              name="used"
              value="true"
              defaultChecked={draft.payload.used}
              className="h-4 w-4 rounded border-zinc-700 bg-zinc-950 text-zinc-100"
            />
            <span>Use medicine</span>
          </label>
          <input type="hidden" name="used" value="false" />
          <PlayerTargetField
            label="Target player"
            players={players}
            defaultValue={draft.payload.targetPlayerId ?? ""}
            includeEmptyOption
            emptyLabel="No target"
          />
        </div>
      );

    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      return (
        <label className="block">
          <span className="text-xs font-medium text-zinc-400">Text</span>
          <textarea
            name="text"
            defaultValue={draft.payload.text}
            rows={5}
            className="mt-1 w-full resize-y rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm leading-6 text-zinc-100 outline-none transition placeholder:text-zinc-600 focus:border-zinc-400"
          />
        </label>
      );

    case "vote_cast":
      return (
        <PlayerTargetField
          label="Vote target"
          players={players}
          defaultValue={draft.payload.targetPlayerId ?? ""}
          includeEmptyOption
          emptyLabel="Abstain"
        />
      );

    default:
      return null;
  }
}

function PlayerTargetField({
  label,
  players,
  defaultValue,
  includeEmptyOption = false,
  emptyLabel = "Select player",
}: {
  readonly label: string;
  readonly players: readonly PlayerSnapshot[];
  readonly defaultValue: string;
  readonly includeEmptyOption?: boolean;
  readonly emptyLabel?: string;
}) {
  return (
    <label className="block">
      <span className="text-xs font-medium text-zinc-400">{label}</span>
      <select
        name="targetPlayerId"
        defaultValue={defaultValue}
        className="mt-1 w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 outline-none transition focus:border-zinc-400"
      >
        {includeEmptyOption ? <option value="">{emptyLabel}</option> : null}
        {players.map((player) => (
          <option key={player.playerId} value={player.playerId}>
            {player.seatNo} · {player.name}
          </option>
        ))}
      </select>
    </label>
  );
}
