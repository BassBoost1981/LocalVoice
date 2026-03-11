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
          <div className="text-4xl text-green-400">✓</div>
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
