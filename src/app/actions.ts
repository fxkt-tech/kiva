"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import type { DraftPayloadEdit } from "@/core/draft-edit";
import type { GamePreset, GamePresetSeatAssignment } from "@/core/game-preset";
import { parseGameRunMode, type GameRunMode } from "@/core/game-run-mode";
import type { DraftId, GameId, PlayerId } from "@/core/types";
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

export async function createGameFromPresetHomeAction(
  presetId: string,
  formData: FormData,
) {
  const record = await libraryActions.createGameFromPreset(
    presetId,
    "",
    formValue(formData, "scriptId"),
    runModeFromForm(formData),
  );
  revalidatePath("/");
  redirect(createdGamePath(record.game.id, record.game.runMode));
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
  const record = await libraryActions.createGameFromTemporaryPreset(
    preset,
    "",
    formValue(formData, "scriptId"),
    runModeFromForm(formData),
  );
  revalidatePath("/");
  redirect(createdGamePath(record.game.id, record.game.runMode));
}

export async function generateEpisodeScriptAction(gameId: GameId) {
  await gameActions.generateEpisodeScript(gameId);
  revalidatePath(`/games/${gameId}/script`);
  revalidatePath("/");
}

export async function approveEpisodeScriptAction(
  gameId: GameId,
  expectedJobId: string,
  expectedScriptId: string,
) {
  await gameActions.approveEpisodeScript(
    gameId,
    expectedJobId,
    expectedScriptId,
  );
  revalidatePath(`/games/${gameId}/script`);
  revalidatePath(`/games/${gameId}/editor`);
  revalidatePath("/");
  redirect(`/games/${gameId}/editor`);
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

export async function confirmDraftAction(gameId: GameId, draftId: DraftId) {
  await gameActions.confirmDraft(gameId, draftId);
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

function runModeFromForm(formData: FormData): GameRunMode {
  return parseGameRunMode(formValue(formData, "runMode"));
}

function createdGamePath(gameId: GameId, runMode: GameRunMode): string {
  return runMode === "scripted"
    ? `/games/${gameId}/script`
    : `/games/${gameId}/editor`;
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

  if (formData.has("leaderPlayerId")) {
    edit.leaderPlayerId = formValue(formData, "leaderPlayerId").trim() as PlayerId;
  }

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
