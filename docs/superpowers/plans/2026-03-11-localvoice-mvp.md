# LocalVoice MVP Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a portable, offline dictation tool for Windows that transcribes speech via whisper.cpp and injects text into the active window.

**Architecture:** Monolith Tauri 2.0 app. Rust backend handles audio capture (cpal), FFT (rustfft), transcription (whisper-rs), and text injection (windows-rs). React 19 frontend for Overlay pill-widget and Settings UI. All data stored portable relative to exe.

**Tech Stack:** Tauri 2.0, React 19, TypeScript, Vite, Tailwind CSS v4, shadcn/ui, whisper-rs, cpal, rustfft, windows-rs, arboard, tauri-plugin-global-shortcut, reqwest, serde, tokio

**Spec:** `docs/superpowers/specs/2026-03-11-localvoice-mvp-design.md`

---

## File Structure

### Rust Backend (`src-tauri/`)

| File | Responsibility |
|------|---------------|
| `src-tauri/src/main.rs` | Windows entry point |
| `src-tauri/src/lib.rs` | Tauri builder, command registry, plugin init |
| `src-tauri/src/config.rs` | Settings struct, read/write JSON, portable path resolution |
| `src-tauri/src/audio.rs` | cpal microphone stream, PCM buffer, FFT → 16 bins, event emission |
| `src-tauri/src/whisper.rs` | Load model, transcribe PCM buffer in spawn_blocking |
| `src-tauri/src/hotkey.rs` | Register/unregister global shortcut, push-to-talk state |
| `src-tauri/src/inject.rs` | Clipboard save/restore, text set, simulate Ctrl+V via SendInput |
| `src-tauri/src/overlay.rs` | Create/position frameless transparent always-on-top window |
| `src-tauri/src/model_manager.rs` | Download models from HuggingFace, progress events |
| `src-tauri/Cargo.toml` | Dependencies |
| `src-tauri/tauri.conf.json` | Tauri config: windows, permissions, bundle |
| `src-tauri/capabilities/default.json` | Tauri v2 permissions |

### Frontend (`src/`)

| File | Responsibility |
|------|---------------|
| `src/main.tsx` | React entry point |
| `src/App.tsx` | Router: overlay vs settings window by Tauri label |
| `src/components/Overlay.tsx` | Pill widget: idle/recording/transcribing/done states |
| `src/components/Settings.tsx` | Settings panel with glasmorph cards |
| `src/components/ModelManager.tsx` | Model list, download button, progress bar |
| `src/components/SetupWizard.tsx` | First-run: hotkey → model → mic test |
| `src/hooks/useRecording.ts` | State machine hook, Tauri event listeners |
| `src/hooks/useSettings.ts` | Load/save settings via Tauri invoke |
| `src/lib/tauri.ts` | Typed wrappers for all Tauri commands and events |
| `src/styles/globals.css` | Tailwind v4 imports, Lexend font-face, CSS variables |

### Config & Assets

| File | Responsibility |
|------|---------------|
| `package.json` | Frontend dependencies |
| `vite.config.ts` | Vite config for Tauri |
| `tsconfig.json` | TypeScript config |
| `Font/Lexend-VariableFont_wght.ttf` | Already exists — Lexend font |

---

## Chunk 1: Phase A — Kern-Engine

### Task 1: Tauri Project Scaffolding

**Files:**
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `src/main.tsx`
- Create: `src/App.tsx`
- Create: `src/styles/globals.css`

- [ ] **Step 1: Initialize Tauri project**

Run:
```bash
npm create tauri-app@latest localvoice-temp -- --template react-ts --manager npm
```

Copy the generated files into `E:/Vibe-Coding/LocalVoice/`, then remove the temp folder.

- [ ] **Step 2: Add Rust dependencies to Cargo.toml**

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
cpal = "0.15"
rustfft = "6"
whisper-rs = "0.13"
arboard = "3"
reqwest = { version = "0.12", features = ["stream"] }
windows = { version = "0.58", features = [
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging",
] }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 3: Configure tauri.conf.json with two windows**

```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/crates/tauri-config-schema/schema.json",
  "productName": "LocalVoice",
  "version": "0.1.0",
  "identifier": "com.localvoice.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "label": "overlay",
        "title": "",
        "width": 200,
        "height": 50,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "resizable": false,
        "skipTaskbar": true,
        "x": 0,
        "y": 0,
        "visible": true
      },
      {
        "label": "settings",
        "title": "LocalVoice — Einstellungen",
        "width": 500,
        "height": 600,
        "decorations": true,
        "visible": false,
        "center": true
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": false
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 4: Create capabilities/default.json**

```json
{
  "identifier": "default",
  "description": "Default permissions",
  "windows": ["overlay", "settings"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:window:default",
    "core:window:allow-set-position",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "global-shortcut:default"
  ]
}
```

- [ ] **Step 5: Create minimal main.rs**

```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    localvoice_lib::run();
}
```

- [ ] **Step 6: Create minimal lib.rs with placeholder commands**

```rust
mod config;

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
```

- [ ] **Step 7: Set up frontend with Tailwind v4 and Lexend**

In `src/styles/globals.css`:
```css
@import "tailwindcss";

@font-face {
  font-family: "Lexend";
  src: url("/Font/Lexend-VariableFont_wght.ttf") format("truetype");
  font-weight: 100 900;
  font-display: swap;
}

:root {
  --color-bg: #09090b;
  --color-accent: #6366f1;
  --color-accent-light: #a78bfa;
  --color-border: rgba(139, 92, 246, 0.12);
  --color-recording: #ef4444;
  --color-done: #22c55e;
}

