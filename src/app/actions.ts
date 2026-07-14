"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import { after } from "next/server";
import type { DraftPayloadEdit } from "@/core/draft-edit";
import type { Lineup, LineupSeat } from "@/core/lineup";
import { parseGameRunMode, type GameRunMode } from "@/core/game-run-mode";
import type { DraftId, GameId, PlayerId } from "@/core/types";
import { createGameActions } from "@/server/game-actions";
import { createGameRepository } from "@/server/game-repository";
import { createContentActions } from "@/server/content-actions";
import { createContentCatalog } from "@/server/content-catalog";
import { createRuntimeLlmClient } from "@/server/llm-runtime";

const dataDir = process.env.KIVA_DATA_DIR;

const gameActions = createGameActions(createGameRepository(dataDir), {
  llmClient: createRuntimeLlmClient(),
  contentCatalog: createContentCatalog(dataDir),
});
const contentActions = createContentActions({
  catalog: createContentCatalog(dataDir),
  gameRepository: createGameRepository(dataDir),
});

export async function createGameAction() {
  redirect("/library?tab=lineups");
}

export async function createGameFromLineupHomeAction(
  lineupId: string,
  formData: FormData,
) {
  const record = await contentActions.createGameFromLineup(
    lineupId,
    "",
    formValue(formData, "scriptId"),
    runModeFromForm(formData),
  );
  revalidatePath("/");
  redirect(createdGamePath(record.game.id, record.game.runMode));
}

export async function createGameFromCustomLineupHomeAction(formData: FormData) {
  const seatAssignments = seatAssignmentsFromForm(formData);
  const lineup: Lineup = {
    id: `temporary_${Date.now()}`,
    name: "随机 12 人狼人杀",
    rulesetId: "classic_twelve",
    seats: seatAssignments,
    enabled: true,
    revision: 1,
  };
  const record = await contentActions.createGameFromTemporaryLineup(
    lineup,
    "",
    formValue(formData, "scriptId"),
    runModeFromForm(formData),
  );
  revalidatePath("/");
  redirect(createdGamePath(record.game.id, record.game.runMode));
}

export async function generateEpisodeScriptAction(gameId: GameId) {
  const jobId = await gameActions.startEpisodeScriptGeneration(gameId);
  after(() => gameActions.runEpisodeScriptGeneration(gameId, jobId));
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

function seatAssignmentsFromForm(formData: FormData): readonly LineupSeat[] {
  return Array.from({ length: 12 }, (_, index) => {
    const seatNo = index + 1;

    return {
      seatNo,
      ruleRoleId: formValue(
        formData,
        `seat.${seatNo}.ruleRoleId`,
      ) as LineupSeat["ruleRoleId"],
      actorId: formValue(formData, `seat.${seatNo}.actorId`),
    };
  });
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
