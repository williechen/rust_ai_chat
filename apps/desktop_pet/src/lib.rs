use ai_core::{ChatModel, ChatRequest, MockChatModel};
use pet_domain::{PetCommand, PetMachine, PetSnapshot};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, PhysicalPosition, State};

const ROAM_STEP_X: i32 = 2;
const ROAM_STEP_Y: i32 = 1;
const ROAM_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);
const ROAM_VERIFY_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);
const ROAM_STALL_LIMIT: u8 = 3;

struct AiState {
    model: Arc<dyn ChatModel>,
}

impl AiState {
    fn new(model: Arc<dyn ChatModel>) -> Self {
        Self { model }
    }
}

async fn roam_desktop(app: tauri::AppHandle) {
    let mut stalled_attempts = 0_u8;

    loop {
        let window = match app.get_webview_window("main") {
            Some(window) => window,
            None => {
                println!("desktop roaming unavailable: main window not found");
                break;
            }
        };

        let before = match window.outer_position() {
            Ok(position) => position,
            Err(error) => {
                eprintln!("desktop roaming unavailable: {error}");
                break;
            }
        };

        let requested = PhysicalPosition {
            x: before.x + ROAM_STEP_X,
            y: before.y + ROAM_STEP_Y,
        };

        if let Err(error) = window.set_position(requested) {
            eprintln!("desktop roaming unavailable: {error}");
            break;
        }

        tokio::time::sleep(ROAM_VERIFY_INTERVAL).await;

        let after = match window.outer_position() {
            Ok(position) => position,
            Err(error) => {
                eprintln!("desktop roaming verification unavailable: {error}");
                break;
            }
        };

        if after == before {
            stalled_attempts += 1;

            if stalled_attempts >= ROAM_STALL_LIMIT {
                eprintln!(
                    "desktop roaming unavailable: \
                     window position did not change after \
                     {ROAM_STALL_LIMIT} attempts"
                );
                break;
            }
        } else {
            stalled_attempts = 0;
        }

        tokio::time::sleep(ROAM_INTERVAL).await;
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
    Ok(response.text)
}

pub fn application() {
    tauri::Builder::default()
        .manage(Mutex::new(PetMachine::default()))
        .manage(AiState::new(Arc::new(MockChatModel)))
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                roam_desktop(app_handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_pet_state,
            send_pet_command,
            chat_with_pet
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Rust AI Desktop Pet");
}
