import "@pixi/layout";
import { LayoutContainer } from "@pixi/layout/components";
import {
  Application,
  Container,
  Graphics,
  Sprite,
  Text,
  Texture,
  type TextStyleOptions,
} from "pixi.js";
import { playbackIndexAtMs } from "@/core/playback";
import { createShotFrame } from "../shot-engine/director";
import type { RenderablePlayer, ShotFrame } from "../shot-engine/types";
import type {
  PreviewRenderFrameInput,
  PreviewRendererHandle,
} from "../preview-renderer";
import {
  caseBoardContentForFrame,
  type CaseBoardContent,
  type CaseBoardMetric,
  type CaseBoardParticipant,
  type CaseBoardRelation,
  type CaseBoardRow,
} from "./case-board-content";
import {
  mansionPixiTheme,
  PIXI_PREVIEW_FONT_FAMILY,
  type PixiPreviewTheme,
} from "./pixi-theme";

const EMPTY_TEXTURE = Texture.EMPTY;
const SEAT_NUMBER_FONT_SIZE = 122;
const SEAT_NUMBER_LINE_HEIGHT = 132;
const CASE_BOARD_ROW_COUNT = 12;

export async function createPixiPreviewRenderer(
  host: HTMLDivElement,
): Promise<PreviewRendererHandle> {
  await ensurePreviewFonts();

  const app = new Application();
  const theme = mansionPixiTheme;

  await app.init({
    width: theme.layout.width,
    height: theme.layout.height,
    background: theme.colors.background,
    antialias: true,
    autoDensity: false,
    resolution: 1,
  });

  app.canvas.setAttribute("aria-hidden", "true");
  app.canvas.className = "h-full w-full bg-black";
  host.replaceChildren(app.canvas);

  const scene = new PixiPreviewScene(theme);
  app.stage.addChild(scene.view);
  app.stage.layout = {
    width: theme.layout.width,
    height: theme.layout.height,
  };

  return new PixiPreviewRenderer(app, scene);
}

async function ensurePreviewFonts(): Promise<void> {
  if (typeof document === "undefined" || !("fonts" in document)) {
    return;
  }

  if (!document.getElementById("kiva-preview-fonts")) {
    const style = document.createElement("style");
    style.id = "kiva-preview-fonts";
    style.textContent = `
      @font-face {
        font-family: "Kiva Noto Sans SC";
        src: url("/kivdb-assets/preview/noto-sans-sc-900.ttf") format("truetype");
        font-style: normal;
        font-weight: 400 900;
        font-display: block;
      }
    `;
    document.head.appendChild(style);
  }

  await Promise.all([
    document.fonts.load('900 32px "Kiva Noto Sans SC"'),
    document.fonts.ready,
  ]);
}

class PixiPreviewRenderer implements PreviewRendererHandle {
  constructor(
    private readonly app: Application,
    private readonly scene: PixiPreviewScene,
  ) {}

  get canvas(): HTMLCanvasElement | null {
    return this.app.canvas instanceof HTMLCanvasElement ? this.app.canvas : null;
  }

  renderFrame(input: PreviewRenderFrameInput): void {
    const scene = input.items[playbackIndexAtMs(input.items, input.timeMs)];
    if (!scene) {
      this.scene.renderEmpty();
      this.app.renderer.render(this.app.stage);
      return;
    }

    this.scene.update(
      createShotFrame({
        gameTitle: input.gameTitle,
        scene,
        items: input.items,
        timeMs: input.timeMs,
        backgroundImages: input.backgroundImages,
        avatarImages: input.avatarImages,
      }),
    );
    this.app.renderer.render(this.app.stage);
  }

  destroy(): void {
    this.app.destroy(true, { children: true, texture: false });
  }
}

class PixiPreviewScene {
  readonly view = new Container();
  private readonly background: BackgroundLayer;
  private readonly topBar: TopBar;
  private readonly leftTrack: SeatTrack;
  private readonly rightTrack: SeatTrack;
  private readonly centerStage: CenterStage;
  private readonly subtitle: SubtitleBand;
  private readonly effects: EffectsLayer;
  private readonly mainArea: LayoutContainer;
  private readonly centerColumn: LayoutContainer;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.background = new BackgroundLayer(theme);
    this.topBar = new TopBar(theme);
    this.leftTrack = new SeatTrack(theme, "left");
    this.rightTrack = new SeatTrack(theme, "right");
    this.centerStage = new CenterStage(theme);
    this.subtitle = new SubtitleBand(theme);
    this.effects = new EffectsLayer(theme);
    this.mainArea = new LayoutContainer();
    this.centerColumn = new LayoutContainer();

    this.view.addChild(this.background.view, this.topBar.view, this.mainArea, this.effects.view);
    this.mainArea.layout = {
      position: "absolute",
      left: contentLeft(theme),
      top: mainAreaTop(theme),
      width: availableWidth(theme),
      height: mainAreaHeight(theme),
    };
    this.centerColumn.layout = {
      position: "absolute",
      left: centerColumnLeft(theme),
      top: 0,
      width: centerColumnWidth(theme),
      height: "100%",
    };
    this.mainArea.addChild(this.leftTrack.view, this.centerColumn, this.rightTrack.view);
    this.centerColumn.addChild(this.centerStage.view, this.subtitle.view);
  }

  update(frame: ShotFrame): void {
    this.background.update(frame);
    this.topBar.update(frame);
    this.leftTrack.update(frame);
    this.rightTrack.update(frame);
    this.centerStage.update(frame);
    this.subtitle.update(frame);
    this.effects.update(frame);
  }

  renderEmpty(): void {
    this.background.renderEmpty();
    this.topBar.renderEmpty();
    this.leftTrack.clear();
    this.rightTrack.clear();
    this.centerStage.renderEmpty();
    this.subtitle.renderEmpty();
    this.effects.clear();
  }
}

class BackgroundLayer {
  readonly view = new Container();
  private readonly base = new Graphics();
  private readonly image = new Sprite(EMPTY_TEXTURE);
  private readonly dim = new Graphics();
  private readonly vignette = new Graphics();

  constructor(private readonly theme: PixiPreviewTheme) {
    this.view.addChild(this.base, this.image, this.dim, this.vignette);
    this.renderBase();
  }

  update(frame: ShotFrame): void {
    const source = frame.scene.phase === "night"
      ? frame.backgroundImages.night
      : frame.backgroundImages.day;

    this.renderBase();
    if (source) {
      this.image.visible = true;
      this.image.texture = Texture.from(source);
      coverSprite(this.image, this.theme.layout.width, this.theme.layout.height);
    } else {
      this.image.visible = false;
    }

    this.drawOverlays();
  }

  renderEmpty(): void {
    this.image.visible = false;
    this.renderBase();
    this.drawOverlays();
  }

  private renderBase(): void {
    this.base
      .clear()
      .rect(0, 0, this.theme.layout.width, this.theme.layout.height)
      .fill({ color: this.theme.colors.background });
  }

  private drawOverlays(): void {
    this.dim.clear();
    this.vignette.clear();
  }
}

class TopBar {
  readonly view = new Container();
  private readonly panel = new Graphics();
  private readonly rule = new Graphics();
  private readonly title: Text;

