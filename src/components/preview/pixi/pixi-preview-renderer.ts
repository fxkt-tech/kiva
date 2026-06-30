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
import { mansionPixiTheme, type PixiPreviewTheme } from "./pixi-theme";

const EMPTY_TEXTURE = Texture.EMPTY;

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
  private readonly mainShot: MainShot;
  private readonly subtitle: SubtitleBand;
  private readonly effects: EffectsLayer;
  private readonly content: LayoutContainer;
  private readonly body: LayoutContainer;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.background = new BackgroundLayer(theme);
    this.topBar = new TopBar(theme);
    this.leftTrack = new SeatTrack(theme, "left");
    this.rightTrack = new SeatTrack(theme, "right");
    this.mainShot = new MainShot(theme);
    this.subtitle = new SubtitleBand(theme);
    this.effects = new EffectsLayer(theme);
    this.content = new LayoutContainer();
    this.body = new LayoutContainer();

    this.view.addChild(this.background.view, this.content, this.effects.view);
    this.content.layout = {
      width: theme.layout.width,
      height: theme.layout.height,
      flexDirection: "column",
    };
    this.body.layout = {
      width: "100%",
      flex: 1,
      flexDirection: "row",
      gap: theme.layout.gap,
      alignItems: "stretch",
      paddingLeft: theme.layout.contentMarginX,
      paddingRight: theme.layout.contentMarginX,
      paddingTop: theme.layout.bodyPaddingTop,
      paddingBottom: theme.layout.bodyPaddingBottom,
    };
    this.content.addChild(this.topBar.view, this.body, this.subtitle.view);
    this.body.addChild(this.leftTrack.view, this.mainShot.view, this.rightTrack.view);
  }

  update(frame: ShotFrame): void {
    this.background.update(frame);
    this.topBar.update(frame);
    this.leftTrack.update(frame);
    this.rightTrack.update(frame);
    this.mainShot.update(frame);
    this.subtitle.update(frame);
    this.effects.update(frame);
  }

  renderEmpty(): void {
    this.background.renderEmpty();
    this.topBar.renderEmpty();
    this.leftTrack.clear();
    this.rightTrack.clear();
    this.mainShot.renderEmpty();
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
    const { width, height } = this.theme.layout;
    this.dim
      .clear()
      .rect(0, 0, width, this.theme.layout.topBarHeight)
      .fill({ color: this.theme.colors.black, alpha: 0.30 })
      .rect(0, height - this.theme.layout.subtitleHeight - 44, width, this.theme.layout.subtitleHeight + 44)
      .fill({ color: this.theme.colors.black, alpha: 0.20 });

    this.vignette
      .clear()
      .rect(0, 0, width, 72)
      .fill({ color: this.theme.colors.black, alpha: 0.26 })
      .rect(0, height - 120, width, 120)
      .fill({ color: this.theme.colors.black, alpha: 0.24 });
  }
}

class TopBar {
  readonly view = new LayoutContainer();
  private readonly panel = new Graphics();
  private readonly rule = new Graphics();
  private readonly brand: Text;
  private readonly title: Text;
  private readonly meta: Text;

  constructor(private readonly theme: PixiPreviewTheme) {
    this.brand = text("四方诛杀(冷冽审讯档案)", theme.typography.brand);
    this.title = text("", theme.typography.title);
    this.meta = text("", theme.typography.meta);
    this.rule.layout = {
      position: "absolute",
      left: 0,
      top: theme.layout.topBarHeight - 2,
      width: "100%",
      height: 2,
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
      height: theme.layout.topBarHeight,
      paddingLeft: 72,
      paddingRight: 72,
      flexDirection: "row",
      alignItems: "center",
      gap: 42,
    };
    this.brand.layout = { width: 460 };
    this.title.layout = { flex: 1 };
    this.meta.layout = { width: 260 };
    this.view.addChild(this.panel, this.brand, this.title, this.meta, this.rule);
  }

