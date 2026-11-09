/*
 * Sternenhimmel für den Hintergrund: Sterne funkeln, verschieben sich leicht
 * mit der Maus und beim Scrollen (Tiefe), leuchten in der Nähe des Zeigers und
 * verbinden sich dort zu kleinen Sternbildern. Gelegentlich fliegt eine
 * Sternschnuppe. Das Skript sammelt nichts und lädt nichts nach. Bei
 * „Bewegung reduzieren“ steht der Himmel still.
 */
(() => {
  const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const canvas = document.createElement("canvas");
  canvas.className = "ds-sky";
  canvas.setAttribute("aria-hidden", "true");
  document.body.prepend(canvas);
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    return;
  }

  const RADIUS = 170;
  const LINK = 105;
  const rand = (min, max) => min + Math.random() * (max - min);
  const mouse = {
    active: false,
    glow: 0,
    tx: -9999,
    ty: -9999,
    x: -9999,
    y: -9999,
  };
  const drift = { tx: 0, ty: 0, x: 0, y: 0 };
  let width = 0;
  let height = 0;
  let stars = [];
  let meteor = null;
  let nextMeteor = performance.now() + rand(3000, 6000);

  const resize = () => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    width = window.innerWidth;
    height = window.innerHeight;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const count = Math.max(
      70,
      Math.min(340, Math.round((width * height) / 6500))
    );
    stars = Array.from({ length: count }, () => ({
      d: rand(0.15, 1),
      phase: rand(0, Math.PI * 2),
      speed: rand(0.5, 1.5),
      warm: Math.random() < 0.12,
      x: rand(-40, width + 40),
      y: Math.random() * height,
    }));
    if (reduce) {
      draw(performance.now());
    }
  };

  const wrapY = (y) => ((y % height) + height) % height;

  const draw = (now) => {
    ctx.clearRect(0, 0, width, height);
    const scroll = window.scrollY || 0;
    const t = now / 1000;
    const near = [];

    for (const star of stars) {
      let x = star.x + drift.x * star.d * 34;
      let y = wrapY(star.y + drift.y * star.d * 26 - scroll * star.d * 0.14);
      let boost = 0;
      if (mouse.glow > 0.01) {
        const dx = x - mouse.x;
        const dy = y - mouse.y;
        const dist = Math.hypot(dx, dy);
        if (dist < RADIUS && dist > 0.01) {
          boost = (1 - dist / RADIUS) * mouse.glow;
          x += (dx / dist) * boost * 14;
          y += (dy / dist) * boost * 14;
        }
      }
      const twinkle = reduce
        ? 0.8
        : 0.62 + 0.38 * Math.sin(t * star.speed + star.phase);
      const alpha = Math.min(1, (0.28 + 0.52 * star.d) * twinkle + boost * 0.7);
      const size = 0.5 + star.d * 1.15 + boost * 1.4;
      ctx.beginPath();
      ctx.arc(x, y, size, 0, Math.PI * 2);
      ctx.fillStyle = star.warm
        ? `rgba(255, 226, 190, ${alpha})`
        : `rgba(226, 232, 244, ${alpha})`;
      ctx.fill();
      if (boost > 0.25) {
        near.push({ boost, x, y });
        ctx.beginPath();
        ctx.arc(x, y, size * 3.2, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(210, 220, 255, ${boost * 0.07})`;
        ctx.fill();
      }
    }

    // Sternbild: nahe Sterne untereinander und mit dem Zeiger verbinden
    if (near.length > 0) {
      ctx.lineWidth = 0.7;
      for (let i = 0; i < near.length; i += 1) {
        const a = near[i];
        ctx.strokeStyle = `rgba(214, 222, 245, ${a.boost * 0.22})`;
        ctx.beginPath();
        ctx.moveTo(a.x, a.y);
        ctx.lineTo(mouse.x, mouse.y);
        ctx.stroke();
        for (let j = i + 1; j < near.length; j += 1) {
          const b = near[j];
          const dist = Math.hypot(a.x - b.x, a.y - b.y);
          if (dist < LINK) {
            ctx.strokeStyle = `rgba(214, 222, 245, ${Math.min(a.boost, b.boost) * (1 - dist / LINK) * 0.35})`;
            ctx.beginPath();
            ctx.moveTo(a.x, a.y);
            ctx.lineTo(b.x, b.y);
            ctx.stroke();
          }
        }
      }
    }

    // weiches Leuchten unter dem Zeiger
    if (mouse.glow > 0.01) {
      const glow = ctx.createRadialGradient(
        mouse.x,
        mouse.y,
        0,
        mouse.x,
        mouse.y,
        RADIUS * 1.4
      );
      glow.addColorStop(0, `rgba(190, 205, 255, ${0.07 * mouse.glow})`);
      glow.addColorStop(1, "rgba(190, 205, 255, 0)");
      ctx.fillStyle = glow;
      ctx.fillRect(
        mouse.x - RADIUS * 1.4,
        mouse.y - RADIUS * 1.4,
        RADIUS * 2.8,
        RADIUS * 2.8
      );
    }

    // Sternschnuppe
    if (meteor) {
      const life = (now - meteor.start) / meteor.duration;
      if (life >= 1) {
        meteor = null;
      } else {
        const head = {
          x: meteor.x + meteor.vx * life,
          y: meteor.y + meteor.vy * life,
        };
        const tail = {
          x: head.x - meteor.vx * 0.18,
          y: head.y - meteor.vy * 0.18,
        };
        const fade = Math.sin(life * Math.PI);
        const line = ctx.createLinearGradient(tail.x, tail.y, head.x, head.y);
        line.addColorStop(0, "rgba(255, 255, 255, 0)");
        line.addColorStop(1, `rgba(255, 255, 255, ${0.75 * fade})`);
        ctx.strokeStyle = line;
        ctx.lineWidth = 1.4;
        ctx.beginPath();
        ctx.moveTo(tail.x, tail.y);
        ctx.lineTo(head.x, head.y);
        ctx.stroke();
      }
    }
  };

  const frame = (now) => {
    if (!document.hidden) {
      mouse.x += (mouse.tx - mouse.x) * 0.18;
      mouse.y += (mouse.ty - mouse.y) * 0.18;
      mouse.glow += ((mouse.active ? 1 : 0) - mouse.glow) * 0.08;
      drift.x += (drift.tx - drift.x) * 0.05;
      drift.y += (drift.ty - drift.y) * 0.05;
      if (!meteor && now > nextMeteor) {
        const angle = rand(0.35, 0.6);
        const length = rand(320, 520);
        meteor = {
          duration: rand(900, 1300),
          start: now,
          vx: Math.cos(angle) * length,
          vy: Math.sin(angle) * length,
          x: rand(width * 0.1, width * 0.8),
          y: rand(0, height * 0.35),
        };
        nextMeteor = now + rand(7000, 14_000);
      }
      draw(now);
    }
    requestAnimationFrame(frame);
  };

  window.addEventListener("resize", resize);
  resize();
  if (reduce) {
    return;
  }
  window.addEventListener(
    "pointermove",
    (event) => {
      if (!mouse.active) {
        mouse.x = event.clientX;
        mouse.y = event.clientY;
      }
      mouse.active = true;
      mouse.tx = event.clientX;
      mouse.ty = event.clientY;
      drift.tx = (event.clientX / width - 0.5) * -2;
      drift.ty = (event.clientY / height - 0.5) * -2;
    },
    { passive: true }
  );
  document.addEventListener("pointerleave", () => {
    mouse.active = false;
    drift.tx = 0;
    drift.ty = 0;
  });
  requestAnimationFrame(frame);
})();
