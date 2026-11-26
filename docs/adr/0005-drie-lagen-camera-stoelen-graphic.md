# Drie losse, koppelbare lagen: Camera, Stoelbeeld en Graphic

Het systeem werkt voortaan met drie zelfstandige lagen op zowel het Voorbeeld (Preview) als de Ether (Live): **Camera** (Standpunt, Camerarit of Vrije camera), **Stoelbeeld** (alleen de 150 Stoelen in de Zaal, met eigen Kleurovergang en Kleurkracht) en **Graphic** (de 2D-overlay). Een Preset kan elke losse laag of combinatie bewaren; bij `Toon` (Take) schakelen alle in het Voorbeeld klaargezette lagen tegelijk door via **Bewegen**, **Overvloeien (Fade)** of **Knippen**, terwijl ongewijzigde lagen op de Ether ongestoord doorlopen. Daarnaast krijgt elke Werkplek een persoonlijk on-demand 720p-**Conceptbeeld** tijdens het invullen van een Graphic of bewerken van een Sjabloon, zodat het gedeelde Voorbeeld van [ADR 0001](0001-server-is-enige-bron-van-de-stand.md) intact blijft.

## Considered Options

- **Huidige Graphic-centrische model met een extra "Alleen Stoelen"-Sjabloon.** Afgewezen: camerabewegingen bleven dan óf direct live óf vastgeklonken aan een Graphic, waardoor je geen Camerarit rustig in Preview kon klaarzetten en er later een Graphic overheen kon starten zonder de beweging te onderbreken.
- **Lokaal HTML/Canvas-voorbeeld in de browser voor het maken en ontwerpen.** Afgewezen: lettertypes, uitlijning en de 3D-Zaal wijken dan af van de Bevy-Renderer; de Regie zag niet wat er werkelijk de uitzending in ging.

## Consequences

- `Stand` en `Opdracht` in `crates/gedeeld` breiden uit met een klaargezette camerakeuze, een los `Stoelbeeld`, en `OvergangSoort::Overvloeien` (Fade met instelbare duur).
- Een klaargezette Camerarit wacht op zijn beginpunt in Preview (met optionele voorvertoning) en start pas op de Ether bij `Toon`; op het camerapaneel kiest de Regie tussen `Preview` (standaard, veilig klaarzetten) en `Direct live`.
- `UitBeeld` (Take out) haalt alleen de 2D-Graphic weg, `KleurUit` alleen de Stoelen, plus een gecombineerde actie voor beide.
- De Preview-renderer schaalt op naar 1280×720 en bedient naast het gedeelde Voorbeeld en Ether ook persoonlijke on-demand Conceptbeelden per Werkplek.
- Het dashboard splitst in twee werkruimtes (**Regie** voor live schakelen en **Studio** voor ruim ontwerpen van Sjablonen, Stijl en camera-opnames) plus **Systeem** achter de statusindicator.
