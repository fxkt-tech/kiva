#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
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
    #[cfg(target_arch = "wasm32")]
    browser_ready();
}
