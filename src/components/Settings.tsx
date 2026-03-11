import { useState, useEffect } from "react";
import { useSettings } from "../hooks/useSettings";
import { ModelManager } from "./ModelManager";
import { SetupWizard } from "./SetupWizard";
import { commands } from "../lib/tauri";

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
  const [firstRun, setFirstRun] = useState(false);

  useEffect(() => {
    // Check if any model is downloaded — if not, it's first run
    // Prüfen ob ein Modell vorhanden ist — sonst Ersteinrichtung
    commands.getAvailableModels().then((models: any) => {
      const hasAny = models.some((m: any) => m.downloaded);
      setFirstRun(!hasAny);
    });
  }, []);

  if (firstRun) {
    return <SetupWizard onComplete={() => setFirstRun(false)} />;
  }

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

      {/* Language / Sprache */}
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

      {/* Model / Modell */}
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
