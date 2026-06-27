import { describe, expect, it } from "vitest";
import { mechanicForDraftType, type ActionDraftType } from "../role-mechanics";
import type { RoleMechanicKey } from "../role-definition";

describe("role mechanics", () => {
  it.each([
    ["wolf_kill_selected", "wolf_kill"],
    ["seer_check_selected", "seer_check"],
    ["witch_antidote_decided", "witch_medicine"],
    ["witch_poison_decided", "witch_medicine"],
    ["vote_cast", "none"],
  ] satisfies readonly (readonly [ActionDraftType, RoleMechanicKey])[])(
    "maps %s to %s",
    (draftType, mechanicKey) => {
      expect(mechanicForDraftType(draftType)).toBe(mechanicKey);
    },
  );
});