  constructor(private readonly theme: PixiPreviewTheme) {
    const stageLeft = mainStageLeft(theme);
    const top = contentTop(theme);
    const stageWidth = mainStageWidth(theme);

    this.title = text("", theme.typography.title);

    this.title.anchor.set(0.5, 0);
    this.title.position.set(stageLeft + stageWidth / 2, top + 23);
    this.view.addChild(this.panel, this.title, this.rule);
  }

  update(frame: ShotFrame): void {
    const stageLeft = mainStageLeft(this.theme);
    const top = contentTop(this.theme);
    const stageWidth = mainStageWidth(this.theme);
    const radius = cardRadius(this.theme);

    this.title.text = previewHeaderTitle(frame.gameTitle);
    fitText(this.title, Math.round(stageWidth * 0.82), 34, 24);
    this.panel
      .clear()
      .roundRect(stageLeft, top, stageWidth, this.theme.layout.topBarHeight, radius)
      .fill({ color: this.theme.colors.black, alpha: 0.30 })
      .rect(stageLeft + radius, top + this.theme.layout.topBarHeight - 1, stageWidth - radius * 2, 1)
      .fill({ color: this.theme.colors.accent, alpha: 0.14 });
    this.rule
      .clear()
      .rect(stageLeft + radius, top + this.theme.layout.topBarHeight - 2, stageWidth - radius * 2, 1)
      .fill({ color: this.theme.colors.brass, alpha: 0.07 });
  }

  renderEmpty(): void {
    this.title.text = previewHeaderTitle("");
  }
}

class SeatTrack {
  readonly view = new LayoutContainer();
  private readonly cards = new Map<RenderablePlayer["playerId"], SeatCard>();

  constructor(
    private readonly theme: PixiPreviewTheme,
    private readonly side: "left" | "right",
  ) {
    this.view.layout = {
      position: "absolute",
      left: side === "left" ? 0 : rightColumnLeft(theme),
      top: contentTop(theme) - mainAreaTop(theme),
      width: sideColumnWidth(theme),
      height: contentHeight(theme),
      flexDirection: "column",
      justifyContent: "space-between",
    };
  }

  update(frame: ShotFrame): void {
    const players = frame.players
      .filter((player) => this.side === "left" ? player.seatNo <= 6 : player.seatNo > 6)
      .sort((left, right) => left.seatNo - right.seatNo);
    const activeKeys = new Set(players.map((player) => player.playerId));

    for (const [key, card] of this.cards) {
      if (!activeKeys.has(key)) {
        this.cards.delete(key);
        this.view.removeChild(card.view);
        card.destroy();
      }
    }

    players.forEach((player) => {
      let card = this.cards.get(player.playerId);
      if (!card) {
        card = new SeatCard(this.theme, this.side);
        this.cards.set(player.playerId, card);
        this.view.addChild(card.view);
      }

      card.update(player, frame);
    });
  }

  clear(): void {
    for (const card of this.cards.values()) {
      this.view.removeChild(card.view);
      card.destroy();
    }
    this.cards.clear();
  }
}

class SeatCard {
  readonly view = new LayoutContainer();
  private readonly panel = new Graphics();
  private readonly activeMark = new Graphics();
  private readonly deadOverlay = new Graphics();
  private readonly deadMark = new Graphics();
  private readonly deadStamp = text("已出局", {
    fill: 0xc56a62,
    fontFamily: PIXI_PREVIEW_FONT_FAMILY,
    fontSize: 34,
    fontWeight: "900",
    letterSpacing: 0,
    stroke: { color: 0x110b0b, width: 4 },
  });
  private readonly avatar: AvatarView;
  private readonly seatBox: LayoutContainer;
  private readonly textColumn: Container;
  private readonly seatNo: Text;
  private readonly name: Text;
  private readonly role: Text;

  constructor(
    private readonly theme: PixiPreviewTheme,
    private readonly side: "left" | "right",
  ) {
    const padding = seatCardPadding(theme);
    const avatarSize = seatAvatarSize(theme);
    this.avatar = new AvatarView({
      fallbackFontSize: Math.round(avatarSize * 0.52),
      radius: theme.layout.radius - 2,
      width: avatarSize,
      height: avatarSize,
    });
    this.seatBox = new LayoutContainer();
    this.textColumn = new Container();
    this.seatNo = text("", {
      ...theme.typography.meta,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: SEAT_NUMBER_FONT_SIZE,
      fontWeight: "900",
      lineHeight: SEAT_NUMBER_LINE_HEIGHT,
    });
    this.name = text("", theme.typography.playerName);
    this.role = text("", theme.typography.body);
    this.activeMark.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.deadOverlay.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.deadMark.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.panel.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };

