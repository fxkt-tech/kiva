"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import type { GameId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";

const gameActions = createGameActions(createGameRepository());

export async function createGameAction() {
  const record = await gameActions.createGame();
  redirect(editorPath(record.game.id));
}

export async function continueGameAction(gameId: GameId) {
  await gameActions.continueGame(gameId);
  revalidatePath(editorPath(gameId));
}

export async function confirmDraftAction(gameId: GameId) {
  await gameActions.confirmDraft(gameId);
  revalidatePath(editorPath(gameId));
  revalidatePath(previewPath(gameId));
}

export async function editDraftDisplayAction(
  gameId: GameId,
  formData: FormData,
) {
  await gameActions.editDraftDisplay(gameId, {
    title: formValue(formData, "title"),
    text: formValue(formData, "text"),
  });
  revalidatePath(editorPath(gameId));
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

function editorPath(gameId: GameId): string {
  return `/games/${gameId}/editor`;
}

function previewPath(gameId: GameId): string {
  return `/games/${gameId}/preview`;
}
