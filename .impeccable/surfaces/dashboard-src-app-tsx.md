---
version: 1
slug: "dashboard-src-app-tsx"
primary_target: "dashboard/src/App.tsx"
related_targets: ["dashboard/src/tablet/Tablet.tsx"]
---

# Surface Brief: uitslagavond-tv Regie Dashboard & Tablet

- **Scope:** Full clean-slate rebuild of `dashboard/` (Regie dashboard + `/tablet`) paired with backend/Preview-renderer support (`crates/gedeeld`, `crates/server`, `crates/renderer`).
- **Visitor Mode:** Operate (live Dutch parliamentary broadcast control room on 1440×900 laptops + Presentator tablet).
- **Audience & Job:** Regie operators switching 3D Tweede Kamer graphics live under time pressure, staging camera moves, lighting seats (`Stoelbeeld`), entering votes/figures, and authoring/editing graphics & Sjablonen without Figma complexity.
- **Constraints:** Live safety invariants (`Enter` never takes, 1–9/Space/X/Z/A–H/Q–U shortcuts, 20s Ctrl+Z undo, ADR 0001 shared Stand + private on-demand 720p Conceptbeeld, ADR 0002 server camera, ADR 0003 block Sjablonen, ADR 0005 3 layers + Fade). Max 1 level of progressive disclosure; zero explanatory prose walls.

## Direction contract

- **THESIS:** DaVinci Deck — an engineered broadcast production deck that refuses flat dark-grey SaaS/Linear tables and narrow center-column multiviewers by putting full-width 720p Preview/Live dual viewers on top, a horizontal OBS/DaVinci Take + Beweeg/Fade/Knip transport bar beneath them, a Media Pool + instant right-hand Inspector below, and a bottom DaVinci page bar.
- **OWN-WORLD:** Solid warm-neutral anthracite studio planes (`#17181b` recessed viewer wells, `#222327` deck ground, `#282a30` elevated bars/inspectors, `#2b2d33` media tiles, crisp 1px dark seams & specular top edges), Archivo + JetBrains Mono typography, `#f0b44c` warm amber focus/editing accent, `#c9373d`/`#e5484d` Live red tally, `#1d8051`/`#2ea66d` Preview green tally, and 3-layer chips (`Camera · Stoelen · Graphic`).
- **STORY:** The Regie immediately reads all three layers on both Preview and Live from the dual viewers and layer badges, stages any combination (including camera moves and seats-only `Stoelbeeld`), picks Beweeg/Fade/Knip, and takes cleanly with zero confusion or tab-hopping.
- **FIRST VIEWPORT:** Twin edge-to-edge 16:9 720p viewers (`Preview` green bezel, `Live` red bezel) over a single horizontal transport bar (`Beweeg / Fade / Knip + duration` · `TAKE Spatie` · `Take out X` · `Kleur uit Z` · `▾ Meer` · clickable `Server · Live · Res · Clock` status pill); below that, visual Preset Media Pool (left) paired with a live-concept Inspector (right), anchored by the bottom DaVinci page bar (`Regie · Zaal & Camera · Stemmingen · Coalitie · Cijfers · Studio`). In `Studio`, the dual viewers collapse into a 36px red Live status strip for full-height drag-and-snap Sjabloon editing over the real 3D Zaal image.
- **FORM:** DaVinci Deck (candidate C, grounded DaVinci Resolve + OBS Studio commitment), seed:9318521e787890e4.
- **FINISH:** unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance.