    this.view.layout = {
      width: "100%",
      height: theme.layout.seatCardHeight,
      flexDirection: "row",
      alignItems: "center",
      gap: seatCardGap(),
      padding,
    };
    this.avatar.view.layout = { width: avatarSize, height: avatarSize };
    this.seatBox.layout = {
      width: seatNumberWidth(theme),
      height: avatarSize,
      alignItems: "center",
      justifyContent: "center",
    };
    this.textColumn.layout = {
      flex: 1,
      height: avatarSize,
    };
    this.seatNo.anchor.set(0.5);
    this.deadStamp.anchor.set(0.5);
    this.deadStamp.rotation = -0.14;
    this.textColumn.addChild(this.name, this.role);
    if (side === "left") {
      this.view.addChild(this.panel, this.activeMark, this.seatBox, this.textColumn, this.avatar.view, this.seatNo, this.deadOverlay, this.deadMark, this.deadStamp);
    } else {
      this.view.addChild(this.panel, this.activeMark, this.avatar.view, this.textColumn, this.seatBox, this.seatNo, this.deadOverlay, this.deadMark, this.deadStamp);
    }
  }

  update(player: RenderablePlayer, frame: ShotFrame): void {
    const dead = player.status === "dead";
    const active = player.emphasis === "active";
    const highlighted = active || player.emphasis === "highlighted";
    const avatarImage = player.avatar ? frame.avatarImages[player.avatar] : null;
    const padding = seatCardPadding(this.theme);
    const avatarSize = seatAvatarSize(this.theme);

    this.view.layout = {
      ...this.view.layout?.style,
      width: "100%",
      height: this.theme.layout.seatCardHeight,
    };
    this.view.alpha = 1;
    const panelFill = seatCardPanelFill(frame.scene.phase, dead, highlighted);
    const panelStroke = seatCardPanelStroke(frame.scene.phase, dead, highlighted, this.theme);
    this.panel
      .clear()
      .roundRect(0, 0, seatCardWidth(this.theme), this.theme.layout.seatCardHeight, cardRadius(this.theme))
      .fill({
        color: panelFill.color,
        alpha: panelFill.alpha,
      })
      .roundRect(0, 0, seatCardWidth(this.theme), this.theme.layout.seatCardHeight, cardRadius(this.theme))
      .stroke({
        color: panelStroke.color,
        alpha: panelStroke.alpha,
        width: dead || highlighted ? 2 : 1,
      });
    this.activeMark.clear();
    if (active && !dead) {
      const markSize = Math.round(avatarSize * 0.30);
      this.activeMark
        .rect(padding, padding, markSize, 2)
        .fill({ color: this.theme.colors.brass, alpha: 0.70 })
        .rect(padding, padding, 2, markSize)
        .fill({ color: this.theme.colors.brass, alpha: 0.70 });
    }
    this.avatar.update(player.name, avatarImage, dead, this.theme);
    this.seatNo.text = String(player.seatNo).padStart(2, "0");
    this.name.text = player.name;
    this.role.text = player.roleName;
    this.avatar.view.layout = { width: avatarSize, height: avatarSize };
    this.seatBox.layout = {
      width: seatNumberWidth(this.theme),
      height: avatarSize,
      alignItems: "center",
      justifyContent: "center",
    };
    this.textColumn.layout = {
      ...this.textColumn.layout?.style,
      height: avatarSize,
    };
    this.role.style = {
      ...this.theme.typography.body,
      fontSize: 28,
      lineHeight: 36,
      align: "center",
      fill: dead ? 0x8e8980 : subtleRoleColor(player.roleName, this.theme),
    };
    this.name.style = {
      ...this.theme.typography.playerName,
      fontSize: active ? 46 : 44,
      lineHeight: active ? 56 : 54,
      align: "center",
      fill: dead ? 0xc6bdb0 : this.theme.typography.playerName.fill,
    };
    this.seatNo.style = {
      ...this.theme.typography.meta,
      fill: dead ? this.theme.colors.danger : highlighted ? this.theme.colors.brass : this.theme.colors.muted,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: SEAT_NUMBER_FONT_SIZE,
      fontWeight: "900",
      lineHeight: SEAT_NUMBER_LINE_HEIGHT,
      align: "center",
    };
    fitText(this.name, textColumnWidth(this.theme), active ? 46 : 44, 26);
    this.seatNo.position.set(
      seatNumberCenterX(this.theme, this.side),
      padding + avatarSize / 2,
    );
    this.name.anchor.set(0.5);
    this.role.anchor.set(0.5);
    this.name.position.set(
      textColumnWidth(this.theme) / 2,
      avatarSize / 2 - Math.round(avatarSize * 0.21),
    );
    this.role.position.set(
      textColumnWidth(this.theme) / 2,
      avatarSize / 2 + Math.round(avatarSize * 0.28),
    );
    this.deadOverlay
      .clear()
      .roundRect(0, 0, seatCardWidth(this.theme), this.theme.layout.seatCardHeight, cardRadius(this.theme))
      .fill({ color: this.theme.colors.black, alpha: dead ? 0.20 : 0 });
    this.deadMark.clear();
    this.deadStamp.visible = dead;
    if (dead) {
      const cardWidth = seatCardWidth(this.theme);
      const cardHeight = this.theme.layout.seatCardHeight;
      const stampWidth = 118;
      const stampHeight = 38;
      const stampX = this.side === "left" ? cardWidth - padding - stampWidth : padding;
      const stampY = padding;
      const stampCenterX = stampX + stampWidth / 2;
      const stampCenterY = stampY + stampHeight / 2;

      this.deadMark
        .moveTo(0, cardHeight - 18)
        .lineTo(cardWidth, 18)
        .stroke({ color: this.theme.colors.danger, alpha: 0.24, width: 3 })
        .moveTo(0, cardHeight - 9)
        .lineTo(cardWidth, 27)
        .stroke({ color: 0x1a0d0d, alpha: 0.60, width: 7 })
        .save()
        .translateTransform(stampCenterX, stampCenterY)
        .rotateTransform(this.deadStamp.rotation)
        .roundRect(-stampWidth / 2, -stampHeight / 2, stampWidth, stampHeight, 8)
        .fill({ color: 0x170b0b, alpha: 0.76 })
        .roundRect(-stampWidth / 2, -stampHeight / 2, stampWidth, stampHeight, 8)
        .stroke({ color: this.theme.colors.danger, alpha: 0.58, width: 2 })
        .restore();
      this.deadStamp.position.set(stampCenterX, stampCenterY + 1);
    }
  }

  destroy(): void {
    this.view.destroy({ children: true });
  }
}

class CenterStage {
  readonly view = new LayoutContainer();
  private readonly speechStage: SpeechStage;
  private readonly caseBoardStage: CaseBoardStage;

  constructor(theme: PixiPreviewTheme) {
    this.speechStage = new SpeechStage(theme);
    this.caseBoardStage = new CaseBoardStage(theme);
    this.view.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: mainStageWidth(theme),
      height: mainStageHeight(theme),
    };
    this.view.addChild(this.speechStage.view, this.caseBoardStage.view);
  }

  update(frame: ShotFrame): void {
    const isSpeech = frame.scene.kind === "speech";
    this.speechStage.view.visible = isSpeech;
    this.caseBoardStage.view.visible = !isSpeech;

    if (isSpeech) {
      this.speechStage.update(frame);
    } else {
      this.caseBoardStage.update(frame);
    }
  }

  renderEmpty(): void {
    this.speechStage.view.visible = false;
    this.caseBoardStage.view.visible = true;
    this.caseBoardStage.renderEmpty();
  }
}

class SpeechStage {
  readonly view = new LayoutContainer();
  private readonly shell: LayoutContainer;
  private readonly panel = new Graphics();
  private readonly slateRule = new Graphics();
  private readonly portrait: AvatarView;
  private readonly slateTitle: Text;
  private readonly slateMeta: Text;
  private readonly focusOverlay = new Graphics();

  constructor(private readonly theme: PixiPreviewTheme) {
    const width = mainStageWidth(theme);
    const height = mainStageHeight(theme);

    this.shell = new LayoutContainer();
    this.portrait = new AvatarView({
      fallbackFontSize: 300,
      radius: 32,
      width,
      height,
    });
    this.slateTitle = text("", {
      ...theme.typography.title,
      fontSize: 38,
      fill: theme.colors.text,
    });
    this.slateMeta = text("INTERROGATION FILE", {
      ...theme.typography.meta,
      fill: theme.colors.brass,
      fontSize: 18,
    });
    this.slateMeta.anchor.set(0.5);
    this.slateTitle.anchor.set(0.5);
    this.slateMeta.position.set(width / 2, height / 2 - 34);
    this.slateTitle.position.set(width / 2, height / 2 + 10);
    this.focusOverlay.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };

    this.view.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width,
      height,
      alignItems: "stretch",
    };
    this.shell.layout = {
      width: "100%",
      height: "100%",
      overflow: "hidden",
      borderRadius: cardRadius(theme),
    };
    this.panel.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.portrait.view.layout = { width: "100%", height: "100%" };
    this.shell.addChild(
      this.panel,
      this.portrait.view,
      this.slateRule,
      this.slateMeta,
      this.slateTitle,
      this.focusOverlay,
    );
    this.view.addChild(this.shell);
  }

  update(frame: ShotFrame): void {
    const player = frame.activePlayer ?? frame.highlightedPlayers[0] ?? null;
    const avatarImage = player?.avatar ? frame.avatarImages[player.avatar] : null;
    this.portrait.view.visible = Boolean(player);
    this.slateTitle.visible = !player;
    this.slateMeta.visible = !player;
    this.slateRule.visible = !player;
    this.slateTitle.text = frame.scene.title;
    if (player) {
      this.portrait.update(player.name, avatarImage, player.status === "dead", this.theme);
    }
    this.panel
      .clear()
      .roundRect(0, 0, mainStageWidth(this.theme), mainStageHeight(this.theme), cardRadius(this.theme))
      .fill({ color: this.theme.colors.black, alpha: player ? 0.30 : 0.14 })
      .roundRect(0, 0, mainStageWidth(this.theme), mainStageHeight(this.theme), cardRadius(this.theme))
      .stroke({
        color: player ? this.theme.colors.accent : this.theme.colors.brass,
        alpha: player ? 0.30 : 0.20,
        width: 1,
      });
    this.slateRule
      .clear()
      .rect(mainStageWidth(this.theme) / 2 - 140, mainStageHeight(this.theme) / 2 - 8, 280, 1)
      .fill({ color: this.theme.colors.brass, alpha: 0.34 });
    this.focusOverlay
      .clear()
      .rect(0, mainStageHeight(this.theme) - 120, mainStageWidth(this.theme), 120)
      .fill({ color: this.theme.colors.black, alpha: player ? 0.20 : 0 });
    this.shell.alpha = 1;
    this.shell.scale.set(1);
  }

  renderEmpty(): void {
    this.portrait.view.visible = false;
    this.slateMeta.visible = true;
    this.slateTitle.visible = true;
    this.slateRule.visible = true;
    this.slateTitle.text = "等待审讯记录";
    this.focusOverlay.clear();
  }
}

