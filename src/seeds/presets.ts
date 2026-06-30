import type { GamePreset } from "../core/game-preset";

const SEED_TIMESTAMP = "2026-06-27T00:00:00.000Z";

const twelvePlayerSeats = [
  { seatNo: 1, roleId: "villager", characterId: "zhou_zhi" },
  { seatNo: 2, roleId: "seer", characterId: "chen_mo" },
  { seatNo: 3, roleId: "werewolf", characterId: "qin_chuan" },
  { seatNo: 4, roleId: "witch", characterId: "lin_xia" },
  { seatNo: 5, roleId: "villager", characterId: "xia_yu" },
  { seatNo: 6, roleId: "werewolf", characterId: "gu_qingyan" },
  { seatNo: 7, roleId: "guard", characterId: "shen_lan" },
  { seatNo: 8, roleId: "hunter", characterId: "xu_yan" },
  { seatNo: 9, roleId: "werewolf", characterId: "bai_qi" },
  { seatNo: 10, roleId: "villager", characterId: "tang_tang" },
  { seatNo: 11, roleId: "werewolf", characterId: "lu_zhao" },
  { seatNo: 12, roleId: "villager", characterId: "su_jin" },
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
