use crate::models::{ApiResponse, ShuttleServiceResult};

const API_BASE: &str = "https://nnextbus.nusmods.com/ShuttleService";

pub fn fetch_shuttle_service(stop_name: &str) -> Result<ShuttleServiceResult, String> {
    let url = format!("{}?busstopname={}", API_BASE, stop_name);
    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Network error: {e}"))?;
    let reader = response.into_reader();
    serde_json::from_reader::<_, ApiResponse>(reader)
        .map(|r| r.result)
        .map_err(|e| format!("Parse error: {e}"))
}
