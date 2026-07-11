export type StageVisualPhase = "day" | "night";

export type StagePalette = {
  readonly majorSurface: string;
  readonly majorBorder: string;
  readonly majorShadow: string;
  readonly cardSurface: string;
  readonly cardBorder: string;
  readonly cardShadow: string;
  readonly text: string;
  readonly number: string;
  readonly accent: string;
  readonly rule: string;
  readonly ruleSoft: string;
  readonly edge: string;
  readonly avatarBorder: string;
  readonly activeCard: string;
  readonly activeBorder: string;
  readonly active: string;
  readonly activeField: string;
  readonly activeShadow: string;
  readonly activeEdgeShadow: string;
  readonly accentGlow: string;
  readonly edgeShadow: string;
  readonly titleShadow: string;
  readonly textShadow: string;
  readonly numberShadow: string;
};

const NIGHT_PALETTE: StagePalette = {
  majorSurface: "rgba(4,7,13,0.70)",
  majorBorder: "rgba(207,189,143,0.38)",
  majorShadow: "0 18px 52px rgba(0,0,0,0.34)",
  cardSurface: "rgba(4,8,14,0.58)",
  cardBorder: "rgba(207,189,143,0.28)",
  cardShadow: "0 8px 24px rgba(0,0,0,0.16)",
  text: "#FFF7E7",
  number: "#E5C67C",
  accent: "#F0D084",
  rule: "rgba(225,197,127,0.55)",
  ruleSoft: "rgba(225,197,127,0.25)",
  edge: "#C09E5D",
  avatarBorder: "#C1A86F",
  activeCard: "rgba(32,25,15,0.78)",
  activeBorder: "#F0CE79",
  active: "#F4CF73",
  activeField: "rgba(242,199,103,0.25)",
  activeShadow: "0 0 46px rgba(226,184,91,0.34)",
  activeEdgeShadow: "0 0 30px rgba(242,197,93,0.60)",
  accentGlow: "rgba(240,208,132,0.34)",
  edgeShadow: "0 0 18px rgba(192,158,93,0.28)",
  titleShadow: "0 3px 14px #000000",
  textShadow: "0 3px 10px #000000",
  numberShadow: "0 0 24px rgba(229,198,124,0.22)",
};

const DAY_PALETTE: StagePalette = {
  majorSurface: "rgba(16,18,15,0.76)",
  majorBorder: "rgba(234,207,143,0.48)",
  majorShadow: "0 18px 48px rgba(0,0,0,0.46)",
  cardSurface: "rgba(13,16,13,0.66)",
  cardBorder: "rgba(220,193,127,0.38)",
  cardShadow: "0 10px 28px rgba(0,0,0,0.28)",
  text: "#FFFAF0",
  number: "#F0C565",
  accent: "#F4C45C",
  rule: "rgba(232,195,112,0.62)",
  ruleSoft: "rgba(232,195,112,0.28)",
  edge: "#C99B46",
  avatarBorder: "#C09C58",
  activeCard: "rgba(51,39,17,0.84)",
  activeBorder: "#F4C764",
  active: "#F7CA62",
  activeField: "rgba(246,197,83,0.22)",
  activeShadow: "0 0 38px rgba(66,42,6,0.45)",
  activeEdgeShadow: "0 0 24px rgba(239,180,53,0.48)",
  accentGlow: "rgba(244,196,92,0.26)",
  edgeShadow: "0 0 12px rgba(100,66,16,0.30)",
  titleShadow: "0 3px 12px #000000",
  textShadow: "0 3px 9px #000000",
  numberShadow: "0 2px 10px #000000",
};

export function stagePaletteForPhase(phase: StageVisualPhase): StagePalette {
  return phase === "night" ? NIGHT_PALETTE : DAY_PALETTE;
}
