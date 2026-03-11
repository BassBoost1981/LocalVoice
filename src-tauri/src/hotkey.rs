use tauri::AppHandle;
use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::str::FromStr;

pub fn register_hotkey(app: &AppHandle, hotkey_str: &str) -> Result<(), String> {
    let shortcut = Shortcut::from_str(hotkey_str)
        .map_err(|e| format!("Invalid hotkey '{}': {}", hotkey_str, e))?;

    app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        use tauri_plugin_global_shortcut::ShortcutState;
        match event.state() {
            ShortcutState::Pressed => {
                let _ = app.emit("recording-started", ());
            }
            ShortcutState::Released => {
                let _ = app.emit("recording-stopped", ());
            }
        }
    }).map_err(|e| format!("Failed to register hotkey: {}", e))?;

    Ok(())
}

pub fn unregister_all(app: &AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister hotkeys: {}", e))
}
