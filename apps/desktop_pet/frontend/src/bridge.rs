use js_sys::{Function, Promise};
use pet_domain::{PetCommand, PetSnapshot};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(
        js_namespace = window,
        js_name = getPetState
    )]
    fn get_pet_state_js() -> Promise;

    #[wasm_bindgen(
        js_namespace = window,
        js_name = sendPetCommand
    )]
    fn send_pet_command_js(command: JsValue) -> Promise;

    #[wasm_bindgen(
        js_namespace = window,
        js_name = listenPetState
    )]
    fn listen_pet_state_js(callback: &Function) -> Promise;
}

fn js_error(value: JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:?}"))
}

pub async fn get_pet_state() -> Result<PetSnapshot, String> {
    let value = JsFuture::from(get_pet_state_js()).await.map_err(js_error)?;

    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn send_pet_command(command: PetCommand) -> Result<PetSnapshot, String> {
    let command = serde_wasm_bindgen::to_value(&command).map_err(|error| error.to_string())?;

    let value = JsFuture::from(send_pet_command_js(command))
        .await
        .map_err(js_error)?;

    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

pub async fn listen_pet_state(
    mut on_snapshot: impl FnMut(PetSnapshot) + 'static,
) -> Result<(), String> {
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |value| {
        if let Ok(snapshot) = serde_wasm_bindgen::from_value(value) {
            on_snapshot(snapshot);
        }
    });

    JsFuture::from(listen_pet_state_js(callback.as_ref().unchecked_ref()))
        .await
        .map_err(js_error)?;

    // listener 跟著整個 pet app 存活。
    callback.forget();

    Ok(())
}
