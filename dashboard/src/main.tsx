// De algemene stijl eerst, zodat de stijl van elk onderdeel erna komt en hem kan aanvullen
// (zoals de groene knop "Vastleggen en klaarzetten" boven de gewone knop).
import "@fontsource-variable/geist";
import "./stijl.css";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { TabletApp } from "./tablet/Tablet";
// Het ontwerp als laatste, zodat het boven de stijl van elk onderdeel komt (DESIGN.md).
import "./ontwerp.css";

// De tablet van de Presentator heeft een eigen adres in dezelfde webapp.
const opTablet = /^\/tablet\/?$/.test(location.pathname);
if (opTablet) document.title = "Tablet · uitslagavond-tv";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    {opTablet ? <TabletApp /> : <App />}
  </StrictMode>,
);
