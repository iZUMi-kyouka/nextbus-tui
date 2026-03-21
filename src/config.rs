#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::models::Config;
use crate::models::SgBusStop;

#[cfg(target_arch = "wasm32")]
const WEB_CONFIG_KEY: &str = "nextbus-tui/config.json";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SgStopsCache {
    pub cached_at: String, // RFC 3339
    pub stops: Vec<SgBusStop>,
}

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

#[cfg(not(target_arch = "wasm32"))]
pub fn sg_stops_path() -> Option<PathBuf> {
    dirs::config_dir().map(|base| base.join("nextbus-tui").join("sg_stops.json"))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_sg_stops() -> Option<SgStopsCache> {
    let path = sg_stops_path()?;
    let s = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&s).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_sg_stops(stops: &[SgBusStop]) {
    let Some(path) = sg_stops_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let cache = SgStopsCache {
        cached_at: chrono::Utc::now().to_rfc3339(),
        stops: stops.to_vec(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&cache) {
        let _ = std::fs::write(path, json);
    }
}