class CaseBoardStage {
  readonly view = new LayoutContainer();
  private readonly panel = new Graphics();
  private readonly accent = new Graphics();
  private readonly heroPanel = new Graphics();
  private readonly kindLabel: Text;
  private readonly title: Text;
  private readonly hero: Text;
  private readonly body: Text;
  private readonly relation: CaseBoardRelationView;
  private readonly metrics: CaseBoardMetricView[];
  private readonly rows: CaseBoardRowView[];

  constructor(private readonly theme: PixiPreviewTheme) {
    this.kindLabel = text("", {
      ...theme.typography.meta,
      fill: theme.colors.brass,
      fontSize: 22,
      letterSpacing: 0,
    });
    this.kindLabel.anchor.set(0, 0.5);
    this.title = text("", {
      ...theme.typography.title,
      fill: theme.colors.text,
      fontSize: 48,
      lineHeight: 58,
      wordWrap: true,
      wordWrapWidth: mainStageWidth(theme) - 96,
    });
    this.title.anchor.set(0, 0);
    this.hero = text("", {
      ...theme.typography.title,
      fill: theme.colors.text,
      fontSize: 78,
      lineHeight: 90,
      wordWrap: true,
      wordWrapWidth: mainStageWidth(theme) - 150,
    });
    this.hero.anchor.set(0.5, 0.5);
    this.body = text("", {
      ...theme.typography.body,
      fill: theme.colors.muted,
      fontSize: 28,
      lineHeight: 38,
      wordWrap: true,
      wordWrapWidth: mainStageWidth(theme) - 150,
    });
    this.body.anchor.set(0.5, 0);
    this.relation = new CaseBoardRelationView(theme);
    this.metrics = Array.from({ length: 3 }, () => new CaseBoardMetricView(theme));
    this.rows = Array.from({ length: CASE_BOARD_ROW_COUNT }, () => (
      new CaseBoardRowView(theme)
    ));

    this.view.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: mainStageWidth(theme),
      height: mainStageHeight(theme),
    };
    this.panel.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.accent.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.view.addChild(
      this.panel,
      this.accent,
      this.heroPanel,
      this.kindLabel,
      this.title,
      this.hero,
      this.body,
      this.relation.view,
      ...this.metrics.map((metric) => metric.view),
      ...this.rows.map((row) => row.view),
    );
  }

  update(frame: ShotFrame): void {
    const content = caseBoardContentForFrame(frame);
    const width = mainStageWidth(this.theme);
    const height = mainStageHeight(this.theme);
    const accentColor = colorForStageTone(content.tone, this.theme);
    const paddingX = 54;
    const heroY = content.variant === "vote" ? 184 : 238;
    const heroPanelHeight = heroPanelHeightForContent(content.variant);
    const metricHeight = metricHeightForContent(content);
    const metricTop = heroY + heroPanelHeight + 18;
    const relationHeight = content.relation ? 116 : 0;
    const relationTop = heroY + heroPanelHeight + 20;
    const rowTop = content.variant === "vote"
      ? 258
      : relationHeight > 0
        ? relationTop + relationHeight + 20
        : metricHeight > 0
        ? metricTop + metricHeight + 18
        : heroY + heroPanelHeight + 24;

    this.renderShell(width, height, accentColor, frame.clock.progress);
    this.renderHeroPanel(width, heroY, heroPanelHeight, accentColor);

    this.kindLabel.text = content.kindLabel;
    this.title.text = content.title;
    this.hero.text = content.hero;
    this.body.text = content.body;

    this.kindLabel.position.set(paddingX, 62);
    this.kindLabel.style = {
      ...this.kindLabel.style,
      fill: accentColor,
    };
    this.title.style = {
      ...this.title.style,
      fill: this.theme.colors.text,
      fontSize: 48,
      lineHeight: 58,
      wordWrapWidth: width - paddingX * 2,
    };
    this.title.position.set(paddingX, 92);
    fitText(this.title, width - paddingX * 2, 48, 32);

    this.hero.style = {
      ...this.hero.style,
      fill: content.variant === "ending" ? accentColor : this.theme.colors.text,
      fontSize: heroSizeForContent(content.variant),
      lineHeight: heroSizeForContent(content.variant) + 12,
      wordWrapWidth: width - paddingX * 2.5,
      align: "center",
    };
    this.hero.position.set(width / 2, heroY + 72);
    fitTextToBox(
      this.hero,
      width - paddingX * 2.5,
      118,
      heroSizeForContent(content.variant),
      34,
    );

    this.body.visible = content.body.length > 0;
    this.body.style = {
      ...this.body.style,
      wordWrapWidth: width - paddingX * 2.5,
      align: "center",
    };
    this.body.position.set(width / 2, heroY + 138);
    fitTextToBox(this.body, width - paddingX * 2.5, 48, 28, 18);

    this.renderMetrics(
      content.metrics,
      paddingX,
      metricTop,
      width - paddingX * 2,
      metricHeight,
    );
    this.relation.update(
      content.relation,
      paddingX,
      relationTop,
      width - paddingX * 2,
      relationHeight,
      accentColor,
    );

    this.renderRows(
      content.rows,
      rowTop,
      width - paddingX * 2,
      height - rowTop - 52,
    );
  }

  renderEmpty(): void {
    const width = mainStageWidth(this.theme);
    const height = mainStageHeight(this.theme);
    const accentColor = this.theme.colors.accent;
    const paddingX = 54;

    this.renderShell(width, height, accentColor, 0);
    this.renderHeroPanel(width, 238, heroPanelHeightForContent("phase"), accentColor);

    this.kindLabel.text = "预览";
    this.title.text = "等待审讯记录";
    this.hero.text = "暂无可播放场景";
    this.body.text = "";
    this.kindLabel.position.set(paddingX, 62);
    this.title.position.set(paddingX, 92);
    this.hero.position.set(width / 2, 310);
    this.body.visible = false;
    this.relation.update(null, 0, 0, 0, 0, accentColor);
    this.title.style = {
      ...this.title.style,
      fontSize: 48,
      lineHeight: 58,
      wordWrapWidth: width - paddingX * 2,
    };
    this.hero.style = {
      ...this.hero.style,
      fontSize: 62,
      lineHeight: 74,
      wordWrapWidth: width - paddingX * 2.5,
    };
    this.renderMetrics([], paddingX, 432, width - paddingX * 2, 0);

    this.renderRows(
      [{ tone: "neutral", label: "--", text: "等待下一段对局记录" }],
      rowTopForContent("phase"),
      width - paddingX * 2,
      height - rowTopForContent("phase") - 54,
    );
  }

  private renderShell(
    width: number,
    height: number,
    accentColor: number,
    progress: number,
  ): void {
    const radius = cardRadius(this.theme);
    const pulse = 0.18 + Math.sin(progress * Math.PI) * 0.12;
    this.panel
      .clear()
      .roundRect(0, 0, width, height, radius)
      .fill({ color: this.theme.colors.black, alpha: 0.60 })
      .roundRect(0, 0, width, height, radius)
      .stroke({ color: accentColor, alpha: 0.18 + pulse, width: 2 })
      .rect(22, 22, width - 44, height - 44)
      .stroke({ color: 0xd4c7ad, alpha: 0.08, width: 1 });
    this.accent
      .clear()
      .rect(54, 156, width - 108, 2)
      .fill({ color: accentColor, alpha: 0.34 })
      .rect(54, height - 98, width - 108, 1)
      .fill({ color: 0xd4c7ad, alpha: 0.12 });
  }

  private renderHeroPanel(
    width: number,
    y: number,
    panelHeight: number,
    accentColor: number,
  ): void {
    const panelWidth = width - 108;
    this.heroPanel
      .clear()
      .roundRect(54, y, panelWidth, panelHeight, cardRadius(this.theme))
      .fill({ color: this.theme.colors.black, alpha: 0.44 })
      .roundRect(54, y, panelWidth, panelHeight, cardRadius(this.theme))
      .stroke({ color: accentColor, alpha: 0.22, width: 1 })
      .rect(54, y, 5, panelHeight)
      .fill({ color: accentColor, alpha: 0.68 });
  }

  private renderMetrics(
    metrics: readonly CaseBoardMetric[],
    x: number,
    y: number,
    width: number,
    height: number,
  ): void {
    const visibleMetrics = metrics.slice(0, this.metrics.length);
    const gap = 14;
    const metricWidth = visibleMetrics.length > 0
      ? Math.floor((width - gap * (visibleMetrics.length - 1)) / visibleMetrics.length)
      : 0;
    this.metrics.forEach((metricView, index) => {
      const metric = visibleMetrics[index] ?? null;
      metricView.update(
        metric,
        x + index * (metricWidth + gap),
        y,
        metricWidth,
        height,
      );
    });
  }

  private renderRows(
    rows: readonly CaseBoardRow[],
    y: number,
    width: number,
    availableHeight: number,
  ): void {
    const visibleRows = rows.slice(0, CASE_BOARD_ROW_COUNT);
    const gap = 12;
    const rowCount = Math.max(1, visibleRows.length);
    const gapsHeight = gap * Math.max(0, visibleRows.length - 1);
    const rowHeight = Math.max(
      54,
      Math.min(88, Math.floor((availableHeight - gapsHeight) / rowCount)),
    );
    const x = (mainStageWidth(this.theme) - width) / 2;

    this.rows.forEach((rowView, index) => {
      const row = visibleRows[index] ?? null;
      rowView.update(row, x, y + index * (rowHeight + gap), width, rowHeight);
    });
  }
}

