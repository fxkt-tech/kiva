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
  type CaseBoardRow,
} from "./case-board-content";
import {
  mansionPixiTheme,
  PIXI_PREVIEW_FONT_FAMILY,
  type PixiPreviewTheme,
} from "./pixi-theme";

const EMPTY_TEXTURE = Texture.EMPTY;
const SEAT_NUMBER_FONT_SIZE = 104;
const SEAT_NUMBER_LINE_HEIGHT = 114;
const CASE_BOARD_ROW_COUNT = 6;

export async function createPixiPreviewRenderer(
  host: HTMLDivElement,
): Promise<PreviewRendererHandle> {
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
      left: theme.layout.outerMargin,
      top: theme.layout.topBarHeight + theme.layout.outerMargin,
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
  private readonly brand: Text;
  private readonly title: Text;
  private readonly meta: Text;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.brand = text("四方诛杀(冷冽审讯档案)", theme.typography.brand);
    this.title = text("", theme.typography.title);
    this.meta = text("", theme.typography.meta);

    this.brand.anchor.set(0, 0);
    this.title.anchor.set(0.5, 0);
    this.meta.anchor.set(1, 0);
    this.brand.position.set(204, 27);
    this.title.position.set(theme.layout.width / 2, 23);
    this.meta.position.set(theme.layout.width - 204, 31);
    this.view.addChild(this.panel, this.brand, this.title, this.meta, this.rule);
  }

  update(frame: ShotFrame): void {
    this.title.text = frame.scene.title;
    this.meta.text = `CASE #${frame.scene.index} / ${frame.scene.kind.toUpperCase()}`;
    fitText(this.title, 720, 34, 24);
    this.panel
      .clear()
      .rect(0, 0, this.theme.layout.width, this.theme.layout.topBarHeight)
      .fill({ color: this.theme.colors.black, alpha: 0.30 })
      .rect(0, this.theme.layout.topBarHeight - 1, this.theme.layout.width, 1)
      .fill({ color: this.theme.colors.accent, alpha: 0.14 });
    this.rule
      .clear()
      .rect(0, this.theme.layout.topBarHeight - 2, this.theme.layout.width, 1)
      .fill({ color: this.theme.colors.brass, alpha: 0.07 });
  }

  renderEmpty(): void {
    this.title.text = "No playable scenes";
    this.meta.text = "";
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
      top: 0,
      width: sideColumnWidth(theme),
      height: mainAreaHeight(theme),
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
      fallbackFontSize: 56,
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
      gap: 12,
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
    this.textColumn.addChild(this.name, this.role);
    if (side === "left") {
      this.view.addChild(this.panel, this.activeMark, this.seatBox, this.textColumn, this.avatar.view, this.seatNo, this.deadOverlay);
    } else {
      this.view.addChild(this.panel, this.activeMark, this.avatar.view, this.textColumn, this.seatBox, this.seatNo, this.deadOverlay);
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
    this.panel
      .clear()
      .roundRect(0, 0, seatCardWidth(this.theme), this.theme.layout.seatCardHeight, this.theme.layout.radius)
      .fill({
        color: this.theme.colors.panel,
        alpha: highlighted ? 0.90 : 0.84,
      })
      .roundRect(0, 0, seatCardWidth(this.theme), this.theme.layout.seatCardHeight, this.theme.layout.radius)
      .stroke({
        color: highlighted ? this.theme.colors.accent : 0xd4c7ad,
        alpha: highlighted ? 0.42 : 0.13,
        width: highlighted ? 2 : 1,
      });
    this.activeMark.clear();
    if (active) {
      this.activeMark
        .rect(padding, padding, 34, 2)
        .fill({ color: this.theme.colors.brass, alpha: 0.70 })
        .rect(padding, padding, 2, 34)
        .fill({ color: this.theme.colors.brass, alpha: 0.70 });
    }
    this.avatar.update(player.name, avatarImage, dead, this.theme);
    this.seatNo.text = String(player.seatNo).padStart(2, "0");
    this.name.text = player.name;
    this.role.text = `身份：${player.roleName}`;
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
      fontSize: 22,
      lineHeight: 30,
      align: "center",
      fill: subtleRoleColor(player.roleName, this.theme),
    };
    this.name.style = {
      ...this.theme.typography.playerName,
      fontSize: active ? 40 : 38,
      lineHeight: active ? 50 : 48,
      align: "center",
    };
    this.seatNo.style = {
      ...this.theme.typography.meta,
      fontFamily: PIXI_PREVIEW_FONT_FAMILY,
      fontSize: SEAT_NUMBER_FONT_SIZE,
      fontWeight: "900",
      lineHeight: SEAT_NUMBER_LINE_HEIGHT,
      align: "center",
    };
    fitText(this.name, 260, active ? 40 : 38, 24);
    this.seatNo.position.set(
      seatNumberCenterX(this.theme, this.side),
      padding + avatarSize / 2,
    );
    this.name.anchor.set(0.5);
    this.role.anchor.set(0.5);
    this.name.position.set(textColumnWidth(this.theme) / 2, avatarSize / 2 - 22);
    this.role.position.set(textColumnWidth(this.theme) / 2, avatarSize / 2 + 31);
    this.deadOverlay
      .clear()
      .roundRect(0, 0, seatCardWidth(this.theme), this.theme.layout.seatCardHeight, this.theme.layout.radius)
      .fill({ color: this.theme.colors.black, alpha: dead ? 0.54 : 0 });
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
      borderRadius: 28,
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
      .roundRect(0, 0, mainStageWidth(this.theme), mainStageHeight(this.theme), 28)
      .fill({ color: this.theme.colors.black, alpha: player ? 0.30 : 0.14 })
      .roundRect(0, 0, mainStageWidth(this.theme), mainStageHeight(this.theme), 28)
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
  private readonly kindLabel: Text;
  private readonly title: Text;
  private readonly body: Text;
  private readonly rowHeader: Text;
  private readonly rows: CaseBoardRowView[];

  constructor(private readonly theme: PixiPreviewTheme) {
    this.kindLabel = text("", {
      ...theme.typography.meta,
      fill: theme.colors.brass,
      fontSize: 22,
      letterSpacing: 0,
    });
    this.title = text("", {
      ...theme.typography.title,
      fill: theme.colors.text,
      fontSize: 48,
      lineHeight: 58,
      wordWrap: true,
      wordWrapWidth: caseBoardLeftColumnWidth(theme),
    });
    this.body = text("", {
      ...theme.typography.body,
      fill: theme.colors.text,
      fontSize: 30,
      lineHeight: 42,
      wordWrap: true,
      wordWrapWidth: caseBoardLeftColumnWidth(theme),
    });
    this.rowHeader = text("CASE NOTES", {
      ...theme.typography.meta,
      fill: theme.colors.muted,
      fontSize: 18,
      letterSpacing: 0,
    });
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
      this.kindLabel,
      this.title,
      this.body,
      this.rowHeader,
      ...this.rows.map((row) => row.view),
    );
  }

  update(frame: ShotFrame): void {
    const content = caseBoardContentForFrame(frame);
    const width = mainStageWidth(this.theme);
    const height = mainStageHeight(this.theme);
    const padding = 46;
    const leftWidth = caseBoardLeftColumnWidth(this.theme);
    const dividerX = padding + leftWidth + 36;
    const rightX = dividerX + 36;
    const rightWidth = width - rightX - padding;
    const rowHeight = 86;
    const rowGap = 14;

    this.panel
      .clear()
      .roundRect(0, 0, width, height, 28)
      .fill({ color: this.theme.colors.black, alpha: 0.46 })
      .roundRect(0, 0, width, height, 28)
      .stroke({ color: 0xd4c7ad, alpha: 0.18, width: 1 });
    this.accent
      .clear()
      .rect(padding, padding, 92, 2)
      .fill({ color: this.theme.colors.brass, alpha: 0.62 })
      .rect(dividerX, padding, 1, height - padding * 2)
      .fill({ color: 0xd4c7ad, alpha: 0.12 })
      .rect(padding, height - padding - 2, leftWidth, 1)
      .fill({ color: this.theme.colors.accent, alpha: 0.18 });

    this.kindLabel.text = content.kindLabel;
    this.title.text = content.title;
    this.body.text = content.body;
    this.rowHeader.text = "CASE NOTES";
    this.kindLabel.position.set(padding, padding + 24);
    this.title.position.set(padding, padding + 76);
    this.body.position.set(padding, padding + 192);
    this.rowHeader.position.set(rightX, padding + 24);
    this.title.style = {
      ...this.title.style,
      wordWrapWidth: leftWidth,
    };
    this.body.style = {
      ...this.body.style,
      wordWrapWidth: leftWidth,
    };
    fitText(this.title, leftWidth, 48, 32);
    fitText(this.body, leftWidth, 30, 22);

    this.rows.forEach((rowView, index) => {
      const row = content.rows[index] ?? null;
      const y = padding + 72 + index * (rowHeight + rowGap);
      rowView.update(row, rightX, y, rightWidth, rowHeight);
    });
  }

  renderEmpty(): void {
    const width = mainStageWidth(this.theme);
    const height = mainStageHeight(this.theme);
    const padding = 46;
    const leftWidth = caseBoardLeftColumnWidth(this.theme);
    const dividerX = padding + leftWidth + 36;
    const rightX = dividerX + 36;
    const rightWidth = width - rightX - padding;
    const rowHeight = 86;

    this.panel
      .clear()
      .roundRect(0, 0, width, height, 28)
      .fill({ color: this.theme.colors.black, alpha: 0.46 })
      .roundRect(0, 0, width, height, 28)
      .stroke({ color: 0xd4c7ad, alpha: 0.18, width: 1 });
    this.accent
      .clear()
      .rect(padding, padding, 92, 2)
      .fill({ color: this.theme.colors.brass, alpha: 0.62 })
      .rect(dividerX, padding, 1, height - padding * 2)
      .fill({ color: 0xd4c7ad, alpha: 0.12 })
      .rect(padding, height - padding - 2, leftWidth, 1)
      .fill({ color: this.theme.colors.accent, alpha: 0.18 });

    this.kindLabel.text = "PREVIEW";
    this.title.text = "等待审讯记录";
    this.body.text = "尚无可播放场景。";
    this.rowHeader.text = "CASE NOTES";
    this.kindLabel.position.set(padding, padding + 24);
    this.title.position.set(padding, padding + 76);
    this.body.position.set(padding, padding + 192);
    this.rowHeader.position.set(rightX, padding + 24);
    this.title.style = {
      ...this.title.style,
      wordWrapWidth: leftWidth,
    };
    this.body.style = {
      ...this.body.style,
      wordWrapWidth: leftWidth,
    };
    fitText(this.title, leftWidth, 48, 32);
    fitText(this.body, leftWidth, 30, 22);

    this.rows[0]?.update(
      { tone: "neutral", label: "FILE", text: "PUBLIC RECORD" },
      rightX,
      padding + 72,
      rightWidth,
      rowHeight,
    );
    this.rows.slice(1).forEach((row) => row.update(null, 0, 0, 0, 0));
  }
}

