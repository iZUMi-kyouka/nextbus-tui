#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::models::Config;

#[cfg(target_arch = "wasm32")]
const WEB_CONFIG_KEY: &str = "nextbus-tui/config.json";

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Config {
    config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn load() -> Config {
    let Some(window) = web_sys::window() else {
        return Config::default();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Config::default();
    };
    storage
        .get_item(WEB_CONFIG_KEY)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save(config: &Config) {
    let Some(path) = config_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(path, json);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn save(config: &Config) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = storage.set_item(WEB_CONFIG_KEY, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("nextbus-tui").join("config.json"))
}
