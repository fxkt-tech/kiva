//! Client-only render style selection. A style combines surface shading, a
//! per-level lighting profile and a camera post chain; each owning module
//! applies its part when [`RenderStyle`] changes. Simulation never reads it.
use bevy::prelude::*;

use crate::gameplay::GameLevel;

#[cfg(not(target_arch = "wasm32"))]
mod shots;
pub(super) mod yancai_post;

/// Registers every style's render passes; they stay idle until a camera
/// carries their components.
pub(super) fn configure(app: &mut App) {
    yancai_post::configure(app);
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::args().any(|arg| arg == shots::FLAG) {
        shots::configure(app);
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(super) enum RenderStyle {
    /// The deferred PBR look the game shipped with.
    #[default]
    Pbr,
    /// Mineral-pigment painting: banded light, cool shadows, ink outlines.
    Yancai,
}

impl RenderStyle {
    /// Stable bridge ids; 0 is reserved for "no request".
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(super) fn id(self) -> u32 {
        match self {
            Self::Pbr => 1,
            Self::Yancai => 2,
        }
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Pbr => "写实",
            Self::Yancai => "岩彩",
        }
    }
}

/// The styles the current level registers, in F8 cycling order.
#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub(super) struct LevelStyles(Vec<RenderStyle>);

impl LevelStyles {
    pub(super) fn for_level(level: GameLevel) -> Self {
        match level {
            GameLevel::BambooForest => Self(vec![RenderStyle::Pbr, RenderStyle::Yancai]),
            // Station keeps its painted PBR look until it gets its own profile.
            GameLevel::Station => Self(vec![RenderStyle::Pbr]),
        }
    }

    pub(super) fn count(&self) -> usize {
        self.0.len()
    }

    /// The style after `current`; a single-style level stays put.
    pub(super) fn next(&self, current: RenderStyle) -> RenderStyle {
        let index = self.0.iter().position(|style| *style == current);
        index.map_or(self.0[0], |index| self.0[(index + 1) % self.0.len()])
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaRenderStyle"], js_name = takeStyle)]
    fn take_style() -> u32;
    #[wasm_bindgen(js_namespace = ["window", "kivaRenderStyle"], js_name = render)]
    fn render(style: u32, count: u32);
}

/// F8 (native: focused and simulating, like F3) or one page request (Web)
/// advances to the level's next style. Requests on single-style levels are
/// consumed without effect.
pub(super) fn sample(
    mut style: ResMut<RenderStyle>,
    styles: Res<LevelStyles>,
    #[cfg(not(target_arch = "wasm32"))] keys: Res<ButtonInput<KeyCode>>,
    #[cfg(not(target_arch = "wasm32"))] window: Single<&Window>,
    #[cfg(not(target_arch = "wasm32"))] active: Res<crate::simulation::SimulationActive>,
) {
    #[cfg(target_arch = "wasm32")]
    let requested = take_style() == 1;
    #[cfg(not(target_arch = "wasm32"))]
    let requested = window.focused && active.0 && keys.just_pressed(KeyCode::F8);
    if requested {
        let next = styles.next(*style);
        style.set_if_neq(next);
    }
    #[cfg(target_arch = "wasm32")]
    render(style.id(), styles.count() as u32);
}

/// True once per real switch; the initial insertion is the shipped setup.
pub(super) fn switched(style: Res<RenderStyle>) -> bool {
    style.is_changed() && !style.is_added()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bamboo_cycles_both_styles_and_station_stays_pbr() {
        let bamboo = LevelStyles::for_level(GameLevel::BambooForest);
        assert_eq!(bamboo.count(), 2);
        assert_eq!(bamboo.next(RenderStyle::Pbr), RenderStyle::Yancai);
        assert_eq!(bamboo.next(RenderStyle::Yancai), RenderStyle::Pbr);
        let station = LevelStyles::for_level(GameLevel::Station);
        assert_eq!(station.count(), 1);
        assert_eq!(station.next(RenderStyle::Pbr), RenderStyle::Pbr);
        // An unregistered style falls back to the level's first one.
        assert_eq!(station.next(RenderStyle::Yancai), RenderStyle::Pbr);
    }

    #[test]
    fn bridge_ids_are_stable_and_never_zero() {
        assert_eq!(RenderStyle::Pbr.id(), 1);
        assert_eq!(RenderStyle::Yancai.id(), 2);
        assert_eq!(RenderStyle::default(), RenderStyle::Pbr);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn app(level: GameLevel, focused: bool, active: bool) -> App {
        let mut app = App::new();
        app.insert_resource(RenderStyle::default())
            .insert_resource(LevelStyles::for_level(level))
            .insert_resource(crate::simulation::SimulationActive(active))
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, sample);
        app.world_mut().spawn(Window {
            focused,
            ..default()
        });
        app
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn press(app: &mut App) -> RenderStyle {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::F8);
        keys.clear();
        keys.press(KeyCode::F8);
        app.update();
        *app.world().resource::<RenderStyle>()
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_f8_needs_focus_and_simulation_and_respects_the_level() {
        let mut bamboo = app(GameLevel::BambooForest, true, true);
        assert_eq!(press(&mut bamboo), RenderStyle::Yancai);
        assert_eq!(press(&mut bamboo), RenderStyle::Pbr);

        for (focused, active) in [(false, true), (true, false)] {
            let mut gated = app(GameLevel::BambooForest, focused, active);
            assert_eq!(press(&mut gated), RenderStyle::Pbr);
        }

        let mut station = app(GameLevel::Station, true, true);
        assert_eq!(press(&mut station), RenderStyle::Pbr);
    }
}
