#!/usr/bin/env node
/**
 * Erzeugt die Produktbilder (Editor, Einstellungen, …) ohne die App zu starten.
 *
 * Die gebaute Oberfläche (`pnpm build`, Ordner `dist/`) läuft in einem
 * Headless-Chromium mit einer Attrappe der Tauri-Schnittstelle
 * (`tauri-mock.js`). Als Bildinhalt dient `sample.html`; die Markierungen im
 * Editor werden per Maus und Tastatur gezeichnet, wie ein Nutzer es täte.
 *
 *   pnpm build
 *   node tools/screenshots/capture.mjs [Zielordner] [Name …]
 *
 * Umgebungsvariablen:
 *   CHROME        Pfad zu chrome-headless-shell oder Chromium.
 *   SYMBOLS_FONT  Schriftdateien (woff2 oder ttf, mit ":" getrennt) für ⌘ ⇧ ⌥ ⌃
 *                 (U+2318, U+21E7, U+2325, U+2303), falls dem Rechner solche
 *                 Zeichen fehlen (Linux), z. B. Noto Sans Symbols 2 (⌘ ⇧ ⌥);
 *                 ⌃ fehlt dort und muss aus einer anderen Quelle kommen.
 *   SAMPLE        anderes Beispielbild (HTML, 1600 × 1068, mit den Kennungen aus
 *                 sample.html).
 * Ohne Namen entstehen alle Bilder.
 */
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { mkdir, readFile, readdir, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import os from "node:os";
import path from "node:path";

const here = import.meta.dirname;
const dist = path.resolve(here, "../../dist");
const args = process.argv.slice(2);
const outDir = path.resolve(args[0] ?? path.join(here, "out"));
const wanted = args.slice(1);

const sleep =  async (ms) => await new Promise((resolve) => setTimeout(resolve, ms));
const MAC_UA =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15";
const MARGIN = 56;
const SCALE = 2;

const findChrome = async () => {
  if (process.env.CHROME) {
    return process.env.CHROME;
  }
  const base = path.join(os.homedir(), ".cache", "ms-playwright");
  if (existsSync(base)) {
    for (const entry of await readdir(base)) {
      const candidate = path.join(
        base,
        entry,
        "chrome-headless-shell-linux64",
        "chrome-headless-shell"
      );
      if (entry.startsWith("chromium_headless_shell") && existsSync(candidate)) {
        return candidate;
      }
    }
  }
  for (const name of ["chromium", "chromium-browser", "google-chrome"]) {
    const candidate = `/usr/bin/${name}`;
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  throw new Error("Kein Chromium gefunden; Umgebungsvariable CHROME setzen.");
};

/* ---------- Mini-Client für das Chrome DevTools Protocol ---------- */

const connect = async (port) => {
  let target;
  for (let attempt = 0; attempt < 100 && !target; attempt += 1) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      target = list.find((entry) => entry.type === "page");
    } catch {
      /* Chrome startet noch */
    }
    if (!target) {
      await sleep(100);
    }
  }
  if (!target) {
    throw new Error("Chrome antwortet nicht.");
  }
  const socket = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve) =>{  socket.addEventListener("open", resolve); });
  let counter = 0;
  const pending = new Map();
  const waiting = new Map();
  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    if (message.id !== undefined && pending.has(message.id)) {
      const { resolve, reject } = pending.get(message.id);
      pending.delete(message.id);
      if (message.error) {
        reject(new Error(`${message.error.message}`));
      } else {
        resolve(message.result);
      }
    } else if (message.method && waiting.has(message.method)) {
      for (const resolve of waiting.get(message.method)) {
        resolve(message.params);
      }
      waiting.delete(message.method);
    }
  });
  const send =  async (method, params = {}) =>
    await new Promise((resolve, reject) => {
      counter += 1;
      pending.set(counter, { reject, resolve });
      socket.send(JSON.stringify({ id: counter, method, params }));
    });
  const once =  async (method) =>
    await new Promise((resolve) => {
      waiting.set(method, [...(waiting.get(method) ?? []), resolve]);
    });
  const evaluate = async (expression) => {
    const result = await send("Runtime.evaluate", {
      awaitPromise: true,
      expression,
      returnByValue: true,
    });
    if (result.exceptionDetails) {
      throw new Error(result.exceptionDetails.exception?.description ?? "Skriptfehler");
    }
    return result.result.value;
  };
  return { close: () =>{  socket.close(); }, evaluate, once, send };
};

