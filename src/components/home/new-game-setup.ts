import type { CharacterDefinition } from "@/core/character-definition";
import type { GamePresetSeatAssignment } from "@/core/game-preset";
import type { RoleDefinition } from "@/core/role-definition";

export const requiredTwelvePlayerRoleCounts = {
  werewolf: 4,
  seer: 1,
  witch: 1,
  hunter: 1,
  guard: 1,
  villager: 4,
} as const;

const requiredRoleLabels: Readonly<Record<string, string>> = {
  werewolf: "狼人",
  seer: "预言家",
  witch: "女巫",
  hunter: "猎人",
  guard: "守卫",
  villager: "平民",
};

export type RandomSeatSetupInput = {
  readonly roles: readonly RoleDefinition[];
  readonly characters: readonly CharacterDefinition[];
  readonly random?: () => number;
};

export function createRandomSeatSetup({
  roles,
  characters,
  random = Math.random,
}: RandomSeatSetupInput): readonly GamePresetSeatAssignment[] {
  const enabledRoleIds = new Set(roles.filter((role) => role.enabled).map((role) => role.id));
  const enabledCharacters = characters.filter((character) => character.enabled);
  const rolePool = Object.entries(requiredTwelvePlayerRoleCounts).flatMap(
    ([roleId, count]) =>
      enabledRoleIds.has(roleId)
        ? Array.from({ length: count }, () => roleId)
        : [],
  );
  const characterPool = shuffle(enabledCharacters, random).slice(0, 12);

  return shuffle(rolePool, random).map((roleId, index) => ({
    seatNo: index + 1,
    roleId,
    characterId: characterPool[index]?.id ?? "",
    modelBindingOverride: null,
  }));
}

export function validateSeatSetup(
  seats: readonly GamePresetSeatAssignment[],
  roles: readonly RoleDefinition[],
  characters: readonly CharacterDefinition[],
): readonly string[] {
  const messages = [
    ...roleCountMessages(seats),
    ...seatReferenceMessages(seats, roles, characters),
  ];

  return [...new Set(messages)];
}

export function roleCountMessages(
  seats: readonly Pick<GamePresetSeatAssignment, "roleId">[],
): readonly string[] {
  const messages: string[] = [];

  for (const [roleId, requiredCount] of Object.entries(requiredTwelvePlayerRoleCounts)) {
    const actualCount = seats.filter((seat) => seat.roleId === roleId).length;
    if (actualCount !== requiredCount) {
      messages.push(
        `${requiredRoleLabels[roleId] ?? roleId}需要 ${requiredCount} 个，当前 ${actualCount} 个。`,
      );
    }
  }

  return messages;
}

function seatReferenceMessages(
  seats: readonly GamePresetSeatAssignment[],
  roles: readonly RoleDefinition[],
  characters: readonly CharacterDefinition[],
): readonly string[] {
  const enabledRoleIds = new Set(roles.filter((role) => role.enabled).map((role) => role.id));
  const enabledCharacterIds = new Set(
    characters.filter((character) => character.enabled).map((character) => character.id),
  );
  const characterIds = seats.map((seat) => seat.characterId).filter(Boolean);
  const messages: string[] = [];

  if (seats.length !== 12) {
    messages.push(`需要 12 个座位，当前 ${seats.length} 个。`);
  }

  for (const seat of seats) {
    if (!enabledRoleIds.has(seat.roleId)) {
      messages.push(`Seat ${seat.seatNo} 选择了不可用职业：${seat.roleId || "空"}`);
    }

    if (!enabledCharacterIds.has(seat.characterId)) {
      messages.push(`Seat ${seat.seatNo} 选择了不可用玩家角色：${seat.characterId || "空"}`);
    }
  }

  if (new Set(characterIds).size !== characterIds.length) {
    messages.push("玩家角色不能重复。");
  }

  return messages;
}

function shuffle<T>(items: readonly T[], random: () => number): T[] {
  const shuffled = [...items];

  for (let index = shuffled.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(random() * (index + 1));
    [shuffled[index], shuffled[swapIndex]] = [shuffled[swapIndex]!, shuffled[index]!];
  }

  return shuffled;
}
