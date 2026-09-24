use pet_domain::{PetCommand, PetMachine, PetSnapshot};
use std::sync::Mutex;
use tauri::{Emitter, State};

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

pub fn application() {
    tauri::Builder::default()
        .manage(Mutex::new(PetMachine::default()))
        .invoke_handler(tauri::generate_handler![get_pet_state, send_pet_command])
        .run(tauri::generate_context!())
        .expect("failed to run Rust AI Desktop Pet");
}
