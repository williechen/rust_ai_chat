use shared::{CreateRoomRequest, CreateRoomResponse};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode};

pub async fn create_room(name: &str) -> Result<CreateRoomResponse, String> {
    let body = serde_json::to_string(&CreateRoomRequest {
        name: name.to_string(),
    })
    .map_err(|error| format!("failed to encode create room request: {error}"))?;

    let init = RequestInit::new();
    init.set_method("POST");
    init.set_mode(RequestMode::SameOrigin);
    init.set_body(&JsValue::from_str(&body));

    let request = Request::new_with_str_and_init("/api/rooms", &init)
        .map_err(|error| format!("failed to build create room request: {error:?}"))?;

    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|error| format!("failed to set content type: {error:?}"))?;

    let window = web_sys::window().ok_or_else(|| "browser window is unavailable".to_string())?;

    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|error| format!("create room request failed: {error:?}"))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|_| "create room response is not an HTTP Response".to_string())?;

    if !response.ok() {
        return Err(format!(
            "create room failed with HTTP status: {}",
            response.status()
        ));
    }

    let response_text = JsFuture::from(
        response
            .text()
            .map_err(|error| format!("failed to read response body: {error:?}"))?,
    )
    .await
    .map_err(|error| format!("failed to await response body: {error:?}"))?
    .as_string()
    .ok_or_else(|| "create room response body is not text".to_string())?;

    serde_json::from_str::<CreateRoomResponse>(&response_text)
        .map_err(|error| format!("failed to decode create room response: {error}"))
}
