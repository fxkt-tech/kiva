"use server";

import { revalidatePath } from "next/cache";
import { redirect } from "next/navigation";
import { createContentActions } from "@/server/content-actions";
import { createContentCatalog } from "@/server/content-catalog";
import { createGameRepository } from "@/server/game-repository";
import {
  actorFromFormData,
  lineupFromFormData,
  presenterFromFormData,
  scriptFromFormData,
} from "./form-parsers";

const dataDir = process.env.KIVA_DATA_DIR;
const actions = createContentActions({
  catalog: createContentCatalog(dataDir),
  gameRepository: createGameRepository(dataDir),
});

export async function saveActorAction(formData: FormData) {
  await actions.saveActor(actorFromFormData(formData));
  revalidatePath("/library");
}

export async function createActorAction(formData: FormData) {
  const actor = actorFromFormData(formData);
  await actions.saveActor(actor);
  revalidatePath("/library");
  redirect(`/library?tab=actors&id=${encodeURIComponent(actor.id)}`);
}

export async function saveLineupAction(formData: FormData) {
  await actions.saveLineup(lineupFromFormData(formData));
  revalidatePath("/library");
  revalidatePath("/");
}

export async function savePresenterAction(formData: FormData) {
  await actions.savePresenter(presenterFromFormData(formData, now()));
  revalidatePath("/library");
  revalidatePath("/");
}

export async function saveScriptAction(formData: FormData) {
  await actions.saveScript(scriptFromFormData(formData, now()));
  revalidatePath("/library");
  revalidatePath("/");
}

export async function duplicateActorAction(actorId: string) {
  await actions.duplicateActor(actorId);
  revalidatePath("/library");
}

export async function duplicateLineupAction(lineupId: string) {
  await actions.duplicateLineup(lineupId);
  revalidatePath("/library");
}

export async function duplicateScriptAction(scriptId: string) {
  await actions.duplicateScript(scriptId);
  revalidatePath("/library");
  revalidatePath("/");
}

export async function setActorEnabledAction(id: string, enabled: boolean) {
  await actions.setActorEnabled(id, enabled);
  revalidatePath("/library");
}

export async function setLineupEnabledAction(id: string, enabled: boolean) {
  await actions.setLineupEnabled(id, enabled);
  revalidatePath("/library");
  revalidatePath("/");
}

export async function createGameFromLineupAction(
  lineupId: string,
  formData: FormData,
) {
  const presenterId = formData.get("presenterId");
  const scriptId = formData.get("scriptId");
  const record = await actions.createGameFromLineup(
    lineupId,
    typeof presenterId === "string" ? presenterId : "",
    typeof scriptId === "string" ? scriptId : undefined,
  );
  revalidatePath("/");
  redirect(`/games/${record.game.id}/editor`);
}

function now(): string {
  return new Date().toISOString();
}
