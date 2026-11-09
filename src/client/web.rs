#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = isActive)]
    fn browser_active() -> bool;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = takeLook)]
    fn browser_look() -> f32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = takeAim)]
    fn browser_aim() -> f32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = keys)]
    fn browser_keys() -> u32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = takeActions)]
    fn browser_actions() -> u32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = ready)]
    fn browser_ready();
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
pub(super) fn take_aim() -> f32 {
    browser_aim()
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
    #[cfg(target_arch = "wasm32")]
    browser_ready();
}
