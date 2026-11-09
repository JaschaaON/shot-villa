/* Attrappe der Tauri-Schnittstelle, damit die Oberfläche ohne App im Browser läuft (nur für Produktbilder). */
(() => {
  const params = new URLSearchParams(location.search);
  const label = params.get("label") ?? "main";
  const sample = params.get("sample") ?? "/sample.png";
  const shot = { height: 2136, id: 1, scale: 2, width: 3200 };
  const settings = {
    afterCapture: { copy: true, save: false, show: true },
    alwaysOnTop: false,
    areaOpens: "editor",
    autoUpdateCheck: true,
    captureCursor: "auto",
    colorFormat: "hex",
    downscaleOnSave: false,
    escCopies: true,
    escSaves: false,
    hotkeys: {
      activeWindow: null,
      anyWindow: null,
      area: "CmdOrCtrl+Shift+2",
      fullscreen: "CmdOrCtrl+Shift+1",
      ocr: "Control+Alt+CmdOrCtrl+O",
      record: "Control+Alt+CmdOrCtrl+R",
      recordFullscreen: "Control+Alt+CmdOrCtrl+F",
      repeatArea: null,
      showApp: null,
      t3Area: "Control+Alt+CmdOrCtrl+T",
    },
    launchAtLogin: true,
    ocrRemoveLineBreaks: true,
    preferActualSize: true,
    saveFolder: "/Users/jascha/Bilder/Pivolo",
    saveFormat: "auto",
    t3Window: "T3 Code",
    thumbnailClosing: "manual",
  };
  let nextCallback = 1;
  let nextListener = 1;
  const handlers = new Map();
  const listeners = new Map();
  window.__emit = (event, payload) => {
    for (const id of listeners.get(event) ?? []) {
      handlers.get(id)?.({ event, id, payload });
    }
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    convertFileSrc: (path) => path,
    async invoke(command, args) {
      switch (command) {
        case "get_settings": { return settings;
        }
        case "set_settings": { return [];
        }
        case "current_shot":
        case "shot_info":
        case "pin_shot": { return params.get("empty") ? null : shot;
        }
        case "shot_png": { return await (await fetch(sample)).arrayBuffer();
        }
        case "overlay_info": {
          return {
            mode: params.get("mode") ?? "area",
            display: {
              displayId: 1,
              shotId: 1,
              frame: { x: 0, y: 0, width: innerWidth, height: innerHeight },
              scale: 2,
            },
            windows: [],
          };
        }
        case "screen_access": { return "denied";
        }
        case "plugin:app|version": { return "0.4.0";
        }
        case "plugin:event|listen": {
          listeners.set(args.event, [...(listeners.get(args.event) ?? []), args.handler]);
          return nextListener++;
        }
        default: { return null;
        }
      }
    },
    metadata: {
      currentWebview: { label, windowLabel: label },
      currentWindow: { label },
    },
    transformCallback(callback) {
      const id = nextCallback++;
      handlers.set(id, callback);
      return id;
    },
    unregisterCallback(id) { handlers.delete(id); },
  };
})();
