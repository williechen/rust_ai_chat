mod chat;
mod file_organizer;
mod roam;

use ai_core::{ChatRequest, MockChatModel};
use pet_domain::{PetCommand, PetMachine, PetSnapshot};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

use crate::{chat::AiState, roam::roam_desktop};

// 派發寵物指令並更新狀態
fn dispatch_pet_command(
    app: &tauri::AppHandle,
    pet: &Mutex<PetMachine>,
    command: PetCommand,
) -> Result<PetSnapshot, String> {
    let snapshot = {
        let mut pet = pet
            .lock()
            .map_err(|_| "pet state lock poisoned".to_string())?;

        pet.dispatch(command).map_err(|error| error.to_string())?
    };

    app.emit("pet://state-changed", snapshot)
        .map_err(|error| error.to_string())?;

    Ok(snapshot)
}

fn authorized_root() -> Result<file_organizer::AuthorizedRoot, String> {
    let path =
        std::env::var("PET_AUTHORIZED_ROOT").map_err(|_| "尚未設定授權掃描目錄".to_string())?;
    file_organizer::AuthorizedRoot::new(path).map_err(|e| format!("無法使用授權掃描目錄：{e}"))
}

// 只接受明確的 true 或 1；其他值均維持不移動。
fn roam_enabled_value(value: Option<&str>) -> bool {
    matches!(value, Some("true" | "1"))
}

fn roam_enabled() -> bool {
    let value = std::env::var("PET_ENABLE_ROAM").ok();
    roam_enabled_value(value.as_deref())
}

fn normalize_new_pet_name(input: &str) -> Result<String, String> {
    let name = input.trim();
    if name.is_empty() || name.chars().count() > 32 {
        return Err("寵物名稱須為 1 到 32 字元".into());
    }
    Ok(name.to_owned())
}

// 取得寵物狀態
#[tauri::command]
fn get_pet_state(pet: State<'_, Mutex<PetMachine>>) -> Result<PetSnapshot, String> {
    let pet = pet.lock().map_err(|_| "pet state unavailable")?;
    Ok(pet.snapshot())
}

// 寵物指令
#[tauri::command]
fn send_pet_command(
    app: tauri::AppHandle,
    pet: State<'_, Mutex<PetMachine>>,
    command: PetCommand,
) -> Result<PetSnapshot, String> {
    dispatch_pet_command(&app, pet.inner(), command)
}

// 與寵物進行聊天
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

#[tauri::command]
fn scan_file_preview() -> Result<file_organizer::ScanPreview, String> {
    let root = authorized_root()?;
    file_organizer::scan_preview(&root).map_err(|e| format!("掃描失敗：{e}"))
}

#[tauri::command]
fn scan_duplicate_preview() -> Result<file_organizer::DuplicatePreview, String> {
    let root = authorized_root()?;
    let scan = file_organizer::scan(&root).map_err(|e| format!("掃描失敗：{e}"))?;
    // 僅回傳 SHA256 分組與警告；不提供刪除或 Trash 能力。
    Ok(file_organizer::duplicate_preview(&scan))
}

#[tauri::command]
fn open_control_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("控制視窗不存在")?;
    window.show().map_err(|_| "無法顯示控制視窗")?;
    window.set_focus().map_err(|_| "無法聚焦控制視窗".into())
}

#[tauri::command]
fn quit_pet(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn validate_new_pet_name(name: String) -> Result<String, String> {
    normalize_new_pet_name(&name)
}

pub fn application() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .manage(Mutex::new(PetMachine::default()))
        .manage(AiState::new(Arc::new(MockChatModel)))
        .setup(|app| {
            /*
             * JS Menu.new() 建立的 native menu item
             * 仍會產生 native menu event。
             *
             * Linux 不再依賴 JS action callback
             * 才能 dispatch PetCommand。
             */
            app.on_menu_event(|app_handle, event| {
                if event.id().0.as_str() == "open_control" {
                    let _ = open_control_window(app_handle.clone());
                    return;
                }
                let command = match event.id().0.as_str() {
                    "interact" => Some(PetCommand::Interact),
                    "sleep" => Some(PetCommand::Sleep),
                    "wake" => Some(PetCommand::Wake),
                    _ => None,
                };

                let Some(command) = command else {
                    return;
                };

                let pet = app_handle.state::<Mutex<PetMachine>>();

                if let Err(error) = dispatch_pet_command(app_handle, pet.inner(), command) {
                    eprintln!("pet context menu command failed: {error}");
                }
            });

            if roam_enabled() {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    roam_desktop(app_handle).await;
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_pet_state,
            send_pet_command,
            chat_with_pet,
            scan_file_preview,
            scan_duplicate_preview,
            open_control_window,
            quit_pet,
            validate_new_pet_name,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Rust AI Desktop Pet");
}

#[cfg(test)]
mod roam_setting_tests {
    use super::roam_enabled_value;
    #[test]
    fn roam_requires_explicit_opt_in() {
        assert!(!roam_enabled_value(None));
        assert!(!roam_enabled_value(Some("")));
        assert!(!roam_enabled_value(Some("false")));
        assert!(!roam_enabled_value(Some("TRUE")));
        assert!(roam_enabled_value(Some("true")));
        assert!(roam_enabled_value(Some("1")));
    }
}

#[cfg(test)]
mod day21_pet_creation_tests {
    use super::normalize_new_pet_name;
    #[test]
    fn day21_accepts_trimmed_chinese_name() {
        assert_eq!(normalize_new_pet_name("  小橘  "), Ok("小橘".into()));
    }
    #[test]
    fn day21_rejects_blank_and_long_names() {
        assert!(normalize_new_pet_name("  ").is_err());
        assert!(normalize_new_pet_name(&"寵".repeat(33)).is_err());
    }
}