/* ---------- Statischer Server für dist/ und das Beispielbild ---------- */

const TYPES = {
  ".css": "text/css",
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript",
  ".png": "image/png",
  ".woff2": "font/woff2",
};

const symbolFonts = (process.env.SYMBOLS_FONT ?? "")
  .split(path.delimiter)
  .filter(Boolean);

const serve = (samplePng) =>
  new Promise((resolve) => {
    const server = createServer(async (request, response) => {
      const url = new URL(request.url ?? "/", "http://localhost");
      try {
        if (url.pathname === "/sample.png") {
          response.writeHead(200, { "content-type": "image/png" });
          response.end(samplePng.current);
          return;
        }
        if (url.pathname === "/inter.woff2") {
          response.writeHead(200, { "content-type": TYPES[".woff2"] });
          response.end(
            await readFile(
              path.resolve(here, "../../node_modules/@fontsource-variable/inter/files/inter-latin-wght-normal.woff2")
            )
          );
          return;
        }
        const symbol = /^\/symbols-(\d+)$/u.exec(url.pathname);
        if (symbol && symbolFonts[Number(symbol[1])]) {
          response.writeHead(200, { "content-type": "application/octet-stream" });
          response.end(await readFile(symbolFonts[Number(symbol[1])]));
          return;
        }
        if (url.pathname === "/sample.html") {
          response.writeHead(200, { "content-type": TYPES[".html"] });
          response.end(await readFile(process.env.SAMPLE ? path.resolve(process.env.SAMPLE) : path.join(here, "sample.html")));
          return;
        }
        const file = path.join(dist, url.pathname === "/" ? "index.html" : url.pathname);
        if (!file.startsWith(dist)) {
          throw new Error("außerhalb");
        }
        const body = await readFile(file);
        response.writeHead(200, {
          "content-type": TYPES[path.extname(file)] ?? "application/octet-stream",
        });
        response.end(body);
      } catch {
        if (!response.headersSent) {
          response.writeHead(404);
        }
        response.end();
      }
    });
    server.listen(0, "127.0.0.1", () =>{  resolve(server); });
  });

/* ---------- Fensterrahmen im Stil von macOS ---------- */

const frameScript = ({ width, height, titlebar, title }) => `(() => {
  const M = ${MARGIN}, W = ${width}, H = ${height}, T = ${titlebar};
  const style = document.createElement("style");
  style.textContent = \`
    html, body { background: #050607 !important; }
    #root { position: fixed; left: \${M}px; top: \${M + T}px; width: \${W}px; height: \${H - T}px;
      overflow: hidden; z-index: 1; border-radius: \${T ? "0 0 12px 12px" : "12px"}; background: var(--onyx); }
    #fake-window { position: fixed; left: \${M}px; top: \${M}px; width: \${W}px; height: \${H}px;
      border-radius: 12px; background: #08090a; z-index: 0;
      box-shadow: 0 0 0 1px #34343a, 0 30px 80px rgb(0 0 0 / 65%); }
    #fake-bar { position: fixed; left: \${M}px; top: \${M}px; width: \${W}px; height: \${T}px; z-index: 2;
      border-radius: 12px 12px 0 0; background: #1c1d20; border-bottom: 1px solid #000;
      display: \${T ? "grid" : "none"}; place-items: center; color: #d0d6e0; font-size: 13px; font-weight: 590; }
    #fake-lights { position: fixed; left: \${M + 20}px; top: \${M + (T ? T / 2 - 6 : 18)}px; z-index: 3; display: flex; gap: 8px; }
    #fake-lights i { width: 12px; height: 12px; border-radius: 50%; display: block; }
  \`;
  document.head.append(style);
  const add = (id, html = "") => { const el = document.createElement("div"); el.id = id; el.innerHTML = html; document.body.append(el); };
  add("fake-window");
  add("fake-bar", ${JSON.stringify(title)});
  add("fake-lights", '<i style="background:#ff5f57"></i><i style="background:#febc2e"></i><i style="background:#28c840"></i>');
})()`;

