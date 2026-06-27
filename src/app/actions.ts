"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import type { DraftPayloadEdit } from "@/core/draft-edit";
import type { GameId, PlayerId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import { createRuntimeLlmClient } from "@/server/llm-runtime";

const dataDir = process.env.KIVA_DATA_DIR;

const gameActions = createGameActions(createGameRepository(dataDir), {
  llmClient: createRuntimeLlmClient(),
});

export async function createGameAction() {
  redirect("/library?tab=presets");
}

export async function confirmDraftAction(gameId: GameId) {
  await gameActions.confirmDraft(gameId);
  await gameActions.continueGame(gameId);
  revalidatePath(editorPath(gameId));
  revalidatePath(previewPath(gameId));
}

export async function editDraftPayloadAction(
  gameId: GameId,
  formData: FormData,
) {
  await gameActions.editDraftPayload(gameId, draftPayloadEditFromForm(formData));
  revalidatePath(editorPath(gameId));
  revalidatePath(previewPath(gameId));
}

export async function regenerateDraftAction(gameId: GameId) {
  await gameActions.regenerateDraft(gameId);
  revalidatePath(editorPath(gameId));
  revalidatePath(previewPath(gameId));
}

export async function deleteDraftAction(gameId: GameId) {
  await gameActions.deleteDraft(gameId);
  revalidatePath(editorPath(gameId));
}

export async function rollbackAfterAction(gameId: GameId, index: number) {
  await gameActions.rollbackAfter(gameId, index);
  revalidatePath(editorPath(gameId));
  revalidatePath(previewPath(gameId));
}

function formValue(formData: FormData, key: string): string {
  const value = formData.get(key);
  return typeof value === "string" ? value : "";
}

function draftPayloadEditFromForm(formData: FormData): DraftPayloadEdit {
  const edit: DraftPayloadEdit = {};

  if (formData.has("targetPlayerId")) {
    const targetPlayerId = formValue(formData, "targetPlayerId").trim();
    edit.targetPlayerId =
      targetPlayerId === "" ? null : (targetPlayerId as PlayerId);
  }

  if (formData.has("used")) {
    const used = formValue(formData, "used").trim().toLowerCase();
    edit.used = used === "true" || used === "on" || used === "1";
  }

  if (formData.has("text")) {
    edit.text = formValue(formData, "text");
  }

  return edit;
}

function editorPath(gameId: GameId): string {
  return `/games/${gameId}/editor`;
}

function previewPath(gameId: GameId): string {
  return `/games/${gameId}/preview`;
}
