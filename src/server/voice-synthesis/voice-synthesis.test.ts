import { describe, expect, it } from "vitest";
import {
  decodeWordBoundaries,
  prepareEdgeSpeechText,
} from "./edge-adapter";
import { aggregateSpeechCues } from "./speech-cues";

describe("Edge voice synthesis adaptation", () => {
  it("keeps provider-specific stage direction handling inside the adapter", () => {
    expect(
      prepareEdgeSpeechText("等一下……（低头小声）我没听懂。（停顿）你再说一遍。"),
    ).toBe("等一下……我没听懂。你再说一遍。");
  });

  it("rejects unordered word boundaries", () => {
    expect(() =>
      decodeWordBoundaries([
        { part: "第一句", start: 500, end: 900 },
        { part: "第二句", start: 800, end: 1200 },
      ]),
    ).toThrow("not ordered");
  });

  it("aggregates real word timings into sentence cues", () => {
    expect(
      aggregateSpeechCues(
        [
          { part: "第一句。", start: 100, end: 900 },
          { part: "第二句", start: 1100, end: 1700 },
          { part: "结束！", start: 1700, end: 2200 },
        ],
        2400,
      ),
    ).toEqual([
      { text: "第一句。", startMs: 100, endMs: 900 },
      { text: "第二句结束！", startMs: 1100, endMs: 2400 },
    ]);
  });
});