/* ---------- Zeichnen im Editor ---------- */

const sampleBoxes = async (cdp, base) => {
  const loaded = cdp.once("Page.loadEventFired");
  await cdp.send("Page.navigate", { url: `${base}/sample.html` });
  await loaded;
  await sleep(300);
  return await cdp.evaluate(`(() => {
    const box = (selector) => { const r = document.querySelector(selector).getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; };
    return {
      kpi1: box("#kpi1"), kpi2: box("#kpi2"), kpi3: box("#kpi3"),
      delta1: box("#delta1"), delta2: box("#delta2"), bar5: box("#bar5"),
      list: box("#milestones"),
      row4: box("#milestones li:nth-child(4)"),
      row1: box("#milestones li:nth-child(1)"),
    };
  })()`);
};

const mouse = (cdp, type, x, y, extra = {}) =>
  cdp.send("Input.dispatchMouseEvent", {
    button: "left",
    buttons: type === "mouseReleased" || type === "mouseMoved" ? 0 : 1,
    clickCount: 1,
    type,
    x,
    y,
    ...extra,
  });

const press = async (cdp, key) => {
  await cdp.send("Input.dispatchKeyEvent", { code: `Key${key.toUpperCase()}`, key, text: key, type: "keyDown" });
  await cdp.send("Input.dispatchKeyEvent", { code: `Key${key.toUpperCase()}`, key, type: "keyUp" });
  await sleep(60);
};

const drawEditor = async (cdp, boxes, options = {}) => {
  await cdp.evaluate(`new Promise((resolve) => {
    const check = () => document.querySelector("canvas.canvas") ? resolve(true) : setTimeout(check, 50);
    check();
  })`);
  await sleep(500);
  const rect = await cdp.evaluate(`(() => { const r = document.querySelector("canvas.canvas").getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width }; })()`);
  const factor = rect.width / 1600;
  const at = (x, y) => [rect.x + x * factor, rect.y + y * factor];
  const drag = async (from, to) => {
    const [x1, y1] = at(...from);
    const [x2, y2] = at(...to);
    await mouse(cdp, "mouseMoved", x1, y1);
    await mouse(cdp, "mousePressed", x1, y1);
    for (let step = 1; step <= 8; step += 1) {
      await mouse(cdp, "mouseMoved", x1 + ((x2 - x1) * step) / 8, y1 + ((y2 - y1) * step) / 8, { buttons: 1 });
    }
    await mouse(cdp, "mouseReleased", x2, y2);
    await sleep(80);
  };
  const click = async (point) => {
    const [x, y] = at(...point);
    await mouse(cdp, "mouseMoved", x, y);
    await mouse(cdp, "mousePressed", x, y);
    await mouse(cdp, "mouseReleased", x, y);
    await sleep(80);
  };

  const px = ([x, y]) => at(x, y).map((value) => Math.round(value * SCALE));
  const geometry = { counters: [], arrow: null, highlight: null, frame: null };
  const marks = options.marks !== false;

  // Zähler an den drei Kacheln
  if (marks) {
    await press(cdp, "c");
  }
  for (const kpi of [boxes.kpi1, boxes.kpi2, boxes.kpi3]) {
    geometry.counters.push(px([kpi.x + 26, kpi.y + 6]));
    if (marks) {
      await click([kpi.x + 26, kpi.y + 6]);
    }
  }
  // Textmarker über den ersten Wert
  const d1 = boxes.delta1;
  geometry.highlight = [px([d1.x - 6, d1.y + 2]), px([d1.x + d1.width + 6, d1.y + d1.height - 2])];
  if (marks) {
    await press(cdp, "h");
    await drag([d1.x - 6, d1.y + 2], [d1.x + d1.width + 6, d1.y + d1.height - 2]);
  }
  // Rahmen um die Meilensteine
  const frameFrom = [boxes.row1.x - 14, boxes.row1.y - 6];
  const frameTo = [boxes.row4.x + boxes.row4.width + 14, boxes.row4.y + boxes.row4.height + 6];
  geometry.frame = [px(frameFrom), px(frameTo)];
  if (marks) {
    await press(cdp, "r");
    await drag(frameFrom, frameTo);
  }
  // Pfeil auf die zweite Kachel
  const d2 = boxes.delta2;
  const arrowFrom = [boxes.kpi3.x + 120, boxes.kpi1.y - 70];
  const arrowTo = [d2.x + d2.width + 16, d2.y + d2.height / 2];
  geometry.arrow = [px(arrowFrom), px(arrowTo)];
  await press(cdp, "a");
  if (marks) {
    await drag(arrowFrom, arrowTo);
  }
  // Auswahl aufheben, Pfeilwerkzeug bleibt aktiv
  if (marks) {
    await press(cdp, "v");
    await click([boxes.list.x + 40, boxes.list.y + boxes.list.height - 20]);
    await press(cdp, "a");
  }
  // Mauszeiger über einen Balken: die Pipette zeigt dessen Farbe
  const bar = boxes.bar5;
  const [bx, by] = at(bar.x + bar.width / 2, bar.y + bar.height * 0.5);
  if (process.env.DEBUG) {
    await cdp.evaluate(`window.__pm = 0; document.querySelector("canvas.canvas").addEventListener("pointermove", () => { window.__pm += 1; });`);
  }
  // Die Pipette braucht manchmal mehrere Mausbewegungen; so lange wiederholen, bis eine Farbe steht
  for (let attempt = 0; attempt < 6; attempt += 1) {
    await mouse(cdp, "mouseMoved", bx - 3 + attempt, by);
    await sleep(120);
    await mouse(cdp, "mouseMoved", bx + attempt, by + 1);
    await sleep(250);
    if (await cdp.evaluate(`document.querySelector(".statusbar button")?.innerText.includes("#") ?? false`)) {
      break;
    }
  }
  if (process.env.DEBUG) {
    process.stdout.write(`pm=${await cdp.evaluate("window.__pm")} status=${await cdp.evaluate("document.querySelector('.statusbar').innerText")}\n`);
    process.stdout.write(`${bx},${by} ${await cdp.evaluate(`document.elementFromPoint(${bx}, ${by})?.tagName`)} ${JSON.stringify(bar)}\n`);
  }
  if (marks && !(await cdp.evaluate(`document.querySelector(".statusbar button")?.innerText.includes("#") ?? false`))) {
    process.stderr.write("Warnung: Die Pipette zeigt keine Farbe; Bild neu erzeugen.\n");
  }
  return geometry;
};

