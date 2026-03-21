//! LTA DataMall proxy client. No API key in the app — key is injected server-side.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::mpsc;

use crate::models::{
    AppEvent, BusFeature, BusLoad, BusOperator, BusType, SgArrivalResult, SgBusArrival, SgBusStop,
    SgService,
};

fn sg_api_base() -> String {
    std::env::var("SG_API_BASE").unwrap_or_else(|_| "https://nusbus.flovt.net".to_string())
}

/// Fetch real-time arrivals for one bus stop.
pub fn fetch_sg_arrival(stop_code: &str) -> Result<SgArrivalResult, String> {
    let url = format!("{}/BusArrival?BusStopCode={}", sg_api_base(), stop_code);
    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Network error: {e}"))?;
    let reader = response.into_reader();
    parse_arrival_response(reader, stop_code)
}

/// Fetch one page of bus stops (up to 500).
pub fn fetch_sg_stops(skip: usize) -> Result<Vec<SgBusStop>, String> {
    let url = format!("{}/BusStops?$skip={}", sg_api_base(), skip);
    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Network error: {e}"))?;
    let reader = response.into_reader();
    parse_stops_response(reader)
}

/// Paginate and fetch all bus stops, sending progress events via the channel.
pub fn fetch_all_sg_stops(tx: mpsc::Sender<AppEvent>) {
    let mut all: Vec<SgBusStop> = Vec::new();
    let mut skip = 0;
    loop {
        match fetch_sg_stops(skip) {
            Ok(batch) => {
                let done = batch.len() < 500;
                all.extend(batch);
                let _ = tx.send(AppEvent::SgStopsLoaded { stops: all.clone() });
                if done {
                    break;
                }
                skip += 500;
            }
            Err(e) => {
                let _ = tx.send(AppEvent::SgStopsError { error: e });
                break;
            }
        }
    }
}

// ── JSON parsing ──────────────────────────────────────────────────────────────

fn parse_arrival_response<R: std::io::Read>(
    reader: R,
    stop_code: &str,
) -> Result<SgArrivalResult, String> {
    let v: serde_json::Value =
        serde_json::from_reader(reader).map_err(|e| format!("Parse error: {e}"))?;

    let services_arr = v
        .get("Services")
        .and_then(|s| s.as_array())
        .ok_or_else(|| "Missing Services array".to_string())?;

    let mut services = Vec::new();
    for svc in services_arr {
        let service_no = svc
            .get("ServiceNo")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let operator = parse_operator(svc.get("Operator").and_then(|s| s.as_str()).unwrap_or(""));
        let next = parse_bus_arrival(svc.get("NextBus"));
        let next2 = parse_bus_arrival(svc.get("NextBus2"));
        let next3 = parse_bus_arrival(svc.get("NextBus3"));
        services.push(SgService {
            service_no,
            operator,
            next,
            next2,
            next3,
        });
    }

    Ok(SgArrivalResult {
        bus_stop_code: stop_code.to_string(),
        services,
    })
}

fn parse_bus_arrival(v: Option<&serde_json::Value>) -> Option<SgBusArrival> {
    let v = v?;
    let eta_str = v
        .get("EstimatedArrival")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let estimated_arrival = chrono::DateTime::parse_from_rfc3339(eta_str).ok();
    // Latitude != "0" means GPS-monitored
    let lat_str = v.get("Latitude").and_then(|s| s.as_str()).unwrap_or("0");
    let monitored = lat_str != "0" && !lat_str.is_empty() && lat_str != "0.0";
    let load = parse_load(v.get("Load").and_then(|s| s.as_str()).unwrap_or(""));
    let feature = parse_feature(v.get("Feature").and_then(|s| s.as_str()).unwrap_or(""));
    let bus_type = parse_bus_type(v.get("Type").and_then(|s| s.as_str()).unwrap_or(""));
    Some(SgBusArrival {
        estimated_arrival,
        monitored,
        load,
        feature,
        bus_type,
    })
}

fn parse_stops_response<R: std::io::Read>(reader: R) -> Result<Vec<SgBusStop>, String> {
    let v: serde_json::Value =
        serde_json::from_reader(reader).map_err(|e| format!("Parse error: {e}"))?;

    let arr = v
        .get("value")
        .and_then(|a| a.as_array())
        .ok_or_else(|| "Missing value array".to_string())?;

    let mut stops = Vec::new();
    for item in arr {
        let code = item
            .get("BusStopCode")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let road_name = item
            .get("RoadName")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let description = item
            .get("Description")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let latitude = item.get("Latitude").and_then(|n| n.as_f64()).unwrap_or(0.0);
        let longitude = item
            .get("Longitude")
            .and_then(|n| n.as_f64())
            .unwrap_or(0.0);
        if !code.is_empty() {
            stops.push(SgBusStop {
                code,
                road_name,
                description,
                latitude,
                longitude,
            });
        }
    }
    Ok(stops)
}

