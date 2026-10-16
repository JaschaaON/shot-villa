import { useEffect, useEffectEvent, useState } from "react";

import { onRecordingStarted, runAction } from "../api";
import { fire } from "../ui/fire";
import { CloseIcon, RecordIcon, StopIcon } from "../ui/icons";

/** „1:05“ aus Sekunden. */
export const formatElapsed = (seconds: number) =>
  `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;

/** Zeitpunkt, ab dem die Aufnahme läuft; `null`, solange sie nur bereitsteht. */
const useRecordingStart = () => {
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const started = useEffectEvent(() => {
    setStartedAt(Date.now());
  });

  useEffect(() => {
    const listening = onRecordingStarted(started);
    return () => {
      fire(async () => {
        (await listening)();
      })();
    };
  }, []);

  return startedAt;
};

const toggle = fire(async () => {
  await runAction("record-area");
});

const cancel = fire(async () => {
  await runAction("record-cancel");
});

/** Rahmen um den gewählten Bereich; Klicks gehen durch ihn hindurch. */
export const RecordingFrame = () => {
  const running = useRecordingStart() !== null;
  return (
    <div className={running ? "recording-frame running" : "recording-frame"} />
  );
};

const Elapsed = ({ startedAt }: { startedAt: number }) => {
  const [seconds, setSeconds] = useState(0);

  useEffect(() => {
    const timer = setInterval(() => {
      setSeconds(Math.floor((Date.now() - startedAt) / 1000));
    }, 250);
    return () => {
      clearInterval(timer);
    };
  }, [startedAt]);

  return <span className="recording-time">{formatElapsed(seconds)}</span>;
};

/** Leiste mittig unter dem Bereich: erst Start/Abbrechen, dann Laufzeit und Stopp. */
export const RecordingBar = () => {
  const startedAt = useRecordingStart();

  useEffect(() => {
    // Esc verwirft den Bereich, solange noch nicht aufgenommen wird.
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        cancel();
      }
    };
    globalThis.addEventListener("keydown", onKey);
    return () => {
      globalThis.removeEventListener("keydown", onKey);
    };
  }, []);

  if (startedAt === null) {
    return (
      <div className="recording-bar">
        <button className="recording-start" onClick={toggle} type="button">
          <RecordIcon />
          Start
        </button>
        <span className="recording-hint">Bereit</span>
        <button
          aria-label="Abbrechen"
          className="recording-cancel"
          onClick={cancel}
          title="Abbrechen (Esc)"
          type="button"
        >
          <CloseIcon />
        </button>
      </div>
    );
  }

  return (
    <div className="recording-bar">
      <span aria-hidden="true" className="recording-dot" />
      <Elapsed startedAt={startedAt} />
      <button className="recording-stop" onClick={toggle} type="button">
        <StopIcon />
        Stopp
      </button>
    </div>
  );
};
