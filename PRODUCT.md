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
- Flow: the Regie puts one Graphic ready, sees it in the small Voorbeeld, and presses "toon". One Graphic on air at a time. No rundown.
- Ether and Reserve follow the same Stand from one server. Renderers can drop out; the dashboard must show that.

## Capabilities and Constraints

- Twelve Sjablonen in four groups (Uitslagavond, Coalitiebouwer, Stemmingen, Algemeen); see issue #6.
- Five kinds of figure sets: peiling, exitpoll, prognose, tussenstand, uitslag. Sets and Stemmingen can be locked.
- Stemmingen are entered by hand per Fractie, following the Voorzitter: tap the named Fracties, then "Overige tegen"; single Kamerleden voting otherwise via Afwijkende stem. The system calculates the outcome. "Vastleggen en klaarzetten" puts the Graphic ready and jumps to the next open motie. After the broadcast the system fetches the official result and shows differences; the Regie decides (issue #27).
- Camera: fixed Standpunten with smooth moves, plus a Vrije camera on a gamepad. A position can be saved as a Standpunt. A recorded move is a Camerarit; it is recorded while Oefenen, so only the Voorbeeld moves. The server computes the camera for every Renderer (issue #23).
- Vormgeving tab: adjust Sjablonen (place, size, colour, font, speed); no timeline editor.
- Tablet for the Presentator: chosen variant T2 (two large tabs, Presets and Coalitiebouwer; a tap puts a Preset ready, the Regie shows it).
- Terminology: use the words in GLOSSARY.md (Stand, Opdracht, Ether, Reserve, Voorbeeld, Graphic, Preset, Standpunt).

## Brand Commitments

None yet. The system is for one tv programme with a configurable style (colours, fonts, logo); the dashboard itself has no brand.

## Evidence on Hand

- Real party names and the 2023 election result. Party logos may be used in prototypes only (owner answer, 2026-11-20); whether the broadcaster may use them is an open question on the map.
- No real broadcast figures, Kamerleden photos or vote data in prototypes: label all of it as example data.

## Product Principles

1. Speed on air beats completeness: the three fastest actions are never more than one click away.
2. What is on air and what is ready must be impossible to confuse.
3. Recognise before reading: parties by logo and colour, sections by symbol.
4. Every change is safe to make live: locks, clear states, and nothing goes on air without "toon".
