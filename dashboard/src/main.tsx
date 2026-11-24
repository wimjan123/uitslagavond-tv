import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { TabletApp } from "./tablet/Tablet";
import "@fontsource-variable/inter/wght.css";
import "./stijl.css";

// De tablet van de Presentator heeft een eigen adres in dezelfde webapp.
const opTablet = /^\/tablet\/?$/.test(location.pathname);
if (opTablet) document.title = "Tablet · uitslagavond-tv";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    {opTablet ? <TabletApp /> : <App />}
  </StrictMode>,
);
