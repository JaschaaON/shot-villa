import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { isMac } from "./platform";

import "./styles.css";
import { Editor } from "./views/editor";
import { Overlay } from "./views/overlay";
import { Permission } from "./views/permission";
import { Pin } from "./views/pin";
import { RecordingBar, RecordingFrame } from "./views/recording";
import { Settings } from "./views/settings";
import { Thumbnail } from "./views/thumbnail";
import { Welcome } from "./views/welcome";

const View = ({ label }: { label: string }) => {
  if (label.startsWith("overlay-")) {
    return <Overlay />;
  }
  if (label.startsWith("pin-")) {
    return <Pin />;
  }
  switch (label) {
    case "permission": {
      return <Permission />;
    }
    case "recording-bar": {
      return <RecordingBar />;
    }
    case "recording-frame": {
      return <RecordingFrame />;
    }
    case "settings": {
      return <Settings />;
    }
    case "thumbnail": {
      return <Thumbnail />;
    }
    case "welcome": {
      return <Welcome />;
    }
    default: {
      return <Editor />;
    }
  }
};

const { label } = getCurrentWebviewWindow();
document.documentElement.dataset.view = label.replace(/-\d+$/u, "");
document.documentElement.dataset.platform = isMac ? "mac" : "windows";

const root = document.querySelector("#root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <View label={label} />
    </StrictMode>
  );
}
