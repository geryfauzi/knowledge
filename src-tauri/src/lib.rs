pub mod drive;
pub mod oauth;
pub mod sync;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You are running Knowledge.", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            oauth::google_start_oauth,
            oauth::google_get_auth_status,
            oauth::google_logout,
            sync::google_sync_now,
            sync::google_sync_file,
            sync::google_delete_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
