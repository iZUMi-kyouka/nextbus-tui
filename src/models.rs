#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::theme::ThemeMode;

/// A bus stop as defined in stops.toml
#[derive(Debug, Clone, Deserialize)]
pub struct BusStop {
    pub caption: String,
    pub name: String,
    pub long_name: String,
    pub short_name: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// Top-level API response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct ApiResponse {
    #[serde(rename = "ShuttleServiceResult")]
    pub result: ShuttleServiceResult,
}

/// Payload inside ShuttleServiceResult
#[derive(Debug, Clone, Deserialize)]
pub struct ShuttleServiceResult {
    pub name: Option<String>,
    pub caption: Option<String>,
    pub shuttles: Vec<Shuttle>,
    #[serde(rename = "TimeStamp")]
    pub timestamp: Option<String>,
}

/// A single shuttle bus entry
#[derive(Debug, Clone, Deserialize)]
pub struct Shuttle {
    /// Bus service name, e.g. "D1", "A2"
    pub name: String,
    /// Unique code per direction, e.g. "COM3-D1-E"
    #[serde(rename = "busstopcode")]
    pub busstopcode: Option<String>,
    /// Next arrival in minutes, or "Arr" / "-"
    #[serde(rename = "arrivalTime")]
    pub arrival_time: String,
    /// Second arrival in minutes, or "Arr" / "-"
    #[serde(rename = "nextArrivalTime")]
    pub next_arrival_time: String,
    #[serde(rename = "arrivalTime_veh_plate")]
    pub arrival_plate: Option<String>,
    #[serde(rename = "nextArrivalTime_veh_plate")]
    pub next_plate: Option<String>,
    pub passengers: Option<String>,
    #[serde(rename = "nextPassengers")]
    pub next_passengers: Option<String>,
}

/// App mode: NUS campus shuttles or SG public bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppMode {
    #[default]
    NusCampus,
    SgPublicBus,
}

/// A SG public bus stop from the LTA DataMall BusStops endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SgBusStop {
    pub code: String,
    pub road_name: String,
    pub description: String,
    pub latitude: f64,
    pub longitude: f64,
}

/// Crowding level of an incoming bus.
#[derive(Debug, Clone, PartialEq)]
pub enum BusLoad {
    SeatsAvailable,
    StandingAvailable,
    LimitedStanding,
    Unknown,
}

/// Deck configuration of an incoming bus.
#[derive(Debug, Clone, PartialEq)]
pub enum BusType {
    SingleDeck,
    DoubleDeck,
    Bendy,
    Unknown,
}

/// Accessibility feature of an incoming bus.
#[derive(Debug, Clone, PartialEq)]
pub enum BusFeature {
    WheelchairAccessible,
    Standard,
}

/// Bus operator identifier.
#[derive(Debug, Clone, PartialEq)]
pub enum BusOperator {
    Sbst,
    Smrt,
    Tts,
    Gas,
    Unknown(String),
}

/// Real-time arrival info for one incoming bus.
#[derive(Debug, Clone)]
pub struct SgBusArrival {
    pub estimated_arrival: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub monitored: bool,
    pub load: BusLoad,
    pub feature: BusFeature,
    pub bus_type: BusType,
}

/// One service (route) at a bus stop, with up to three incoming buses.
#[derive(Debug, Clone)]
pub struct SgService {
    pub service_no: String,
    pub operator: BusOperator,
    pub next: Option<SgBusArrival>,
    pub next2: Option<SgBusArrival>,
    pub next3: Option<SgBusArrival>,
}

/// Full arrival result for one bus stop code.
#[derive(Debug, Clone)]
pub struct SgArrivalResult {
    pub bus_stop_code: String,
    pub services: Vec<SgService>,
}

/// Internal event bus between background threads and the main loop
pub enum AppEvent {
    Tick,
    DataReceived {
        stop_name: String,
        data: ShuttleServiceResult,
    },
    FetchError {
        stop_name: String,
        error: String,
    },
    SgDataReceived {
        stop_code: String,
        data: SgArrivalResult,
    },
    SgFetchError {
        stop_code: String,
        error: String,
    },
    SgStopsLoaded {
        stops: Vec<SgBusStop>,
    },
    SgStopsError {
        error: String,
    },
    TrainAlertsReceived {
        disrupted: bool,
        summary: String,
    },
    TrainAlertsFetchError {
        error: String,
    },
}

/// A single stop along a bus route
#[derive(Debug, Clone, Deserialize)]
pub struct RouteStop {
    pub seq: u32,
    pub stop_name: String,
    pub busstopcode: String,
}

/// A bus route with its colour and ordered stop list
#[derive(Debug, Clone, Deserialize)]
pub struct Route {
    pub name: String,
    /// Hex colour string, e.g. "#fe0000"
    pub color: String,
    pub stops: Vec<RouteStop>,
}

fn default_refresh_interval() -> u64 {
    20
}

fn default_language() -> String {
    "en".to_owned()
}

fn default_theme_mode() -> ThemeMode {
    ThemeMode::Dark
}

fn default_theme_idx() -> usize {
    0
}

fn default_app_mode() -> AppMode {
    AppMode::NusCampus
}

/// Persisted user configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub favourites: Vec<String>,
    #[serde(default)]
    pub sg_favourites: Vec<String>,
    /// Auto-refresh interval in seconds (5–300).
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
    /// Whether the app opens in favourites-only view by default.
    #[serde(default)]
    pub default_fav_view: bool,
    /// UI language code, e.g. "en" or "ja".
    #[serde(default = "default_language")]
    pub language: String,
    /// Theme mode: dark, light, or auto (switches based on local time).
    #[serde(default = "default_theme_mode")]
    pub theme_mode: ThemeMode,
    /// Index of the active theme in the themes list.
    #[serde(default = "default_theme_idx")]
    pub theme_idx: usize,
    /// Default app mode on startup.
    #[serde(default = "default_app_mode")]
    pub default_mode: AppMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            favourites: Vec::new(),
            sg_favourites: Vec::new(),
            refresh_interval_secs: default_refresh_interval(),
            default_fav_view: false,
            language: default_language(),
            theme_mode: default_theme_mode(),
            theme_idx: default_theme_idx(),
            default_mode: default_app_mode(),
        }
    }
}
