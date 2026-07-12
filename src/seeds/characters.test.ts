import { describe, expect, it } from "vitest";
import {
  EDGE_VOICE_RATE,
  FEMALE_PLAYER_EDGE_VOICE,
  MALE_PLAYER_EDGE_VOICE,
} from "@/core/voice";
import { seedCharacters } from "./characters";

const FEMALE_CHARACTER_IDS = new Set([
  "lin_xia",
  "gu_qingyan",
  "shen_lan",
  "tang_tang",
  "su_jin",
]);

describe("seed character voices", () => {
  it("uses one verified voice for each player gender", () => {
    for (const character of seedCharacters) {
      expect(character.voiceProfile.voice).toBe(
        FEMALE_CHARACTER_IDS.has(character.id)
          ? FEMALE_PLAYER_EDGE_VOICE
          : MALE_PLAYER_EDGE_VOICE,
      );
      expect(character.voiceProfile.rate).toBe(EDGE_VOICE_RATE);
    }
  });
});
