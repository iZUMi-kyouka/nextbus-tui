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

// ── Locale detection ──────────────────────────────────────────────────────────

/// Map a raw locale/BCP-47 tag to one of the supported language codes.
/// Falls back to `"en"` for anything unrecognised.
pub fn detect_language_from_locale(locale: &str) -> &'static str {
    // Normalise: strip encoding suffix (e.g. ".UTF-8") and take first subtag pair.
    let bare = locale.split('.').next().unwrap_or(locale);
    // Normalise underscores to hyphens for BCP-47 comparison.
    // We operate on the bare slice directly to avoid allocation where possible.
    let (lang, region) = if let Some(pos) = bare.find(['_', '-']) {
        (&bare[..pos], bare[pos + 1..].to_ascii_uppercase())
    } else {
        (bare, String::new())
    };

    match lang.to_ascii_lowercase().as_str() {
        "zh" => match region.as_str() {
            "TW" | "HK" | "MO" | "HANT" => "zh-TW",
            _ => "zh-CN", // zh, zh-CN, zh-Hans, zh-SG → Simplified
        },
        "ja" => "ja",
        "ms" => "ms",
        "ta" => "ta",
        "vi" => "vi",
        _ => "en",
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn system_locale() -> String {
    // Standard Unix locale env vars, checked in priority order.
    // `LANGUAGE` can be colon-separated (e.g. "zh_CN:zh:en"); take the first entry.
    for var in &["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            let first = val.split(':').next().unwrap_or("").trim().to_owned();
            if !first.is_empty() && first != "C" && first != "POSIX" {
                return first;
            }
        }
    }
    String::new()
}

// ── Config load / save ────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub fn load() -> Config {
    // If the config file already exists, load it as-is (user's saved preference).
    if let Some(path) = config_path() {
        if let Ok(s) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<Config>(&s) {
                return cfg;
            }
        }
    }
    // First startup — apply locale detection so the UI opens in the user's language.
    let mut cfg = Config::default();
    let locale = system_locale();
    if !locale.is_empty() {
        cfg.language = detect_language_from_locale(&locale).to_owned();
    }
    cfg
}

#[cfg(target_arch = "wasm32")]
pub fn load() -> Config {
    let Some(window) = web_sys::window() else {
        return Config::default();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Config::default();
    };
    // If the config is already in localStorage, respect it.
    if let Some(json) = storage.get_item(WEB_CONFIG_KEY).ok().flatten() {
        if let Ok(cfg) = serde_json::from_str::<Config>(&json) {
            return cfg;
        }
    }
    // First visit — detect language from the browser.
    let mut cfg = Config::default();
    let locale = window.navigator().language().unwrap_or_default();
    if !locale.is_empty() {
        cfg.language = detect_language_from_locale(&locale).to_owned();
    }
    cfg
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

#[cfg(test)]
mod tests {
    use super::detect_language_from_locale;

    #[test]
    fn detect_english_fallback() {
        assert_eq!(detect_language_from_locale("en_US.UTF-8"), "en");
        assert_eq!(detect_language_from_locale("en"), "en");
        assert_eq!(detect_language_from_locale("fr_FR.UTF-8"), "en");
        assert_eq!(detect_language_from_locale("de"), "en");
    }

    #[test]
    fn detect_japanese() {
        assert_eq!(detect_language_from_locale("ja_JP.UTF-8"), "ja");
        assert_eq!(detect_language_from_locale("ja"), "ja");
    }

    #[test]
    fn detect_simplified_chinese() {
        assert_eq!(detect_language_from_locale("zh_CN.UTF-8"), "zh-CN");
        assert_eq!(detect_language_from_locale("zh_SG.UTF-8"), "zh-CN");
        assert_eq!(detect_language_from_locale("zh-Hans"), "zh-CN");
        assert_eq!(detect_language_from_locale("zh"), "zh-CN");
    }

    #[test]
    fn detect_traditional_chinese() {
        assert_eq!(detect_language_from_locale("zh_TW.UTF-8"), "zh-TW");
        assert_eq!(detect_language_from_locale("zh_HK.UTF-8"), "zh-TW");
        assert_eq!(detect_language_from_locale("zh-Hant"), "zh-TW");
    }

    #[test]
    fn detect_malay() {
        assert_eq!(detect_language_from_locale("ms_MY.UTF-8"), "ms");
        assert_eq!(detect_language_from_locale("ms"), "ms");
    }

    #[test]
    fn detect_tamil() {
        assert_eq!(detect_language_from_locale("ta_IN.UTF-8"), "ta");
        assert_eq!(detect_language_from_locale("ta_SG.UTF-8"), "ta");
    }

    #[test]
    fn detect_vietnamese() {
        assert_eq!(detect_language_from_locale("vi_VN.UTF-8"), "vi");
        assert_eq!(detect_language_from_locale("vi"), "vi");
    }

    #[test]
    fn detect_bcp47_browser_tags() {
        // Browser navigator.language returns BCP-47 tags
        assert_eq!(detect_language_from_locale("zh-CN"), "zh-CN");
        assert_eq!(detect_language_from_locale("zh-TW"), "zh-TW");
        assert_eq!(detect_language_from_locale("ja"), "ja");
        assert_eq!(detect_language_from_locale("ms-MY"), "ms");
        assert_eq!(detect_language_from_locale("en-SG"), "en");
        assert_eq!(detect_language_from_locale("en-GB"), "en");
    }

    #[test]
    fn detect_empty_locale_returns_default() {
        assert_eq!(detect_language_from_locale(""), "en");
        assert_eq!(detect_language_from_locale("C"), "en");
    }
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
