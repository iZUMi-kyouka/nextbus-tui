#![allow(dead_code)]

use serde::{Deserialize, Serialize};

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
    30
}

fn default_language() -> String {
    "en".to_owned()
}

/// Persisted user configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub favourites: Vec<String>,
    /// Auto-refresh interval in seconds (5–300).
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
    /// Whether the app opens in favourites-only view by default.
    #[serde(default)]
    pub default_fav_view: bool,
    /// UI language code, e.g. "en" or "ja".
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            favourites: Vec::new(),
            refresh_interval_secs: default_refresh_interval(),
            default_fav_view: false,
            language: default_language(),
        }
    }
}
