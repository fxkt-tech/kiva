import type { StageVisualPhase } from "./stage-palette";

export type IdentityTone = {
  readonly foreground: string;
  readonly glow: string;
};

const ROLE_IDENTITY_TONES = {
  night: {
    werewolf: { foreground: "#FF767B", glow: "rgba(255,118,123,0.30)" },
    villager: { foreground: "#D0D5D1", glow: "rgba(208,213,209,0.18)" },
    seer: { foreground: "#76E2B6", glow: "rgba(118,226,182,0.28)" },
    witch: { foreground: "#CBA4F4", glow: "rgba(203,164,244,0.28)" },
    guard: { foreground: "#80C8FB", glow: "rgba(128,200,251,0.28)" },
    hunter: { foreground: "#E5BD6E", glow: "rgba(229,189,110,0.26)" },
    unknown: { foreground: "#D0D5D1", glow: "rgba(208,213,209,0.18)" },
  },
  day: {
    werewolf: { foreground: "#FF696E", glow: "rgba(255,105,110,0.28)" },
    villager: { foreground: "#E0E1DB", glow: "rgba(224,225,219,0.18)" },
    seer: { foreground: "#59DFA4", glow: "rgba(89,223,164,0.26)" },
    witch: { foreground: "#CF93F5", glow: "rgba(207,147,245,0.27)" },
    guard: { foreground: "#66C1FA", glow: "rgba(102,193,250,0.27)" },
    hunter: { foreground: "#E8B44C", glow: "rgba(232,180,76,0.25)" },
    unknown: { foreground: "#E0E1DB", glow: "rgba(224,225,219,0.18)" },
  },
} as const satisfies Readonly<Record<StageVisualPhase, Record<string, IdentityTone>>>;

export function roleIdentityTone(
  roleName: string,
  phase: StageVisualPhase = "night",
): IdentityTone {
  const tones = ROLE_IDENTITY_TONES[phase];
  if (roleName.includes("狼")) {
    return tones.werewolf;
  }
  if (roleName.includes("预言")) {
    return tones.seer;
  }
  if (roleName.includes("女巫")) {
    return tones.witch;
  }
  if (roleName.includes("守卫")) {
    return tones.guard;
  }
  if (roleName.includes("猎人")) {
    return tones.hunter;
  }
  if (roleName.includes("平民") || roleName.includes("村民")) {
    return tones.villager;
  }
  return tones.unknown;
}

export function presenterIdentityTone(phase: StageVisualPhase): IdentityTone {
  return phase === "night"
    ? { foreground: "#F0D084", glow: "rgba(240,208,132,0.34)" }
    : { foreground: "#F4C45C", glow: "rgba(244,196,92,0.26)" };
}

export function identityTextStyle(tone: IdentityTone) {
  return {
    color: tone.foreground,
    textShadow: `0 0 18px ${tone.glow}, 0 1px 0 rgba(0,0,0,0.72)`,
  } as const;
}
