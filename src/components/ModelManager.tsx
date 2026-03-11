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
      // Refresh model list / Modellliste aktualisieren
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
