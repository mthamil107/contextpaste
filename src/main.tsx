import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { RegionSelectorOverlay } from "./components/RegionSelector/RegionSelectorOverlay";
import "./index.css";

const windowLabel = getCurrentWindow().label;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {windowLabel === "region-selector" ? <RegionSelectorOverlay /> : <App />}
  </React.StrictMode>,
);
