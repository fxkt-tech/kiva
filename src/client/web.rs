//! Small platform bridge; DOM state never enters the simulation module.
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
fn root() -> Option<web_sys::Element> {
    web_sys::window()?.document()?.document_element()
}

pub(super) fn ready() {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window()
        && let Ok(event) = web_sys::CustomEvent::new("kiva-ready")
    {
        let _ = window.dispatch_event(&event);
    }
}
/// Winit's web window follows canvas focus. HUD controls belong to the same
/// game document, so focusing a button must not be treated as leaving the app.
pub(super) fn document_focused() -> Option<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()?.document()?.has_focus().ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
pub(super) fn paused() -> Option<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        Some(root()?.get_attribute("data-paused").as_deref() != Some("false"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
pub(super) fn set_paused(paused: bool) {
    #[cfg(target_arch = "wasm32")]
    if let Some(root) = root() {
        let _ = root.set_attribute("data-paused", if paused { "true" } else { "false" });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = paused;
}
pub(super) fn pointer_over_ui(point: Vec2) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.element_from_point(point.x, point.y))
            .and_then(|e| e.closest("[data-ui]").ok().flatten())
            .is_some()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = point;
        false
    }
}
pub(super) fn take_skill_request() -> Option<usize> {
    #[cfg(target_arch = "wasm32")]
    {
        let root = root()?;
        let value = root.get_attribute("data-skill")?;
        let _ = root.remove_attribute("data-skill");
        value.parse().ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
pub(super) fn set_camera_lock(locked: bool) {
    #[cfg(target_arch = "wasm32")]
    if let Some(root) = root() {
        let _ = root.set_attribute("data-camera-locked", if locked { "true" } else { "false" });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = locked;
}
pub(super) fn publish(json: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let init = web_sys::CustomEventInit::new();
        init.set_detail(&wasm_bindgen::JsValue::from_str(json));
        if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict("kiva-state", &init) {
            let _ = window.dispatch_event(&event);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = json;
}
