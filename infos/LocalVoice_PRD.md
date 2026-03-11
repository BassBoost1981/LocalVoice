# LocalVoice – Product Requirements Document (PRD)

**Version:** 0.1.0  
**Status:** Draft  
**Datum:** 2026-03-11  
**Autor:** tbd  

---

## 1. Produktübersicht

### 1.1 Vision

LocalVoice ist ein vollständig offline-fähiges, system-weites Diktierwerkzeug für den Desktop. Es ermöglicht Nutzern, per Sprache Text direkt in jede Anwendung zu diktieren – genau dort, wo der Cursor sich befindet. Kein Cloud-Zwang, keine API-Keys, keine Datenweitergabe.

### 1.2 Elevator Pitch

> "Drücke eine Taste, sprich, und deine Worte erscheinen – überall, sofort, komplett privat."

### 1.3 Inspiration & Abgrenzung

| Produkt | Unterschied zu LocalVoice |
|---|---|
| WisperFlow | Cloud-basiert, kostenpflichtig |
| OpenWhispr | Python-Backend, optional Cloud |
| Windows Diktat (Win+H) | Begrenzte App-Kompatibilität, Online |
| **LocalVoice** | 100% offline, whisper.cpp, Tauri, Windows-first |

---

## 2. Zielgruppe

### 2.1 Primäre Nutzer

- **Knowledge Worker** – schreiben viele E-Mails, Dokumente, Slack-Nachrichten
- **Entwickler** – diktieren Code-Kommentare, Commit-Messages, Prompts an LLMs
- **Content Creator** – Blog-Posts, Skripte, Social Media schnell per Stimme erfassen
- **Privacy-bewusste Nutzer** – wollen keine Stimmdaten in der Cloud

### 2.2 Persona

**Max, 34, Software-Entwickler**
- Schreibt täglich 50+ Nachrichten in Slack, GitHub, Claude
- Tippt schnell, aber Sprache wäre noch schneller
- Misstraut Cloud-Diensten mit Audiodaten
- Nutzt Windows 11, gelegentlich Linux

---

## 3. Kernfunktionen (MVP)

### 3.1 System-weite Spracheingabe

- Hotkey aktiviert Aufnahme (global, auch wenn App im Hintergrund)
- Nach Transkription wird Text **direkt in das aktive Fenster** injiziert
- Funktioniert in: Browser, IDEs, Office, Chat-Apps, Terminals

### 3.2 Aktivierungsmodi

| Modus | Beschreibung |
|---|---|
| **Push-to-Talk** | Taste gedrückt halten → aufnehmen → loslassen → transkribieren |
| **Toggle** | Einmal drücken = Start, nochmal drücken = Stop & transkribieren |

Umschaltbar in den Einstellungen. Standard: Push-to-Talk.

### 3.3 Overlay-Indikator

- Kleines, schwebendes Fenster (always-on-top)
- Halbtransparent, nicht störend
- Zeigt Zustand:
  - 🔴 `Aufnahme läuft...` + Wellenform-Animation
  - ⚡ `Transkribiere...` + Ladeanimation
  - ✅ `Fertig` + kurze Einblendung, dann ausblenden
- Position frei wählbar (Ecke des Bildschirms)

### 3.4 Text-Injection

**Strategie (Windows-first):**
1. **Primär:** Text in Zwischenablage → `Ctrl+V` simulieren
2. **Fallback:** `SendInput` (Win32 API) für Apps die Clipboard blockieren
3. **Fallback 2:** Benutzer muss manuell `Ctrl+V` drücken (mit Hinweis im Overlay)

### 3.5 Modell-Management

