import type { GamePreset } from "../core/game-preset";

const SEED_TIMESTAMP = "2026-06-27T00:00:00.000Z";

const twelvePlayerSeats = [
  { seatNo: 1, roleId: "villager", characterId: "zhou_xu" },
  { seatNo: 2, roleId: "seer", characterId: "qiao_ke" },
  { seatNo: 3, roleId: "werewolf", characterId: "qin_chuan" },
  { seatNo: 4, roleId: "witch", characterId: "xia_mi" },
  { seatNo: 5, roleId: "villager", characterId: "ren_ye" },
  { seatNo: 6, roleId: "werewolf", characterId: "gu_ling" },
  { seatNo: 7, roleId: "guard", characterId: "cheng_wu" },
  { seatNo: 8, roleId: "hunter", characterId: "ye_chen" },
  { seatNo: 9, roleId: "werewolf", characterId: "chi_mu" },
  { seatNo: 10, roleId: "villager", characterId: "tang_li" },
  { seatNo: 11, roleId: "werewolf", characterId: "lu_ran" },
  { seatNo: 12, roleId: "villager", characterId: "su_xian" },
] as const;

export const seedPresets: readonly GamePreset[] = [
  {
    id: "twelve_player_standard",
    name: "12人狼人杀标准局",
    rulesetId: "classic_twelve",
    playerCount: 12,
    roleIds: twelvePlayerSeats.map((seat) => seat.roleId),
    characterIds: twelvePlayerSeats.map((seat) => seat.characterId),
    seatAssignments: twelvePlayerSeats.map((seat) => ({
      ...seat,
      modelBindingOverride: null,
    })),
    enabled: true,
    createdAt: SEED_TIMESTAMP,
    updatedAt: SEED_TIMESTAMP,
  },
];
