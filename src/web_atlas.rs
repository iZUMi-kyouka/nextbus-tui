#![cfg(target_arch = "wasm32")]

use ratzilla::backend::webgl2::{FontAtlasData, WebGl2BackendOptions};
use ratzilla::WebGl2Backend;

// This file is generated/replaced by atlas tooling.
const CUSTOM_ATLAS_BYTES: &[u8] = include_bytes!("../assets/atlas/multilang.atlas");

pub(crate) fn create_backend() -> Result<WebGl2Backend, String> {
    if CUSTOM_ATLAS_BYTES.is_empty() {
        return WebGl2Backend::new().map_err(|e| format!("webgl backend init failed: {e}"));
    }

    match FontAtlasData::from_binary(CUSTOM_ATLAS_BYTES) {
        Ok(atlas) => {
            let opts = WebGl2BackendOptions::new().font_atlas(atlas);
            WebGl2Backend::new_with_options(opts)
                .map_err(|e| format!("webgl backend init with custom atlas failed: {e}"))
        }
        Err(e) => {
            eprintln!("custom atlas parse failed, falling back to default atlas: {e:?}");
            WebGl2Backend::new().map_err(|err| format!("webgl backend init failed: {err}"))
        }
    }
}
