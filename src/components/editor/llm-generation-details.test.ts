import { describe, expect, it } from "vitest";
import { generationMarkdown } from "./llm-generation-details";

describe("generationMarkdown", () => {
  it("formats the system prompt, user messages, and raw response as Markdown", () => {
    const generation = {
      request: {
        schemaName: "speech",
        systemPrompt: "Keep the game moving.",
        messages: [
          { role: "assistant", content: "Earlier reply" },
          { role: "user", content: "What should I say?" },
          { role: "user", content: "Keep it brief." },
        ],
      },
      rawOutput: '{"text":"Trust me."}',
    } as const;

    expect(generationMarkdown(generation)).toBe(
      '## Schema\n\nspeech\n\n## System prompt\n\nKeep the game moving.\n\n## User message\n\nWhat should I say?\n\n## User message\n\nKeep it brief.\n\n## Raw response\n\n{"text":"Trust me."}\n',
    );
  });

  it("marks unavailable snapshots as not recorded", () => {
    const generation = { request: null, rawOutput: null };

    expect(generationMarkdown(generation)).toBe(
      "## Schema\n\nNot recorded.\n\n## System prompt\n\nNot recorded.\n\n## User message\n\nNot recorded.\n\n## Raw response\n\nNot recorded.\n",
    );
  });

  it("includes invalid and accepted repair attempts", () => {
    const generation = {
      request: {
        schemaName: "speech_v2",
        systemPrompt: "Original prompt",
        messages: [{ role: "user", content: "Speak." }],
      },
      rawOutput: '{"text":"Fixed."}',
      attempts: [
        {
          request: {
            schemaName: "speech_v2",
            systemPrompt: "Original prompt",
            messages: [{ role: "user", content: "Speak." }],
          },
          tokenUsage: null,
          rawOutput: '{"text":""}',
          parsedOutput: { text: "" },
          error: "text must not be empty",
        },
        {
          request: {
            schemaName: "speech_v2",
            systemPrompt: "Repair prompt",
            messages: [{ role: "user", content: "Fix JSON." }],
          },
          tokenUsage: null,
          rawOutput: '{"text":"Fixed."}',
          parsedOutput: { text: "Fixed." },
          error: null,
        },
      ],
    } as const;

    const markdown = generationMarkdown(generation);

    expect(markdown).toContain(
      '## Attempt 1\n\nStatus: invalid\n\nRaw response:\n\n{"text":""}\n\nError: text must not be empty',
    );
    expect(markdown).toContain(
      '## Attempt 2\n\nStatus: accepted\n\nRaw response:\n\n{"text":"Fixed."}',
    );
  });
});
