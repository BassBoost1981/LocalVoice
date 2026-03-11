use tauri::{AppHandle, Manager, PhysicalPosition};

/// Position the overlay window in one of 4 screen corners
/// Overlay-Fenster in einer der 4 Bildschirmecken positionieren
pub fn set_overlay_position(app: &AppHandle, position: &str) -> Result<(), String> {
    let overlay = app
        .get_webview_window("overlay")
        .ok_or("Overlay window not found")?;

    // Get primary monitor dimensions
    let monitor = overlay
        .primary_monitor()
        .map_err(|e| format!("Monitor error: {}", e))?
        .ok_or("No primary monitor")?;

    let screen = monitor.size();
    let scale = monitor.scale_factor();
    let screen_w = (screen.width as f64 / scale) as i32;
    let screen_h = (screen.height as f64 / scale) as i32;

    let overlay_w = 200;
    let overlay_h = 50;
    let margin = 20;

    let (x, y) = match position {
        "top-left" => (margin, margin),
        "top-right" => (screen_w - overlay_w - margin, margin),
        "bottom-left" => (margin, screen_h - overlay_h - margin),
        _ => (screen_w - overlay_w - margin, screen_h - overlay_h - margin), // bottom-right default
    };

    overlay
        .set_position(PhysicalPosition::new(
            (x as f64 * scale) as i32,
            (y as f64 * scale) as i32,
        ))
        .map_err(|e| format!("Failed to set position: {}", e))?;

    overlay
        .show()
        .map_err(|e| format!("Failed to show overlay: {}", e))?;

    Ok(())
}
