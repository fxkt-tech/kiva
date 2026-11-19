//! Illustrated native entry and pause menu. The same 16:9 page coordinates drive
//! the painting, typesetting, and hit testing whatever the window size or scale.
mod ink;
mod pages;
mod shop;

use super::{
    Assembled, Manifest, bundle,
    controls::{CaptureState, ScrollCapture},
    loading::{Readiness, StartupSignal},
    menu_loadout::MenuLoadout,
};
use bevy::{asset::LoadState, input::InputSystems, prelude::*, window::PrimaryWindow};
use pages::Page;
use shop::Shop;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Leaf {
    #[default]
    Home,
    Map,
    Persona,
    Armory,
    Birds,
    Closeup,
    Supply,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Open(Leaf),
    Back,
    Stage(usize),
    Enter,
    Portrait(usize),
    Pair(usize),
    Equip,
    Bird(usize),
    Stall(usize),
    Ware(usize),
    Adjust(i32),
    Buy,
}

#[derive(Resource)]
pub(super) struct NativeHandscroll {
    open: bool,
    playing_before: bool,
    leaf: Leaf,
    stage: usize,
    portrait: usize,
    pair: usize,
    bird: usize,
    shop: Shop,
}

impl Default for NativeHandscroll {
    fn default() -> Self {
        Self {
            open: true,
            playing_before: false,
            leaf: Leaf::Home,
            stage: 0,
            portrait: 0,
            pair: 0,
            bird: 0,
            shop: Shop::default(),
        }
    }
}

impl NativeHandscroll {
    pub(super) fn is_open(&self) -> bool {
        self.open
    }

    fn can_enter(&self, status: &EntryStatus) -> bool {
        *status == EntryStatus::Ready
            && match self.leaf {
                Leaf::Home => self.playing_before,
                Leaf::Map => self.stage == 0,
                _ => false,
            }
    }

    fn back(&mut self) {
        self.leaf = if self.leaf == Leaf::Closeup {
            Leaf::Birds
        } else {
            Leaf::Home
        };
    }
}

const ART: [&str; 13] = [
    "duangui-key-art.png",
    "stage-select-landscape.webp",
    "persona-zhuque.webp",
    "persona-qingluan.webp",
    "armory-hammer.webp",
    "armory-axe.webp",
    "bestiary-landscape.webp",
    "bestiary-crow.webp",
    "bestiary-vulture.webp",
    "bestiary-owl.webp",
    "supply-travel.webp",
    "supply-plume.webp",
    "supply-parts.webp",
];

#[derive(Resource)]
struct ScrollAssets {
    paintings: [Handle<Image>; ART.len()],
    font: Handle<Font>,
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
enum EntryStatus {
    #[default]
    Loading,
    Files(u32, u32),
    Preparing,
    Ready,
    Failed(String),
}
impl EntryStatus {
    fn label(&self) -> String {
        match self {
            Self::Loading => "展开绘卷…".into(),
            Self::Files(done, total) => format!("载入资源 {done} / {total}"),
            Self::Preparing => "准备画面…".into(),
            Self::Ready => String::new(),
            Self::Failed(path) => format!("载入失败 · {path}"),
        }
    }
}

#[derive(Resource, Default)]
struct Navigation {
    focus: Option<Action>,
    origin: Option<Vec2>,
}
#[derive(Component)]
struct ScrollCamera;
#[derive(Component)]
struct Paper;

pub(super) fn configure(app: &mut App, direct: bool) {
    app.insert_resource(NativeHandscroll {
        open: !direct,
        ..default()
    })
    .insert_resource(ScrollCapture::new(direct))
    .init_resource::<EntryStatus>()
    .init_resource::<Navigation>()
    .init_resource::<Page>()
    .add_systems(Startup, setup)
    .add_systems(
        PreUpdate,
        (update_status, input).chain().after(InputSystems),
    )
    .add_systems(
        RunFixedMainLoop,
        observe_capture.in_set(bevy::app::RunFixedMainLoopSystems::AfterFixedMainLoop),
    )
    .add_systems(Update, (draw, ink::advance).chain());
    ink::configure(app);
}

fn setup(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 20,
            clear_color: ClearColorConfig::Custom(Color::srgb_u8(32, 31, 33)),
            ..default()
        },
        Msaa::Off,
        ScrollCamera,
    ));
    commands.insert_resource(ScrollAssets {
        paintings: ART.map(|name| server.load(format!("handscroll/{name}"))),
        font: server.load("handscroll/scroll-font.otf"),
    });
}

