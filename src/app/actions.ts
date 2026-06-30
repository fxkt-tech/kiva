"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import type { DraftPayloadEdit } from "@/core/draft-edit";
import type { GamePreset, GamePresetSeatAssignment } from "@/core/game-preset";
import type { GameId, PlayerId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import { createLibraryActions } from "@/server/library-actions";
import { createLibraryRepository } from "@/server/library-repository";
import { createRuntimeLlmClient } from "@/server/llm-runtime";

const dataDir = process.env.KIVA_DATA_DIR;

const gameActions = createGameActions(createGameRepository(dataDir), {
  llmClient: createRuntimeLlmClient(),
});
const libraryActions = createLibraryActions({
  libraryRepository: createLibraryRepository(dataDir),
  gameRepository: createGameRepository(dataDir),
});

export async function createGameAction() {
  redirect("/library?tab=presets");
}

export async function createGameFromPresetHomeAction(presetId: string) {
  const record = await libraryActions.createGameFromPreset(presetId);
  revalidatePath("/");
  redirect(`/games/${record.game.id}/editor`);
}

export async function createGameFromSeatAssignmentsAction(formData: FormData) {
  const seatAssignments = seatAssignmentsFromForm(formData);
  const preset: GamePreset = {
    id: `temporary_${Date.now()}`,
    name: "随机 12 人狼人杀",
    rulesetId: "classic_twelve",
    playerCount: 12,
    roleIds: seatAssignments.map((seat) => seat.roleId),
    characterIds: seatAssignments.map((seat) => seat.characterId),
    seatAssignments,
    enabled: true,
    createdAt: now(),
    updatedAt: now(),
  };
  const record = await libraryActions.createGameFromTemporaryPreset(preset);
  revalidatePath("/");
  redirect(`/games/${record.game.id}/editor`);
}

export async function deleteGameAction(gameId: GameId) {
  await gameActions.deleteGame(gameId);
  revalidatePath("/");
}

export async function renameGameAction(gameId: GameId, formData: FormData) {
  await gameActions.renameGame(gameId, formValue(formData, "title"));
  revalidatePath("/");
  revalidatePath(editorPath(gameId));
  revalidatePath(previewPath(gameId));
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

function seatAssignmentsFromForm(
  formData: FormData,
): readonly GamePresetSeatAssignment[] {
  return Array.from({ length: 12 }, (_, index) => {
    const seatNo = index + 1;

    return {
      seatNo,
      roleId: formValue(formData, `seat.${seatNo}.roleId`),
      characterId: formValue(formData, `seat.${seatNo}.characterId`),
      modelBindingOverride: null,
    };
  });
}

function now(): string {
  return new Date().toISOString();
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
