/*
 * Bilder vergrößern: Ein Klick (oder Enter) auf einen Screenshot lässt ihn aus
 * seiner Position heraus auf Bildschirmgröße wachsen. Esc, Klick daneben oder
 * die Schließen-Taste bringen ihn zurück; mit den Pfeiltasten wechselt man
 * zwischen den Bildern. Das Skript lädt nichts nach und speichert nichts.
 */
(() => {
  const thumbs = [
    ...document.querySelectorAll(
      ".ds-gallery .ds-shot, .guide .ds-figure .ds-shot, .ds-proposal .ds-shot"
    ),
  ];
  if (thumbs.length === 0) {
    return;
  }
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const MS = reduce ? 0 : 420;
  const EASE = "cubic-bezier(0.2, 0.8, 0.2, 1)";

  let box = null;
  let big = null;
  let counter = null;
  let caption = null;
  let index = -1;
  let busy = false;

  const target = (thumb) => {
    const naturalWidth =
      Number(thumb.getAttribute("width")) || thumb.naturalWidth || 1600;
    const naturalHeight =
      Number(thumb.getAttribute("height")) || thumb.naturalHeight || 1000;
    const scale = Math.min(
      (window.innerWidth * 0.94) / naturalWidth,
      (window.innerHeight * 0.82) / naturalHeight
    );
    const width = naturalWidth * scale;
    const height = naturalHeight * scale;
    return {
      height,
      left: (window.innerWidth - width) / 2,
      top: (window.innerHeight * 0.92 - height) / 2,
      width,
    };
  };

  const place = (rect) => {
    big.style.left = `${rect.left}px`;
    big.style.top = `${rect.top}px`;
    big.style.width = `${rect.width}px`;
    big.style.height = `${rect.height}px`;
  };

  const fromThumb = (thumb, rect) => {
    const from = thumb.getBoundingClientRect();
    return `translate(${from.left - rect.left}px, ${from.top - rect.top}px) scale(${from.width / rect.width}, ${from.height / rect.height})`;
  };

  const label = (thumb) =>
    thumb.closest("figure")?.querySelector("figcaption")?.textContent?.trim() ||
    thumb.alt;

  const show = (next) => {
    index = next;
    const thumb = thumbs[index];
    counter.textContent = `${index + 1} / ${thumbs.length}`;
    caption.textContent = label(thumb);
    big.alt = thumb.alt;
    big.src = thumb.currentSrc || thumb.src;
  };

  const onKey = (event) => {
    if (event.key === "Escape") {
      close();
    } else if (event.key === "ArrowRight") {
      step(1);
    } else if (event.key === "ArrowLeft") {
      step(-1);
    } else if (event.key === "Tab") {
      const items = [...box.querySelectorAll("button")];
      const at = items.indexOf(document.activeElement);
      event.preventDefault();
      items[
        (at + (event.shiftKey ? items.length - 1 : 1)) % items.length
      ].focus();
    }
  };

  const build = () => {
    box = document.createElement("div");
    box.className = "lb";
    box.setAttribute("role", "dialog");
    box.setAttribute("aria-modal", "true");
    box.setAttribute("aria-label", "Bildansicht");
    const backdrop = document.createElement("div");
    backdrop.className = "lb-backdrop";
    backdrop.addEventListener("click", () => close());
    big = document.createElement("img");
    big.className = "lb-img";
    big.addEventListener("click", () => close());
    const bar = document.createElement("div");
    bar.className = "lb-bar";
    const make = (cls, text, name, handler) => {
      const button = document.createElement("button");
      button.type = "button";
      button.className = cls;
      button.textContent = text;
      button.setAttribute("aria-label", name);
      button.addEventListener("click", handler);
      return button;
    };
    counter = document.createElement("span");
    counter.className = "lb-count";
    caption = document.createElement("span");
    caption.className = "lb-cap";
    bar.append(
      make("lb-nav", "←", "Vorheriges Bild", () => step(-1)),
      counter,
      caption,
      make("lb-nav", "→", "Nächstes Bild", () => step(1))
    );
    const closer = make("lb-close", "×", "Schließen", () => close());
    box.append(backdrop, big, bar, closer);
    box.addEventListener("wheel", (event) => event.preventDefault(), {
      passive: false,
    });
  };

  const open = (start) => {
    if (box || busy) {
      return;
    }
    build();
    document.body.append(box);
    document.addEventListener("keydown", onKey);
    show(start);
    const thumb = thumbs[start];
    const rect = target(thumb);
    place(rect);
    big.style.transformOrigin = "0 0";
    big.style.transition = "none";
    big.style.transform = fromThumb(thumb, rect);
    thumb.style.visibility = "hidden";
    busy = true;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        box.classList.add("is-open");
        big.style.transition = `transform ${MS}ms ${EASE}`;
        big.style.transform = "none";
        setTimeout(() => {
          busy = false;
          box.querySelector(".lb-close").focus();
        }, MS);
      });
    });
  };

  const close = () => {
    if (!box || busy) {
      return;
    }
    busy = true;
    const thumb = thumbs[index];
    const rect = target(thumb);
    const done = () => {
      thumb.style.visibility = "";
      document.removeEventListener("keydown", onKey);
      box.remove();
      box = null;
      busy = false;
      thumb.focus({ preventScroll: true });
    };
    box.classList.remove("is-open");
    big.style.transition = `transform ${MS}ms ${EASE}`;
    big.style.transform = fromThumb(thumb, rect);
    setTimeout(done, MS + 20);
  };

  const step = (direction) => {
    if (!box || busy) {
      return;
    }
    busy = true;
    const previous = thumbs[index];
    const next = (index + direction + thumbs.length) % thumbs.length;
    big.style.transition = "opacity 140ms ease";
    big.style.opacity = "0";
    setTimeout(() => {
      previous.style.visibility = "";
      show(next);
      const rect = target(thumbs[next]);
      big.style.transition = "none";
      big.style.transform = "none";
      place(rect);
      thumbs[next].style.visibility = "hidden";
      requestAnimationFrame(() => {
        big.style.transition = "opacity 220ms ease";
        big.style.opacity = "1";
        setTimeout(() => {
          busy = false;
        }, 220);
      });
    }, 150);
  };

  for (const thumb of thumbs) {
    thumb.classList.add("is-zoomable");
    thumb.tabIndex = 0;
    thumb.setAttribute("role", "button");
    thumb.setAttribute("aria-label", `${thumb.alt} – Bild vergrößern`);
    thumb.addEventListener("click", () => open(thumbs.indexOf(thumb)));
    thumb.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        open(thumbs.indexOf(thumb));
      }
    });
  }
})();