body {
  font-family: "Lexend", sans-serif;
  background: var(--color-bg);
  color: white;
  margin: 0;
  overflow: hidden;
}
```

Minimal `src/App.tsx`:
```tsx
function App() {
  return <div className="p-4">LocalVoice running</div>;
}
export default App;
```

- [ ] **Step 8: Verify build compiles**

Run:
```bash
cd E:/Vibe-Coding/LocalVoice && npm install && npm run tauri dev
```
Expected: Tauri window opens showing "LocalVoice running".

- [ ] **Step 9: Init git and commit**

```bash
cd E:/Vibe-Coding/LocalVoice
git init
echo "node_modules/\ndist/\ntarget/\n.firecrawl/\n.superpowers/\nmodels/*.bin" > .gitignore
git add -A
git commit -m "feat: scaffold Tauri 2.0 project with React 19, Tailwind v4, Lexend font

Initialize project structure with two windows (overlay + settings),
global-shortcut plugin, and all Rust dependencies."
```

---

### Task 2: Config Module (Portable Settings)

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create config.rs with Settings struct**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: String,
    pub language: String,
    pub model: String,
    pub overlay_position: String,
    pub activation_mode: String,
    pub autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "ctrl+shift+space".to_string(),
            language: "de".to_string(),
            model: "base".to_string(),
            overlay_position: "bottom-right".to_string(),
            activation_mode: "push-to-talk".to_string(),
            autostart: false,
        }
    }
}

/// Get the portable base path (next to the exe)
pub fn get_base_path() -> PathBuf {
    std::env::current_exe()
        .expect("failed to get exe path")
        .parent()
        .expect("failed to get exe directory")
        .to_path_buf()
}

pub fn get_config_path() -> PathBuf {
    get_base_path().join("config").join("settings.json")
}

pub fn get_models_path() -> PathBuf {
    get_base_path().join("models")
}

pub fn load_settings() -> Settings {
    let path = get_config_path();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        let settings = Settings::default();
        save_settings(&settings);
        settings
    }
}

pub fn save_settings(settings: &Settings) {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let data = serde_json::to_string_pretty(settings).expect("failed to serialize settings");
    std::fs::write(&path, data).expect("failed to write settings");
}
```

- [ ] **Step 2: Register get_settings and save_settings commands in lib.rs**

```rust
mod config;

use std::sync::Mutex;

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
```

- [ ] **Step 3: Verify build and test config creation**

Run: `npm run tauri dev`
Expected: `config/settings.json` is created next to the exe with default values.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/lib.rs
git commit -m "feat: add portable config module with settings read/write"
```

---

### Task 3: Audio Capture Module

**Files:**
- Create: `src-tauri/src/audio.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create audio.rs with mic recording and FFT**

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::{FftPlanner, num_complex::Complex};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        }
    }

    pub fn start(&mut self, app_handle: AppHandle) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or("No input device found")?;

        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(16000),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples = self.samples.clone();
        let app = app_handle.clone();
        let fft_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let fft_buf_clone = fft_buffer.clone();

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Store PCM samples
                    if let Ok(mut s) = samples.lock() {
                        s.extend_from_slice(data);
                    }
                    // Accumulate for FFT
                    if let Ok(mut fb) = fft_buf_clone.lock() {
                        fb.extend_from_slice(data);
                        // Process FFT every 512 samples (~30fps at 16kHz)
                        while fb.len() >= 512 {
                            let chunk: Vec<f32> = fb.drain(..512).collect();
                            let bins = compute_fft(&chunk, 16);
                            let _ = app.emit("fft-data", &bins);
                        }
                    }
                },
                |err| eprintln!("Audio stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;
        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) -> Vec<f32> {
        self.stream = None; // Drop stops the stream
        let mut samples = self.samples.lock().unwrap();
        std::mem::take(&mut *samples)
    }
}

fn compute_fft(samples: &[f32], num_bins: usize) -> Vec<f32> {
    let len = samples.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(len);

    let mut buffer: Vec<Complex<f32>> = samples
        .iter()
        .map(|&s| Complex { re: s, im: 0.0 })
        .collect();

    fft.process(&mut buffer);

    // Take first half (positive frequencies), group into bins
    let half = len / 2;
    let bin_size = half / num_bins;
    (0..num_bins)
        .map(|i| {
            let start = i * bin_size;
            let end = start + bin_size;
            let magnitude: f32 = buffer[start..end]
                .iter()
                .map(|c| (c.re * c.re + c.im * c.im).sqrt())
                .sum::<f32>()
                / bin_size as f32;
            // Normalize to 0.0-1.0 range (clamp)
            (magnitude * 10.0).min(1.0)
        })
        .collect()
}
```

- [ ] **Step 2: Add AudioRecorder as Tauri managed state in lib.rs**

```rust
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
    let mut recorder = state.lock().map_err(|e| e.to_string())?;
    recorder.start(app_handle)?;
    Ok(())
}