/* ---------- Bilder ---------- */

const SETTINGS_VIEW = { bar: "native", height: 580, label: "settings", title: "Einstellungen", width: 820 };

const SCENARIOS = {
  "editor": { bar: "overlay", height: 720, label: "main", run: drawEditor, width: 1100 },
  "editor-clean": { bar: "overlay", height: 720, label: "main", run: async (cdp, boxes) => {
      const result = await drawEditor(cdp, boxes, { marks: false });
      // Ohne Markierungen soll „Markierungen 0“ nicht stören; die Website zeichnet sie darüber
      await cdp.evaluate(`document.querySelector(".statusbar .count").style.visibility = "hidden"`);
      return result;
    }, width: 1100 },
  "editor-leer": { bar: "overlay", height: 720, label: "main", query: "&empty=1", width: 1100 },
  "permission": { bar: "native", height: 360, label: "permission", title: "Bildschirmaufnahme erlauben", width: 460 },
  "recording-bereit": { bar: "none", height: 48, label: "recording-bar", width: 230 },
  "recording-laeuft": {
    bar: "none",
    height: 48,
    label: "recording-bar",
    run: async (cdp) => {
      await cdp.evaluate(`window.__emit("recording-started", null)`);
      await sleep(2400);
    },
    width: 230,
  },
  "settings-advanced": { ...SETTINGS_VIEW, query: "&tab=advanced" },
  "settings-general": { ...SETTINGS_VIEW, query: "&tab=general" },
  "settings-shortcuts": { ...SETTINGS_VIEW, query: "&tab=hotkeys" },
  "settings-t3": { ...SETTINGS_VIEW, query: "&tab=t3" },
  "settings-updates": { ...SETTINGS_VIEW, query: "&tab=updates" },
  "thumbnail": { bar: "none", height: 213, label: "thumbnail", width: 260 },
  "welcome": { bar: "native", height: 500, label: "welcome", title: "Willkommen bei Pivolo", width: 520 },
};

