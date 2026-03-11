mod config;

#[tauri::command]
fn get_settings() -> config::Settings {
    config::load_settings()
}

#[tauri::command]
fn save_settings_cmd(settings: config::Settings) {
    config::save_settings(&settings);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![get_settings, save_settings_cmd])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