fn update_status(world: &mut World) {
    if *world.resource::<EntryStatus>() == EntryStatus::Ready {
        return;
    }
    let status = {
        let assets = world.resource::<ScrollAssets>();
        let server = world.resource::<AssetServer>();
        let mut loaded = 0;
        let mut failed = None;
        for (id, path) in assets
            .paintings
            .iter()
            .zip(ART)
            .map(|(h, name)| (h.id().untyped(), name))
            .chain(std::iter::once((
                assets.font.id().untyped(),
                "scroll-font.otf",
            )))
        {
            match server.load_state(id) {
                LoadState::Loaded => loaded += 1,
                LoadState::Failed(_) => {
                    failed = Some(path);
                    break;
                }
                _ => {}
            }
        }
        let count = ART.len() as u32 + 1;
        let game = bundle::status(world, &world.resource::<Manifest>().0);
        match (failed, game, world.resource::<StartupSignal>().get()) {
            (Some(path), _, _) | (_, bundle::Status::Failed(path), _) => {
                EntryStatus::Failed(path.to_string())
            }
            (_, _, Readiness::Failed) => EntryStatus::Failed("画面准备失败".into()),
            (
                _,
                bundle::Status::Pending {
                    loaded: files,
                    total,
                },
                _,
            ) => EntryStatus::Files(loaded + files, count + total),
            _ if loaded != count => {
                let game_count = world.resource::<Manifest>().0.len() as u32;
                EntryStatus::Files(loaded + game_count, count + game_count)
            }
            (_, _, Readiness::Ready) if world.contains_resource::<Assembled>() => {
                EntryStatus::Ready
            }
            _ => EntryStatus::Preparing,
        }
    };
    world.resource_mut::<EntryStatus>().set_if_neq(status);
}

fn observe_capture(
    capture: Res<CaptureState>,
    gate: Res<ScrollCapture>,
    mut scroll: ResMut<NativeHandscroll>,
    mut nav: ResMut<Navigation>,
) {
    if capture.is_active() && !scroll.playing_before {
        scroll.playing_before = true;
    }
    let show = !capture.is_active() && !gate.awaiting_direct_click;
    if scroll.open != show {
        scroll.open = show;
        scroll.leaf = Leaf::Home;
        scroll.stage = 0;
        nav.focus = None;
    }
}

fn fit(window: &Window) -> (Vec2, f32) {
    let size = Vec2::new(window.width(), window.height());
    let scale = (size.x / 1600.).min(size.y / 900.);
    ((size - Vec2::new(1600., 900.) * scale) * 0.5, scale)
}