- Beim ersten Start: Setup-Wizard lädt `base`-Modell herunter (~150 MB)
- Weitere Modelle in Einstellungen herunterladbar
- Modelle werden lokal gespeichert (`%USERPROFILE%\.localvoice\models\`)

| Modell | Größe | Geschwindigkeit | Qualität |
|---|---|---|---|
| tiny | ~75 MB | ⚡⚡⚡⚡ | ⭐⭐ |
| base | ~150 MB | ⚡⚡⚡ | ⭐⭐⭐ |
| small | ~500 MB | ⚡⚡ | ⭐⭐⭐⭐ |
| medium | ~1.5 GB | ⚡ | ⭐⭐⭐⭐⭐ |
| large-v3 | ~3 GB | 🐌 | ⭐⭐⭐⭐⭐ |

### 3.6 Spracheinstellung

- Sprache manuell festlegbar (kein Auto-Detect im MVP)
- Unterstützte Sprachen: alle 58 Whisper-Sprachen
- Standard: Systemsprache

### 3.7 System Tray

- App läuft im System Tray (kein permanentes Fenster)
- Rechtsklick-Menü: Einstellungen öffnen, Pause, Beenden
- Tray-Icon zeigt Status (aktiv / pausiert)

---

## 4. Einstellungen

### 4.1 Übersicht

| Einstellung | Optionen | Standard |
|---|---|---|
| Hotkey | Frei belegbar | `Alt + Space` |
| Aktivierungsmodus | Push-to-Talk / Toggle | Push-to-Talk |
| Whisper-Modell | tiny / base / small / medium / large | base |
| Sprache | 58 Sprachen | Systemsprache |
| Overlay-Position | Oben-Links / Oben-Rechts / Unten-Links / Unten-Rechts | Unten-Rechts |
| Autostart | Ein / Aus | Ein |
| Text-Injection | Clipboard / SendInput / Manuell | Clipboard |

---

## 5. Technischer Stack

### 5.1 Architektur

```
┌─────────────────────────────────────────┐
│           LocalVoice Desktop App        │
│                                         │
│  ┌─────────────┐    ┌────────────────┐  │
│  │  React 19   │    │   Rust Backend │  │
│  │  + Vite     │◄──►│   (Tauri 2.0) │  │
│  │  + shadcn/ui│    │               │  │
│  │  + Tailwind │    │  ┌──────────┐ │  │
│  └─────────────┘    │  │whisper.rs│ │  │
│                     │  │(whisper- │ │  │
│                     │  │   cpp)   │ │  │
│                     │  └──────────┘ │  │
│                     │  ┌──────────┐ │  │
│                     │  │audio.rs  │ │  │
│                     │  │(cpal)    │ │  │
│                     │  └──────────┘ │  │
│                     │  ┌──────────┐ │  │
│                     │  │inject.rs │ │  │
│                     │  │(windows- │ │  │
│                     │  │   rs)    │ │  │
│                     │  └──────────┘ │  │
│                     └────────────────┘  │
└─────────────────────────────────────────┘
```

### 5.2 Dependencies

**Frontend:**
- React 19
- TypeScript
- Vite
- Tailwind CSS v4
- shadcn/ui
- Tauri JS API (`@tauri-apps/api`)

**Backend (Rust):**
- `tauri` 2.0
- `whisper-rs` – whisper.cpp Bindings
- `cpal` – Cross-platform Audio Input
- `windows-rs` – Win32 API für Text-Injection & Hotkeys
- `tauri-plugin-global-shortcut` – System-weite Hotkeys
- `serde` / `serde_json` – Konfiguration
- `tokio` – Async Runtime

### 5.3 Projektstruktur

```
localvoice/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # App-Einstiegspunkt
│   │   ├── lib.rs            # Tauri Commands Registry
│   │   ├── whisper.rs        # whisper.cpp Integration
│   │   ├── audio.rs          # Mikrofon-Aufnahme (cpal)
│   │   ├── inject.rs         # Text-Injection (Windows/Mac/Linux)
│   │   ├── hotkey.rs         # Global Shortcut Handling
│   │   ├── overlay.rs        # Overlay-Fenster Management
│   │   └── config.rs         # Einstellungen (JSON)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── components/
│   │   ├── Overlay.tsx       # Aufnahme-Indikator UI
│   │   ├── Settings.tsx      # Einstellungen-Panel
│   │   ├── ModelManager.tsx  # Modell Download & Verwaltung
│   │   ├── SetupWizard.tsx   # First-Run Onboarding
│   │   └── TrayMenu.tsx      # System Tray Menü
│   ├── hooks/
│   │   ├── useRecording.ts   # Aufnahme State
│   │   └── useSettings.ts    # Einstellungen State
│   ├── lib/
│   │   └── tauri.ts          # Tauri IPC Helpers
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── vite.config.ts
├── tailwind.config.ts
└── README.md
```

---

## 6. User Flows

### 6.1 First Run

```
App starten
    │
    ▼
Setup Wizard
    │
    ├── Schritt 1: Willkommen & Erklärung
    ├── Schritt 2: Hotkey festlegen
    ├── Schritt 3: Modell wählen & herunterladen
    ├── Schritt 4: Mikrofon-Berechtigung
    └── Schritt 5: Fertig → App läuft im Tray
```

### 6.2 Diktat (Push-to-Talk)

```
Hotkey gedrückt halten
    │
    ▼
Overlay erscheint → 🔴 "Aufnahme..."
    │
    ▼
Nutzer spricht
    │
    ▼
Hotkey loslassen
    │
    ▼
Overlay → ⚡ "Transkribiere..."
    │
    ▼
whisper.cpp verarbeitet Audio (lokal)
    │
    ▼
Text wird in aktives Fenster injiziert
    │
    ▼
Overlay → ✅ "Fertig" (1 Sek.) → ausblenden
```

### 6.3 Diktat (Toggle)

```
Hotkey einmal drücken → Aufnahme startet
    │
    ▼
Hotkey nochmal drücken → Aufnahme stoppt → Transkription
```

---

## 7. Nicht-funktionale Anforderungen

| Anforderung | Zielwert |
|---|---|
| Transkriptionslatenz (base-Modell, 5 Sek. Audio) | < 2 Sekunden |
| App-Startzeit | < 1 Sekunde |
| RAM-Verbrauch (idle) | < 80 MB |
| Installationsgröße | < 50 MB (ohne Modelle) |
| Offline-Betrieb | 100% – keine Netzwerkanfragen |
| Plattform (MVP) | Windows 10/11 (64-bit) |
| Plattform (v2) | macOS 12+, Linux (Ubuntu 22+) |

---

## 8. Out of Scope (MVP)

- Cloud-API Integration (OpenAI, etc.)
- Transkriptions-Historie / Verlauf
- KI-Nachbearbeitung (Grammatikkorrektur)
- Mehrere Mikrofone gleichzeitig
- Meeting-Transkription
- Mobile App
- Auto-Detect Sprache

---

## 9. Roadmap

### v0.1 – MVP (Windows)
- [ ] Tauri Projektstruktur
- [ ] whisper.cpp Integration (whisper-rs)
- [ ] Mikrofon-Aufnahme (cpal)
- [ ] Push-to-Talk Hotkey
- [ ] Text-Injection via Clipboard
- [ ] Basis-Overlay
- [ ] Modell-Download (base)
- [ ] Einstellungen (Hotkey, Sprache, Modell)
- [ ] System Tray

### v0.2 – Polish
- [ ] Toggle-Modus
- [ ] Setup-Wizard (First Run)
- [ ] Alle Modelle (tiny → large)
- [ ] Overlay-Animationen
- [ ] SendInput Fallback
- [ ] Autostart mit Windows

### v0.3 – Plattformen
- [ ] macOS Support
- [ ] Linux Support
- [ ] Auto-Update

### v1.0 – Feature Complete
- [ ] Transkriptions-Historie
- [ ] Custom Dictionary
- [ ] KI-Nachbearbeitung (optional, lokal via llama.cpp)
- [ ] VAD (Voice Activity Detection)

---

## 10. Erfolgsmetriken

| Metrik | Ziel |
|---|---|
| GitHub Stars (3 Monate) | 500+ |
| Downloads (3 Monate) | 1.000+ |
| Transkriptionsgenauigkeit (base, Englisch) | > 90% |
| Crash-Rate | < 0.1% |
| Nutzer-Retention (7 Tage) | > 40% |

---

## 11. Offene Fragen

- [ ] Lizenz: MIT oder Apache 2.0?
- [ ] Windows Code-Signing Zertifikat notwendig für Distribution?
- [ ] Soll whisper.cpp als vorcompiliertes Binary mitgeliefert oder zur Laufzeit kompiliert werden?
- [ ] GPU-Beschleunigung (CUDA/Metal) in v1 oder erst später?
- [ ] Branding: Logo, Farben, Icon?

---

*LocalVoice – Deine Stimme. Dein Gerät. Kein Cloud.*