fn parse_load(s: &str) -> BusLoad {
    match s {
        "SEA" => BusLoad::SeatsAvailable,
        "SDA" => BusLoad::StandingAvailable,
        "LSD" => BusLoad::LimitedStanding,
        _ => BusLoad::Unknown,
    }
}

fn parse_feature(s: &str) -> BusFeature {
    if s == "WAB" {
        BusFeature::WheelchairAccessible
    } else {
        BusFeature::Standard
    }
}

fn parse_bus_type(s: &str) -> BusType {
    match s {
        "SD" => BusType::SingleDeck,
        "DD" => BusType::DoubleDeck,
        "BD" => BusType::Bendy,
        _ => BusType::Unknown,
    }
}

fn parse_operator(s: &str) -> BusOperator {
    match s {
        "SBST" => BusOperator::Sbst,
        "SMRT" => BusOperator::Smrt,
        "TTS" => BusOperator::Tts,
        "GAS" => BusOperator::Gas,
        other => BusOperator::Unknown(other.to_string()),
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_load_sea() {
        assert_eq!(parse_load("SEA"), BusLoad::SeatsAvailable);
    }

    #[test]
    fn parse_load_sda() {
        assert_eq!(parse_load("SDA"), BusLoad::StandingAvailable);
    }

    #[test]
    fn parse_load_lsd() {
        assert_eq!(parse_load("LSD"), BusLoad::LimitedStanding);
    }

    #[test]
    fn parse_feature_wab() {
        assert_eq!(parse_feature("WAB"), BusFeature::WheelchairAccessible);
    }

    #[test]
    fn parse_feature_other() {
        assert_eq!(parse_feature("N/A"), BusFeature::Standard);
    }

    #[test]
    fn parse_bus_type_sd() {
        assert_eq!(parse_bus_type("SD"), BusType::SingleDeck);
    }

    #[test]
    fn parse_bus_type_dd() {
        assert_eq!(parse_bus_type("DD"), BusType::DoubleDeck);
    }

    #[test]
    fn parse_bus_type_bd() {
        assert_eq!(parse_bus_type("BD"), BusType::Bendy);
    }

    #[test]
    fn parse_arrival_from_sample_json() {
        let json = r#"{
            "BusStopCode": "83139",
            "Services": [
                {
                    "ServiceNo": "15",
                    "Operator": "GAS",
                    "NextBus": {
                        "OriginCode": "77009",
                        "DestinationCode": "77009",
                        "EstimatedArrival": "2017-04-29T07:20:24+08:00",
                        "Latitude": "1.314452211",
                        "Longitude": "103.910587000",
                        "VisitNumber": "1",
                        "Load": "SEA",
                        "Feature": "WAB",
                        "Type": "SD"
                    },
                    "NextBus2": {
                        "EstimatedArrival": "",
                        "Latitude": "0",
                        "Longitude": "0",
                        "Load": "",
                        "Feature": "",
                        "Type": ""
                    },
                    "NextBus3": {
                        "EstimatedArrival": "",
                        "Latitude": "0",
                        "Longitude": "0",
                        "Load": "",
                        "Feature": "",
                        "Type": ""
                    }
                }
            ]
        }"#;
        let result = parse_arrival_response(json.as_bytes(), "83139").unwrap();
        assert_eq!(result.bus_stop_code, "83139");
        assert_eq!(result.services.len(), 1);
        let svc = &result.services[0];
        assert_eq!(svc.service_no, "15");
        assert_eq!(svc.operator, BusOperator::Gas);
        let next = svc.next.as_ref().unwrap();
        assert!(next.monitored);
        assert_eq!(next.load, BusLoad::SeatsAvailable);
        assert_eq!(next.feature, BusFeature::WheelchairAccessible);
        assert_eq!(next.bus_type, BusType::SingleDeck);
        assert!(next.estimated_arrival.is_some());
    }

    #[test]
    fn parse_stops_from_sample_json() {
        let json = r#"{
            "odata.metadata": "...",
            "value": [
                {
                    "BusStopCode": "01012",
                    "RoadName": "Victoria St",
                    "Description": "Hotel Grand Pacific",
                    "Latitude": 1.29684825487647,
                    "Longitude": 103.85253591654006
                }
            ]
        }"#;
        let stops = parse_stops_response(json.as_bytes()).unwrap();
        assert_eq!(stops.len(), 1);
        assert_eq!(stops[0].code, "01012");
        assert_eq!(stops[0].road_name, "Victoria St");
        assert_eq!(stops[0].description, "Hotel Grand Pacific");
    }
}
