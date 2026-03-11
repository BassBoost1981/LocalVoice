use arboard::Clipboard;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};

/// Inject text into the active window via clipboard + Ctrl+V
/// Text in aktives Fenster einfügen via Clipboard + Ctrl+V
pub fn inject_text(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;

    // Save current clipboard content / Clipboard-Inhalt sichern
    let previous = clipboard.get_text().ok();

    // Set new text
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    // Small delay to ensure clipboard is ready
    thread::sleep(Duration::from_millis(50));

    // Simulate Ctrl+V
    #[cfg(windows)]
    simulate_ctrl_v()?;

    // Restore previous clipboard after a delay
    // Clipboard wiederherstellen nach kurzer Verzögerung
    if let Some(prev) = previous {
        thread::sleep(Duration::from_millis(200));
        let _ = clipboard.set_text(prev);
    }

    Ok(())
}

#[cfg(windows)]
fn simulate_ctrl_v() -> Result<(), String> {
    unsafe {
        let inputs = [
            // Ctrl down
            create_key_input(VK_CONTROL, false),
            // V down
            create_key_input(VK_V, false),
            // V up
            create_key_input(VK_V, true),
            // Ctrl up
            create_key_input(VK_CONTROL, true),
        ];

        let result = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if result != inputs.len() as u32 {
            return Err("SendInput failed".to_string());
        }
    }
    Ok(())
}

#[cfg(windows)]
unsafe fn create_key_input(key: VIRTUAL_KEY, key_up: bool) -> INPUT {
    let mut flags = windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS(0);
    if key_up {
        flags = KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
