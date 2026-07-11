import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { MockLlmClient } from "@/core/llm";
import { compilePublicPlayback } from "@/core/playback";
import type { GameId } from "@/core/types";
import { createGameActions } from "../game-actions";
import { createGameRepository, type GameRecord } from "../game-repository";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(
    tempDirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
  );
});

describe("LLM integration", () => {
  it("confirms generated speech into public playback", async () => {
    const repository = createGameRepository(await createTempDir());
    const created = await createGameActions(repository).createGame();
    const actions = createGameActions(repository, {
      llmClient: new MockLlmClient([
        {
          text: "白天发言由模型生成。",
          reasoning: "根据当前可见信息生成白天发言。",
        },
      ]),
    });

    const withSpeech = await continueUntilDraftType(
      actions,
      created.game.id,
      "day_speech_given",
    );
    expect(withSpeech.draft).toMatchObject({
      type: "day_speech_given",
    });

    const generated = await actions.regenerateDraft(created.game.id);
    expect(generated.draft).toMatchObject({
      type: "day_speech_given",
      payload: { text: "白天发言由模型生成。" },
    });
    const confirmed = await actions.confirmDraft(created.game.id);
    const playback = compilePublicPlayback(
      confirmed.events,
      confirmed.game.players,
      { presenter: confirmed.game.presenter },
    );

    expect(
      playback.some((item) => item.text.includes("白天发言由模型生成。")),
    ).toBe(true);
  });
});

async function createTempDir(): Promise<string> {
  const dir = await mkdtemp(join(tmpdir(), "kiva-llm-integration-"));
  tempDirs.push(dir);
  return dir;
}

async function continueUntilDraftType(
  actions: ReturnType<typeof createGameActions>,
  gameId: GameId,
  draftType: NonNullable<GameRecord["draft"]>["type"],
): Promise<GameRecord> {
  for (let step = 0; step < 40; step += 1) {
    const record = await actions.continueGame(gameId);
    if (record.draft?.type === draftType) {
      return record;
    }
    await actions.confirmDraft(gameId);
  }

  throw new Error(`Draft not reached: ${draftType}`);
}
