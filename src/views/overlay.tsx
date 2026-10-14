import { useEffect, useEffectEvent, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";

import { overlayFinish, overlayInfo, overlayReady, shotPng } from "../api";
import type { OverlayInfo, Rect, Selection, WindowInfo } from "../api";
import { rectFromPoints } from "../editor/geometry";
import type { Point } from "../editor/model";
import { fire } from "../ui/fire";

const LOUPE_SIZE = 120;
const LOUPE_PIXELS = 15;
const LOUPE_OFFSET = 24;
const LOUPE_LABEL = 24;
const MIN_SELECTION = 2;

interface Size {
  width: number;
  height: number;
}

const toPoint = (event: ReactPointerEvent): Point => ({
  x: event.clientX,
  y: event.clientY,
});

const windowAt = (
  windows: WindowInfo[],
  point: Point
): WindowInfo | undefined =>
  windows.find(
    ({ frame }) =>
      point.x >= frame.x &&
      point.x < frame.x + frame.width &&
      point.y >= frame.y &&
      point.y < frame.y + frame.height
  );

/** Position der Lupe neben dem Mauszeiger, gespiegelt an den Bildschirmrändern. */
const loupePosition = (point: Point, size: Size): Point => {
  const flipX = point.x + LOUPE_OFFSET + LOUPE_SIZE > size.width;
  const flipY = point.y + LOUPE_OFFSET + LOUPE_SIZE + LOUPE_LABEL > size.height;
  return {
    x: flipX ? point.x - LOUPE_OFFSET - LOUPE_SIZE : point.x + LOUPE_OFFSET,
    y: flipY
      ? point.y - LOUPE_OFFSET - LOUPE_SIZE - LOUPE_LABEL
      : point.y + LOUPE_OFFSET,
  };
};

interface LoupeProps {
  image: HTMLImageElement;
  point: Point;
  scale: number;
  size: Size;
}

/** Vergrößerte Pixel unter dem Mauszeiger. */
const Loupe = ({ image, point, scale, size }: LoupeProps) => {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const ctx = ref.current?.getContext("2d");
    if (!ctx) {
      return;
    }
    const half = Math.floor(LOUPE_PIXELS / 2);
    const px = Math.floor(point.x * scale) - half;
    const py = Math.floor(point.y * scale) - half;
    const cell = LOUPE_SIZE / LOUPE_PIXELS;
    ctx.imageSmoothingEnabled = false;
    ctx.fillStyle = "#000000";
    ctx.fillRect(0, 0, LOUPE_SIZE, LOUPE_SIZE);
    ctx.drawImage(
      image,
      px,
      py,
      LOUPE_PIXELS,
      LOUPE_PIXELS,
      0,
      0,
      LOUPE_SIZE,
      LOUPE_SIZE
    );
    ctx.strokeStyle = "rgba(255, 255, 255, 0.9)";
    ctx.lineWidth = 1;
    ctx.strokeRect(half * cell + 0.5, half * cell + 0.5, cell - 1, cell - 1);
  }, [image, point, scale]);

  const position = loupePosition(point, size);
  return (
    <div className="loupe" style={{ left: position.x, top: position.y }}>
      <canvas height={LOUPE_SIZE} ref={ref} width={LOUPE_SIZE} />
      <span>
        {Math.round(point.x)}, {Math.round(point.y)}
      </span>
    </div>
  );
};

/** Abdunklung mit Aussparung für Auswahl oder Fenster. */
const Shade = ({ size, highlight }: { size: Size; highlight: Rect | null }) => {
  const hole = highlight
    ? `M${highlight.x} ${highlight.y}h${highlight.width}v${highlight.height}h${-highlight.width}Z`
    : "";
  return (
    <svg className="shade" height={size.height} width={size.width}>
      <path
        d={`M0 0H${size.width}V${size.height}H0Z${hole}`}
        fillRule="evenodd"
      />
      {highlight ? (
        <rect
          className="outline"
          height={highlight.height}
          width={highlight.width}
          x={highlight.x}
          y={highlight.y}
        />
      ) : null}
    </svg>
  );
};

