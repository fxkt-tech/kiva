const ROLE_IDENTITY_COLORS: Readonly<Record<string, string>> = {
  werewolf: "var(--role-werewolf, #b4233e)",
  seer: "var(--role-seer, #0f766e)",
  witch: "var(--role-witch, #7e22ce)",
  hunter: "var(--role-hunter, #8a5300)",
  guard: "var(--role-guard, #0369a1)",
  villager: "var(--role-villager, #52525b)",
};

export function roleIdentityColor(roleId: string): string {
  return ROLE_IDENTITY_COLORS[roleId] ?? "var(--foreground, #171621)";
}
