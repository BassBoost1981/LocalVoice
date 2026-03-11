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
