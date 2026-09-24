use pet_domain::{PetCommand, PetMachine, PetSnapshot};
use std::sync::Mutex;
use tauri::{Emitter, State};

#[tauri::command]
fn get_pet_state(machine: State<'_, Mutex<PetMachine>>) -> Result<PetSnapshot, String> {
    machine
        .lock()
        .map(|pet| pet.snapshot())
        .map_err(|_| "pet state unavailable".into())
}

#[tauri::command]
fn send_pet_command(
    app: tauri::AppHandle,
    machine: State<'_, Mutex<PetMachine>>,
    command: PetCommand,
) -> Result<PetSnapshot, String> {
    // 同一把鎖序列化命令與事件，避免並行呼叫亂序。
    let mut pet = machine.lock().map_err(|_| "pet state unavailable")?;
    let event = pet.dispatch(command).map_err(|error| error.to_string())?;
    let snapshot = pet.snapshot();
    app.emit("pet://state-changed", event)
        .map_err(|error| error.to_string())?;
    Ok(snapshot)
}

pub fn application() {
    tauri::Builder::default()
        .manage(Mutex::new(PetMachine::default()))
        .invoke_handler(tauri::generate_handler![get_pet_state, send_pet_command])
        .run(tauri::generate_context!())
        .expect("failed to run Rust AI Desktop Pet");
}
