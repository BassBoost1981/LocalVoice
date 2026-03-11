import { useState, useEffect } from "react";
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
        // Auto-return to idle after 1.5s fade-out
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
