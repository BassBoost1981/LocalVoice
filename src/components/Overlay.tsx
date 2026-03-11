import { useState, useEffect } from "react";
import { useRecording, type RecordingState } from "../hooks/useRecording";

function Overlay() {
  const { state, fftData } = useRecording();

  return (
    <div className="flex items-center justify-center w-screen h-screen">
      <PillWidget state={state} fftData={fftData} />
    </div>
  );
}

function PillWidget({
  state,
  fftData,
}: {
  state: RecordingState;
  fftData: number[];
}) {
  const isActive = state === "recording" || state === "transcribing";
  const isDone = state === "done";
  const [fading, setFading] = useState(false);

  // Fade out after 1s in done state / Nach 1s im Done-Zustand ausblenden
  useEffect(() => {
    if (state === "done") {
      const timer = setTimeout(() => setFading(true), 1000);
      return () => clearTimeout(timer);
    }
    setFading(false);
  }, [state]);

  return (
    <div
      className={`
        flex items-center justify-center gap-[3px] rounded-full
        border backdrop-blur-xl transition-all duration-300 ease-out
        ${isDone
          ? "w-[140px] h-[40px] border-green-500/20 bg-green-500/4"
          : isActive
            ? "w-[180px] h-[44px] border-violet-500/20 bg-violet-500/4"
            : "w-[140px] h-[40px] border-white/8 bg-white/2"
        }
        ${fading ? "opacity-0 transition-opacity duration-500" : ""}
      `}
      style={{
        animation: isDone ? "pop-in 0.3s cubic-bezier(0.34, 1.56, 0.64, 1)" : undefined,
      }}
    >
      {state === "idle" && null}

      {state === "recording" && <FrequencyDots fftData={fftData} />}

      {state === "transcribing" && <TranscribingView />}

      {state === "done" && <DoneCheck />}
    </div>
  );
}

function FrequencyDots({ fftData }: { fftData: number[] }) {
  // Map 16 FFT bins to 12 dots via neighbor interpolation
  // 16 FFT-Bins auf 12 Dots mappen via Nachbar-Interpolation
  const dots = Array.from({ length: 12 }, (_, i) => {
    const idx = (i / 12) * 16;
    const lo = Math.floor(idx);
    const hi = Math.min(lo + 1, 15);
    const frac = idx - lo;
    return fftData[lo] * (1 - frac) + fftData[hi] * frac;
  });

  return (
    <div className="flex items-center gap-[4px] px-6">
      {dots.map((value, i) => (
        <div
          key={i}
          className="w-[5px] rounded-full bg-violet-400/85 transition-all duration-75"
          style={{
            height: `${Math.max(5, value * 28)}px`,
            opacity: 0.4 + value * 0.6,
          }}
        />
      ))}
    </div>
  );
}

function TranscribingView() {
  return (
    <div className="flex items-center gap-2 px-4">
      <div className="flex items-center gap-[2.5px]">
        {Array.from({ length: 12 }).map((_, i) => (
          <div
            key={i}
            className="w-[4px] h-[4px] rounded-full bg-violet-400/50"
          />
        ))}
      </div>
      <div className="w-[18px] h-[18px] border-2 border-violet-500/15 border-t-violet-400/70 rounded-full animate-spin" />
    </div>
  );
}

function DoneCheck() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <path
        d="M3.5 8l3 3 6-6"
        stroke="#22c55e"
        strokeWidth="1.8"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export default Overlay;