class CaseBoardRelationView {
  readonly view = new Container();
  private readonly panel = new Graphics();
  private readonly arrow = new Graphics();
  private readonly actorSeat: Text;
  private readonly actorName: Text;
  private readonly actorRole: Text;
  private readonly action: Text;
  private readonly result: Text;
  private readonly targetSeat: Text;
  private readonly targetName: Text;
  private readonly targetRole: Text;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.actorSeat = relationSeatText(theme);
    this.actorName = relationNameText(theme);
    this.actorRole = relationRoleText(theme);
    this.action = text("", {
      ...theme.typography.title,
      fill: theme.colors.text,
      fontSize: 34,
      lineHeight: 42,
      letterSpacing: 0,
    });
    this.result = text("", {
      ...theme.typography.meta,
      fill: theme.colors.brass,
      fontSize: 18,
      lineHeight: 24,
    });
    this.targetSeat = relationSeatText(theme);
    this.targetName = relationNameText(theme);
    this.targetRole = relationRoleText(theme);
    this.action.anchor.set(0.5);
    this.result.anchor.set(0.5);
    this.view.addChild(
      this.panel,
      this.arrow,
      this.actorSeat,
      this.actorName,
      this.actorRole,
      this.action,
      this.result,
      this.targetSeat,
      this.targetName,
      this.targetRole,
    );
  }

  update(
    relation: CaseBoardRelation | null,
    x: number,
    y: number,
    width: number,
    height: number,
    accentColor: number,
  ): void {
    this.view.visible = Boolean(relation) && width > 0 && height > 0;
    if (!relation) {
      this.clearText();
      this.panel.clear();
      this.arrow.clear();
      return;
    }

    const radius = cardRadius(this.theme);
    const cardWidth = Math.floor((width - 190) / 2);
    const centerX = width / 2;
    const actorX = 0;
    const targetX = width - cardWidth;
    this.view.position.set(x, y);

    this.panel
      .clear()
      .roundRect(actorX, 0, cardWidth, height, radius)
      .fill({ color: this.theme.colors.panel, alpha: 0.66 })
      .roundRect(actorX, 0, cardWidth, height, radius)
      .stroke({ color: participantColor(relation.actor, this.theme), alpha: 0.28, width: 1 })
      .rect(actorX, radius, 4, height - radius * 2)
      .fill({ color: participantColor(relation.actor, this.theme), alpha: 0.62 })
      .roundRect(targetX, 0, cardWidth, height, radius)
      .fill({ color: this.theme.colors.panel, alpha: 0.66 })
      .roundRect(targetX, 0, cardWidth, height, radius)
      .stroke({ color: participantColor(relation.target, this.theme), alpha: 0.28, width: 1 })
      .rect(targetX, radius, 4, height - radius * 2)
      .fill({ color: participantColor(relation.target, this.theme), alpha: 0.62 });

    this.arrow
      .clear()
      .moveTo(actorX + cardWidth + 24, height / 2)
      .lineTo(targetX - 24, height / 2)
      .stroke({ color: accentColor, alpha: 0.82, width: 4, cap: "round" })
      .moveTo(targetX - 40, height / 2 - 12)
      .lineTo(targetX - 24, height / 2)
      .lineTo(targetX - 40, height / 2 + 12)
      .stroke({ color: accentColor, alpha: 0.82, width: 4, cap: "round", join: "round" });

    this.action.text = relation.action;
    this.result.text = relation.result ?? "";
    this.action.style = { ...this.action.style, fill: accentColor };
    this.action.position.set(centerX, height / 2 - (relation.result ? 10 : 0));
    this.result.visible = Boolean(relation.result);
    this.result.position.set(centerX, height / 2 + 28);

    this.renderParticipant(relation.actor, actorX, cardWidth, height, "行动者");
    this.renderParticipant(relation.target, targetX, cardWidth, height, "目标");
  }

  private renderParticipant(
    participant: CaseBoardParticipant | null,
    x: number,
    width: number,
    height: number,
    fallback: string,
  ): void {
    const isActor = x === 0;
    const seat = isActor ? this.actorSeat : this.targetSeat;
    const name = isActor ? this.actorName : this.targetName;
    const role = isActor ? this.actorRole : this.targetRole;
    const color = participantColor(participant, this.theme);

    seat.text = participant?.label ?? "--";
    name.text = participant?.name ?? fallback;
    role.text = participant?.roleName ?? "未指定";
    seat.style = { ...seat.style, fill: color };
    name.style = { ...name.style, fill: participant ? this.theme.colors.text : this.theme.colors.muted };
    role.style = { ...role.style, fill: participant ? this.theme.colors.muted : this.theme.colors.accent };
    seat.position.set(x + 24, 18);
    name.position.set(x + 86, 22);
    role.position.set(x + 86, 68);
    fitTextToBox(name, width - 116, 40, 34, 22);
    fitTextToBox(role, width - 116, 26, 18, 14);
    seat.position.set(x + 24, height / 2 - 26);
  }

  private clearText(): void {
    this.actorSeat.text = "";
    this.actorName.text = "";
    this.actorRole.text = "";
    this.action.text = "";
    this.result.text = "";
    this.targetSeat.text = "";
    this.targetName.text = "";
    this.targetRole.text = "";
  }
}

