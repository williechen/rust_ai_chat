use ai_code::{ChatModel, ChatRequest, MockChatModel};
use pet_domain::{PetCommand, PetMachine, PetSnapshot};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

struct AiState {
    model: Arc<dyn ChatModel>,
}

impl AiState {
    fn new(model: Arc<dyn ChatModel>) -> Self {
        Self { model }
    }
}

#[tauri::command]
fn get_pet_state(pet: State<'_, Mutex<PetMachine>>) -> Result<PetSnapshot, String> {
    let pet = pet.lock().map_err(|_| "pet state unavailable")?;
    Ok(pet.snapshot())
}

#[tauri::command]
fn send_pet_command(
    app: tauri::AppHandle,
    pet: State<'_, Mutex<PetMachine>>,
    command: PetCommand,
) -> Result<PetSnapshot, String> {
    let snapshot = {
        let mut pet = pet.lock().map_err(|_| "pet state lock poisoned")?;
        pet.dispatch(command).map_err(|error| error.to_string())?
    };

    app.emit("pet://state-changed", snapshot.clone())
        .map_err(|error| error.to_string())?;

    Ok(snapshot)
}

#[tauri::command]
async fn chat_with_pet(
    message: String,
    state: tauri::State<'_, AiState>,
) -> Result<String, String> {
    let response = state
        .model
        .chat(ChatRequest::new(message))
        .await
        .map_err(|error| error.to_string())?;
    Ok(response)
}

pub fn application() {
    tauri::Builder::default()
        .manage(AiState::new(Arc::new(MockChatModel)))
        .invoke_handler(tauri::generate_handler![
            get_pet_state,
            send_pet_command,
            chat_with_pet
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Rust AI Desktop Pet");
}
