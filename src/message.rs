use crate::models::ShuttleServiceResult;

/// Every mutation that can happen to `App` is expressed as one of these variants.
/// Input translators (`key_to_message`, `mouse_to_message`) produce `Option<Message>`;
/// `App::update()` is the single place that consumes them.
#[derive(Debug)]
pub enum Message {
    // ── Navigation ──────────────────────────────────────────────────────────
    MoveUp,
    MoveDown,
    GoFirst,
    GoLast,
    JumpDigit(char),
    CommitJump,
    CancelJump,

    // ── Search ───────────────────────────────────────────────────────────────
    OpenSearch,
    /// `keep_filter: true` = Enter (keep text); `false` = Esc (clear text).
    CloseSearch {
        keep_filter: bool,
    },
    SearchChar(char),
    SearchBackspace,

    // ── List view ────────────────────────────────────────────────────────────
    ToggleFavourite,
    ToggleFavView,

    // ── Theme ────────────────────────────────────────────────────────────────
    CycleTheme,
    OpenThemePicker,
    CloseThemePicker,
    ThemePickerUp,
    ThemePickerDown,
    ThemePickerApply,

    // ── Language picker ──────────────────────────────────────────────────────
    CloseLangPicker,
    LangPickerUp,
    LangPickerDown,
    LangPickerApply,

    // ── Settings overlay ─────────────────────────────────────────────────────
    OpenSettings,
    CloseSettings,
    SettingsUp,
    SettingsDown,
    SettingsActivateRow,
    SettingsEditChar(char),
    SettingsEditBackspace,
    SettingsEditCancel,
    SettingsEditCommit,

    // ── Mode switching ────────────────────────────────────────────────────────
    SwitchMode,

    // ── Background events ────────────────────────────────────────────────────
    Tick,
    DataReceived {
        stop_name: String,
        data: ShuttleServiceResult,
    },
    FetchError {
        stop_name: String,
        error: String,
    },

    // ── SG background events ─────────────────────────────────────────────────
    SgDataReceived {
        stop_code: String,
        data: crate::models::SgArrivalResult,
    },
    SgFetchError {
        stop_code: String,
        error: String,
    },
    SgStopsLoaded {
        stops: Vec<crate::models::SgBusStop>,
    },
    SgStopsError {
        error: String,
    },

    // ── Mouse ────────────────────────────────────────────────────────────────
    /// Click on an item at visual position `target` in the list.
    ListClick(usize),
    /// Scroll the list viewport up without moving the selection.
    ScrollListUp,
    /// Scroll the list viewport down without moving the selection.
    ScrollListDown,

    // ── Focus ─────────────────────────────────────────────────────────────────
    FocusGained,
    FocusLost,

    // ── Control ──────────────────────────────────────────────────────────────
    RefreshCurrent,
    Quit,
}

impl From<crate::models::AppEvent> for Message {
    fn from(ev: crate::models::AppEvent) -> Self {
        match ev {
            crate::models::AppEvent::Tick => Message::Tick,
            crate::models::AppEvent::DataReceived { stop_name, data } => {
                Message::DataReceived { stop_name, data }
            }
            crate::models::AppEvent::FetchError { stop_name, error } => {
                Message::FetchError { stop_name, error }
            }
            crate::models::AppEvent::SgDataReceived { stop_code, data } => {
                Message::SgDataReceived { stop_code, data }
            }
            crate::models::AppEvent::SgFetchError { stop_code, error } => {
                Message::SgFetchError { stop_code, error }
            }
            crate::models::AppEvent::SgStopsLoaded { stops } => Message::SgStopsLoaded { stops },
            crate::models::AppEvent::SgStopsError { error } => Message::SgStopsError { error },
        }
    }
}
