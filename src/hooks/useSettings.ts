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
