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

    // ── Mouse ────────────────────────────────────────────────────────────────
    /// Click on an item at visual position `target` in the list.
    ListClick(usize),

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
        }
    }
}
