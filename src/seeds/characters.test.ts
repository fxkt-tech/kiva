import { describe, expect, it } from "vitest";
import { seedCharacters } from "./characters";

describe("seed character voices", () => {
  it("uses a diverse, complete voice palette", () => {
    expect(seedCharacters).toHaveLength(12);
    expect(new Set(seedCharacters.map((character) => character.voiceProfile.voice)).size)
      .toBeGreaterThanOrEqual(6);
    for (const character of seedCharacters) {
      expect(character.voiceProfile.voice).toMatch(/^zh-CN-/);
      expect(character.voiceProfile.rate).toMatch(/^[+-]\d+%$/);
    }
  });

  it("keeps Qin Chuan as the constrained high-intelligence protagonist", () => {
    const qinChuan = seedCharacters.find((character) => character.id === "qin_chuan");
    expect(qinChuan?.name).toBe("秦川");
    expect(`${qinChuan?.persona}${qinChuan?.systemPrompt}`).toContain("高智商主角");
    expect(qinChuan?.systemPrompt).toContain("不能拥有上帝视角");
  });
});