class CaseBoardMetricView {
  readonly view = new Container();
  private readonly panel = new Graphics();
  private readonly label: Text;
  private readonly value: Text;
  private readonly caption: Text;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.label = text("", {
      ...theme.typography.meta,
      fill: theme.colors.muted,
      fontSize: 18,
      lineHeight: 24,
    });
    this.value = text("", {
      ...theme.typography.title,
      fill: theme.colors.text,
      fontSize: 48,
      lineHeight: 56,
    });
    this.caption = text("", {
      ...theme.typography.meta,
      fill: theme.colors.muted,
      fontSize: 16,
      lineHeight: 22,
    });
    this.value.anchor.set(1, 0.5);
    this.caption.anchor.set(1, 0.5);
    this.view.addChild(this.panel, this.label, this.value, this.caption);
  }

  update(
    metric: CaseBoardMetric | null,
    x: number,
    y: number,
    width: number,
    height: number,
  ): void {
    this.view.visible = Boolean(metric) && width > 0 && height > 0;
    if (!metric) {
      this.panel.clear();
      this.label.text = "";
      this.value.text = "";
      this.caption.text = "";
      return;
    }

    const toneColor = colorForCaseBoardTone(metric.tone, this.theme);
    this.view.position.set(x, y);
    this.panel
      .clear()
      .roundRect(0, 0, width, height, cardRadius(this.theme))
      .fill({ color: this.theme.colors.panel, alpha: 0.58 })
      .roundRect(0, 0, width, height, cardRadius(this.theme))
      .stroke({ color: toneColor, alpha: 0.24, width: 1 });

    this.label.text = metric.label;
    this.value.text = metric.value;
    this.caption.text = metric.caption ?? "";
    this.label.style = { ...this.label.style, fill: toneColor };
    this.value.style = { ...this.value.style, fill: toneColor };
    this.label.position.set(22, 16);
    this.value.position.set(width - 22, height / 2 + 4);
    this.caption.position.set(width - 24, height - 18);
  }
}

class CaseBoardRowView {
  readonly view = new Container();
  private readonly panel = new Graphics();
  private readonly voteBar = new Graphics();
  private readonly label: Text;
  private readonly body: Text;
  private readonly meta: Text;
  private readonly count: Text;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.label = text("", {
      ...theme.typography.meta,
      fill: theme.colors.brass,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: 26,
      fontWeight: "900",
      lineHeight: 34,
    });
    this.body = text("", {
      ...theme.typography.body,
      fill: theme.colors.text,
      fontSize: 25,
      lineHeight: 34,
      wordWrap: true,
    });
    this.meta = text("", {
      ...theme.typography.meta,
      fill: theme.colors.muted,
      fontSize: 17,
      lineHeight: 24,
    });
    this.count = text("", {
      ...theme.typography.meta,
      fill: theme.colors.text,
      fontSize: 42,
      fontWeight: "900",
      lineHeight: 44,
    });
    this.count.anchor.set(1, 0.5);
    this.label.anchor.set(0.5);
    this.view.addChild(this.panel, this.voteBar, this.label, this.body, this.meta, this.count);
  }

  update(
    row: CaseBoardRow | null,
    x: number,
    y: number,
    width: number,
    height: number,
  ): void {
    this.view.visible = Boolean(row);
    if (!row) {
      this.panel.clear();
      this.voteBar.clear();
      this.label.text = "";
      this.body.text = "";
      this.meta.text = "";
      this.count.text = "";
      return;
    }

    const toneColor = colorForCaseBoardTone(row.tone, this.theme);
    const radius = cardRadius(this.theme);
    const labelWidth = 92;
    const textX = labelWidth + 22;
    const countWidth = row.count ? 92 : 0;
    const textWidth = width - textX - countWidth - 22;

    this.view.position.set(x, y);
    this.panel
      .clear()
      .roundRect(0, 0, width, height, radius)
      .fill({ color: this.theme.colors.panel, alpha: 0.64 })
      .roundRect(0, 0, width, height, radius)
      .stroke({ color: toneColor, alpha: 0.26, width: 1 })
      .rect(0, radius, 4, height - radius * 2)
      .fill({ color: toneColor, alpha: 0.62 });
    this.voteBar.clear();
    if (row.count) {
      const barWidth = Math.min(width - labelWidth - 126, row.count * 76);
      this.voteBar
        .roundRect(textX, height - 13, barWidth, 5, 3)
        .fill({ color: toneColor, alpha: 0.48 });
    }

    this.label.text = row.label;
    this.body.text = row.text;
    this.meta.text = row.meta ?? "";
    this.count.text = row.count ? `${row.count}票` : "";
    this.label.style = {
      ...this.label.style,
      fill: toneColor,
    };
    this.body.style = {
      ...this.body.style,
      fill: row.tone === "dead-player" ? this.theme.colors.muted : this.theme.colors.text,
      wordWrapWidth: textWidth,
    };
    this.count.style = {
      ...this.count.style,
      fill: toneColor,
    };
    this.meta.visible = Boolean(row.meta);
    this.count.visible = Boolean(row.count);
    this.label.position.set(labelWidth / 2, height / 2);
    const bodyY = row.meta ? 12 : Math.max(12, height / 2 - 17);
    const bodyHeight = row.meta ? 32 : height - bodyY - 18;
    this.body.position.set(textX, bodyY);
    this.meta.position.set(textX, Math.min(height - 28, bodyY + 34));
    this.count.position.set(width - 22, height / 2);
    fitTextToBox(this.body, textWidth, bodyHeight, 25, 17);
  }
}