const main = async () => {
  await mkdir(outDir, { recursive: true });
  const chrome = await findChrome();
  const port = 9300 + Math.floor(Math.random() * 500);
  const profile = path.join(os.tmpdir(), `pivolo-shots-${process.pid}`);
  const browser = spawn(
    chrome,
    [
      "--headless",
      "--no-sandbox",
      "--disable-gpu",
      "--hide-scrollbars",
      "--force-color-profile=srgb",
      `--remote-debugging-port=${port}`,
      `--user-data-dir=${profile}`,
      "about:blank",
    ],
    { stdio: "ignore" }
  );
  const samplePng = { current: Buffer.alloc(0) };
  const server = await serve(samplePng);
  const base = `http://127.0.0.1:${server.address().port}`;
  const mock = await readFile(path.join(here, "tauri-mock.js"), "utf-8");

  try {
    const cdp = await connect(port);
    await cdp.send("Page.enable");
    await cdp.send("Runtime.enable");
    await cdp.send("Emulation.setUserAgentOverride", { userAgent: MAC_UA });

    // Beispielbild rendern und die Lage der Elemente merken
    await cdp.send("Emulation.setDeviceMetricsOverride", { deviceScaleFactor: SCALE, height: 1068, mobile: false, width: 1600 });
    const boxes = await sampleBoxes(cdp, base);
    const shot = await cdp.send("Page.captureScreenshot", { format: "png" });
    samplePng.current = Buffer.from(shot.data, "base64");

    const faces = symbolFonts
      .map((file, index) => `@font-face { font-family: "Pivolo Symbols ${index}"; src: url(/symbols-${index}) format("${file.endsWith(".ttf") ? "truetype" : "woff2"}"); unicode-range: U+2303, U+2318, U+2325, U+21E7; }`)
      .join(" ");
    const family = symbolFonts.map((_, index) => `"Pivolo Symbols ${index}"`).join(", ");
    const symbols = `document.addEventListener("DOMContentLoaded", () => {
      const style = document.createElement("style");
      style.textContent = ${JSON.stringify(faces)} + ${JSON.stringify(`:root { --font: "Inter Variable", ${family}, system-ui, sans-serif; }`)};
      document.head.append(style);
    });`;
    await cdp.send("Page.addScriptToEvaluateOnNewDocument", { source: mock + symbols });
    for (const [name, scenario] of Object.entries(SCENARIOS)) {
      if (wanted.length > 0 && !wanted.includes(name)) {
        continue;
      }
      const framed = scenario.bar !== "none";
      const titlebar = scenario.bar === "native" ? 28 : 0;
      const viewport = framed
        ? { height: scenario.height + MARGIN * 2, width: scenario.width + MARGIN * 2 }
        : { height: scenario.height, width: scenario.width };
      await cdp.send("Emulation.setDeviceMetricsOverride", { deviceScaleFactor: SCALE, mobile: false, ...viewport });
      await cdp.send("Emulation.setDefaultBackgroundColorOverride", { color: { a: 1, b: 7, g: 6, r: 5 } });
      const loaded = cdp.once("Page.loadEventFired");
      await cdp.send("Page.navigate", { url: `${base}/?label=${scenario.label}${scenario.query ?? ""}` });
      await loaded;
      await sleep(900);
      if (framed) {
        await cdp.evaluate(frameScript({ height: scenario.height, title: scenario.title ?? "", titlebar, width: scenario.width }));
      }
      if (scenario.run) {
        const result = await scenario.run(cdp, boxes);
        if (result && name === "editor") {
          // Lage der Markierungen in Bildpunkten, für die Bewegtbild-Animation der Website
          await writeFile(path.join(outDir, "editor-marks.json"), `${JSON.stringify(result, null, 2)}\n`);
        }
      }
      await sleep(250);
      const image = await cdp.send("Page.captureScreenshot", { format: "jpeg", quality: 92 });
      const file = path.join(outDir, `${name}.jpg`);
      await writeFile(file, Buffer.from(image.data, "base64"));
      process.stdout.write(`${file}\n`);
    }
    cdp.close();
  } finally {
    browser.kill();
    server.close();
  }
};

await main();
