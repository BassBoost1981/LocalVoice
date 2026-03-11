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
