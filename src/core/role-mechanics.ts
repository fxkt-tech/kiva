import type { RuleRoleMechanicKey } from "./rule-role";

export type ActionDraftType =
  | "seer_check_selected"
  | "wolf_vote_cast"
  | "vote_cast"
  | "witch_antidote_decided"
  | "witch_poison_decided"
  | "guard_protect_selected"
  | "hunter_shot_decided";

const MECHANIC_BY_DRAFT_TYPE = {
  seer_check_selected: "seer_check",
  wolf_vote_cast: "wolf_kill",
  vote_cast: "none",
  witch_antidote_decided: "witch_medicine",
  witch_poison_decided: "witch_medicine",
  guard_protect_selected: "guard_protect",
  hunter_shot_decided: "hunter_shot",
} satisfies Record<ActionDraftType, RuleRoleMechanicKey>;

export function mechanicForDraftType(
  type: ActionDraftType,
): RuleRoleMechanicKey {
  return MECHANIC_BY_DRAFT_TYPE[type];
}
