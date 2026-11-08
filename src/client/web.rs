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

/// The browser can acquire the native lock before it dispatches the change
/// event. Only that event lets the UI submit Launch or finish a resume.
#[derive(Clone, Copy, Default)]
pub(super) struct PointerCapture {
    pub actual: bool,
    pub event_confirmed: bool,
}
impl PointerCapture {
    pub fn confirmed(self) -> bool {
        self.actual && self.event_confirmed
    }
}
pub(super) fn pointer_capture() -> Option<PointerCapture> {
    #[cfg(target_arch = "wasm32")]
    {
        Some(PointerCapture {
            actual: pointer_captured()?,
            event_confirmed: root()?.get_attribute("data-pointer-locked").as_deref()
                == Some("true"),
        })
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
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaPointer"], js_name = sync)]
    fn browser_pointer_sync(active: bool);
    #[wasm_bindgen(js_namespace = ["window", "kivaPointer"], js_name = position)]
    fn browser_pointer_position() -> Vec<f32>;
    #[wasm_bindgen(js_namespace = ["window", "kivaPointer"], js_name = takePress)]
    fn browser_pointer_press(button: u16) -> Vec<f32>;
    #[wasm_bindgen(js_namespace = ["window", "kivaPointer"], js_name = clearPresses)]
    fn browser_pointer_clear();
}

#[cfg(target_arch = "wasm32")]
fn pointer_point(values: Vec<f32>) -> Option<Vec2> {
    let [x, y] = values.as_slice() else {
        return None;
    };
    let point = Vec2::new(*x, *y);
    point.is_finite().then_some(point)
}

pub(super) fn sync_game_pointer(active: bool) {
    #[cfg(target_arch = "wasm32")]
    browser_pointer_sync(active);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = active;
}

pub(super) fn game_pointer_position() -> Option<Vec2> {
    #[cfg(target_arch = "wasm32")]
    {
        pointer_point(browser_pointer_position())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

pub(super) struct PointerPress {
    pub position: Vec2,
    pub captured_by_ui: bool,
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_pointer_press(values: &[f32]) -> Option<PointerPress> {
    let [x, y, captured_by_ui] = *values else {
        return None;
    };
    let position = Vec2::new(x, y);
    if !position.is_finite() || (captured_by_ui != 0.0 && captured_by_ui != 1.0) {
        return None;
    }
    Some(PointerPress {
        position,
        captured_by_ui: captured_by_ui == 1.0,
    })
}

pub(super) fn take_pointer_press(button: MouseButton) -> Option<PointerPress> {
    #[cfg(target_arch = "wasm32")]
    {
        let button = match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            MouseButton::Back => 3,
            MouseButton::Forward => 4,
            MouseButton::Other(value) => value,
        };
        decode_pointer_press(&browser_pointer_press(button))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = button;
        None
    }
}

pub(super) fn clear_pointer_presses() {
    #[cfg(target_arch = "wasm32")]
    browser_pointer_clear();
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
pub(super) fn take_session_requests() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let root = root()?;
        let value = root.get_attribute("data-session-requests")?;
        let _ = root.remove_attribute("data-session-requests");
        Some(value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
pub(super) fn take_equipment_requests() -> Option<String> {
    take_attribute("data-equipment-requests")
}
pub(super) fn take_equipment_skill_request() -> Option<String> {
    take_attribute("data-equipment-skill")
}
/// Latest minimap navigation intent; consuming also discards rejected/stale input.
pub(super) fn take_camera_request() -> Option<String> {
    take_attribute("data-camera-request")
}
pub(super) fn minimap_dragging() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        root().is_some_and(|root| {
            root.get_attribute("data-minimap-dragging").as_deref() == Some("true")
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}
fn take_attribute(name: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let root = root()?;
        let value = root.get_attribute(name)?;
        let _ = root.remove_attribute(name);
        Some(value)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = name;
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
#[cfg(target_arch = "wasm32")]
pub(super) fn publish_camera(json: &str) {
    dispatch_json("kiva-camera", json);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_bridge_preserves_position_and_event_time_ui_ownership() {
        for captured_by_ui in [false, true] {
            let press = decode_pointer_press(&[120.5, 72.25, f32::from(captured_by_ui)]).unwrap();
            assert_eq!(press.position, Vec2::new(120.5, 72.25));
            assert_eq!(press.captured_by_ui, captured_by_ui);
        }
        for values in [
            &[][..],
            &[1.0, 2.0][..],
            &[1.0, 2.0, 0.0, 1.0][..],
            &[f32::NAN, 2.0, 0.0][..],
            &[1.0, f32::INFINITY, 0.0][..],
            &[1.0, 2.0, f32::NAN][..],
            &[1.0, 2.0, 2.0][..],
        ] {
            assert!(decode_pointer_press(values).is_none());
        }
    }
}