#[allow(clippy::too_many_arguments)]
fn input(
    mut scroll: ResMut<NativeHandscroll>,
    page: Res<Page>,
    mut nav: ResMut<Navigation>,
    mut last_pointer: Local<Option<Vec2>>,
    window: Single<&Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    status: Res<EntryStatus>,
    mut gate: ResMut<ScrollCapture>,
    mut loadout: ResMut<MenuLoadout>,
) {
    if !scroll.open {
        if gate.awaiting_direct_click
            && (matches!(*status, EntryStatus::Failed(_))
                || (window.focused && keys.just_pressed(KeyCode::Escape)))
        {
            gate.awaiting_direct_click = false;
        }
        return;
    }
    if !window.focused {
        return;
    }
    let (offset, scale) = fit(&window);
    let pointer = window.cursor_position().map(|pos| (pos - offset) / scale);
    if *last_pointer != pointer {
        *last_pointer = pointer;
        let hover = pointer.and_then(|pos| page.hit(pos));
        if nav.focus != hover {
            nav.focus = hover;
        }
    }
    let action = if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::ArrowRight) {
        Some(Action::Back)
    } else if mouse.just_pressed(MouseButton::Left) {
        pointer.and_then(|pos| page.hit(pos))
    } else if keys.just_pressed(KeyCode::Tab)
        || keys.just_pressed(KeyCode::ArrowDown)
        || keys.just_pressed(KeyCode::ArrowUp)
    {
        let previous = keys.pressed(KeyCode::ShiftLeft)
            || keys.pressed(KeyCode::ShiftRight)
            || keys.just_pressed(KeyCode::ArrowUp);
        nav.focus = page.step(nav.focus, previous);
        None
    } else if keys.just_pressed(KeyCode::ArrowLeft) && scroll.leaf == Leaf::Home {
        Some(Action::Open(Leaf::Map))
    } else if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]) {
        nav.focus.or_else(|| page.step(None, false))
    } else {
        None
    };
    let Some(action) = action else { return };
    let origin = if mouse.just_pressed(MouseButton::Left) {
        pointer
    } else {
        page.center(action)
    };
    nav.origin = origin.map(|p| p / Vec2::new(1600., 900.));
    let old_leaf = scroll.leaf;
    match action {
        Action::Open(leaf) => scroll.leaf = leaf,
        Action::Back => scroll.back(),
        Action::Stage(index) => scroll.stage = index,
        Action::Enter if scroll.can_enter(&status) => gate.request(),
        Action::Enter => {}
        Action::Portrait(index) => scroll.portrait = index,
        Action::Pair(index) => scroll.pair = index,
        Action::Equip => loadout.select(crate::simulation::WeaponPair::ALL[scroll.pair]),
        Action::Bird(index) => {
            scroll.bird = index;
            scroll.leaf = Leaf::Closeup;
        }
        Action::Stall(index) => scroll.shop.select_stall(index),
        Action::Ware(index) => scroll.shop.select(index),
        Action::Adjust(delta) => scroll.shop.adjust(delta),
        Action::Buy => scroll.shop.purchase(),
    }
    nav.focus = if old_leaf == scroll.leaf {
        Some(action)
    } else {
        None
    };
}

#[allow(clippy::too_many_arguments)]
fn draw(
    mut commands: Commands,
    scroll: Res<NativeHandscroll>,
    loadout: Res<MenuLoadout>,
    status: Res<EntryStatus>,
    assets: Res<ScrollAssets>,
    server: Res<AssetServer>,
    mut nav: ResMut<Navigation>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut camera: Single<(Entity, &mut Camera), With<ScrollCamera>>,
    mut page: ResMut<Page>,
    roots: Query<Entity, With<Paper>>,
    mut old_view: Local<Option<(usize, Vec2, crate::simulation::WeaponPair)>>,
    mut materials: ResMut<Assets<ink::Ink>>,
) {
    let (camera_id, ref mut camera) = *camera;
    camera.is_active = scroll.open;
    if !scroll.open {
        if old_view.take().is_some() {
            for entity in &roots {
                commands.entity(entity).despawn();
            }
        }
        return;
    }
    let window_size = Vec2::new(window.width(), window.height());
    let equipped = loadout.displayed();
    let picture = pages::picture(&scroll);
    let identity = (picture, window_size, equipped);
    if !scroll.is_changed()
        && !status.is_changed()
        && !nav.is_changed()
        && old_view.as_ref() == Some(&identity)
    {
        return;
    }
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    let previous = old_view.replace(identity);
    let (offset, scale) = fit(&window);
    if scale <= 0.0 {
        return;
    }
    let layout = pages::compose(&scroll, equipped, &status);
    let paper = commands
        .spawn((
            Paper,
            UiTargetCamera(camera_id),
            Node {
                position_type: PositionType::Absolute,
                left: px(offset.x),
                top: px(offset.y),
                width: px(1600. * scale),
                height: px(900. * scale),
                overflow: Overflow::clip(),
                ..default()
            },
            ImageNode::new(assets.paintings[picture].clone()),
            BackgroundColor(pages::PAPER),
        ))
        .id();
    for element in &layout.elements {
        element.draw(&mut commands, paper, &assets.font, scale, nav.focus);
    }
    if matches!(server.load_state(assets.font.id()), LoadState::Failed(_)) {
        commands.spawn((
            ChildOf(paper),
            Text::new("Failed to load scroll-font.otf"),
            TextColor(pages::INK),
            TextFont::from_font_size(24.),
        ));
    }
    if let Some((from, _, _)) = previous
        && from != picture
    {
        let origin = nav.origin.take().unwrap_or(Vec2::splat(0.5));
        ink::reveal(
            &mut commands,
            &mut materials,
            paper,
            assets.paintings[from].clone(),
            origin,
        );
    }
    *page = layout;
}

#[cfg(test)]
mod tests;
