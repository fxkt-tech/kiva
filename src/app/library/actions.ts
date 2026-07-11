"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import { createGameRepository } from "@/server/game-repository";
import { createLibraryActions } from "@/server/library-actions";
import { createLibraryRepository } from "@/server/library-repository";
import {
  characterFromFormData,
  presenterFromFormData,
  presetFromFormData,
  roleFromFormData,
} from "./form-parsers";

const dataDir = process.env.KIVA_DATA_DIR;

const libraryActions = createLibraryActions({
  libraryRepository: createLibraryRepository(dataDir),
  gameRepository: createGameRepository(dataDir),
});

export async function saveRoleAction(formData: FormData) {
  await libraryActions.saveRole(roleFromFormData(formData, now()));
  revalidatePath("/library");
}

export async function saveCharacterAction(formData: FormData) {
  await libraryActions.saveCharacter(characterFromFormData(formData, now()));
  revalidatePath("/library");
}

export async function savePresetAction(formData: FormData) {
  await libraryActions.savePreset(presetFromFormData(formData, now()));
  revalidatePath("/library");
}

export async function savePresenterAction(formData: FormData) {
  await libraryActions.savePresenter(presenterFromFormData(formData, now()));
  revalidatePath("/library");
  revalidatePath("/");
}

export async function duplicateRoleAction(roleId: string) {
  await libraryActions.duplicateRole(roleId);
  revalidatePath("/library");
}

export async function duplicateCharacterAction(characterId: string) {
  await libraryActions.duplicateCharacter(characterId);
  revalidatePath("/library");
}

export async function duplicatePresetAction(presetId: string) {
  await libraryActions.duplicatePreset(presetId);
  revalidatePath("/library");
}

export async function setRoleEnabledAction(roleId: string, enabled: boolean) {
  await libraryActions.setRoleEnabled(roleId, enabled);
  revalidatePath("/library");
}

export async function setCharacterEnabledAction(
  characterId: string,
  enabled: boolean,
) {
  await libraryActions.setCharacterEnabled(characterId, enabled);
  revalidatePath("/library");
}

export async function setPresetEnabledAction(presetId: string, enabled: boolean) {
  await libraryActions.setPresetEnabled(presetId, enabled);
  revalidatePath("/library");
}

export async function createGameFromPresetAction(
  presetId: string,
  formData: FormData,
) {
  const presenterId = formData.get("presenterId");
  const record = await libraryActions.createGameFromPreset(
    presetId,
    typeof presenterId === "string" ? presenterId : "",
  );
  revalidatePath("/");
  revalidatePath("/library");
  redirect(`/games/${record.game.id}/editor`);
}

function now(): string {
  return new Date().toISOString();
}