#[tauri::command]
fn stop_recording(
    state: tauri::State<'_, Mutex<AudioRecorder>>,
) -> Result<Vec<f32>, String> {
    let mut recorder = state.lock().map_err(|e| e.to_string())?;
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
```

- [ ] **Step 3: Verify build compiles**

Run: `npm run tauri dev`
Expected: Compiles without errors. No runtime test yet (needs mic).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/audio.rs src-tauri/src/lib.rs
git commit -m "feat: add audio capture with cpal and FFT frequency analysis"
```

---

### Task 4: Whisper Transcription Module

**Files:**
- Create: `src-tauri/src/whisper.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create whisper.rs**

```rust
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperTranscriber {
    ctx: WhisperContext,
}

impl WhisperTranscriber {
    pub fn new(model_path: &Path) -> Result<Self, String> {
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or("Invalid model path")?,
            WhisperContextParameters::default(),
        )
        .map_err(|e| format!("Failed to load whisper model: {}", e))?;

        Ok(Self { ctx })
    }

    pub fn transcribe(&self, audio_data: &[f32], language: &str) -> Result<String, String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(language));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_single_segment(false);

        let mut state = self.ctx.create_state()
            .map_err(|e| format!("Failed to create state: {}", e))?;

        state
            .full(params, audio_data)
            .map_err(|e| format!("Transcription failed: {}", e))?;

        let num_segments = state
            .full_n_segments()
            .map_err(|e| format!("Failed to get segments: {}", e))?;

        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        Ok(text.trim().to_string())
    }
}
```

- [ ] **Step 2: Integrate whisper into lib.rs with spawn_blocking**

Add to `lib.rs`:
```rust
mod whisper;

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// Change managed state to include optional whisper transcriber
struct AppState {
    recorder: std::sync::Mutex<audio::AudioRecorder>,
    transcriber: TokioMutex<Option<whisper::WhisperTranscriber>>,
}
```

Update `run()` setup to load whisper model on startup:
```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Arc::new(AppState {
            recorder: std::sync::Mutex::new(audio::AudioRecorder::new()),
            transcriber: TokioMutex::new(None),
        }))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings_cmd,
            start_recording,
            stop_recording,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let settings = config::load_settings();
                let model_path = config::get_models_path()
                    .join(format!("ggml-{}.bin", settings.model));
                if model_path.exists() {
                    match whisper::WhisperTranscriber::new(&model_path) {
                        Ok(t) => {
                            println!("Whisper model loaded: {}", settings.model);
                            // Store in state - get from handle
                            let state = handle.state::<Arc<AppState>>();
                            *state.transcriber.lock().await = Some(t);
                        }
                        Err(e) => eprintln!("Failed to load whisper: {}", e),
                    }
                } else {
                    eprintln!("Model not found: {:?}", model_path);
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Update `start_recording` and `stop_recording` to use `AppState`:
```rust
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
    // Stop recording and get samples
    let samples = {
        let mut recorder = state.recorder.lock().map_err(|e| e.to_string())?;
        recorder.stop()
    };

    // Transcribe in blocking thread
    let transcriber_guard = state.transcriber.lock().await;
    let transcriber = transcriber_guard
        .as_ref()
        .ok_or("Whisper model not loaded")?;

    let settings = config::load_settings();
    let lang = settings.language.clone();

    // whisper-rs is not Send, so we clone the needed data
    let audio = samples.clone();

    // Transcribe — whisper-rs is CPU-bound, use spawn_blocking
    // We need to work around WhisperTranscriber not being Send
    // by doing the transcription while holding the lock
    let text = tokio::task::spawn_blocking(move || {
        transcriber.transcribe(&audio, &lang)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))??;

    // Emit event
    let _ = app_handle.emit("transcription-done", &text);
    Ok(text)
}
```

- [ ] **Step 3: Download a test model to verify**

Run:
```bash
mkdir -p E:/Vibe-Coding/LocalVoice/models
curl -L -o E:/Vibe-Coding/LocalVoice/models/ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

- [ ] **Step 4: Verify build compiles**

Run: `npm run tauri dev`
Expected: Console shows "Whisper model loaded: base".

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/whisper.rs src-tauri/src/lib.rs
git commit -m "feat: add whisper.rs transcription module with async model loading"
```

---

### Task 5: Hotkey Module

**Files:**
- Create: `src-tauri/src/hotkey.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create hotkey.rs**

```rust
use tauri::AppHandle;
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
```

- [ ] **Step 2: Register hotkey in lib.rs setup and add set_hotkey command**

Add to setup:
```rust
// In .setup(|app| { ... })
let settings = config::load_settings();
hotkey::register_hotkey(&app.handle(), &settings.hotkey)
    .unwrap_or_else(|e| eprintln!("Hotkey error: {}", e));
```

Add command:
```rust
#[tauri::command]
fn set_hotkey(app_handle: tauri::AppHandle, hotkey: String) -> Result<(), String> {
    hotkey::unregister_all(&app_handle)?;
    hotkey::register_hotkey(&app_handle, &hotkey)?;
    let mut settings = config::load_settings();
    settings.hotkey = hotkey;
    config::save_settings(&settings);
    Ok(())
}
```

- [ ] **Step 3: Verify hotkey works**

Run: `npm run tauri dev`
Test: Press `Ctrl+Shift+Space` — should see events in console/devtools.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/hotkey.rs src-tauri/src/lib.rs
git commit -m "feat: add global hotkey with push-to-talk support"
```

---

### Task 6: Wire Up Full Recording Flow (Phase A Complete)

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Connect hotkey events to recording + transcription pipeline**

In `lib.rs` setup, add event listeners that connect the pipeline:

```rust
// In .setup(|app| { ... })
let handle = app.handle().clone();
let state = handle.state::<Arc<AppState>>().inner().clone();

// Listen for recording-started (from hotkey press)
let h = handle.clone();
let s = state.clone();
app.listen("recording-started", move |_| {
    let h = h.clone();
    let s = s.clone();
    tauri::async_runtime::spawn(async move {
        let mut recorder = s.recorder.lock().unwrap();
        if let Err(e) = recorder.start(h.clone()) {
            eprintln!("Failed to start recording: {}", e);
        }
    });
});

// Listen for recording-stopped (from hotkey release)
let h = handle.clone();
let s = state.clone();
app.listen("recording-stopped", move |_| {
    let h = h.clone();
    let s = s.clone();
    tauri::async_runtime::spawn(async move {
        // Stop recording
        let samples = {
            let mut recorder = s.recorder.lock().unwrap();
            recorder.stop()
        };

        if samples.is_empty() {
            return;
        }

        let _ = h.emit("transcribing-started", ());

        // Transcribe
        let transcriber_guard = s.transcriber.lock().await;
        if let Some(transcriber) = transcriber_guard.as_ref() {
            let settings = config::load_settings();
            let lang = settings.language.clone();
            // Clone audio data for spawn_blocking (WhisperTranscriber is not Send)
            let audio = samples.clone();
            // Note: We must transcribe while holding the lock since transcriber isn't Send
            // In production, consider a dedicated transcription thread
            match transcriber.transcribe(&audio, &lang) {
                Ok(text) => {
                    println!("Transcribed: {}", text);
                    let _ = h.emit("transcription-done", &text);
                }
                Err(e) => {
                    eprintln!("Transcription error: {}", e);
                    let _ = h.emit("transcription-error", &e);
                }
            }
        } else {
            eprintln!("No whisper model loaded");
            let _ = h.emit("transcription-error", "Model not loaded");
        }
    });
});
```

- [ ] **Step 2: Test full flow**

Run: `npm run tauri dev`
Test: Press and hold `Ctrl+Shift+Space`, speak, release. Console should print transcribed text.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: wire up full recording→transcription pipeline via hotkey events

Phase A complete: hotkey → audio capture → whisper transcription → console output"
```

---

## Chunk 2: Phase B — Overlay + Text-Injection

### Task 7: Text Injection Module

**Files:**
- Create: `src-tauri/src/inject.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create inject.rs**

```rust
use arboard::Clipboard;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};

pub fn inject_text(text: &str) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;

    // Save current clipboard content
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
```

- [ ] **Step 2: Integrate inject into transcription pipeline in lib.rs**

In the `recording-stopped` listener, after successful transcription:
```rust
Ok(text) => {
    println!("Transcribed: {}", text);
    if !text.is_empty() {
        if let Err(e) = inject::inject_text(&text) {
            eprintln!("Injection failed: {}", e);
            let _ = h.emit("transcription-error",
                &format!("Text kopiert, bitte manuell Ctrl+V drücken: {}", e));
        }
    }
    let _ = h.emit("transcription-done", &text);
}
```

- [ ] **Step 3: Test text injection**

Run: `npm run tauri dev`
Test: Open Notepad, press hotkey, speak, release. Text should appear in Notepad.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/inject.rs src-tauri/src/lib.rs
git commit -m "feat: add text injection via clipboard + simulated Ctrl+V"
```

---

### Task 8: Overlay Window Management

**Files:**
- Create: `src-tauri/src/overlay.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create overlay.rs**

```rust
use tauri::{AppHandle, Manager, PhysicalPosition};

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
```

- [ ] **Step 2: Call overlay positioning in lib.rs setup**

```rust
// In .setup(|app| { ... }), after hotkey registration
let settings = config::load_settings();
overlay::set_overlay_position(&app.handle(), &settings.overlay_position)
    .unwrap_or_else(|e| eprintln!("Overlay error: {}", e));
```

- [ ] **Step 3: Verify overlay window appears**

Run: `npm run tauri dev`
Expected: Small frameless window appears in bottom-right corner.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/overlay.rs src-tauri/src/lib.rs
git commit -m "feat: add overlay window positioning for all 4 screen corners"
```

---

### Task 9: Tauri IPC Helpers (Frontend)

**Files:**
- Create: `src/lib/tauri.ts`

- [ ] **Step 1: Create typed Tauri wrapper**

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Types
export interface Settings {
  hotkey: string;
  language: string;
  model: string;
  overlay_position: string;
  activation_mode: string;
  autostart: boolean;
}

// Commands
export const commands = {
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) =>
    invoke("save_settings_cmd", { settings }),
  startRecording: () => invoke("start_recording"),
  stopRecording: () => invoke<string>("stop_recording"),
  setHotkey: (hotkey: string) => invoke("set_hotkey", { hotkey }),
  downloadModel: (model: string) =>
    invoke("download_model_cmd", { model }),
  getAvailableModels: () => invoke<string[]>("get_available_models_cmd"),
};

// Events
export const events = {
  onFftData: (cb: (data: number[]) => void): Promise<UnlistenFn> =>
    listen<number[]>("fft-data", (e) => cb(e.payload)),
  onRecordingStarted: (cb: () => void): Promise<UnlistenFn> =>
    listen("recording-started", () => cb()),
  onRecordingStopped: (cb: () => void): Promise<UnlistenFn> =>
    listen("recording-stopped", () => cb()),
  onTranscribingStarted: (cb: () => void): Promise<UnlistenFn> =>
    listen("transcribing-started", () => cb()),
  onTranscriptionDone: (cb: (text: string) => void): Promise<UnlistenFn> =>
    listen<string>("transcription-done", (e) => cb(e.payload)),
  onTranscriptionError: (cb: (error: string) => void): Promise<UnlistenFn> =>
    listen<string>("transcription-error", (e) => cb(e.payload)),
  onModelDownloadProgress: (
    cb: (percent: number) => void
  ): Promise<UnlistenFn> =>
    listen<number>("model-download-progress", (e) => cb(e.payload)),
};
```

- [ ] **Step 2: Commit**

```bash
git add src/lib/tauri.ts
git commit -m "feat: add typed Tauri IPC wrappers for commands and events"
```

---

### Task 10: useRecording Hook (State Machine)

**Files:**
- Create: `src/hooks/useRecording.ts`

- [ ] **Step 1: Create state machine hook**

```typescript
import { useState, useEffect, useCallback } from "react";
import { events } from "../lib/tauri";

export type RecordingState = "idle" | "recording" | "transcribing" | "done";

export function useRecording() {
  const [state, setState] = useState<RecordingState>("idle");
  const [fftData, setFftData] = useState<number[]>(new Array(16).fill(0));
  const [lastText, setLastText] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisteners: Promise<() => void>[] = [];

    unlisteners.push(
      events.onRecordingStarted(() => {
        setState("recording");
        setError(null);
      })
    );

    unlisteners.push(
      events.onFftData((data) => {
        setFftData(data);
      })
    );

    unlisteners.push(
      events.onRecordingStopped(() => {
        // Don't set state here — wait for transcribing-started
      })
    );

    unlisteners.push(
      events.onTranscribingStarted(() => {
        setState("transcribing");
        setFftData(new Array(16).fill(0)); // Freeze dots
      })
    );

    unlisteners.push(
      events.onTranscriptionDone((text) => {
        setState("done");
        setLastText(text);
        // Auto-return to idle after 1.5s
        setTimeout(() => setState("idle"), 1500);
      })
    );

    unlisteners.push(
      events.onTranscriptionError((err) => {
        setError(err);
        setState("idle");
      })
    );

    return () => {
      unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
    };
  }, []);

  return { state, fftData, lastText, error };
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/useRecording.ts
git commit -m "feat: add useRecording state machine hook with FFT data"
```

---

### Task 11: Overlay Pill Component

**Files:**
- Create: `src/components/Overlay.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Create Overlay.tsx pill widget**

```tsx
import { useRecording, type RecordingState } from "../hooks/useRecording";

function Overlay() {
  const { state, fftData } = useRecording();

  return (
    <div className="flex items-center justify-center w-screen h-screen">
      <PillWidget state={state} fftData={fftData} />
    </div>
  );
}

function PillWidget({
  state,
  fftData,
}: {
  state: RecordingState;
  fftData: number[];
}) {
  const isActive = state === "recording" || state === "transcribing";
  const isDone = state === "done";

  return (
    <div
      className={`
        flex items-center justify-center gap-[3px] rounded-full
        border backdrop-blur-xl transition-all duration-300 ease-out
        ${isDone
          ? "w-[140px] h-[40px] border-green-500/20 bg-green-500/4"
          : isActive
            ? "w-[180px] h-[44px] border-violet-500/20 bg-violet-500/4"
            : "w-[140px] h-[40px] border-white/8 bg-white/2"
        }
      `}
      style={{
        animation: isDone ? "pop-in 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)" : undefined,
      }}
    >
      {state === "idle" && null}

      {state === "recording" && <FrequencyDots fftData={fftData} />}

      {state === "transcribing" && <TranscribingView />}

      {state === "done" && <DoneCheck />}
    </div>
  );
}

function FrequencyDots({ fftData }: { fftData: number[] }) {
  // Map 16 bins to 12 dots via neighbor averaging
  const dots = Array.from({ length: 12 }, (_, i) => {
    const idx = (i / 12) * 16;
    const lo = Math.floor(idx);
    const hi = Math.min(lo + 1, 15);
    const frac = idx - lo;
    return fftData[lo] * (1 - frac) + fftData[hi] * frac;
  });

  return (
    <div className="flex items-center gap-[4px] px-6">
      {dots.map((value, i) => (
        <div
          key={i}
          className="w-[5px] rounded-full bg-violet-400/85 transition-all duration-75"
          style={{
            height: `${Math.max(5, value * 28)}px`,
            opacity: 0.4 + value * 0.6,
          }}
        />
      ))}
    </div>
  );
}

function TranscribingView() {
  return (
    <div className="flex items-center gap-2 px-4">
      <div className="flex items-center gap-[2.5px]">
        {Array.from({ length: 12 }).map((_, i) => (
          <div
            key={i}
            className="w-[4px] h-[4px] rounded-full bg-violet-400/50"
          />
        ))}
      </div>
      <div className="w-[18px] h-[18px] border-2 border-violet-500/15 border-t-violet-400/70 rounded-full animate-spin" />
    </div>
  );
}

function DoneCheck() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <path
        d="M3.5 8l3 3 6-6"
        stroke="#22c55e"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export default Overlay;
```

- [ ] **Step 2: Update App.tsx to route by window label**

```tsx
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import Overlay from "./components/Overlay";

function App() {
  const [windowLabel, setWindowLabel] = useState<string>("");

  useEffect(() => {
    setWindowLabel(getCurrentWindow().label);
  }, []);

  if (windowLabel === "overlay") {
    return <Overlay />;
  }

  // Settings window (Phase C)
  return (
    <div className="p-6">
      <h1 className="text-xl font-semibold">LocalVoice Einstellungen</h1>
      <p className="text-white/50 mt-2">Kommt in Phase C...</p>
    </div>
  );
}

export default App;
```

- [ ] **Step 3: Add pop-in keyframe to globals.css**

```css
@keyframes pop-in {
  from {
    opacity: 0;
    transform: scale(0.9);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
```

- [ ] **Step 4: Verify overlay displays**

Run: `npm run tauri dev`
Expected: Pill appears in corner. When hotkey is pressed, dots animate with mic FFT data.

- [ ] **Step 5: Commit**

```bash
git add src/components/Overlay.tsx src/App.tsx src/styles/globals.css
git commit -m "feat: add overlay pill widget with FFT dots, transcribing spinner, done check

Phase B complete: full dictation pipeline with visual overlay and text injection"
```

---

## Chunk 3: Phase C — Settings + Polish

### Task 12: useSettings Hook

**Files:**
- Create: `src/hooks/useSettings.ts`

- [ ] **Step 1: Create settings hook**

```typescript
import { useState, useEffect, useCallback } from "react";
import { commands, type Settings } from "../lib/tauri";

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    commands.getSettings().then((s) => {
      setSettings(s);
      setLoading(false);
    });
  }, []);

  const update = useCallback(
    async (partial: Partial<Settings>) => {
      if (!settings) return;
      const updated = { ...settings, ...partial };
      setSettings(updated);
      await commands.saveSettings(updated);

      // If hotkey changed, re-register it
      if (partial.hotkey) {
        await commands.setHotkey(partial.hotkey);
      }
    },
    [settings]
  );

  return { settings, loading, update };
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/useSettings.ts
git commit -m "feat: add useSettings hook with auto-persist and hotkey re-registration"
```

---

### Task 13: Model Manager

**Files:**
- Create: `src-tauri/src/model_manager.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/components/ModelManager.tsx`

- [ ] **Step 1: Create model_manager.rs**

```rust
use reqwest;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

pub struct ModelInfo {
    pub name: String,
    pub filename: String,
    pub url: String,
    pub size_mb: u64,
}

pub fn get_available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            name: "tiny".to_string(),
            filename: "ggml-tiny.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".to_string(),
            size_mb: 75,
        },
        ModelInfo {
            name: "base".to_string(),
            filename: "ggml-base.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string(),
            size_mb: 148,
        },
        ModelInfo {
            name: "small".to_string(),
            filename: "ggml-small.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string(),
            size_mb: 488,
        },
        ModelInfo {
            name: "medium".to_string(),
            filename: "ggml-medium.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
            size_mb: 1533,
        },
        ModelInfo {
            name: "large-v3".to_string(),
            filename: "ggml-large-v3.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".to_string(),
            size_mb: 3095,
        },
    ]
}