  update(frame: ShotFrame): void {
    this.title.text = frame.scene.title;
    this.meta.text = `CASE #${frame.scene.index} / ${frame.scene.kind.toUpperCase()}`;
    fitText(this.title, 720, 34, 24);
    this.panel
      .clear()
      .rect(0, 0, this.theme.layout.width, this.theme.layout.topBarHeight)
      .fill({ color: this.theme.colors.black, alpha: 0.90 })
      .rect(0, this.theme.layout.topBarHeight - 1, this.theme.layout.width, 1)
      .fill({ color: this.theme.colors.accent, alpha: 0.24 });
    this.rule
      .clear()
      .rect(0, this.theme.layout.topBarHeight - 2, this.theme.layout.width, 2)
      .fill({ color: this.theme.colors.brass, alpha: 0.16 });
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
      width: theme.layout.sideTrackWidth,
      height: "100%",
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
  private readonly caseLine = new Graphics();
  private readonly activeMark = new Graphics();
  private readonly avatar: AvatarView;
  private readonly textColumn: LayoutContainer;
  private readonly seatNo: Text;
  private readonly name: Text;
  private readonly role: Text;
  private readonly status: Text;

  constructor(
    private readonly theme: PixiPreviewTheme,
    private readonly side: "left" | "right",
  ) {
    this.avatar = new AvatarView({
      fallbackFontSize: 48,
      radius: theme.layout.radius,
      width: 62,
      height: 76,
    });
    this.textColumn = new LayoutContainer();
    this.seatNo = text("", theme.typography.meta);
    this.name = text("", theme.typography.playerName);
    this.role = text("", theme.typography.body);
    this.status = text("", {
      ...theme.typography.meta,
      fontSize: 12,
      fill: theme.colors.good,
    });
    this.caseLine.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.activeMark.layout = {
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
      flexDirection: side === "left" ? "row-reverse" : "row",
      alignItems: "center",
      gap: 10,
      padding: 12,
    };
    this.avatar.view.layout = { width: 62, height: 76 };
    this.textColumn.layout = {
      flex: 1,
      height: "100%",
      flexDirection: "column",
      justifyContent: "center",
      gap: 2,
    };
    this.seatNo.layout = { width: "100%" };
    this.name.layout = { width: "100%" };
    this.role.layout = { width: "100%" };
    this.status.layout = { width: "100%" };
    this.textColumn.addChild(this.seatNo, this.name, this.role, this.status);
    this.view.addChild(this.panel, this.caseLine, this.activeMark, this.avatar.view, this.textColumn);
  }

  update(player: RenderablePlayer, frame: ShotFrame): void {
    const dead = player.status === "dead";
    const active = player.emphasis === "active";
    const highlighted = active || player.emphasis === "highlighted";
    const avatarImage = player.avatar ? frame.avatarImages[player.avatar] : null;

    this.view.layout = {
      ...this.view.layout?.style,
      width: "100%",
      height: this.theme.layout.seatCardHeight,
    };
    this.view.alpha = dead ? this.theme.alpha.dead : 1;
    this.panel
      .clear()
      .roundRect(0, 0, this.theme.layout.sideTrackWidth, this.theme.layout.seatCardHeight, this.theme.layout.radius)
      .fill({
        color: this.theme.colors.panel,
        alpha: highlighted ? 0.90 : 0.84,
      })
      .roundRect(0, 0, this.theme.layout.sideTrackWidth, this.theme.layout.seatCardHeight, this.theme.layout.radius)
      .stroke({
        color: highlighted ? this.theme.colors.accent : 0xd4c7ad,
        alpha: highlighted ? 0.42 : 0.13,
        width: highlighted ? 2 : 1,
      });
    this.caseLine
      .clear()
      .roundRect(
        this.side === "left" ? this.theme.layout.sideTrackWidth - 12 : 8,
        12,
        4,
        this.theme.layout.seatCardHeight - 24,
        2,
      )
      .fill({
        color: roleColor(player.roleName, this.theme),
        alpha: highlighted ? 0.72 : 0.34,
      });
    this.activeMark.clear();
    if (active) {
      this.activeMark
        .rect(12, 12, 30, 2)
        .fill({ color: this.theme.colors.brass, alpha: 0.70 })
        .rect(12, 12, 2, 30)
        .fill({ color: this.theme.colors.brass, alpha: 0.70 });
    }
    this.avatar.update(player.name, avatarImage, dead, this.theme);
    this.seatNo.text = `Seat ${player.seatNo}`;
    this.name.text = player.name;
    this.role.text = `身份：${player.roleName}`;
    this.status.text = dead ? "DEAD" : "ALIVE";
    this.status.style = {
      ...this.theme.typography.meta,
      fontSize: 12,
      fill: dead ? this.theme.colors.danger : 0x7f897f,
    };
    this.role.style = {
      ...this.theme.typography.body,
      fontSize: 14,
      fill: subtleRoleColor(player.roleName, this.theme),
    };
    this.seatNo.style = {
      ...this.theme.typography.meta,
      fontSize: 13,
    };
    fitText(this.name, 170, active ? 24 : 22, 17);
    if (this.side === "left") {
      this.textColumn.scale.x = 1;
    }
  }

  destroy(): void {
    this.view.destroy({ children: true });
  }
}

class MainShot {
  readonly view = new LayoutContainer();
  private readonly shell: LayoutContainer;
  private readonly panel = new Graphics();
  private readonly slateRule = new Graphics();
  private readonly portrait: AvatarView;
  private readonly slateTitle: Text;
  private readonly slateMeta: Text;
  private readonly lensOverlay = new Graphics();

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
    this.lensOverlay.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };

