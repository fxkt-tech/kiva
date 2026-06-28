import type { ShowThemePack, ThemeRenderInput } from "../show-theme";

export const mansionMurderTheme: ShowThemePack = {
  id: "mansion_murder",
  name: "Mansion Murder",
  tokens: {
    background: "#050506",
    panel: "#15110f",
    text: "#f4f1ea",
    muted: "#8b8178",
    accent: "#8fd3ff",
    danger: "#b4232a",
    good: "#86efac",
    wolf: "#f87171",
  },
  render: {
    renderPhase: renderStubScene,
    renderAnnouncement: renderStubScene,
    renderSpeech: renderStubScene,
    renderVote: renderStubScene,
    renderResolution: renderStubScene,
    renderEnd: renderStubScene,
  },
};

function renderStubScene({ ctx, scene }: ThemeRenderInput): void {
  ctx.fillStyle = mansionMurderTheme.tokens.background;
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height);
  ctx.fillStyle = mansionMurderTheme.tokens.text;
  ctx.font = "700 72px sans-serif";
  ctx.fillText(scene.title, 96, 220);
}
