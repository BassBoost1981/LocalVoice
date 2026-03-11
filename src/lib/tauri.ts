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
  setHotkey: (hotkeyStr: string) => invoke("set_hotkey", { hotkeyStr }),
  downloadModel: (model: string) =>
    invoke("download_model_cmd", { model }),
  getAvailableModels: () => invoke<string[]>("get_available_models_cmd"),
  reloadModel: (model: string) => invoke("reload_model", { model }),
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