    this.view.layout = {
      flex: 1,
      height: "100%",
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
      this.lensOverlay,
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
    this.lensOverlay
      .clear()
      .rect(
        mainStageWidth(this.theme) / 2 - 230,
        mainStageHeight(this.theme) / 2 - 70,
        460,
        126,
      )
      .fill({ color: this.theme.colors.black, alpha: player ? 0 : 0.34 })
      .rect(0, mainStageHeight(this.theme) - 120, mainStageWidth(this.theme), 120)
      .fill({ color: this.theme.colors.black, alpha: player ? 0.20 : 0 });
    this.shell.alpha = 0.88 + frame.clock.enterProgress * 0.12;
    this.shell.scale.set(0.985 + frame.clock.enterProgress * 0.015);
  }

  renderEmpty(): void {
    this.portrait.view.visible = false;
    this.slateMeta.visible = true;
    this.slateTitle.visible = true;
    this.slateRule.visible = true;
    this.slateTitle.text = "等待审讯记录";
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
      width: theme.layout.width - theme.layout.contentMarginX * 2,
      height: theme.layout.subtitleHeight,
      marginLeft: theme.layout.contentMarginX,
      marginRight: theme.layout.contentMarginX,
      paddingLeft: 220,
      paddingRight: 220,
      paddingTop: 22,
      paddingBottom: 28,
      gap: 10,
      flexDirection: "column",
    };
    this.speaker.layout = { width: "100%" };
    this.content.layout = { width: "100%" };
    this.view.addChild(this.panel, this.rule, this.speaker, this.content);
  }

  update(frame: ShotFrame): void {
    const width = this.theme.layout.width - this.theme.layout.contentMarginX * 2;
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
    fontFamily: "Georgia, serif",
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

function mainStageWidth(theme: PixiPreviewTheme): number {
  return theme.layout.width
    - theme.layout.contentMarginX * 2
    - theme.layout.sideTrackWidth * 2
    - theme.layout.gap * 2;
}

function mainStageHeight(theme: PixiPreviewTheme): number {
  return theme.layout.height
    - theme.layout.topBarHeight
    - theme.layout.subtitleHeight
    - theme.layout.bodyPaddingTop
    - theme.layout.bodyPaddingBottom;
}
