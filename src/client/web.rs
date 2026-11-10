#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaPage"], js_name = isViewer)]
    fn browser_viewer_mode() -> bool;
    #[wasm_bindgen(js_namespace = ["window", "kivaViewer"], js_name = takeCommands)]
    fn browser_viewer_commands() -> String;
    #[wasm_bindgen(js_namespace = ["window", "kivaViewer"], js_name = render)]
    fn browser_viewer_render(snapshot: &str);
    #[wasm_bindgen(js_namespace = ["window", "kivaViewer"], js_name = ready)]
    fn browser_viewer_ready();
    #[wasm_bindgen(js_namespace = ["window", "kivaViewer"], js_name = fail)]
    fn browser_viewer_fail(reason: &str);
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = isActive)]
    fn browser_active() -> bool;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = takeLook)]
    fn browser_look() -> f32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = takeDefenseEdges)]
    fn browser_defense_edges() -> Vec<u8>;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = keys)]
    fn browser_keys() -> u32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = takeActions)]
    fn browser_actions() -> u32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = ready)]
    fn browser_ready();
    #[wasm_bindgen(js_namespace = ["window", "kivaStartup"], js_name = progress)]
    fn browser_progress(phase: &str, loaded: u32, total: u32);
    #[wasm_bindgen(js_namespace = ["window", "kivaStartup"], js_name = fail)]
    fn browser_fail(reason: &str);
}

#[cfg(target_arch = "wasm32")]
pub(super) fn active() -> bool {
    browser_active()
}
#[cfg(target_arch = "wasm32")]
pub(super) fn take_look() -> f32 {
    browser_look()
}
#[cfg(target_arch = "wasm32")]
pub(super) fn take_defense_edges() -> Vec<u8> {
    browser_defense_edges()
}
#[cfg(target_arch = "wasm32")]
pub(super) fn keys() -> u32 {
    browser_keys()
}

#[cfg(target_arch = "wasm32")]
pub(super) fn take_actions() -> u32 {
    browser_actions()
}

pub(super) fn ready() {
    #[cfg(test)]
    STARTUP_NOTIFICATIONS.with(|counts| counts.set((counts.get().0 + 1, counts.get().1)));
    #[cfg(target_arch = "wasm32")]
    if viewer_mode() {
        browser_viewer_ready();
    } else {
        browser_ready();
    }
}

pub(super) fn progress(_phase: &str, _loaded: u32, _total: u32) {
    #[cfg(target_arch = "wasm32")]
    browser_progress(_phase, _loaded, _total);
}

pub(super) fn fail(_reason: &str) {
    #[cfg(test)]
    STARTUP_NOTIFICATIONS.with(|counts| counts.set((counts.get().0, counts.get().1 + 1)));
    #[cfg(target_arch = "wasm32")]
    browser_fail(_reason);
}

#[cfg(test)]
thread_local! {
    static STARTUP_NOTIFICATIONS: std::cell::Cell<(u32, u32)> = const { std::cell::Cell::new((0, 0)) };
}

#[cfg(test)]
pub(super) fn take_startup_notifications() -> (u32, u32) {
    STARTUP_NOTIFICATIONS.with(|counts| counts.replace((0, 0)))
}

/// Page mode is fixed before the App and any game browser listeners exist.
pub(super) fn viewer_mode() -> bool {
    #[cfg(target_arch = "wasm32")]
    return browser_viewer_mode();
    #[cfg(not(target_arch = "wasm32"))]
    false
}

pub(super) fn viewer_take_commands() -> String {
    #[cfg(target_arch = "wasm32")]
    return browser_viewer_commands();
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

pub(super) fn viewer_render(_snapshot: &str) {
    #[cfg(target_arch = "wasm32")]
    browser_viewer_render(_snapshot);
}

pub(super) fn viewer_fail(_reason: &str) {
    #[cfg(target_arch = "wasm32")]
    browser_viewer_fail(_reason);
}
