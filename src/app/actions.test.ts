import { beforeEach, describe, expect, it, vi } from "vitest";
import type { GameId } from "@/core/types";

const actionMocks = vi.hoisted(() => ({
  createGameFromPreset: vi.fn(),
  createGameFromTemporaryPreset: vi.fn(),
  generateEpisodeScript: vi.fn(),
  startEpisodeScriptGeneration: vi.fn(),
  runEpisodeScriptGeneration: vi.fn(),
  after: vi.fn(),
  redirect: vi.fn(),
  revalidatePath: vi.fn(),
}));

vi.mock("next/cache", () => ({
  revalidatePath: actionMocks.revalidatePath,
}));

vi.mock("next/navigation", () => ({
  redirect: actionMocks.redirect,
}));

vi.mock("next/server", () => ({
  after: actionMocks.after,
}));

vi.mock("@/server/game-actions", () => ({
  createGameActions: () => ({
    generateEpisodeScript: actionMocks.generateEpisodeScript,
    startEpisodeScriptGeneration: actionMocks.startEpisodeScriptGeneration,
    runEpisodeScriptGeneration: actionMocks.runEpisodeScriptGeneration,
  }),
}));

vi.mock("@/server/game-repository", () => ({
  createGameRepository: () => ({}),
}));

vi.mock("@/server/library-actions", () => ({
  createLibraryActions: () => ({
    createGameFromPreset: actionMocks.createGameFromPreset,
    createGameFromTemporaryPreset: actionMocks.createGameFromTemporaryPreset,
  }),
}));

vi.mock("@/server/library-repository", () => ({
  createLibraryRepository: () => ({}),
}));

vi.mock("@/server/llm-runtime", () => ({
  createRuntimeLlmClient: () => ({}),
}));

import {
  createGameFromPresetHomeAction,
  createGameFromSeatAssignmentsAction,
  generateEpisodeScriptAction,
} from "./actions";

const scriptedRecord = {
  game: {
    id: "scripted-game-1",
    runMode: "scripted",
  },
};

beforeEach(() => {
  vi.clearAllMocks();
  actionMocks.createGameFromPreset.mockResolvedValue(scriptedRecord);
  actionMocks.createGameFromTemporaryPreset.mockResolvedValue(scriptedRecord);
  actionMocks.startEpisodeScriptGeneration.mockResolvedValue("episode_job_1");
  actionMocks.runEpisodeScriptGeneration.mockResolvedValue(scriptedRecord);
});

describe("Episode Script server actions", () => {
  it("returns after persisting the job and schedules long authoring after the response", async () => {
    await generateEpisodeScriptAction("scripted-game-1" as GameId);

    expect(actionMocks.startEpisodeScriptGeneration).toHaveBeenCalledWith(
      "scripted-game-1",
    );
    expect(actionMocks.runEpisodeScriptGeneration).not.toHaveBeenCalled();
    expect(actionMocks.after).toHaveBeenCalledOnce();

    const scheduled = actionMocks.after.mock.calls[0]?.[0];
    expect(scheduled).toBeTypeOf("function");
    await scheduled();
    expect(actionMocks.runEpisodeScriptGeneration).toHaveBeenCalledWith(
      "scripted-game-1",
      "episode_job_1",
    );
  });
});

describe("New Game server actions", () => {
  it("creates a scripted preset game without generating its episode script", async () => {
    const formData = scriptedFormData();

    await createGameFromPresetHomeAction("twelve_player_standard", formData);

    expect(actionMocks.createGameFromPreset).toHaveBeenCalledWith(
      "twelve_player_standard",
      "",
      "midnight_archive",
      "scripted",
    );
    expect(actionMocks.generateEpisodeScript).not.toHaveBeenCalled();
    expect(actionMocks.redirect).toHaveBeenCalledWith(
      "/games/scripted-game-1/script",
    );
  });

  it("creates a scripted random-seat game without generating its episode script", async () => {
    const formData = scriptedFormData();
    for (let seatNo = 1; seatNo <= 12; seatNo += 1) {
      formData.set(`seat.${seatNo}.roleId`, "villager");
      formData.set(`seat.${seatNo}.characterId`, `character_${seatNo}`);
    }

    await createGameFromSeatAssignmentsAction(formData);

    expect(actionMocks.createGameFromTemporaryPreset).toHaveBeenCalledOnce();
    expect(actionMocks.generateEpisodeScript).not.toHaveBeenCalled();
    expect(actionMocks.redirect).toHaveBeenCalledWith(
      "/games/scripted-game-1/script",
    );
  });
});

function scriptedFormData(): FormData {
  const formData = new FormData();
  formData.set("scriptId", "midnight_archive");
  formData.set("runMode", "scripted");
  return formData;
}