pub async fn download_model(
    app: AppHandle,
    model_name: &str,
    models_dir: PathBuf,
) -> Result<(), String> {
    let models = get_available_models();
    let model = models
        .iter()
        .find(|m| m.name == model_name)
        .ok_or(format!("Unknown model: {}", model_name))?;

    std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    let dest = models_dir.join(&model.filename);

    let response = reqwest::get(&model.url)
        .await
        .map_err(|e| format!("Download failed: {}", e))?;

    let total = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(&dest)
        .await
        .map_err(|e| format!("File create failed: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;

        if total > 0 {
            let percent = (downloaded as f64 / total as f64 * 100.0) as u32;
            let _ = app.emit("model-download-progress", percent);
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Add futures-util to Cargo.toml**

```toml
futures-util = "0.3"
```

- [ ] **Step 3: Register download commands in lib.rs**

```rust
#[tauri::command]
async fn download_model_cmd(
    app_handle: tauri::AppHandle,
    model: String,
) -> Result<(), String> {
    let models_dir = config::get_models_path();
    model_manager::download_model(app_handle, &model, models_dir).await
}

#[tauri::command]
fn get_available_models_cmd() -> Vec<serde_json::Value> {
    model_manager::get_available_models()
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "name": m.name,
                "filename": m.filename,
                "size_mb": m.size_mb,
                "downloaded": config::get_models_path().join(&m.filename).exists(),
            })
        })
        .collect()
}
```

- [ ] **Step 4: Create ModelManager.tsx**

```tsx
import { useState, useEffect } from "react";
import { commands, events } from "../lib/tauri";

interface ModelEntry {
  name: string;
  filename: string;
  size_mb: number;
  downloaded: boolean;
}

export function ModelManager({ currentModel }: { currentModel: string }) {
  const [models, setModels] = useState<ModelEntry[]>([]);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    commands.getAvailableModels().then((m) => setModels(m as any));
    const unlisten = events.onModelDownloadProgress((p) => setProgress(p));
    return () => { unlisten.then((u) => u()); };
  }, []);

  const handleDownload = async (model: string) => {
    setDownloading(model);
    setProgress(0);
    try {
      await commands.downloadModel(model);
      // Refresh model list
      const updated = await commands.getAvailableModels();
      setModels(updated as any);
    } catch (e) {
      console.error("Download failed:", e);
    }
    setDownloading(null);
  };

  return (
    <div className="flex flex-col gap-3">
      {models.map((m) => (
        <div
          key={m.name}
          className="flex items-center justify-between p-3 rounded-xl
                     bg-white/4 border border-white/6"
        >
          <div>
            <div className="text-sm font-medium text-white/80">{m.name}</div>
            <div className="text-xs text-white/35">{m.size_mb} MB</div>
          </div>
          <div className="flex items-center gap-2">
            {m.name === currentModel && (
              <span className="text-xs px-2 py-1 rounded bg-violet-500/20 border border-violet-500/30 text-white/70">
                Aktiv
              </span>
            )}
            {m.downloaded ? (
              <span className="text-xs text-green-400/60">✓</span>
            ) : downloading === m.name ? (
              <div className="flex items-center gap-2">
                <div className="w-20 h-1.5 bg-white/10 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-violet-500 rounded-full transition-all duration-300"
                    style={{ width: `${progress}%` }}
                  />
                </div>
                <span className="text-xs text-white/40">{progress}%</span>
              </div>
            ) : (
              <button
                onClick={() => handleDownload(m.name)}
                className="text-xs px-3 py-1 rounded-lg bg-violet-500/15 border border-violet-500/20
                           text-violet-300 hover:bg-violet-500/25 transition-colors"
              >
                Download
              </button>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/model_manager.rs src-tauri/Cargo.toml src-tauri/src/lib.rs src/components/ModelManager.tsx
git commit -m "feat: add model manager with HuggingFace download and progress UI"
```

---

### Task 14: Settings UI

**Files:**
- Create: `src/components/Settings.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Create Settings.tsx**

```tsx
import { useSettings } from "../hooks/useSettings";
import { ModelManager } from "./ModelManager";

const LANGUAGES = [
  { code: "de", label: "Deutsch" },
  { code: "en", label: "English" },
  { code: "fr", label: "Français" },
  { code: "es", label: "Español" },
  { code: "it", label: "Italiano" },
  { code: "pt", label: "Português" },
  { code: "nl", label: "Nederlands" },
  { code: "pl", label: "Polski" },
  { code: "ru", label: "Русский" },
  { code: "ja", label: "日本語" },
  { code: "zh", label: "中文" },
  { code: "ko", label: "한국어" },
];

const POSITIONS = [
  { value: "top-left", label: "Oben Links" },
  { value: "top-right", label: "Oben Rechts" },
  { value: "bottom-left", label: "Unten Links" },
  { value: "bottom-right", label: "Unten Rechts" },
];

export function Settings() {
  const { settings, loading, update } = useSettings();

  if (loading || !settings) {
    return <div className="p-6 text-white/40">Laden...</div>;
  }

  return (
    <div className="p-6 max-w-lg mx-auto flex flex-col gap-6">
      <h1 className="text-lg font-semibold text-white/90 tracking-tight">
        Einstellungen
      </h1>

      {/* Hotkey */}
      <SettingCard label="Hotkey">
        <input
          type="text"
          value={settings.hotkey}
          onChange={(e) => update({ hotkey: e.target.value })}
          className="text-sm bg-violet-500/10 border border-violet-500/20 rounded-lg
                     px-3 py-1.5 text-white/80 outline-none focus:border-violet-500/40
                     w-48 text-right"
          placeholder="z.B. ctrl+shift+space"
        />
      </SettingCard>

      {/* Language */}
      <SettingCard label="Sprache">
        <select
          value={settings.language}
          onChange={(e) => update({ language: e.target.value })}
          className="text-sm bg-violet-500/10 border border-violet-500/20 rounded-lg
                     px-3 py-1.5 text-white/80 outline-none"
        >
          {LANGUAGES.map((l) => (
            <option key={l.code} value={l.code}>{l.label}</option>
          ))}
        </select>
      </SettingCard>

      {/* Overlay Position */}
      <SettingCard label="Overlay Position">
        <select
          value={settings.overlay_position}
          onChange={(e) => update({ overlay_position: e.target.value })}
          className="text-sm bg-violet-500/10 border border-violet-500/20 rounded-lg
                     px-3 py-1.5 text-white/80 outline-none"
        >
          {POSITIONS.map((p) => (
            <option key={p.value} value={p.value}>{p.label}</option>
          ))}
        </select>
      </SettingCard>

      {/* Model */}
      <div className="flex flex-col gap-3">
        <span className="text-xs text-white/40 uppercase tracking-wider">
          Whisper Modell
        </span>
        <ModelManager currentModel={settings.model} />
      </div>

      {/* Version */}
      <div className="text-xs text-white/20 text-center mt-4">
        LocalVoice v0.1.0 — 100% Offline
      </div>
    </div>
  );
}

function SettingCard({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between p-3 rounded-xl bg-white/4 border border-white/6">
      <span className="text-sm text-white/50">{label}</span>
      {children}
    </div>
  );
}
```

- [ ] **Step 2: Update App.tsx to render Settings**

```tsx
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import Overlay from "./components/Overlay";
import { Settings } from "./components/Settings";

function App() {
  const [windowLabel, setWindowLabel] = useState<string>("");

  useEffect(() => {
    setWindowLabel(getCurrentWindow().label);
  }, []);

  if (windowLabel === "overlay") {
    return <Overlay />;
  }

  return <Settings />;
}

export default App;
```

- [ ] **Step 3: Verify settings window**

Run: `npm run tauri dev`
Expected: Settings window shows when opened from tray. Overlay pill shows in corner.

- [ ] **Step 4: Commit**

```bash
git add src/components/Settings.tsx src/App.tsx
git commit -m "feat: add settings UI with hotkey, language, model, overlay position"
```

---

### Task 15: System Tray

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add tray icon setup to lib.rs**

```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

// In .setup(|app| { ... })
let settings_item = MenuItem::with_id(app, "settings", "Einstellungen", true, None::<&str>)
    .map_err(|e| e.to_string())?;
let quit_item = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)
    .map_err(|e| e.to_string())?;

let menu = Menu::with_items(app, &[&settings_item, &quit_item])
    .map_err(|e| e.to_string())?;

TrayIconBuilder::new()
    .menu(&menu)
    .tooltip("LocalVoice — Bereit")
    .on_menu_event(|app, event| match event.id.as_ref() {
        "settings" => {
            if let Some(win) = app.get_webview_window("settings") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            if let Some(win) = app.get_webview_window("settings") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
    })
    .build(app)
    .map_err(|e| e.to_string())?;
```

- [ ] **Step 2: Create a placeholder tray icon**

Create `src-tauri/icons/icon.png` — a simple 32x32 PNG. Can be generated or use a placeholder.

- [ ] **Step 3: Verify tray**

Run: `npm run tauri dev`
Expected: Tray icon appears. Right-click shows "Einstellungen" and "Beenden". Left-click opens settings.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/icons/
git commit -m "feat: add system tray with settings and quit menu"
```

---

### Task 16: Setup Wizard (First Run)

**Files:**
- Create: `src/components/SetupWizard.tsx`
- Modify: `src/components/Settings.tsx`

- [ ] **Step 1: Create SetupWizard.tsx**

```tsx
import { useState } from "react";
import { commands } from "../lib/tauri";
import { ModelManager } from "./ModelManager";

type Step = "welcome" | "hotkey" | "model" | "done";

export function SetupWizard({ onComplete }: { onComplete: () => void }) {
  const [step, setStep] = useState<Step>("welcome");
  const [hotkey, setHotkey] = useState("ctrl+shift+space");

  return (
    <div className="p-8 max-w-md mx-auto flex flex-col items-center justify-center min-h-screen gap-6">
      {step === "welcome" && (
        <>
          <h1 className="text-2xl font-semibold text-white/90">
            Willkommen bei LocalVoice
          </h1>
          <p className="text-sm text-white/50 text-center leading-relaxed">
            Deine Stimme. Dein Gerät. Kein Cloud.
            <br />
            Drücke eine Taste, sprich, und deine Worte erscheinen — überall.
          </p>
          <button
            onClick={() => setStep("hotkey")}
            className="mt-4 px-6 py-2.5 rounded-xl bg-violet-500/20 border border-violet-500/30
                       text-white/80 hover:bg-violet-500/30 transition-colors text-sm"
          >
            Los geht's
          </button>
        </>
      )}

      {step === "hotkey" && (
        <>
          <h2 className="text-lg font-semibold text-white/90">Hotkey festlegen</h2>
          <p className="text-sm text-white/40 text-center">
            Diese Tastenkombination startet und stoppt die Aufnahme.
          </p>
          <input
            type="text"
            value={hotkey}
            onChange={(e) => setHotkey(e.target.value)}
            className="text-sm bg-violet-500/10 border border-violet-500/20 rounded-lg
                       px-4 py-2 text-white/80 outline-none focus:border-violet-500/40
                       text-center w-64"
            placeholder="z.B. ctrl+shift+space"
          />
          <button
            onClick={async () => {
              await commands.setHotkey(hotkey);
              setStep("model");
            }}
            className="px-6 py-2 rounded-xl bg-violet-500/20 border border-violet-500/30
                       text-white/80 hover:bg-violet-500/30 transition-colors text-sm"
          >
            Weiter
          </button>
        </>
      )}

      {step === "model" && (
        <>
          <h2 className="text-lg font-semibold text-white/90">Modell herunterladen</h2>
          <p className="text-sm text-white/40 text-center">
            Das Sprachmodell wird lokal gespeichert (~150 MB für base).
          </p>
          <div className="w-full">
            <ModelManager currentModel="base" />
          </div>
          <button
            onClick={() => setStep("done")}
            className="px-6 py-2 rounded-xl bg-violet-500/20 border border-violet-500/30
                       text-white/80 hover:bg-violet-500/30 transition-colors text-sm"
          >
            Weiter
          </button>
        </>
      )}

      {step === "done" && (
        <>
          <div className="text-4xl">✓</div>
          <h2 className="text-lg font-semibold text-white/90">Fertig!</h2>
          <p className="text-sm text-white/40 text-center">
            LocalVoice läuft jetzt im System Tray.
            <br />
            Drücke <span className="text-violet-400">{hotkey}</span> zum Diktieren.
          </p>
          <button
            onClick={onComplete}
            className="mt-2 px-6 py-2 rounded-xl bg-violet-500/20 border border-violet-500/30
                       text-white/80 hover:bg-violet-500/30 transition-colors text-sm"
          >
            Starten
          </button>
        </>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Add first-run detection in Settings.tsx**

At the top of `Settings`:
```tsx
import { SetupWizard } from "./SetupWizard";

// In the Settings component, check if first run
const [firstRun, setFirstRun] = useState(false);

useEffect(() => {
  // Check if model exists — if not, it's first run
  commands.getAvailableModels().then((models: any) => {
    const hasAny = models.some((m: any) => m.downloaded);
    setFirstRun(!hasAny);
  });
}, []);

if (firstRun) {
  return <SetupWizard onComplete={() => setFirstRun(false)} />;
}
```

- [ ] **Step 3: Verify setup wizard**

Run: `npm run tauri dev` (without any model in models/)
Expected: Settings window shows wizard. After downloading model, shows normal settings.

- [ ] **Step 4: Commit**

```bash
git add src/components/SetupWizard.tsx src/components/Settings.tsx
git commit -m "feat: add first-run setup wizard with hotkey + model download

Phase C complete: settings UI, model manager, system tray, setup wizard.
LocalVoice MVP v0.1.0 is feature-complete."
```

---

### Task 17: Final Wiring — generate_handler and polish

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/components/Overlay.tsx`
- Modify: `src/components/SetupWizard.tsx`

- [ ] **Step 1: Update lib.rs with complete generate_handler**

Ensure the final `invoke_handler` includes ALL commands:
```rust
.invoke_handler(tauri::generate_handler![
    get_settings,
    save_settings_cmd,
    start_recording,
    stop_recording,
    set_hotkey,
    download_model_cmd,
    get_available_models_cmd,
])
```

- [ ] **Step 2: Add fade-out animation to Overlay Done state**

In `Overlay.tsx`, update the PillWidget done state:
```tsx
// Add to the done state div:
const [fading, setFading] = useState(false);

// In useEffect, when state becomes "done", start fade after 1s:
useEffect(() => {
  if (state === "done") {
    const timer = setTimeout(() => setFading(true), 1000);
    return () => clearTimeout(timer);
  }
  setFading(false);
}, [state]);

// Add to the pill div className:
${fading ? "opacity-0 transition-opacity duration-500" : ""}
```

- [ ] **Step 3: Add mic test step to SetupWizard**

Add a "mic-test" step between "model" and "done":
```tsx
{step === "mic-test" && (
  <>
    <h2 className="text-lg font-semibold text-white/90">Mikrofon testen</h2>
    <p className="text-sm text-white/40 text-center">
      Drücke den Button und sprich kurz. Du solltest die Dots animieren sehen.
    </p>
    <div className="w-[180px] h-[44px] rounded-full border border-violet-500/20 bg-violet-500/4
                    flex items-center justify-center gap-[4px] px-6">
      {/* Show FFT dots here — reuse FrequencyDots */}
    </div>
    <button onClick={() => setStep("done")} className="...">Weiter</button>
  </>
)}
```

- [ ] **Step 4: Add reload_model command for model switching**

In `lib.rs`:
```rust
#[tauri::command]
async fn reload_model(
    model: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let model_path = config::get_models_path()
        .join(format!("ggml-{}.bin", model));
    if !model_path.exists() {
        return Err(format!("Model not found: {}", model));
    }
    let transcriber = whisper::WhisperTranscriber::new(&model_path)?;
    *state.transcriber.lock().await = Some(transcriber);
    let mut settings = config::load_settings();
    settings.model = model;
    config::save_settings(&settings);
    Ok(())
}
```

Add `reload_model` to `generate_handler!` and to `tauri.ts`:
```typescript
reloadModel: (model: string) => invoke("reload_model", { model }),
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: final wiring — complete command registry, fade-out, mic test, model reload"
```

---

## Post-Implementation Checklist

- [ ] **Manual test:** Full flow — hotkey → record → transcribe → inject into Notepad
- [ ] **Manual test:** Settings changes persist across restart
- [ ] **Manual test:** Overlay shows correct states (idle/recording/transcribing/done)
- [ ] **Manual test:** Tray icon works (left-click opens settings, right-click menu)
- [ ] **Manual test:** Model download with progress bar
- [ ] **Manual test:** Setup wizard on first run
- [ ] **Build release:** `npm run tauri build` — verify portable exe
- [ ] **Verify portable:** Copy build output to new folder, run — should work without install
