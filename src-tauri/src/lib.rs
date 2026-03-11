mod audio;
mod config;
mod whisper;

use audio::AudioRecorder;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};

struct AppState {
    recorder: Mutex<AudioRecorder>,
    transcriber: Mutex<Option<whisper::WhisperTranscriber>>,
}

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
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    recorder.start(app_handle)?;
    Ok(())
}

#[tauri::command]
async fn stop_recording(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<String, String> {
    // Stop recording and get PCM samples
    let samples = {
        let mut recorder = state.recorder.lock().map_err(|e| e.to_string())?;
        recorder.stop()
    };

    if samples.is_empty() {
        return Err("No audio recorded".to_string());
    }

    let settings = config::load_settings();
    let lang = settings.language.clone();

    // Clone state Arc for the blocking task
    let state_clone = state.inner().clone();

    // Transcribe in blocking thread (CPU-bound)
    // Transkription im Blocking-Thread (CPU-intensiv)
    let text = tokio::task::spawn_blocking(move || {
        let guard = state_clone.transcriber.lock().map_err(|e| e.to_string())?;
        let transcriber = guard.as_ref().ok_or("Whisper model not loaded")?;
        transcriber.transcribe(&samples, &lang)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    // Emit transcription result to frontend
    let _ = app_handle.emit("transcription-done", &text);
    Ok(text)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Arc::new(AppState {
            recorder: Mutex::new(AudioRecorder::new()),
            transcriber: Mutex::new(None),
        }))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings_cmd,
            start_recording,
            stop_recording,
        ])
        .setup(|app| {
            // Load whisper model on startup if available
            // Whisper-Modell beim Start laden (falls vorhanden)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let settings = config::load_settings();
                let model_path = config::get_models_path()
                    .join(format!("ggml-{}.bin", settings.model));
                if model_path.exists() {
                    match whisper::WhisperTranscriber::new(&model_path) {
                        Ok(t) => {
                            println!("Whisper model loaded: {}", settings.model);
                            let state: Arc<AppState> = handle.state::<Arc<AppState>>().inner().clone();
                            state.transcriber.lock().unwrap().replace(t);
                        }
                        Err(e) => eprintln!("Failed to load whisper: {}", e),
                    }
                } else {
                    eprintln!("Model not found: {:?} — download via Settings", model_path);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
