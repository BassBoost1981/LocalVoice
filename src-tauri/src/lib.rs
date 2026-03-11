mod audio;
mod config;

use audio::AudioRecorder;
use std::sync::Mutex;

#[tauri::command]
fn get_settings() -> config::Settings {
    config::load_settings()
}

#[tauri::command]
fn save_settings_cmd(settings: config::Settings) {
    config::save_settings(&settings);
}

#[tauri::command]
fn start_recording(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AudioRecorder>>,
) -> Result<(), String> {
    let mut recorder = state.inner().lock().map_err(|e| e.to_string())?;
    recorder.start(app_handle)?;
    Ok(())
}

#[tauri::command]
fn stop_recording(
    state: tauri::State<'_, Mutex<AudioRecorder>>,
) -> Result<Vec<f32>, String> {
    let mut recorder = state.inner().lock().map_err(|e| e.to_string())?;
    Ok(recorder.stop())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Mutex::new(AudioRecorder::new()))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings_cmd,
            start_recording,
            stop_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
