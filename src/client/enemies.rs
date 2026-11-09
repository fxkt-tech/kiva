//! Enemy-mode input is a request to gameplay; the displayed selection follows Rust state.
use bevy::prelude::*;

use crate::gameplay::{EnemyMode, set_enemy_mode};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaEnemies"], js_name = takeMode)]
    fn take_mode() -> u32;
    #[wasm_bindgen(js_namespace = ["window", "kivaEnemies"], js_name = render)]
    fn render(mode: u32);
}

pub(super) fn sample(
    mut commands: Commands,
    mode: Res<EnemyMode>,
    #[cfg(not(target_arch = "wasm32"))] keys: Res<ButtonInput<KeyCode>>,
    #[cfg(not(target_arch = "wasm32"))] window: Single<&Window>,
) {
    #[cfg(target_arch = "wasm32")]
    let requested = {
        render(match *mode {
            EnemyMode::Target => 1,
            EnemyMode::Combat => 2,
        });
        match take_mode() {
            1 => Some(EnemyMode::Target),
            2 => Some(EnemyMode::Combat),
            _ => None,
        }
    };
    #[cfg(not(target_arch = "wasm32"))]
    let requested = (window.focused && keys.just_pressed(KeyCode::F2)).then(|| mode.toggled());

    if let Some(requested) = requested {
        commands.queue(move |world: &mut World| set_enemy_mode(world, requested));
    }
}
