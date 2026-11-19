//! Once startup has observed a submitted scene, the opaque scroll can cover it.
use super::{
    camera::GameCamera,
    loading::{Readiness, StartupSignal},
};
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaHandscroll"], js_name = isOpen)]
    fn scroll_is_open() -> bool;
}

pub(super) fn configure(app: &mut App) {
    app.add_systems(
        Update,
        sync_camera.run_if(resource_exists::<super::Assembled>),
    );
}

fn sync_camera(
    signal: Res<StartupSignal>,
    #[cfg(not(target_arch = "wasm32"))] scroll: Option<Res<super::handscroll::NativeHandscroll>>,
    mut camera: Single<&mut Camera, With<GameCamera>>,
) {
    if signal.get() != Readiness::Ready {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    let covered = scroll_is_open();
    #[cfg(not(target_arch = "wasm32"))]
    let covered = scroll.is_some_and(|scroll| scroll.is_open());
    let active = !covered;
    if camera.is_active != active {
        camera.is_active = active;
    }
}