class SubtitleBand {
  readonly view = new LayoutContainer();
  private readonly panel = new Graphics();
  private readonly rule = new Graphics();
  private readonly speaker: Text;
  private readonly content: Text;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.speaker = text("", {
      ...theme.typography.meta,
      fill: theme.colors.brass,
      fontSize: 22,
    });
    this.content = text("", theme.typography.subtitle);
    this.panel.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.rule.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: 2,
    };

    this.view.layout = {
      position: "absolute",
      left: 0,
      top: subtitleTop(theme),
      width: mainStageWidth(theme),
      height: theme.layout.subtitleHeight,
      paddingLeft: 44,
      paddingRight: 44,
      paddingTop: 20,
      paddingBottom: 24,
      gap: 10,
      flexDirection: "column",
    };
    this.speaker.layout = { width: "100%" };
    this.content.layout = { width: "100%" };
    this.view.addChild(this.panel, this.rule, this.speaker, this.content);
  }

  update(frame: ShotFrame): void {
    if (frame.scene.kind !== "speech" || !frame.subtitle) {
      this.renderEmpty();
      return;
    }

    this.view.visible = true;
    const width = mainStageWidth(this.theme);
    const radius = cardRadius(this.theme);
    this.panel
      .clear()
      .roundRect(0, 0, width, this.theme.layout.subtitleHeight, radius)
      .fill({ color: this.theme.colors.black, alpha: 0.82 })
      .roundRect(0, 0, width, this.theme.layout.subtitleHeight, radius)
      .stroke({ color: 0xd4c7ad, alpha: 0.18, width: 1 });
    this.rule
      .clear()
      .rect(radius, 0, width - radius * 2, 2)
      .fill({ color: this.theme.colors.accent, alpha: 0.18 });
    this.speaker.text = frame.subtitle.speaker;
    this.content.text = frame.subtitle.lines.join("\n");
  }

  renderEmpty(): void {
    this.view.visible = false;
    this.panel.clear();
    this.rule.clear();
    this.speaker.text = "";
    this.content.text = "";
  }
}

class EffectsLayer {
  readonly view = new Container();
  private readonly rule: Graphics;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.rule = new Graphics();
    this.view.addChild(this.rule);
  }

  update(frame: ShotFrame): void {
    const stageLeft = mainStageLeft(this.theme);
    const top = contentTop(this.theme);
    const stageWidth = mainStageWidth(this.theme);
    const pulse = 0.22 + Math.sin(frame.clock.progress * Math.PI) * 0.18;
    this.rule
      .clear()
      .rect(stageLeft, top + this.theme.layout.topBarHeight - 2, stageWidth, 2)
      .fill({ color: this.theme.colors.accent, alpha: pulse * 0.36 });
  }

  clear(): void {
    this.rule.clear();
  }
}

class AvatarView {
  readonly view = new Container();
  private readonly image = new Sprite(EMPTY_TEXTURE);
  private readonly fallback = new Graphics();
  private readonly clip = new Graphics();
  private readonly label = text("", {
    fill: 0xf4f1ea,
    fontFamily: PIXI_PREVIEW_FONT_FAMILY,
    fontSize: 82,
    fontWeight: "900",
    letterSpacing: 0,
  });
  private readonly deadOverlay = new Graphics();
  private readonly fallbackFontSize: number;
  private readonly radius: number;
  private readonly width: number;
  private readonly height: number;

  constructor(options: {
    readonly fallbackFontSize: number;
    readonly radius: number;
    readonly width: number;
    readonly height: number;
  }) {
    this.fallbackFontSize = options.fallbackFontSize;
    this.radius = options.radius;
    this.width = options.width;
    this.height = options.height;
    this.view.addChild(this.fallback, this.image, this.label, this.deadOverlay);
    this.view.addChild(this.clip);
    this.view.mask = this.clip;
  }

  update(
    label: string,
    image: HTMLImageElement | null,
    dead: boolean,
    theme: PixiPreviewTheme,
  ): void {
    const width = this.width;
    const height = this.height;

    this.clip
      .clear()
      .roundRect(0, 0, width, height, this.radius)
      .fill({ color: theme.colors.white });

    this.fallback
      .clear()
      .roundRect(0, 0, width, height, this.radius)
      .fill({ color: dead ? 0x241f1f : theme.colors.panelAlt, alpha: 0.94 })
      .roundRect(0, 0, width, height, this.radius)
      .stroke({ color: 0xd4c7ad, alpha: dead ? 0.10 : 0.18, width: 1 });

    if (image) {
      this.image.visible = true;
      this.image.texture = Texture.from(image);
      coverSprite(this.image, width, height);
      this.label.visible = false;
    } else {
      this.image.visible = false;
      this.label.visible = true;
      this.label.text = Array.from(label)[0] ?? "?";
      this.label.style = {
        ...this.label.style,
        fontSize: this.fallbackFontSize,
        fill: dead ? theme.colors.muted : theme.colors.text,
      };
      this.label.anchor.set(0.5);
      this.label.position.set(width / 2, height / 2 + this.fallbackFontSize * 0.05);
    }

    this.deadOverlay
      .clear()
      .roundRect(0, 0, width, height, this.radius)
      .fill({
        color: theme.colors.black,
        alpha: dead ? 0.42 : 0,
      });
  }
}

function text(value: string, style: TextStyleOptions): Text {
  return new Text({
    text: value,
    style,
  });
}

function relationSeatText(theme: PixiPreviewTheme): Text {
  return text("", {
    ...theme.typography.title,
    fill: theme.colors.brass,
    fontSize: 44,
    lineHeight: 50,
    letterSpacing: 0,
  });
}

function relationNameText(theme: PixiPreviewTheme): Text {
  return text("", {
    ...theme.typography.title,
    fill: theme.colors.text,
    fontSize: 34,
    lineHeight: 42,
    letterSpacing: 0,
    wordWrap: true,
  });
}

function relationRoleText(theme: PixiPreviewTheme): Text {
  return text("", {
    ...theme.typography.meta,
    fill: theme.colors.muted,
    fontSize: 18,
    lineHeight: 24,
    letterSpacing: 0,
    wordWrap: true,
  });
}

function fitText(
  target: Text,
  maxWidth: number,
  maxSize: number,
  minSize: number,
): void {
  let nextSize = maxSize;
  while (nextSize > minSize) {
    target.style = {
      ...target.style,
      fontSize: nextSize,
    };
    if (target.width <= maxWidth) {
      return;
    }
    nextSize -= 1;
  }
  target.style = {
    ...target.style,
    fontSize: minSize,
  };
}

function fitTextToBox(
  target: Text,
  maxWidth: number,
  maxHeight: number,
  maxSize: number,
  minSize: number,
): void {
  let nextSize = maxSize;
  while (nextSize > minSize) {
    target.style = {
      ...target.style,
      fontSize: nextSize,
      lineHeight: Math.round(nextSize * 1.28),
      wordWrapWidth: maxWidth,
    };
    if (target.width <= maxWidth && target.height <= maxHeight) {
      return;
    }
    nextSize -= 1;
  }
  target.style = {
    ...target.style,
    fontSize: minSize,
    lineHeight: Math.round(minSize * 1.28),
    wordWrapWidth: maxWidth,
  };
}

function coverSprite(sprite: Sprite, width: number, height: number): void {
  const textureWidth = sprite.texture.width || width;
  const textureHeight = sprite.texture.height || height;
  const scale = Math.max(width / textureWidth, height / textureHeight);
  sprite.scale.set(scale);
  sprite.position.set(
    (width - textureWidth * scale) / 2,
    (height - textureHeight * scale) / 2,
  );
}

function subtleRoleColor(roleName: string, theme: PixiPreviewTheme): number {
  if (roleName.includes("狼")) {
    return 0x8f2f2f;
  }
  if (roleName.includes("预言")) {
    return 0x44e36e;
  }
  if (roleName.includes("守卫")) {
    return 0x4169e1;
  }
  if (roleName.includes("猎人")) {
    return 0xc69b3a;
  }
  if (roleName.includes("女巫")) {
    return 0x9b5de5;
  }
  if (roleName.includes("村民") || roleName.includes("平民")) {
    return 0xd8d6cf;
  }
  return theme.colors.muted;
}

