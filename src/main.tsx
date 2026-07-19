import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "./styles.css";
import "./desktop-tuning.css";
import "./time-machine.css";

// Keep the desktop shell and storage-intelligence styles loaded as one application surface.
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
