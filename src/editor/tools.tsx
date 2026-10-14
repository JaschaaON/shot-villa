import type { ReactNode } from "react";

import {
  ArrowIcon,
  ConcealIcon,
  CounterIcon,
  FreehandIcon,
  HighlightIcon,
  LineIcon,
  OvalIcon,
  RectIcon,
  SelectIcon,
  SpotlightIcon,
  TextIcon,
} from "../ui/icons";
import type { Tool } from "./model";

export interface ToolInfo {
  tool: Tool;
  label: string;
  /** Taste ohne Modifier. */
  key: string;
  icon: ReactNode;
}

export const TOOLS: readonly ToolInfo[] = [
  { icon: <SelectIcon />, key: "v", label: "Auswahl", tool: "select" },
  { icon: <ArrowIcon />, key: "a", label: "Pfeil", tool: "arrow" },
  { icon: <TextIcon />, key: "t", label: "Text", tool: "text" },
  { icon: <RectIcon />, key: "r", label: "Rechteck", tool: "rect" },
  { icon: <OvalIcon />, key: "o", label: "Oval", tool: "oval" },
  { icon: <LineIcon />, key: "l", label: "Linie", tool: "line" },
  { icon: <FreehandIcon />, key: "d", label: "Freihand", tool: "freehand" },
  { icon: <HighlightIcon />, key: "h", label: "Textmarker", tool: "highlight" },
  {
    icon: <ConcealIcon />,
    key: "b",
    label: "Unkenntlich machen",
    tool: "conceal",
  },
  { icon: <SpotlightIcon />, key: "s", label: "Spotlight", tool: "spotlight" },
  { icon: <CounterIcon />, key: "c", label: "Zähler", tool: "counter" },
];

export const toolForKey = (key: string): Tool | undefined =>
  TOOLS.find((info) => info.key === key.toLowerCase())?.tool;
