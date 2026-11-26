// DaVinci Deck entrypoint: self-hosted Archivo + JetBrains Mono, één ontwerpbestand (deck.css),
// ondersteunt Regie (/), Presentator Tablet (/tablet) en losse monitoren (/monitors).
import "@fontsource-variable/archivo";
import "@fontsource-variable/jetbrains-mono";
import "./deck.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { TabletApp } from "./tablet/Tablet";

const pad = location.pathname;
const opTablet = /^\/tablet\/?$/.test(pad);
const opMonitors = /^\/monitors\/?$/.test(pad);
if (opTablet) document.title = "Tablet · uitslagavond-tv";
if (opMonitors) document.title = "Monitoren · uitslagavond-tv";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    {opTablet ? <TabletApp /> : <App popoutMonitors={opMonitors} />}
  </StrictMode>,
);
