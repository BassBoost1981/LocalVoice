#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! LocalVoice is running.", name)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
