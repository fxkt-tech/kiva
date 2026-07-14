import { describe, expect, it } from "vitest";
import { mechanicForDraftType, type ActionDraftType } from "../role-mechanics";
import type { RuleRoleMechanicKey } from "../rule-role";

describe("role mechanics", () => {
  it.each([
    ["wolf_vote_cast", "wolf_kill"],
    ["seer_check_selected", "seer_check"],
    ["witch_antidote_decided", "witch_medicine"],
    ["witch_poison_decided", "witch_medicine"],
    ["guard_protect_selected", "guard_protect"],
    ["hunter_shot_decided", "hunter_shot"],
    ["vote_cast", "none"],
  ] satisfies readonly (readonly [ActionDraftType, RuleRoleMechanicKey])[])(
    "maps %s to %s",
    (draftType, mechanicKey) => {
      expect(mechanicForDraftType(draftType)).toBe(mechanicKey);
    },
  );
});
