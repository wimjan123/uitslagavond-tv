# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Dashboard and tablet: one web app in TypeScript with React, served by the Rust server (see docs/adr/0001-server-is-enige-bron-van-de-stand.md). Prototypes are single static HTML files on throwaway branches.

## Users

- **Regie**: the operators in the control room of a Dutch broadcaster. They prepare and show graphics, type in election figures and votes, and steer the camera through the 3D Zaal, live on air. One or two people, chosen per broadcast (issue #21). Each works on their own Werkplek (a laptop with a name, no login) with the same full dashboard and no roles; who does what is agreed between them. Two people may edit the same figure set: per field the last change wins, and each sees which set the other has open. "Toon" only shows the Graphic the pressing screen had in view; if it changed in between, the server refuses and the button says who changed it.
- **Presentator**: the presenter in the studio. Uses a tablet to put a Preset ready (the Regie then shows it) and to add or remove parties in the Coalitiebouwer.

## Product Purpose

Live tv-graphics of the Tweede Kamer: a realtime 3D plenary hall whose seats colour per party, plus 2D graphics, for an Uitslagavond, the Coalitiebouwer and Stemmingen. The dashboard is where the Regie controls everything. Success: during a live broadcast the right graphic is on air within seconds, without mistakes.

## Positioning

A broadcast graphics system built around the real Zaal of the Tweede Kamer, with Dutch parliamentary concepts (Fractie, Zetel, Stemming, Coalitiebouwer) as first-class objects instead of generic text layers.

## Operating Context

- Live broadcast, under time pressure, in a dim control room.
- **The Regie works on a laptop** (owner answer, 2026-11-20). Compact screen, mouse/trackpad and keyboard.
- Fastest actions, in the owner's words: putting a Preset ready and showing it, entering or updating figures, and camera and Standpunten.
- Flow: drie zelfstandige, koppelbare lagen op zowel Preview als Live — **Camera** (Standpunt of Camerarit), **Stoelbeeld** (alleen de 150 Stoelen met eigen Kleurovergang en Kleurkracht) en **Graphic** (2D-overlay). De Regie zet één of meer lagen klaar in Preview, kiest de overgang (**Beweeg**, **Fade** of **Knip**), en drukt "Take". Ongewijzigde lagen lopen op Live door (zodat een Graphic of Stoelbeeld over een lopende Camerarit heen kan starten). Geen rundown.
- Twee hoofdwerkruimtes: **Regie** (grote 720p Preview + Take + Live in OBS-stijl, met daaronder Presets, Zaal & Camera, Stemmingen, Coalitie en Cijfers) en **Studio** (volle schermhoogte voor Sjablonen, Programmastijl en Camera-opnames, met alleen een smalle Live-balk bovenin). **Systeem** zit achter de statusindicator rechtsboven.
- Tijdens het invullen van een Graphic of het bewerken van een Sjabloon toont het scherm een persoonlijk on-demand 720p **Conceptbeeld** uit de Preview-renderer, zonder de gedeelde Preview van een andere Werkplek te overschrijven.
- Ether and Reserve follow the same Stand from one server. Renderers can drop out; the dashboard must show that.

## Capabilities and Constraints

- Twelve Sjablonen in four groups (Uitslagavond, Coalitiebouwer, Stemmingen, Algemeen); see issue #6.
- Five kinds of figure sets: peiling, exitpoll, prognose, tussenstand, uitslag. Sets and Stemmingen can be locked.
- Stemmingen are entered by hand per Fractie, following the Voorzitter: tap the named Fracties, then "Overige tegen"; single Kamerleden voting otherwise via Afwijkende stem. The system calculates the outcome. "Vastleggen en klaarzetten" puts the Graphic ready and jumps to the next open motie. After the broadcast the system fetches the official result and shows differences; the Regie decides (issue #27).
- Camera: fixed Standpunten and Cameraritten can be staged in Preview (default, starting on Take via Beweeg, Fade or Knip) or fired Direct live via a clear mode toggle; plus Vrije camera on a gamepad and Oefenen/recording. The server computes the camera for every Renderer (issue #23).
- Presets: visual 16:9 cards (with 1-click list view), drag-and-drop ordering (top 9 get keys 1–9), direct creation/editing/duplicating/renaming/deleting without leaving Regie, saving any combination of Camera, Stoelbeeld and Graphic.
- Studio (Vormgeving): intuitive drag-and-snap Sjabloon editor over the real 3D Zaal image (with title-safe guides, anchor presets, undo/redo, live animation preview, and a compact 4-section collapsible inspector); no timeline editor.
- Tablet for the Presentator: chosen variant T2 (two large tabs, Presets and Coalitiebouwer; a tap puts a Preset ready, the Regie shows it), styled in the same visual system as the Regie dashboard.
- Terminology: use the words in GLOSSARY.md (Stand, Opdracht, Ether, Reserve, Voorbeeld, Conceptbeeld, Stoelbeeld, Overvloeien, Graphic, Preset, Standpunt, Camerarit).

## Brand Commitments

None for the broadcast itself (configurable programme Stijl); for the Regie dashboard and tablet, the owner wants a calm, solid, high-craft studio production tool inspired by **DaVinci Resolve** and **OBS Studio** — significantly simpler than Figma, never crowded with tiny buttons, with at most one level of progressive disclosure, and letting the 16:9 broadcast imagery lead.

## Evidence on Hand

- Real party names and the 2023 election result. Party logos may be used in prototypes only (owner answer, 2026-11-20); whether the broadcaster may use them is an open question on the map.
- No real broadcast figures, Kamerleden photos or vote data in prototypes: label all of it as example data.

## Product Principles

1. Speed on air beats completeness: the three fastest actions are never more than one click away.
2. What is on air and what is ready must be impossible to confuse.
3. Recognise before reading: parties by logo and colour, sections by symbol.
4. Every change is safe to make live: locks, clear states, and nothing goes on air without "toon".
