use std::collections::HashMap;

use fluent::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

/// Per-language metadata loaded from `assets/i18n/config.toml`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LangMeta {
    /// Name in the language itself, e.g. "日本語"
    pub native_name: String,
}

#[derive(serde::Deserialize)]
struct I18nConfigFile {
    languages: HashMap<String, LangMeta>,
}

static I18N_CONFIG: &str = include_str!("../assets/i18n/config.toml");
static FTL_EN: &str = include_str!("../assets/i18n/en/main.ftl");
static FTL_JA: &str = include_str!("../assets/i18n/ja/main.ftl");
static FTL_ZH_CN: &str = include_str!("../assets/i18n/zh-CN/main.ftl");
static FTL_ZH_TW: &str = include_str!("../assets/i18n/zh-TW/main.ftl");
static FTL_MS: &str = include_str!("../assets/i18n/ms/main.ftl");
static FTL_TA: &str = include_str!("../assets/i18n/ta/main.ftl");
static FTL_VI: &str = include_str!("../assets/i18n/vi/main.ftl");

/// Language codes in selection order (used when cycling through languages).
pub const LANGUAGES: &[&str] = &["en", "zh-CN", "zh-TW", "ja", "ms", "ta", "vi"];

/// Active i18n bundle: holds the Fluent message bundle for the current language.
pub struct I18n {
    /// Active language code, e.g. "en" or "ja".
    pub lang: String,
    /// Metadata for the active language (font, native name, …).
    pub lang_meta: LangMeta,
    bundle: FluentBundle<FluentResource>,
}

impl I18n {
    /// Build an `I18n` for `lang`. Falls back to `"en"` for unknown codes.
    pub fn new(lang: &str) -> Self {
        let config: I18nConfigFile =
            toml::from_str(I18N_CONFIG).expect("assets/i18n/config.toml is invalid");
        let mut all_meta = config.languages;

        // Fall back to English for unrecognised language codes.
        let lang = if all_meta.contains_key(lang) {
            lang
        } else {
            "en"
        };
        let lang_meta = all_meta
            .remove(lang)
            .expect("'en' must exist in i18n config");

        let ftl_src = match lang {
            "ja" => FTL_JA,
            "zh-CN" => FTL_ZH_CN,
            "zh-TW" => FTL_ZH_TW,
            "ms" => FTL_MS,
            "ta" => FTL_TA,
            "vi" => FTL_VI,
            _ => FTL_EN,
        };

        let langid: LanguageIdentifier = lang.parse().unwrap_or_else(|_| "en".parse().unwrap());
        let resource =
            FluentResource::try_new(ftl_src.to_owned()).expect("failed to parse FTL source");
        let mut bundle = FluentBundle::new(vec![langid]);
        // Disable Unicode bidi isolation marks — they corrupt terminal output.
        bundle.set_use_isolating(false);
        bundle
            .add_resource(resource)
            .expect("failed to add FTL resource to bundle");

        Self {
            lang: lang.to_owned(),
            lang_meta,
            bundle,
        }
    }

    /// Translate `id` with no variable substitutions.
    pub fn t(&self, id: &str) -> String {
        self.format(id, None)
    }

    /// Translate `id` substituting `args` for any `{ $var }` placeables.
    pub fn t_args(&self, id: &str, args: &FluentArgs<'_>) -> String {
        self.format(id, Some(args))
    }

    /// Returns the language code that follows the current one in the cycle.
    pub fn next_lang(&self) -> &str {
        let pos = LANGUAGES.iter().position(|&l| l == self.lang).unwrap_or(0);
        LANGUAGES[(pos + 1) % LANGUAGES.len()]
    }

    fn format(&self, id: &str, args: Option<&FluentArgs<'_>>) -> String {
        let msg = match self.bundle.get_message(id) {
            Some(m) => m,
            None => return format!("[{id}]"),
        };
        let pattern = match msg.value() {
            Some(p) => p,
            None => return format!("[{id}]"),
        };
        let mut errors = Vec::new();
        self.bundle
            .format_pattern(pattern, args, &mut errors)
            .into_owned()
    }
}
