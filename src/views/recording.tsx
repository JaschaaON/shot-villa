import { useEffect, useState } from "react";

import { runAction } from "../api";
import { fire } from "../ui/fire";
import { StopIcon } from "../ui/icons";

/** „1:05“ aus Sekunden. */
export const formatElapsed = (seconds: number) =>
  `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;

/** Rahmen um den aufgenommenen Bereich; Klicks gehen durch ihn hindurch. */
export const RecordingFrame = () => <div className="recording-frame" />;

/** Laufzeit und Stopp-Knopf unter dem aufgenommenen Bereich. */
export const RecordingBar = () => {
  const [seconds, setSeconds] = useState(0);

  useEffect(() => {
    const started = Date.now();
    const timer = setInterval(() => {
      setSeconds(Math.floor((Date.now() - started) / 1000));
    }, 250);
    return () => {
      clearInterval(timer);
    };
  }, []);

  return (
    <div className="recording-bar">
      <span aria-hidden="true" className="recording-dot" />
      <span className="recording-time">{formatElapsed(seconds)}</span>
      <button
        className="recording-stop"
        onClick={fire(async () => {
          await runAction("record-area");
        })}
        type="button"
      >
        <StopIcon />
        Stopp
      </button>
    </div>
  );
};
