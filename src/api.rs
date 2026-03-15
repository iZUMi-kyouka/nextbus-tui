use crate::models::{ApiResponse, ShuttleServiceResult};

fn api_base() -> String {
    std::env::var("API_BASE")
        .unwrap_or_else(|_| "https://nnextbus.nusmods.com/ShuttleService".to_string())
}

pub fn fetch_shuttle_service(stop_name: &str) -> Result<ShuttleServiceResult, String> {
    let url = format!("{}?busstopname={}", api_base(), stop_name);
    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Network error: {e}"))?;
    let reader = response.into_reader();
    serde_json::from_reader::<_, ApiResponse>(reader)
        .map(|r| r.result)
        .map_err(|e| format!("Parse error: {e}"))
}
