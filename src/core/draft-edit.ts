import type { DraftEvent } from "./drafts";
import type { PlayerId } from "./types";

export type DraftPayloadEdit = {
  targetPlayerId?: PlayerId | null;
  used?: boolean;
  text?: string;
} & Record<string, unknown>;

type DraftOf<Type extends DraftEvent["type"]> = Extract<DraftEvent, { type: Type }>;

export function applyDraftPayloadEdit(
  draft: DraftEvent,
  edit: DraftPayloadEdit,
): DraftEvent {
  switch (draft.type) {
    case "wolf_kill_selected":
    case "seer_check_selected":
      assertSupportedEdit(draft.type, edit, ["targetPlayerId"]);
      return editTargetDraft(draft, edit);

    case "witch_antidote_decided":
    case "witch_poison_decided":
      assertSupportedEdit(draft.type, edit, ["used", "targetPlayerId"]);
      return editWitchMedicineDraft(draft, edit);

    case "last_words_given":
    case "day_speech_given":
    case "pk_speech_given":
      assertSupportedEdit(draft.type, edit, ["text"]);
      return editTextDraft(draft, edit);

    case "vote_cast":
      assertSupportedEdit(draft.type, edit, ["targetPlayerId"]);
      return editVoteDraft(draft, edit);

    default:
      throw unsupportedEdit(draft.type);
  }
}

function editTargetDraft(
  draft: DraftOf<"wolf_kill_selected" | "seer_check_selected">,
  edit: DraftPayloadEdit,
): DraftEvent {
  if (!hasOwn(edit, "targetPlayerId")) {
    return draft;
  }

  const targetPlayerId = edit.targetPlayerId;
  if (targetPlayerId === null || targetPlayerId === undefined) {
    throw new Error(`targetPlayerId is required for ${draft.type}`);
  }

  return {
    ...draft,
    targetPlayerIds: [targetPlayerId],
    payload: {
      ...draft.payload,
      targetPlayerId,
    },
  };
}

function editWitchMedicineDraft(
  draft: DraftOf<"witch_antidote_decided" | "witch_poison_decided">,
  edit: DraftPayloadEdit,
): DraftEvent {
  const used = hasOwn(edit, "used") ? edit.used : draft.payload.used;
  const targetPlayerId = hasOwn(edit, "targetPlayerId")
    ? edit.targetPlayerId
    : draft.payload.targetPlayerId;

  if (typeof used !== "boolean") {
    throw new Error(`used is required for ${draft.type}`);
  }

  if (!used) {
    return {
      ...draft,
      targetPlayerIds: [],
      payload: {
        ...draft.payload,
        used: false,
        targetPlayerId: null,
      },
    };
  }

  if (targetPlayerId === null || targetPlayerId === undefined) {
    throw new Error(`targetPlayerId is required when ${draft.type} is used`);
  }

  return {
    ...draft,
    targetPlayerIds: [targetPlayerId],
    payload: {
      ...draft.payload,
      used: true,
      targetPlayerId,
    },
  };
}

function editTextDraft(
  draft: DraftOf<"last_words_given" | "day_speech_given" | "pk_speech_given">,
  edit: DraftPayloadEdit,
): DraftEvent {
  if (!hasOwn(edit, "text")) {
    return draft;
  }

  return {
    ...draft,
    payload: {
      ...draft.payload,
      text: String(edit.text ?? "").trim(),
    },
  } as DraftEvent;
}

function editVoteDraft(
  draft: DraftOf<"vote_cast">,
  edit: DraftPayloadEdit,
): DraftEvent {
  if (!hasOwn(edit, "targetPlayerId")) {
    return draft;
  }

  const targetPlayerId = edit.targetPlayerId ?? null;
  return {
    ...draft,
    targetPlayerIds: targetPlayerId ? [targetPlayerId] : [],
    payload: {
      ...draft.payload,
      targetPlayerId,
    },
  };
}

function assertSupportedEdit(
  draftType: DraftEvent["type"],
  edit: DraftPayloadEdit,
  supportedKeys: readonly string[],
): void {
  for (const key of Object.keys(edit)) {
    if (!supportedKeys.includes(key)) {
      throw unsupportedEdit(draftType);
    }
  }
}

function unsupportedEdit(draftType: DraftEvent["type"]): Error {
  return new Error(`Unsupported draft edit for ${draftType}`);
}

function hasOwn<T extends object, Key extends PropertyKey>(
  value: T,
  key: Key,
): value is T & Record<Key, unknown> {
  return Object.prototype.hasOwnProperty.call(value, key);
}
