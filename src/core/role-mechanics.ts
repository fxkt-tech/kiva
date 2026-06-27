import type { RoleMechanicKey } from "./role-definition";

export type ActionDraftType =
  | "seer_check_selected"
  | "wolf_kill_selected"
  | "vote_cast"
  | "witch_antidote_decided"
  | "witch_poison_decided";

const MECHANIC_BY_DRAFT_TYPE = {
  seer_check_selected: "seer_check",
  wolf_kill_selected: "wolf_kill",
  vote_cast: "none",
  witch_antidote_decided: "witch_medicine",
  witch_poison_decided: "witch_medicine",
} satisfies Record<ActionDraftType, RoleMechanicKey>;

export function mechanicForDraftType(type: ActionDraftType): RoleMechanicKey {
  return MECHANIC_BY_DRAFT_TYPE[type];
}
