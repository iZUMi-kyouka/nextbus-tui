#![cfg(target_arch = "wasm32")]

use ratzilla::backend::webgl2::{FontAtlasData, WebGl2BackendOptions};
use ratzilla::WebGl2Backend;

const ATLAS_JA: &[u8] = include_bytes!("../assets/atlas/atlas.ja.atlas");
const ATLAS_ZH_CN: &[u8] = include_bytes!("../assets/atlas/atlas.zh-CN.atlas");
const ATLAS_ZH_TW: &[u8] = include_bytes!("../assets/atlas/atlas.zh-TW.atlas");
const ATLAS_VI: &[u8] = include_bytes!("../assets/atlas/atlas.vi.atlas");

pub(crate) fn create_backend_for_lang(lang: &str) -> Result<WebGl2Backend, String> {
    let Some(bytes) = atlas_bytes_for_lang(lang) else {
        return WebGl2Backend::new().map_err(|e| format!("webgl backend init failed: {e}"));
    };

    match FontAtlasData::from_binary(bytes) {
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

fn atlas_bytes_for_lang(lang: &str) -> Option<&'static [u8]> {
    match lang {
        "ja" => Some(ATLAS_JA),
        "zh-CN" => Some(ATLAS_ZH_CN),
        "zh-TW" => Some(ATLAS_ZH_TW),
        "vi" => Some(ATLAS_VI),
        // Latin-focused locales use ratzilla's built-in default atlas for stable metrics.
        _ => None,
    }
}
