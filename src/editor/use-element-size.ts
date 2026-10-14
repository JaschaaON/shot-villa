import { useEffect, useState } from "react";
import type { RefObject } from "react";

import type { Size } from "./zoom";

export const useElementSize = (ref: RefObject<HTMLElement | null>): Size => {
  const [size, setSize] = useState<Size>({ height: 0, width: 0 });

  useEffect(() => {
    const observer = new ResizeObserver(([entry]) => {
      if (entry) {
        const { height, width } = entry.contentRect;
        setSize({ height, width });
      }
    });
    if (ref.current) {
      observer.observe(ref.current);
    }
    return () => {
      observer.disconnect();
    };
  }, [ref]);

  return size;
};
