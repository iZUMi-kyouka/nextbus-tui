#[cfg(not(target_arch = "wasm32"))]
use crate::models::ApiResponse;
#[cfg(target_arch = "wasm32")]
use crate::models::ApiResponse;
use crate::models::ShuttleServiceResult;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

fn api_base() -> String {
    #[cfg(target_arch = "wasm32")]
    let default_base = "https://nusbus.flovt.net/ShuttleService";
    #[cfg(not(target_arch = "wasm32"))]
    let default_base = "https://nnextbus.nusmods.com/ShuttleService";

    std::env::var("API_BASE").unwrap_or_else(|_| default_base.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
pub async fn fetch_shuttle_service_async(stop_name: &str) -> Result<ShuttleServiceResult, String> {
    let url = format!("{}?busstopname={}", api_base(), stop_name);

    let window = web_sys::window().ok_or_else(|| "window is unavailable".to_string())?;
    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| format!("Network error: {e:?}"))?;
    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "failed to convert fetch response".to_string())?;

    if !resp.ok() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let body_text = JsFuture::from(
        resp.text()
            .map_err(|e| format!("Failed to read response body: {e:?}"))?,
    )
    .await
    .map_err(|e| format!("Body read error: {e:?}"))?
    .as_string()
    .ok_or_else(|| "response body is not valid UTF-8 text".to_string())?;

    serde_json::from_str::<ApiResponse>(&body_text)
        .map(|r| r.result)
        .map_err(|e| format!("Parse error: {e}"))
}
