# LocalVoice MVP — Design Specification

**Version:** 0.1.0
**Datum:** 2026-03-11
**Status:** Approved

---

## 1. Überblick

LocalVoice ist ein portables, vollständig offline-fähiges Diktiertool für Windows. Per Hotkey wird eine Aufnahme gestartet, lokal via whisper.cpp transkribiert und der Text direkt in das aktive Fenster injiziert.

**Problem:** Existierende Diktiertools sind cloud-basiert, kostenpflichtig oder haben eingeschränkte App-Kompatibilität.

**User:** Privacy-bewusste Knowledge Worker, Entwickler, Content Creator auf Windows 10/11.

**Erfolg:** Hotkey drücken → sprechen → loslassen → Text erscheint. Unter 2 Sekunden Latenz, 100% offline.

---

## 2. Architektur

### 2.1 Ansatz: Monolith

Alles in einem Tauri 2.0 Prozess. Kein Sidecar, kein WASM. whisper-rs läuft in `tokio::spawn_blocking`.

### 2.2 Tech Stack

**Frontend:**
- React 19, TypeScript, Vite
- Tailwind CSS v4, shadcn/ui
- Tauri JS API (`@tauri-apps/api`)
- Font: Lexend (Variable, alle Gewichte)

**Backend (Rust):**
- `tauri` 2.0
- `whisper-rs` — whisper.cpp Bindings
- `cpal` — Audio Input
- `rustfft` — FFT für Frequency Visualizer
- `windows-rs` — Text-Injection (SendInput)
- `arboard` — Clipboard
- `tauri-plugin-global-shortcut` — Hotkeys
- `reqwest` — Modell-Download
- `serde` / `serde_json` — Config
- `tokio` — Async Runtime

### 2.3 Datenfluss

```
Hotkey gedrückt
    │
    ▼
hotkey.rs → Event "recording-started"
    │
    ▼
audio.rs startet cpal-Stream (16kHz Mono f32)
    │
    ├──► FFT (rustfft, 512 Samples) → 16 Bins → Event "fft-data" @30fps
    │                                                    │
    │                                              Frontend: Overlay
    │                                              zeigt Dots mit
    │                                              echten FFT-Daten
    │
    └──► PCM-Samples → Buffer (Vec<f32>)
              │
Hotkey losgelassen
              │
              ▼
        whisper.rs (spawn_blocking)
              │
              ▼
        Event "transcription-done" { text }
              │
              ▼
        inject.rs: Text → Clipboard → Ctrl+V simulieren
              │
              ▼
        Event → Frontend: Overlay zeigt "Done" → Fade-Out
```

### 2.4 Portabilität

Kein `%USERPROFILE%`, kein Registry. Alles relativ zum Exe-Pfad:

```
LocalVoice/
├── LocalVoice.exe
├── config/
│   └── settings.json
├── models/
│   └── ggml-base.bin
└── data/
```

---

## 3. Design-System

### 3.1 Visuell

- **Theme:** Dark-only, Basis `#09090b` (zinc-950)
- **Akzent:** Indigo/Violet (`#6366f1` → `#a78bfa`)
- **Font:** Lexend (Variable Font, alle Gewichte)
- **Borders:** `rgba(139, 92, 246, 0.12)`, glasmorph
- **Radii:** 16px (Cards), 10px (Inputs), 6px (Badges), 999px (Pill/Overlay)
- **States:** Rot=Recording, Violet=Transcribing, Grün=Done

### 3.2 Overlay (Pill-Widget)

Kompakte Pill-Form, always-on-top, frameless, transparent. Position wählbar (4 Ecken).

| State | Aussehen | Größe |
|-------|----------|-------|
| **Idle** | Dünner Border, fast unsichtbar | ~140x40px |
| **Recording** | Dots-Visualizer (echte FFT-Daten), Violet | ~180x44px |
| **Transcribing** | Eingefrorene Dots + Spinner rechts | ~180x44px |
| **Done** | Grüner Checkmark, Pop-In → Fade-Out 1.5s | ~140x40px |

**Visualizer:** 12 Dots, angetrieben von echten FFT-Frequency-Bands vom Mikrofon (16 Bins → 12 Dots via Nachbar-Bin-Mittelung). CSS-Animation nur als Fallback.

### 3.3 Settings-Fenster

Normales Tauri-Fenster (~500x600px), glasmorphe Input-Cards auf dunklem Hintergrund.

---

## 4. Rust-Module

| Modul | Crate | Verantwortung |
|-------|-------|---------------|
| `main.rs` / `lib.rs` | `tauri` | App Bootstrap, Command Registry |
| `audio.rs` | `cpal`, `rustfft` | Mikrofon-Stream, PCM-Buffer, FFT → 16 Bins @30fps |
| `whisper.rs` | `whisper-rs` (CPU-only, kein CUDA) | Modell laden, Transkription in `spawn_blocking` |
| `hotkey.rs` | `tauri-plugin-global-shortcut` | Global Shortcut, Push-to-Talk Events |
| `inject.rs` | `windows-rs`, `arboard` | Clipboard sichern → Text setzen → Ctrl+V simulieren → Clipboard wiederherstellen |
| `config.rs` | `serde` | settings.json lesen/schreiben, portabler Pfad |
| `overlay.rs` | `tauri` Window API | Frameless, transparent, always-on-top, Positionierung |
| `model_manager.rs` | `reqwest` | Download von Hugging Face, Progress-Events |

