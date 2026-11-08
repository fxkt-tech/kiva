#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = isActive)]
    fn browser_active() -> bool;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = takeLook)]
    fn browser_look() -> f32;
    #[wasm_bindgen(js_namespace = ["window", "kivaInput"], js_name = keys)]
    fn browser_keys() -> u8;
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
pub(super) fn keys() -> u8 {
    browser_keys()
}

pub(super) fn ready() {
    #[cfg(target_arch = "wasm32")]
    browser_ready();
}
