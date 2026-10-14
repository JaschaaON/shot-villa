import type { ReactNode } from "react";

/** Eigene, schlichte Linien-Icons (20 × 20). */
const Icon = ({ children }: { children: ReactNode }) => (
  <svg
    aria-hidden="true"
    fill="none"
    height="20"
    stroke="currentColor"
    strokeLinecap="round"
    strokeLinejoin="round"
    strokeWidth="1.6"
    viewBox="0 0 20 20"
    width="20"
  >
    {children}
  </svg>
);

export const SelectIcon = () => (
  <Icon>
    <path d="M4 3l11 6-5 1.5L8 16z" />
  </Icon>
);

export const ArrowIcon = () => (
  <Icon>
    <path d="M4 16L15 5M8 5h7v7" />
  </Icon>
);

export const TextIcon = () => (
  <Icon>
    <path d="M4 5V4h12v1M10 4v12M7.5 16h5" />
  </Icon>
);

export const RectIcon = () => (
  <Icon>
    <rect height="11" rx="1.5" width="14" x="3" y="4.5" />
  </Icon>
);

export const OvalIcon = () => (
  <Icon>
    <ellipse cx="10" cy="10" rx="7" ry="5.5" />
  </Icon>
);

export const LineIcon = () => (
  <Icon>
    <path d="M4 16L16 4" />
  </Icon>
);

export const FreehandIcon = () => (
  <Icon>
    <path d="M3 14c2-5 4-7 6-4s4 3 8-5" />
  </Icon>
);

export const HighlightIcon = () => (
  <Icon>
    <path d="M12 3l5 5-7 7H5v-5z" />
    <path d="M3 17h7" />
  </Icon>
);

export const ConcealIcon = () => (
  <Icon>
    <rect height="12" width="12" x="4" y="4" />
    <path d="M4 8h12M4 12h12M8 4v12M12 4v12" />
  </Icon>
);

export const SpotlightIcon = () => (
  <Icon>
    <circle cx="10" cy="10" r="3.5" />
    <path d="M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" />
  </Icon>
);

export const CounterIcon = () => (
  <Icon>
    <circle cx="10" cy="10" r="7" />
    <path d="M9 7.5l1.5-1v7" />
  </Icon>
);

export const CopyIcon = () => (
  <Icon>
    <rect height="10" rx="1.5" width="9" x="7" y="7" />
    <path d="M13 7V4.5A1.5 1.5 0 0011.5 3h-6A1.5 1.5 0 004 4.5v7A1.5 1.5 0 005.5 13H7" />
  </Icon>
);

export const SaveIcon = () => (
  <Icon>
    <path d="M10 3v9M6.5 8.5L10 12l3.5-3.5M4 14v2h12v-2" />
  </Icon>
);

export const PinIcon = () => (
  <Icon>
    <path d="M12 3l5 5-3 1-3 3 .5 3.5L4.5 8.5 8 9l3-3z" />
    <path d="M7 13l-4 4" />
  </Icon>
);

export const OcrIcon = () => (
  <Icon>
    <path d="M3 6V3h3M14 3h3v3M17 14v3h-3M6 17H3v-3" />
    <path d="M7 8h6M7 10.5h6M7 13h4" />
  </Icon>
);

export const CropIcon = () => (
  <Icon>
    <path d="M5 2v13h13M2 5h13v13" />
  </Icon>
);

export const UndoIcon = () => (
  <Icon>
    <path d="M7 5L3.5 8.5 7 12" />
    <path d="M4 8.5h8a4 4 0 010 8H9" />
  </Icon>
);

export const RedoIcon = () => (
  <Icon>
    <path d="M13 5l3.5 3.5L13 12" />
    <path d="M16 8.5H8a4 4 0 000 8h3" />
  </Icon>
);

export const CloseIcon = () => (
  <Icon>
    <path d="M5 5l10 10M15 5L5 15" />
  </Icon>
);

export const EditIcon = () => (
  <Icon>
    <path d="M13 3.5l3.5 3.5L7 16.5H3.5V13z" />
  </Icon>
);