class CaseBoardRowView {
  readonly view = new Container();
  private readonly panel = new Graphics();
  private readonly label: Text;
  private readonly body: Text;
  private readonly meta: Text;

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
    this.label.anchor.set(0.5);
    this.view.addChild(this.panel, this.label, this.body, this.meta);
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
      this.label.text = "";
      this.body.text = "";
      this.meta.text = "";
      return;
    }

    const toneColor = colorForCaseBoardTone(row.tone, this.theme);
    const labelWidth = 92;
    const textX = labelWidth + 22;
    const textWidth = width - textX - 22;

    this.view.position.set(x, y);
    this.panel
      .clear()
      .roundRect(0, 0, width, height, 12)
      .fill({ color: this.theme.colors.panel, alpha: 0.64 })
      .roundRect(0, 0, width, height, 12)
      .stroke({ color: toneColor, alpha: 0.26, width: 1 })
      .rect(0, 0, 4, height)
      .fill({ color: toneColor, alpha: 0.62 });

    this.label.text = row.label;
    this.body.text = row.text;
    this.meta.text = row.meta ?? "";
    this.label.style = {
      ...this.label.style,
      fill: toneColor,
    };
    this.body.style = {
      ...this.body.style,
      fill: row.tone === "dead-player" ? this.theme.colors.muted : this.theme.colors.text,
      wordWrapWidth: textWidth,
    };
    this.meta.visible = Boolean(row.meta);
    this.label.position.set(labelWidth / 2, height / 2);
    const bodyY = row.meta ? 14 : 20;
    const bodyHeight = row.meta ? 32 : height - bodyY - 18;
    this.body.position.set(textX, bodyY);
    this.meta.position.set(textX, 50);
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
    const width = mainStageWidth(this.theme);
    this.panel
      .clear()
      .rect(0, 0, width, this.theme.layout.subtitleHeight)
      .fill({ color: this.theme.colors.black, alpha: 0.82 })
      .rect(0, 0, width, this.theme.layout.subtitleHeight)
      .stroke({ color: 0xd4c7ad, alpha: 0.18, width: 1 });
    this.rule
      .clear()
      .rect(0, 0, width, 2)
      .fill({ color: this.theme.colors.accent, alpha: 0.18 });
    this.speaker.text = frame.subtitle?.speaker ?? frame.scene.title;
    this.content.text = frame.subtitle?.lines.join("\n") ?? frame.scene.text;
  }

  renderEmpty(): void {
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
    const pulse = 0.22 + Math.sin(frame.clock.progress * Math.PI) * 0.18;
    this.rule
      .clear()
      .rect(0, this.theme.layout.topBarHeight - 2, this.theme.layout.width, 2)
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

function roleColor(roleName: string, theme: PixiPreviewTheme): number {
  if (roleName.includes("狼")) {
    return theme.colors.wolf;
  }
  if (roleName.includes("预言") || roleName.includes("女巫") || roleName.includes("猎人")) {
    return theme.colors.good;
  }
  return 0xc9a85a;
}

function subtleRoleColor(roleName: string, theme: PixiPreviewTheme): number {
  if (roleName.includes("狼")) {
    return 0xc78984;
  }
  if (roleName.includes("预言") || roleName.includes("女巫") || roleName.includes("猎人")) {
    return 0x9fbfaf;
  }
  return theme.colors.muted;
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
  }
}

function caseBoardLeftColumnWidth(theme: PixiPreviewTheme): number {
  return Math.round(mainStageWidth(theme) * 0.53);
}

function mainStageWidth(theme: PixiPreviewTheme): number {
  return centerColumnWidth(theme);
}

function mainStageHeight(theme: PixiPreviewTheme): number {
  return mainAreaHeight(theme);
}

function mainAreaHeight(theme: PixiPreviewTheme): number {
  return theme.layout.height
    - theme.layout.topBarHeight
    - theme.layout.outerMargin * 2;
}

function availableWidth(theme: PixiPreviewTheme): number {
  return theme.layout.width
    - theme.layout.outerMargin * 2;
}

function contentWidth(theme: PixiPreviewTheme): number {
  return availableWidth(theme) - theme.layout.columnGap * 2;
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
    - 24;
}

function subtitleTop(theme: PixiPreviewTheme): number {
  return mainAreaHeight(theme) * theme.layout.subtitleTopRatio;
}
