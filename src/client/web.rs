//! Small platform bridge; DOM state never enters the simulation module.
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
fn root() -> Option<web_sys::Element> {
    web_sys::window()?.document()?.document_element()
}

pub(super) fn canvas_size() -> Option<Vec2> {
    #[cfg(target_arch = "wasm32")]
    {
        let canvas = web_sys::window()?.document()?.get_element_by_id("game")?;
        let size = Vec2::new(canvas.client_width() as f32, canvas.client_height() as f32);
        size.cmpgt(Vec2::ZERO).all().then_some(size)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// Camera viewport units may differ from CSS pixels at non-default DPI.
pub(super) fn viewport_to_css(point: Vec2, viewport: Vec2) -> Vec2 {
    point * canvas_size().unwrap_or(viewport) / viewport
}
pub(super) fn css_to_viewport(point: Vec2, viewport: Vec2) -> Vec2 {
    point * viewport / canvas_size().unwrap_or(viewport)
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
pub(super) fn pointer_captured() -> Option<bool> {
    #[cfg(target_arch = "wasm32")]
    {
        Some(
            web_sys::window()?
                .document()?
                .pointer_lock_element()
                .is_some_and(|element| element.id() == "game"),
        )
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
pub(super) fn release_pointer() {
    #[cfg(target_arch = "wasm32")]
    if pointer_captured() == Some(true)
        && let Some(document) = web_sys::window().and_then(|window| window.document())
    {
        document.exit_pointer_lock();
    }
}
pub(super) fn click_ui(point: Vec2) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        if let Some(button) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.element_from_point(point.x, point.y))
            .and_then(|element| element.closest("[data-ui] button").ok().flatten())
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        {
            button.click();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = point;
}
pub(super) fn draw_pointer(point: Option<Vec2>) {
    #[cfg(target_arch = "wasm32")]
    if let Some(document) = web_sys::window().and_then(|window| window.document())
        && let Some(cursor) = document.get_element_by_id("game-cursor")
    {
        if let Some(point) = point.filter(|_| pointer_captured() == Some(true)) {
            let size = document
                .document_element()
                .map(|root| Vec2::new(root.client_width() as f32, root.client_height() as f32))
                .unwrap_or(Vec2::ZERO);
            let flip_x = if point.x > size.x - 24.0 { -1 } else { 1 };
            let flip_y = if point.y > size.y - 24.0 { -1 } else { 1 };
            let _ = cursor.set_attribute(
                "style",
                &format!(
                    "transform:translate3d({}px,{}px,0);--flip-x:{flip_x};--flip-y:{flip_y}",
                    point.x, point.y
                ),
            );
            let _ = cursor.remove_attribute("hidden");
            let hovered = document
                .element_from_point(point.x, point.y)
                .and_then(|element| element.closest("[data-ui] button").ok().flatten());
            if let Ok(Some(previous)) = document.query_selector("[data-virtual-hover]")
                && hovered.as_ref() != Some(&previous)
            {
                let _ = previous.remove_attribute("data-virtual-hover");
            }
            if let Some(button) = hovered {
                let _ = button.set_attribute("data-virtual-hover", "");
            }
        } else {
            let _ = cursor.set_attribute("hidden", "");
            if let Ok(Some(previous)) = document.query_selector("[data-virtual-hover]") {
                let _ = previous.remove_attribute("data-virtual-hover");
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = point;
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
        let value = if paused { "true" } else { "false" };
        if root.get_attribute("data-paused").as_deref() != Some(value) {
            let _ = root.set_attribute("data-paused", value);
        }
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
    dispatch_json("kiva-state", json);
}
#[cfg(target_arch = "wasm32")]
pub(super) fn publish_labels(json: &str) {
    dispatch_json("kiva-labels", json);
}
fn dispatch_json(name: &str, json: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        let init = web_sys::CustomEventInit::new();
        init.set_detail(&wasm_bindgen::JsValue::from_str(json));
        if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict(name, &init) {
            let _ = window.dispatch_event(&event);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (name, json);
}
