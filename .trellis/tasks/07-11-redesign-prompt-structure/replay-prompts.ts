import { readFile } from "node:fs/promises";
import { generateActionDraft } from "../../../src/core/action-generation";
import type { DraftEvent } from "../../../src/core/drafts";
import type { GameEvent } from "../../../src/core/events";
import type { Game } from "../../../src/core/game";
import { OpenAICompatibleLlmClient } from "../../../src/core/llm";
import { isLlmActionDraft, isLlmSpeechDraft } from "../../../src/core/llm-task-specs";
import { generateSpeechDraft } from "../../../src/core/speech-generation";

type StoredGeneration = {
  readonly draftId: string;
  readonly rawOutput: string | null;
  readonly parsedOutput: Record<string, unknown> | null;
};

type StoredRecord = {
  readonly game: Game;
  readonly events: readonly GameEvent[];
  readonly draft: DraftEvent | null;
  readonly generations: readonly StoredGeneration[];
};

const scenarios = [
  {
    name: "首夜狼队战术",
    eventIndex: 16,
    legacyDraftId: "draft_bed31555-392a-49c2-aaf7-efde068115fd",
  },
  {
    name: "首夜狼队意见",
    eventIndex: 17,
    legacyDraftId: "draft_b675a9a1-b8d8-4d08-9b5e-77bb8257356d",
  },
  {
    name: "首夜狼人 4 号密票",
    eventIndex: 20,
    legacyDraftId: "draft_8cd00332-7617-4364-8700-e636134dc1a3",
  },
  {
    name: "首夜狼人 7 号密票",
    eventIndex: 21,
    legacyDraftId: "draft_17dc4254-5fc6-4553-a9e8-c0f01c908fe5",
  },
  {
    name: "首夜狼人 10 号密票",
    eventIndex: 22,
    legacyDraftId: "draft_9fd569cb-a85f-4e0b-90b6-dd30ff07fec5",
  },
  {
    name: "首夜狼人 12 号密票",
    eventIndex: 23,
    legacyDraftId: "draft_9a1ac9b4-7076-4b23-a1b1-7569394996c9",
  },
  {
    name: "首夜预言家查验",
    eventIndex: 25,
    legacyDraftId: "draft_0c72ee3b-8e08-4b12-8fad-7f314e4e699c",
  },
  {
    name: "首夜女巫解药",
    eventIndex: 28,
    legacyDraftId: "draft_e366601f-d095-4c40-9a1b-729ffd92b1da",
  },
  {
    name: "首日第一位公开发言",
    eventIndex: 33,
    legacyDraftId: "draft_61171b6d-eef6-46c5-9dae-2558f4aeac38",
  },
  {
    name: "首日狼人公开发言",
    eventIndex: 36,
    legacyDraftId: "draft_3208ee4b-37db-4d5f-a434-6e3f7ba35fe8",
  },
] as const;

async function main(): Promise<void> {
  const path = process.argv[2];
  if (!path) throw new Error("Usage: tsx replay-prompts.ts <game-record.json>");

  const baseUrl = process.env.OPENAI_COMPATIBLE_BASE_URL?.trim();
  const apiKey = process.env.OPENAI_COMPATIBLE_API_KEY?.trim();
  if (!baseUrl || !apiKey) {
    throw new Error("OpenAI-compatible credentials are not configured");
  }

  const record = JSON.parse(await readFile(path, "utf8")) as StoredRecord;
  const llmClient = new OpenAICompatibleLlmClient({ baseUrl, apiKey });
  const results = [];
  const selectedIndexes = new Set(
    process.argv.slice(3).map((value) => Number.parseInt(value, 10)),
  );

  for (const scenario of scenarios) {
    if (selectedIndexes.size > 0 && !selectedIndexes.has(scenario.eventIndex)) {
      continue;
    }
    const event = record.events.find(
      (candidate) => candidate.index === scenario.eventIndex,
    );
    const currentDraft =
      record.draft?.id === scenario.legacyDraftId ? record.draft : null;
    if (!event && !currentDraft) {
      throw new Error(`Missing event or current Draft ${scenario.eventIndex}`);
    }
    const draft = event
      ? eventAsDraft(event, scenario.legacyDraftId)
      : currentDraft!;
    const events = record.events.filter(
      (candidate) => candidate.index < scenario.eventIndex,
    );
    const legacy = [...record.generations]
      .reverse()
      .find((generation) => generation.draftId === scenario.legacyDraftId);
    if (!legacy) throw new Error(`Missing generation ${scenario.legacyDraftId}`);

    const generated = isLlmSpeechDraft(draft)
      ? await generateSpeechDraft({
          game: record.game,
          events,
          draft,
          llmClient,
          generationId: `replay_${scenario.eventIndex}`,
          createdAt: new Date().toISOString(),
          promptMode: "v2",
        })
      : isLlmActionDraft(draft)
        ? await generateActionDraft({
            game: record.game,
            events,
            draft,
            llmClient,
            generationId: `replay_${scenario.eventIndex}`,
            createdAt: new Date().toISOString(),
            promptMode: "v2",
          })
        : null;
    if (!generated?.generation) {
      throw new Error(`Event ${scenario.eventIndex} is not an LLM draft`);
    }

    results.push({
      scenario: scenario.name,
      draftType: draft.type,
      legacy: {
        rawOutput: legacy.rawOutput,
        parsedOutput: legacy.parsedOutput,
      },
      v2: {
        status: generated.generation.status,
        promptVersion: generated.generation.promptVersion,
        schemaName: generated.generation.request?.schemaName ?? null,
        rawOutput: generated.generation.rawOutput,
        parsedOutput: generated.generation.parsedOutput,
        error: generated.generation.error,
        attempts: generated.generation.attempts?.length ?? 1,
      },
    });
  }

  process.stdout.write(`${JSON.stringify(results, null, 2)}\n`);
}

function eventAsDraft(event: GameEvent, id: string): DraftEvent {
  const { index: _index, ...withoutIndex } = event;
  return {
    ...withoutIndex,
    id: id as DraftEvent["id"],
    status: "draft",
  } as DraftEvent;
}

void main().catch((error: unknown) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  process.stderr.write(`${message}\n`);
  process.exitCode = 1;
});
