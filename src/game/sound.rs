//! Short sound cues are synthesized by WebAudio; there are no audio downloads.
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(super) struct Sounds(pub Vec<&'static str>);

pub(super) fn play_sounds(mut sounds: ResMut<Sounds>) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        for sound in &sounds.0 {
            let options = web_sys::CustomEventInit::new();
            options.set_detail(&wasm_bindgen::JsValue::from_str(sound));
            if let Ok(event) =
                web_sys::CustomEvent::new_with_event_init_dict("kiva-sound", &options)
            {
                let _ = window.dispatch_event(&event);
            }
        }
    }
    sounds.0.clear();
}
