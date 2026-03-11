mod audio;
mod config;
mod hotkey;
mod whisper;

use audio::AudioRecorder;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager};

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
    let samples = {
        let mut recorder = state.recorder.lock().map_err(|e| e.to_string())?;
        recorder.stop()
    };

    if samples.is_empty() {
        return Err("No audio recorded".to_string());
    }

    let settings = config::load_settings();
    let lang = settings.language.clone();
    let state_clone = state.inner().clone();

    // Transcribe in blocking thread (CPU-bound)
    let text = tokio::task::spawn_blocking(move || {
        let guard = state_clone.transcriber.lock().map_err(|e| e.to_string())?;
        let transcriber = guard.as_ref().ok_or("Whisper model not loaded")?;
        transcriber.transcribe(&samples, &lang)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    let _ = app_handle.emit("transcription-done", &text);
    Ok(text)
}

#[tauri::command]
fn set_hotkey(app_handle: tauri::AppHandle, hotkey_str: String) -> Result<(), String> {
    hotkey::unregister_all(&app_handle)?;
    hotkey::register_hotkey(&app_handle, &hotkey_str)?;
    let mut settings = config::load_settings();
    settings.hotkey = hotkey_str;
    config::save_settings(&settings);
    Ok(())
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
            set_hotkey,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let settings = config::load_settings();

            // Register global hotkey / Globalen Hotkey registrieren
            hotkey::register_hotkey(&handle, &settings.hotkey)
                .unwrap_or_else(|e| eprintln!("Hotkey error: {}", e));

            // Load whisper model async / Whisper-Modell async laden
            let h = handle.clone();
            tauri::async_runtime::spawn(async move {
                let settings = config::load_settings();
                let model_path = config::get_models_path()
                    .join(format!("ggml-{}.bin", settings.model));
                if model_path.exists() {
                    match whisper::WhisperTranscriber::new(&model_path) {
                        Ok(t) => {
                            println!("Whisper model loaded: {}", settings.model);
                            let state: Arc<AppState> = h.state::<Arc<AppState>>().inner().clone();
                            state.transcriber.lock().unwrap().replace(t);
                        }
                        Err(e) => eprintln!("Failed to load whisper: {}", e),
                    }
                } else {
                    eprintln!("Model not found: {:?} — download via Settings", model_path);
                }
            });

            // Wire hotkey events to recording pipeline
            // Hotkey-Events mit Recording-Pipeline verbinden
            let h = handle.clone();
            let state: Arc<AppState> = handle.state::<Arc<AppState>>().inner().clone();

            // Hotkey pressed -> start recording
            let h1 = h.clone();
            let s1 = state.clone();
            app.listen("recording-started", move |_| {
                let h = h1.clone();
                let s = s1.clone();
                tauri::async_runtime::spawn(async move {
                    let mut recorder = s.recorder.lock().unwrap();
                    if let Err(e) = recorder.start(h.clone()) {
                        eprintln!("Failed to start recording: {}", e);
                    }
                });
            });

            // Hotkey released -> stop recording + transcribe
            let h2 = h.clone();
            let s2 = state.clone();
            app.listen("recording-stopped", move |_| {
                let h = h2.clone();
                let s = s2.clone();
                tauri::async_runtime::spawn(async move {
                    // Stop and get audio samples
                    let samples = {
                        let mut recorder = s.recorder.lock().unwrap();
                        recorder.stop()
                    };

                    if samples.is_empty() {
                        return;
                    }

                    let _ = h.emit("transcribing-started", ());

                    // Transcribe in blocking thread
                    let s_clone = s.clone();
                    let settings = config::load_settings();
                    let lang = settings.language.clone();

                    let result = tokio::task::spawn_blocking(move || {
                        let guard = s_clone.transcriber.lock().unwrap();
                        if let Some(transcriber) = guard.as_ref() {
                            transcriber.transcribe(&samples, &lang)
                        } else {
                            Err("Model not loaded".to_string())
                        }
                    })
                    .await;

                    match result {
                        Ok(Ok(text)) => {
                            println!("Transcribed: {}", text);
                            let _ = h.emit("transcription-done", &text);
                        }
                        Ok(Err(e)) => {
                            eprintln!("Transcription error: {}", e);
                            let _ = h.emit("transcription-error", &e);
                        }
                        Err(e) => {
                            let msg = format!("Task error: {}", e);
                            eprintln!("{}", msg);
                            let _ = h.emit("transcription-error", &msg);
                        }
                    }
                });
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