const Label = ({ at, children }: { at: Point; children: string }) => (
  <div
    className="selection-size"
    style={{ left: at.x, top: Math.max(0, at.y) }}
  >
    {children}
  </div>
);

const windowLabel = (window: WindowInfo) =>
  window.title === "" ? window.app : `${window.app} – ${window.title}`;

interface SelectorProps {
  info: OverlayInfo;
  image: HTMLImageElement;
  onFinish: (selection: Selection) => void;
}

const Selector = ({ info, image, onFinish }: SelectorProps) => {
  const [pointer, setPointer] = useState<Point | null>(null);
  const [start, setStart] = useState<Point | null>(null);
  const { frame, scale } = info.display;
  const selection = start && pointer ? rectFromPoints(start, pointer) : null;
  const hovered =
    info.mode === "window" && pointer
      ? windowAt(info.windows, pointer)
      : undefined;
  const showsGuides =
    pointer !== null && selection === null && info.mode !== "window";

  return (
    <div
      className={`overlay mode-${info.mode}`}
      onPointerDown={(event) => {
        if (event.button !== 0) {
          onFinish({ kind: "cancel" });
          return;
        }
        event.currentTarget.setPointerCapture(event.pointerId);
        setStart(toPoint(event));
      }}
      onPointerMove={(event) => {
        setPointer(toPoint(event));
      }}
      onPointerUp={(event) => {
        const rect = start ? rectFromPoints(start, toPoint(event)) : null;
        setStart(null);
        if (
          rect &&
          rect.width >= MIN_SELECTION &&
          rect.height >= MIN_SELECTION
        ) {
          onFinish({ kind: "area", rect });
        } else if (hovered) {
          onFinish({ id: hovered.id, kind: "window" });
        }
      }}
      style={{
        backgroundImage: `url(${image.src})`,
        height: frame.height,
        width: frame.width,
      }}
    >
      <Shade highlight={selection ?? hovered?.frame ?? null} size={frame} />
      {showsGuides ? (
        <>
          <div className="guide horizontal" style={{ top: pointer.y }} />
          <div className="guide vertical" style={{ left: pointer.x }} />
        </>
      ) : null}
      {selection ? (
        <Label at={{ x: selection.x, y: selection.y - 26 }}>
          {`${Math.round(selection.width)} × ${Math.round(selection.height)}`}
        </Label>
      ) : null}
      {hovered && !selection ? (
        <Label at={{ x: hovered.frame.x + 8, y: hovered.frame.y + 8 }}>
          {windowLabel(hovered)}
        </Label>
      ) : null}
      {pointer && info.mode !== "window" ? (
        <Loupe image={image} point={pointer} scale={scale} size={frame} />
      ) : null}
    </div>
  );
};

/** Auswahl über dem eingefrorenen Bildschirm. */
export const Overlay = () => {
  const [info, setInfo] = useState<OverlayInfo | null>(null);
  const [image, setImage] = useState<HTMLImageElement | null>(null);
  const finished = useRef(false);

  const finish = (selection: Selection) => {
    if (!finished.current) {
      finished.current = true;
      fire(async () => {
        await overlayFinish(selection);
      })();
    }
  };

  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (event.key === "Escape") {
      finish({ kind: "cancel" });
    }
  });

  useEffect(() => {
    const load = async () => {
      const overlay = await overlayInfo();
      if (!overlay) {
        return;
      }
      setInfo(overlay);
      // Die URL bleibt gültig, bis das Overlay-Fenster geschlossen wird.
      const frozen = new Image();
      frozen.src = URL.createObjectURL(await shotPng(overlay.display.shotId));
      await frozen.decode();
      setImage(frozen);
      await overlayReady();
    };
    fire(load)();
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  return info && image ? (
    <Selector image={image} info={info} onFinish={finish} />
  ) : null;
};