**Audio-Details:**
- Sample-Rate: 16kHz Mono
- Format: f32
- FFT: 512-Sample-Fenster, 16 Frequency-Bins
- Whisper erwartet 16kHz Mono f32

---

## 5. Frontend-Komponenten

| Komponente | Zweck |
|-----------|-------|
| `App.tsx` | Router: Overlay vs. Settings (via Tauri Window-Label) |
| `Overlay.tsx` | Pill-Widget, FFT-Event Listener, 4 States |
| `Settings.tsx` | Einstellungen-Panel |
| `ModelManager.tsx` | Modell-Liste, Download, Progress |
| `SetupWizard.tsx` | First-Run: Hotkey → Modell → Mikrofon-Test |
| `useRecording.ts` | State-Machine Hook (idle→recording→transcribing→done→idle) |
| `useSettings.ts` | Settings laden/speichern via invoke |
| `tauri.ts` | Typisierte Wrapper für Commands & Events |

### 5.1 Tauri-Fenster

1. **Overlay** — frameless, transparent, always-on-top, ~200x50px, nicht resizable
2. **Settings** — normales Fenster, ~500x600px, aus Tray öffenbar

### 5.2 Tauri Commands (invoke)

- `start_recording` / `stop_recording`
- `get_settings` / `save_settings`
- `download_model` / `get_available_models`
- `set_hotkey`

### 5.3 Tauri Events (listen)

- `fft-data` → `number[]` (16 Bands, ~30fps)
- `recording-started` / `recording-stopped`
- `transcription-done` → `{ text: string }`
- `transcription-error` → `{ error: string }`
- `model-download-progress` → `{ percent: number }`

---

## 6. Einstellungen

| Einstellung | Optionen | Standard |
|-------------|----------|----------|
| Hotkey | Frei konfigurierbar | `Ctrl+Shift+Space` |
| Aktivierungsmodus | Push-to-Talk | Push-to-Talk (Toggle in v0.2) |
| Whisper-Modell | tiny/base/small/medium/large | base |
| Sprache | 58 Whisper-Sprachen | Systemsprache |
| Overlay-Position | 4 Ecken | Unten-Rechts |
| Autostart | Ein/Aus | Aus (Phase C) |
| Text-Injection | Clipboard | Clipboard |

---

## 7. Phasen

### Phase A — Kern-Engine
- Tauri Projektstruktur
- `audio.rs`: cpal Aufnahme + FFT
- `whisper.rs`: Modell laden, Transkription
- `hotkey.rs`: Global Shortcut
- `config.rs`: Portable settings.json
- Ergebnis: Hotkey → Aufnahme → Text in Konsole

### Phase B — Overlay + Injection
- `inject.rs`: Clipboard + Ctrl+V
- `overlay.rs`: Frameless always-on-top Fenster
- `Overlay.tsx`: Pill-Widget mit echten FFT-Dots
- `useRecording.ts`: State-Machine
- `tauri.ts`: IPC Wrapper
- Ergebnis: Sichtbares, nutzbares Diktiertool

### Phase C — Settings + Polish
- `Settings.tsx`: Einstellungen-UI
- `ModelManager.tsx`: Download + Progress
- `SetupWizard.tsx`: First-Run Onboarding
- System Tray: Icon, Rechtsklick-Menü
- `useSettings.ts`: Persistenz
- Ergebnis: Fertiges MVP

---

## 8. Nicht-funktionale Anforderungen

| Anforderung | Zielwert |
|-------------|----------|
| Transkriptionslatenz (base, 5s Audio) | < 2 Sekunden |
| App-Startzeit | < 1 Sekunde |
| RAM-Verbrauch (idle) | < 80 MB |
| Installationsgröße (ohne Modelle) | < 50 MB |
| Offline-Betrieb | 100% (außer Modell-Download) |
| Plattform | Windows 10/11 (64-bit) |
| Portabilität | Alles in einem Ordner, kein Installer nötig |

---

## 9. Out of Scope (MVP)

- Cloud-API Integration
- Transkriptions-Historie
- KI-Nachbearbeitung
- Toggle-Modus (v0.2)
- macOS/Linux Support (v0.3)
- Auto-Detect Sprache
- GPU-Beschleunigung (CUDA)
- Cursor-attached Overlay

---

## 10. Bekannte Einschränkungen

- **UAC-Fenster:** Text-Injection via Ctrl+V funktioniert nicht in Admin-erhöhten Fenstern (Windows-Limitierung). Overlay zeigt Fallback-Hinweis "Bitte manuell Ctrl+V drücken".
- **Modell-Quelle:** Downloads von `huggingface.co/ggerganov/whisper.cpp` (ggml-Format).
- **Clipboard:** Vorheriger Clipboard-Inhalt wird vor Injection gesichert und danach wiederhergestellt.
- **Fehler-Handling:** Mikrofon nicht vorhanden → Overlay zeigt Fehlermeldung. Modell fehlt → Setup-Wizard startet. Transkriptionsfehler → Overlay zeigt kurzen Error-State.