function seatCardPanelFill(
  phase: ShotFrame["scene"]["phase"],
  dead: boolean,
  highlighted: boolean,
): { readonly color: number; readonly alpha: number } {
  if (isDaytimePreviewPhase(phase)) {
    return {
      color: dead ? 0x150c0b : 0x16120d,
      alpha: dead ? 0.68 : highlighted ? 0.78 : 0.66,
    };
  }

  return {
    color: dead ? 0x090707 : 0x05090d,
    alpha: dead ? 0.78 : highlighted ? 0.90 : 0.84,
  };
}

function seatCardPanelStroke(
  phase: ShotFrame["scene"]["phase"],
  dead: boolean,
  highlighted: boolean,
  theme: PixiPreviewTheme,
): { readonly color: number; readonly alpha: number } {
  if (dead) {
    return { color: theme.colors.danger, alpha: isDaytimePreviewPhase(phase) ? 0.38 : 0.34 };
  }

  if (highlighted) {
    return {
      color: isDaytimePreviewPhase(phase) ? theme.colors.brass : theme.colors.accent,
      alpha: isDaytimePreviewPhase(phase) ? 0.46 : 0.42,
    };
  }

  return {
    color: isDaytimePreviewPhase(phase) ? theme.colors.brass : 0xd4c7ad,
    alpha: isDaytimePreviewPhase(phase) ? 0.18 : 0.13,
  };
}

function isDaytimePreviewPhase(phase: ShotFrame["scene"]["phase"]): boolean {
  return phase !== "night" && phase !== "setup";
}

function colorForCaseBoardTone(
  tone: CaseBoardRow["tone"],
  theme: PixiPreviewTheme,
): number {
  switch (tone) {
    case "detail":
      return theme.colors.brass;
    case "alive-player":
      return theme.colors.good;
    case "dead-player":
      return theme.colors.muted;
    case "neutral":
      return theme.colors.accent;
    case "result":
      return theme.colors.danger;
  }
}

function participantColor(
  participant: CaseBoardParticipant | null,
  theme: PixiPreviewTheme,
): number {
  return participant
    ? colorForCaseBoardTone(participant.tone, theme)
    : theme.colors.accent;
}

function colorForStageTone(
  tone: CaseBoardContent["tone"],
  theme: PixiPreviewTheme,
): number {
  switch (tone) {
    case "day":
      return theme.colors.brass;
    case "night":
      return theme.colors.accent;
    case "danger":
      return theme.colors.danger;
    case "good":
      return theme.colors.good;
    case "neutral":
      return theme.colors.muted;
  }
}

function heroPanelHeightForContent(
  variant: CaseBoardContent["variant"],
): number {
  switch (variant) {
    case "vote":
      return 96;
    case "standard":
      return 142;
    default:
      return 176;
  }
}

function metricHeightForContent(content: CaseBoardContent): number {
  return content.metrics.length > 0 && content.variant !== "vote" ? 86 : 0;
}

function rowTopForContent(
  variant: CaseBoardContent["variant"],
): number {
  switch (variant) {
    case "phase":
      return 520;
    case "vote":
      return 258;
    case "ending":
      return 542;
    case "standard":
      return 428;
    case "night-action":
    case "death":
      return 542;
  }
}

function heroSizeForContent(
  variant: CaseBoardContent["variant"],
): number {
  switch (variant) {
    case "phase":
      return 76;
    case "ending":
      return 74;
    case "vote":
      return 54;
    case "night-action":
      return 68;
    case "death":
      return 70;
    case "standard":
      return 52;
  }
}

function previewHeaderTitle(gameTitle: string): string {
  const title = gameTitle.trim() || "凌冽审讯档案";
  return `四方诛杀(${title})`;
}

function cardRadius(theme: PixiPreviewTheme): number {
  return theme.layout.radius;
}

function mainStageWidth(theme: PixiPreviewTheme): number {
  return centerColumnWidth(theme);
}

function mainStageLeft(theme: PixiPreviewTheme): number {
  return contentLeft(theme) + centerColumnLeft(theme);
}

function mainStageHeight(theme: PixiPreviewTheme): number {
  return mainAreaHeight(theme);
}

function mainAreaTop(theme: PixiPreviewTheme): number {
  return contentTop(theme)
    + theme.layout.topBarHeight
    + theme.layout.columnGap;
}

function mainAreaHeight(theme: PixiPreviewTheme): number {
  return theme.layout.height
    - theme.layout.topBarHeight
    - theme.layout.columnGap
    - theme.layout.outerMargin * 2;
}

function availableWidth(theme: PixiPreviewTheme): number {
  return contentWidthFrame(theme);
}

function contentWidth(theme: PixiPreviewTheme): number {
  return availableWidth(theme) - theme.layout.columnGap * 2;
}

function contentLeft(theme: PixiPreviewTheme): number {
  return theme.layout.outerMargin;
}

function contentTop(theme: PixiPreviewTheme): number {
  return theme.layout.outerMargin;
}

function contentWidthFrame(theme: PixiPreviewTheme): number {
  return theme.layout.width - theme.layout.outerMargin * 2;
}

function contentHeight(theme: PixiPreviewTheme): number {
  return theme.layout.height - theme.layout.outerMargin * 2;
}

function sideColumnWidth(theme: PixiPreviewTheme): number {
  return contentWidth(theme) * theme.layout.sideColumnRatio;
}

function centerColumnWidth(theme: PixiPreviewTheme): number {
  return contentWidth(theme) * theme.layout.centerColumnRatio;
}

function centerColumnLeft(theme: PixiPreviewTheme): number {
  return sideColumnWidth(theme) + theme.layout.columnGap;
}

function rightColumnLeft(theme: PixiPreviewTheme): number {
  return centerColumnLeft(theme)
    + centerColumnWidth(theme)
    + theme.layout.columnGap;
}

function seatCardWidth(theme: PixiPreviewTheme): number {
  return sideColumnWidth(theme);
}

function seatCardPadding(theme: PixiPreviewTheme): number {
  return Math.round(theme.layout.seatCardHeight * 0.05);
}

function seatAvatarSize(theme: PixiPreviewTheme): number {
  return theme.layout.seatCardHeight - seatCardPadding(theme) * 2;
}

function seatNumberWidth(theme: PixiPreviewTheme): number {
  return seatAvatarSize(theme);
}

function seatNumberCenterX(
  theme: PixiPreviewTheme,
  side: "left" | "right",
): number {
  const padding = seatCardPadding(theme);
  const centerOffset = seatNumberWidth(theme) / 2;
  if (side === "left") {
    return padding + centerOffset;
  }

  return seatCardWidth(theme) - padding - centerOffset;
}

function textColumnWidth(theme: PixiPreviewTheme): number {
  return seatCardWidth(theme)
    - seatCardPadding(theme) * 2
    - seatAvatarSize(theme)
    - seatNumberWidth(theme)
    - seatCardGap() * 2;
}

function seatCardGap(): number {
  return 14;
}

function subtitleTop(theme: PixiPreviewTheme): number {
  return mainAreaHeight(theme) * theme.layout.subtitleTopRatio;
}
