---
name: uitslagavond-tv Regie dashboard
description: The Regie's live control surface for the Tweede Kamer graphics, laid out like broadcast software, with a multiviewer on top and lists below.
colors:
  graphite-field: "#0c0e10"
  graphite-ground: "#121417"
  graphite-panel: "#181b1f"
  graphite-raised: "#1f2328"
  graphite-divider: "#20242a"
  selection-fill: "#26303d"
  graphite-line: "#2a2f36"
  paper-ink: "#e4e7eb"
  slate-mute: "#8c939d"
  steel-blue: "#6f9fe0"
  steel-ink: "#08111e"
  ether-red: "#d3302f"
  ether-red-ink: "#ffffff"
  ether-red-text: "#ff7a78"
  klaar-green: "#22844f"
  klaar-green-ink: "#ffffff"
  klaar-green-text: "#5fd093"
  warn-amber: "#d89b2b"
  warn-amber-ink: "#1b1200"
  logo-tile: "#ffffff"
  monitor-black: "#000000"
  preview-canvas: "#0d1220"
  seat-empty: "#3a404c"
  seat-dim: "#1d2130"
typography:
  display:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, Helvetica Neue, sans-serif"
    fontSize: "56px"
    fontWeight: 800
    lineHeight: 1
    fontFeature: "\"tnum\", \"cv11\""
  headline:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "17px"
    fontWeight: 650
    lineHeight: 1.35
  action:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "16px"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "0.08em"
  outcome:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 700
    lineHeight: 1.35
    letterSpacing: "0.06em"
  body:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, Helvetica Neue, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.35
    fontFeature: "\"tnum\", \"cv11\""
  title:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 600
    lineHeight: 1.35
  control:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 550
    lineHeight: 1.35
  small:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.35
  label:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "10.5px"
    fontWeight: 600
    lineHeight: 1.35
    letterSpacing: "0.08em"
  stamp:
    fontFamily: "Inter, system-ui, sans-serif"
    fontSize: "10.5px"
    fontWeight: 700
    lineHeight: 1
    letterSpacing: "0.08em"
  keycap:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace"
    fontSize: "10px"
    fontWeight: 600
    lineHeight: 1
rounded:
  none: "0px"
  key: "2px"
  control: "3px"
spacing:
  hair: "4px"
  tight: "6px"
  sm: "8px"
  md: "10px"
  gutter: "12px"
  lg: "14px"
components:
  tab-strip:
    backgroundColor: "{colors.graphite-field}"
    height: "34px"
  tab:
    textColor: "{colors.slate-mute}"
    typography: "{typography.control}"
    padding: "0 12px"
  tab-on:
    backgroundColor: "{colors.graphite-panel}"
    textColor: "{colors.paper-ink}"
  tab-count:
    backgroundColor: "{colors.graphite-raised}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.key}"
    height: "16px"
  multiviewer:
    backgroundColor: "{colors.graphite-panel}"
    padding: "10px 12px"
  monitor:
    backgroundColor: "{colors.monitor-black}"
    rounded: "{rounded.none}"
  monitor-label:
    backgroundColor: "{colors.graphite-field}"
    textColor: "{colors.slate-mute}"
    typography: "{typography.small}"
    height: "26px"
    padding: "0 8px"
  toon-button:
    backgroundColor: "{colors.ether-red}"
    textColor: "{colors.ether-red-ink}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    height: "62px"
  button:
    backgroundColor: "{colors.graphite-raised}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    padding: "0 10px"
    height: "32px"
  button-primary:
    backgroundColor: "{colors.paper-ink}"
    textColor: "{colors.graphite-ground}"
    rounded: "{rounded.control}"
  button-commit:
    backgroundColor: "{colors.klaar-green}"
    textColor: "{colors.klaar-green-ink}"
    rounded: "{rounded.control}"
    height: "36px"
    padding: "0 16px"
  filter-toggle:
    textColor: "{colors.slate-mute}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    padding: "4px 9px"
  filter-toggle-on:
    backgroundColor: "{colors.selection-fill}"
    textColor: "{colors.paper-ink}"
  segment:
    backgroundColor: "{colors.graphite-raised}"
    textColor: "{colors.slate-mute}"
    rounded: "{rounded.control}"
    padding: "6px 12px"
  segment-voor-on:
    backgroundColor: "{colors.klaar-green}"
    textColor: "{colors.klaar-green-ink}"
  segment-tegen-on:
    backgroundColor: "{colors.ether-red}"
    textColor: "{colors.ether-red-ink}"
  input-field:
    backgroundColor: "{colors.graphite-field}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.control}"
    padding: "6px 9px"
  table-head:
    backgroundColor: "{colors.graphite-panel}"
    textColor: "{colors.slate-mute}"
    typography: "{typography.label}"
    padding: "6px 10px"
  table-row:
    textColor: "{colors.paper-ink}"
    typography: "{typography.body}"
    padding: "5px 10px"
  table-row-selected:
    backgroundColor: "{colors.selection-fill}"
  stamp-live:
    backgroundColor: "{colors.ether-red}"
    textColor: "{colors.ether-red-ink}"
    typography: "{typography.stamp}"
    rounded: "{rounded.key}"
    padding: "4px 7px"
  stamp-klaar:
    backgroundColor: "{colors.klaar-green}"
    textColor: "{colors.klaar-green-ink}"
    typography: "{typography.stamp}"
    rounded: "{rounded.key}"
    padding: "4px 7px"
  stamp-warn:
    backgroundColor: "{colors.warn-amber}"
    textColor: "{colors.warn-amber-ink}"
    typography: "{typography.stamp}"
    rounded: "{rounded.key}"
    padding: "4px 7px"
  keycap:
    typography: "{typography.keycap}"
    rounded: "{rounded.key}"
    padding: "3px 5px"
  standpunt-key:
    backgroundColor: "{colors.graphite-raised}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.small}"
    rounded: "{rounded.control}"
    height: "30px"
  standpunt-key-on:
    backgroundColor: "{colors.selection-fill}"
  fractie-key:
    backgroundColor: "{colors.graphite-raised}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.title}"
    rounded: "{rounded.control}"
    height: "42px"
  logo-tile:
    backgroundColor: "{colors.logo-tile}"
    rounded: "{rounded.key}"
    width: "30px"
    height: "19px"
  floor-plan:
    backgroundColor: "{colors.graphite-field}"
    rounded: "{rounded.control}"
  floor-plan-pin:
    backgroundColor: "{colors.graphite-raised}"
    textColor: "{colors.paper-ink}"
  floor-plan-pin-on:
    backgroundColor: "{colors.paper-ink}"
    textColor: "{colors.graphite-ground}"
  thumbnail:
    backgroundColor: "{colors.preview-canvas}"
    rounded: "{rounded.none}"
---

# Design System: uitslagavond-tv Regie dashboard

Chosen by the owner on 2026-11-22: design "Omroepsoftware", layout 1 (multiviewer on top). It replaces the earlier design "Plenaire Zaal", which the owner found childish for a control room.

## Overview

**Creative North Star: "The multiviewer in the gallery"**

The Regie dashboard looks like the broadcast software that already sits in a control room: a dark graphite surface, a row of monitors with labels under them, and plain lists below. Nothing on screen is playful. The only strong colours are the two that every control room already knows: red for what is on air and green for what is ready. Everything else is grey, so those two colours always mean something.

The shell never moves. A strip of text tabs runs along the top, and under it the multiviewer row holds Voorbeeld, TOON, Ether and the camera in every section. Only the work area below changes and scrolls. The work is set as tables and compact panels, not as cards: one row per Preset, one row per Stemming. Density is high, so the operator sees a whole evening of Presets at once, and every row carries its key.

The signature is the multiviewer. Voorbeeld and Ether are black-framed 16:9 pictures with a label strip under them, as on a real multiviewer: a coloured top line (green or red), the word VOORBEELD or ETHER, the name of the Graphic and the time on air. Next to them, the camera block holds the floor plan of the Zaal and a row of Standpunt keys.

**Key Characteristics:**
- Graphite ground, paper-white ink, slate grey for everything secondary.
- Two broadcast colours only: Ether red for on air, Klaar green for ready. Amber marks attention.
- Selection is a quiet blue-grey fill with a thin steel-blue line, never a bright colour block.
- Text tabs on top, multiviewer row under them, one work area that scrolls.
- Tables and rows, not tiles. Every Preset has a number, every Standpunt a letter, every shortcut a key cap.
- Square monitors with a label strip; small 2 to 3px corners on controls; no shadows.
- Party logos small on white tiles; input controls use a party-colour stripe and the short name.

## Colors

A graphite palette of six steps, one steel-blue line for selection, and the two broadcast state colours with lighter text variants for use on dark ground.

### Primary
- **Steel Blue** (steel-blue): the operator's selection. The 2px line under the active tab, the 3px line at the left of a selected row, the border of a selected Standpunt key, the focus outline and the tablet toggle when on. It is a line, never a fill. Text in steel blue on the ground reaches 6.8:1.

### Secondary (broadcast states)
- **Ether Red** (ether-red): on air. The TOON button, the top line of the Ether label strip, the Live stamp, the left line and tint of the on-air row, Tegen in Stemmingen. White ink (ether-red-ink) on top (5.0:1). On dark ground, red text uses Ether Red Text (ether-red-text, 7.6:1).
- **Klaar Green** (klaar-green): ready in the Voorbeeld. The top line of the Voorbeeld label strip, the Klaar stamp, the left line and tint of the ready row, Voor in Stemmingen, and the button "Vastleggen en klaarzetten". White ink (klaar-green-ink) on top (4.7:1). On dark ground, green text uses Klaar Green Text (klaar-green-text, 10.1:1).
- **Attention Amber** (warn-amber): something needs a look. Nieuwere cijfers, a Renderer with another version, the star on a motie for the broadcast, Oefenen, import notices. Near-black ink (warn-amber-ink) on top (7.6:1).

### Neutral
- **Graphite Field** (graphite-field): the tab strip, inputs, the search box, the monitor label strips, the floor plan and the Voorzitter sentence.
- **Graphite Ground** (graphite-ground): the page and the work area.
- **Graphite Panel** (graphite-panel): the multiviewer row, toolbars, sticky table heads, the action bar and the active tab.
- **Graphite Raised** (graphite-raised): buttons, segments, Standpunt keys and Fractie keys.
- **Graphite Divider** (graphite-divider): the line between table rows.
- **Graphite Line** (graphite-line): every other 1px border.
- **Selection Fill** (selection-fill): the fill of a selected row, filter or Standpunt key (paper ink on it: 10.8:1).
- **Paper Ink** (paper-ink): all primary text and icons (14.9:1 on the ground).
- **Slate** (slate-mute): secondary text, column heads, labels, idle tabs and filters (6.0:1 on the ground, 5.1:1 on raised).
- **Logo White** (logo-tile): the tile under every party logo.
- **Monitor Black** (monitor-black): behind a monitor picture.
- **Preview Canvas** (preview-canvas), **Empty Seat** (seat-empty), **Dimmed Seat** (seat-dim): only inside thumbnails and the floor plan, which imitate the Renderer. Seats take party colours from the party data and the programme style supplies the accent, so neither is a dashboard token.

### Named Rules
**The Two Colours Rule.** Only on air and ready get a strong colour, and they always get the same one: red for Ether, green for Klaar. Selection, hover and navigation stay grey with a steel-blue line, so a coloured thing on screen is always a broadcast state.

**The Tint Not Fill Rule.** A ready or on-air row is a light tint of its state colour with a 3px line at its left edge, never a solid block. Solid state fills are only for TOON, stamps, the Voor/Tegen segment and the commit button.

**The Ink Pair Rule.** Every state fill carries its own ink: white on Ether Red and Klaar Green, near-black on amber. State colours used as text on dark ground take their Text variant.

## Typography

**Body Font:** Inter (self-hosted variable woff2, weights 100 to 900, `font-display: block`), with system-ui, -apple-system, Segoe UI, Roboto and Helvetica Neue as fallback. Features `tnum` (tabular figures) and `cv11` (single-storey a) are on for the whole page.
**Label/Mono Font:** system monospace (ui-monospace, SFMono-Regular, Menlo) for key caps only.

**Character:** A neutral, dense interface face, like the software around it. Tabular figures make Zetels, times and counts line up in columns.

### Hierarchy
- **Display** (800, 56px, line-height 1): the coalition count in the Coalitiebouwer.
- **Headline** (650, 17px): the title of the open Stemming or set.
- **Action** (700, 16px, uppercase, 0.08em): the word TOON. Nothing else uses it.
- **Outcome** (700, 15px, uppercase, 0.06em): Aangenomen or Verworpen, in the state text colour.
- **Body** (400, 13px, line-height 1.35): default text and table cells.
- **Title** (600, 13px): the main line of a table row (Graphic name, motie title), Fractie names.
- **Control** (550, 13px): tabs, filters, buttons.
- **Small** (400, 12px, Slate): the second line of a row, hints, monitor label text.
- **Label** (600, 10.5 to 11px, uppercase, 0.08em, Slate): column heads, toolbar titles, VOORBEELD and ETHER in the label strip, CAMERA.
- **Stamp** (700, 10.5px, uppercase, 0.08em): state stamps.
- **Keycap** (600, 10px, monospace, 1px border in currentColor at 70% opacity): printed keys.

### Named Rules
**The One Family Rule.** Inter sets every word in the dashboard. Weight and the uppercase label style make hierarchy; only key caps switch to monospace.

**The Uppercase Is A Label Rule.** Uppercase is only for labels, stamps, the outcome word and TOON. Running text and names stay in sentence case, as the Dutch UI writes them.

## Layout

The shell is a CSS grid of three rows (tab strip, multiviewer, work area) over the full viewport. The page targets a laptop at 1440x900 and is checked at 1280x800; there are no breakpoints. The shell never scrolls; only the work area does.

**Tab strip (34px, Graphite Field).** Text tabs from the left: Presets, Maken, Cijfers, Stemmingen, Coalitie, Camera, Vormgeving, Systeem. The active tab has Graphite Panel fill, paper ink and a 2px steel-blue line at its foot. A tab with open work shows a small count box after its name. At the right: a status dot with label for Server, Ether and Reserve (green fine, amber needs a look), then the clock in 600. There is no Wand dot: the Videowand is out of scope for now.

**Multiviewer row (Graphite Panel, 10px 12px padding, 10px gaps, 1px line below).** Four columns, `1fr 170px 1fr 330px`, always visible in every section:
1. **Voorbeeld:** a monitor (see Components) with label "VOORBEELD", the Graphic name, a green top line when something is ready. During Oefenen the top line and label turn amber, the label reads OEFENEN (or a red dot with the recording time), and the label text says the gamepad only drives this picture. While the Regie tries a Standpunt or plays a Camerarit here, the top line is dashed.
2. **Take column (170px):** TOON (62px), then Bijwerken (B) and Uit beeld (X) as 34px buttons with the key cap at the right.
3. **Ether:** a monitor with label "ETHER", the Graphic name, "sinds hh.mm" at the right and a red top line. When figures changed after showing, the right side reads "Nieuwere cijfers" in amber.
4. **Camera (330px):** a label row "CAMERA" with the current Standpunt or Camerarit name in paper ink, the floor plan of the Zaal, then a grid of keys in three columns: one per Standpunt and Camerarit with its letter, then Vrij (V), Oefenen (O) and Opnemen (R, only enabled during Oefenen, reads Stop while recording).

**Work area.** A toolbar (Graphite Panel, 7px 12px padding, 1px line below): the section name as a label, then the section's filters as text toggles, a 200px search box where useful, and a key hint in Slate at the right. Under it the section body fills the rest and scrolls.
- **Presets:** one table (see Preset Table), ending in a row "Nieuwe Graphic maken".
- **Stemmingen** (behaviour from Proef: Stemmingen live invoeren): two panes, `minmax(300px, 36%) 1fr`, split by a 1px line. Left: the motie list of the day as a table (star, title with number below, status). Toolbar filters: "Voor de uitzending (n)" and "Alle n", plus "Met de hand" and the day and import time at the right. Right: the input pane (see Stemming Input) with a fixed action bar at its foot, so "Vastleggen en klaarzetten" is always visible. A sub-view "Controle" lists each entered Stemming next to the official result as a table, most serious difference first.
- **Cijfers, Coalitie, Maken, Vormgeving, Systeem** follow the same pattern: a toolbar, then tables or a list plus a detail pane. Cijfers: sets as a list left, party rows as a table right. Coalitie: the arc (see Coalitiebouwer Arc) with a side list. Camera: the Camera Werkbank.

**Spacing rhythm.** 4, 6, 8, 10, 12, 14px. Table cells pad 5px 10px; controls sit 6 to 8px apart; panes pad 12px 14px.

### Keyboard
The keyboard works anywhere outside a text field. Inside a field keys type, and Escape leaves the field. Combinations with Cmd, Ctrl or Alt are ignored.

| Key | Action |
| --- | --- |
| 1 to 9 | Put the Preset with that number ready (Klaar). It does not show. |
| Space | TOON: put the ready Graphic in the Ether. |
| Enter | Never shows. It only presses the focused button. |
| A, S, D, F, G, H | Pick a fixed Standpunt: Totaal vanaf voorzitter, Laag over de banken, Linkerflank, Rechterflank, Bovenaf, Katheder. |
| J, K, L | Pick an own Standpunt or Camerarit. A new one gets the first free letter; the Regie can move or clear a letter. |
| Shift with a letter | Pick that Standpunt or Camerarit with a cut, without movement. |
| V | Vrije camera on or off (gamepad). |
| O | Oefenen on or off: the gamepad drives only the Voorbeeld camera, the Ether stays. |
| R | Start or stop recording a Camerarit. Only during Oefenen. B on the gamepad does the same; B on the keyboard stays Bijwerken. |
| B | Bijwerken: refresh the figures of the Graphic on air. |
| X | Uit beeld: take the Graphic off air. |

When focus sits on a button, Space and Enter press that button and do not show. Every shortcut is printed on its control as a key cap: Spatie on TOON, B and X under it, the letter on each Standpunt key and pin, the number in the first column of each Preset row.

## Elevation & Depth

The system is flat. Depth comes from the graphite steps, darkest at the back of controls: field, ground, panel, raised. Borders are 1px Graphite Line; rows are split by Graphite Divider. Nothing casts a shadow, TOON included.

### Shadow Vocabulary
- **Row marker** (`box-shadow: inset 3px 0 0 <colour>`): the left line of a selected (steel blue), ready (green) or on-air (red) table row.
- **Logo hairline** (`box-shadow: inset 0 0 0 1px #0000001a`): edge of every white logo tile.

### Named Rules
**The No Lift Rule.** No element casts a shadow. TOON stands out by being the only large red control, not by height.

## Shapes

Almost square. Monitors and thumbnails have no rounding, like the pictures on a multiviewer. Controls (buttons, inputs, segments, Standpunt and Fractie keys, the floor plan) have 3px corners; key caps, stamps, logo tiles and count boxes have 2px. There are no pills and no round chips. Circles appear only for status dots and the Standpunt pins on the floor plan. The Zaal is the one organic shape: the floor plan draws the hemicycle, and the Coalitiebouwer lays parties on a semicircle.

A dashed border means "filled in by the rest": a Fractie key set by "Overige tegen" (or voor) instead of tapped.

## Components

### Buttons
- **Shape:** 3px corners, 32px tall in toolbars and action bars.
- **Default:** Graphite Raised fill, 1px line, paper ink 550 text, 0 10px padding, no wrap.
- **Primary:** paper-ink fill with graphite text. For "zet klaar" actions and a lock that is on ("Vergrendeld").
- **Commit:** "Vastleggen en klaarzetten" only: Klaar Green with white ink, 36px tall, 0 16px. Disabled while Fracties are open: Graphite Raised with Slate text.
- **Hover / Focus:** border turns Slate; focus shows a 2px steel-blue outline 2px out.
- **Disabled:** 40% opacity, default cursor. Disabled actions stay visible.

### TOON
The one large red control. 62px tall, full width of the take column, 3px corners, Ether Red with white ink, the word TOON in the action style at the left and a "Spatie" key cap at the right. No icon, no shadow, no press animation beyond the browser's own. Without a ready Graphic it dims to 40%.

### Monitor (signature)
Voorbeeld and Ether, as on a multiviewer.
- **Picture:** the 16:9 thumbnail on Monitor Black, no rounding, 1px Graphite Line frame. The picture keeps its ratio and is centred; black fills the rest.
- **Label strip (26px, Graphite Field):** a 3px top line in the state colour (green Voorbeeld with something ready, red Ether with something on air, Graphite Line when empty), the label word in 700 11px uppercase in the state text colour, the Graphic name in Slate with an ellipsis, and at the right the time on air or "Nieuwere cijfers" in amber.

### Filters and Segments
- **Filter toggle:** text only, Slate, 4px 9px, 3px corners. Hover: paper ink. On: Selection Fill with paper ink. Used in toolbars for Preset groups, "Op de tablet", and the Stemming lists.
- **Segment:** one bordered group of 13px Slate buttons divided by Graphite Line. A chosen option gets Selection Fill and a 2px steel-blue line at its foot. In Stemmingen the Voor option fills Klaar Green and Tegen fills Ether Red, both with white ink.

### Stamps
States stamp; nothing silently disappears. 10.5px 700 uppercase, 2px corners, 4px 7px padding.
- **Live** (Ether Red, white): on the on-air Preset row.
- **Klaar** (Klaar Green, white): on the ready Preset row.
- **Was live hh.mm:** plain Slate text, no box.
- **Warn** (amber, near-black): "Andere versie, geweigerd" and similar.

### Preset Table
The default work area. Columns: Toets (44px), Beeld (108px), Graphic, Gegevens, Camera, Tablet (60px), Status (120px).
- **Head:** sticky, Graphite Panel, label style, 1px line below.
- **Row:** 5px 10px cells, Graphite Divider between rows. Toets is a 22px key cap box with the number. Beeld is an 88px thumbnail with a 1px line. Graphic is the Sjabloon name in the title style over its group (or the Titelkaart text) in Slate. Gegevens shows up to four 30px logo tiles and the set name in Slate. Camera shows the letter and the Standpunt name in Slate. Tablet is a 26x24px toggle: Slate tablet icon, steel-blue icon and border when the Preset is on the Presentator's tablet. Status holds the stamp.
- **States:** hover tints the row 3% white. Klaar: green tint and green left line plus the Klaar stamp. Live: red tint and red left line plus the Live stamp. Earlier on air: "was live hh.mm" in the status column.
- **Click:** a click anywhere on the row puts the Preset ready. It never shows. The tablet toggle only toggles the tablet.

### Stemming Input
The right pane in Stemmingen, top to bottom, 12px 14px padding, 10px gaps:
- **Header:** the motie number and indiener in Slate small, the title in the headline style, and at the right the lock button.
- **Outcome:** the outcome word in the outcome style (green Aangenomen, red Verworpen, amber while open), "n voor · n tegen", and "uit de Zetels van 17 Fracties" in Slate.
- **Seat bar:** 8px tall, 1px corners: green Voor, empty open, red Tegen, with the majority marker.
- **Voorzitter sentence:** Graphite Field, 3px corners, 14px text, a microphone icon; Fractie names in it sit on a light state tint in the state text colour.
- **Side switch:** "Ik tik nu" with the Voor/Tegen segment and a one-line hint in Slate.
- **Fractie keys:** a grid of 6 columns, 4px gaps. Each key is 42px, Graphite Raised, 3px corners: a 4px stripe in the party colour at the left, the short name in the title style, the Zetels in Slate at the bottom right, the tap order as a small number at the top left, and VOOR or TEGEN as a small label at the top right once set. Voor tints green with a green border, Tegen tints red with a red border. A key filled by "Overige" has a dashed border. No logos here: the stripe and the name are faster to read.
- **Action bar (fixed at the pane foot, Graphite Panel):** Overige tegen (or voor), Met algemene stemmen, Afwijkende stem, Wissen; at the right Volgende and the Commit button "Vastleggen en klaarzetten".

### Logo Tile
Party logos sit on a white tile, 30px wide by default, height 62% of the width, 2px corners, 1px inset hairline, the logo contained at 84% by 78%. They appear where they identify the content of a Graphic: the Gegevens column of the Preset table, Cijfers rows and the Coalitiebouwer. A party without a supplied logo gets a tile of the same size in its party colour with its short name in white 700.

### Seat Bar
A bar of the 150 Zetels in party colours, in Uitslagindeling order from left to right, 1px corners, with a 2px paper-ink marker at 75.5 of 150 for the majority line. 8px in Stemmingen, up to 14px in Cijfers and the Coalitiebouwer. In the Coalitiebouwer the coalition parties come first and the rest stay empty.

### Coalitiebouwer Arc
Parties on a semicircle, each a 40px logo tile over its Zetel count. A chosen party gets Selection Fill, a 1px steel-blue border and a small check; the others show their logo at 85%. The centre shows the total in the display style, green text from 76, with "meerderheid" or "nog N nodig voor 76" below in Slate.

### Floor Plan
The floor plan of the Zaal is part of the camera block in the multiviewer row and large in the Camera section.
- **Drawing:** a top-down SVG on Graphite Field with 3px corners. The hemicycle floor in a slightly lighter graphite with a Graphite Line edge, 150 Stoelen as dots in the colours of what is in the Ether now, and the voorzitter desk in Graphite Line.
- **Pins:** each Standpunt is a round pin with its letter. Idle: Graphite Raised fill, 1px Slate stroke, paper-ink letter. Hover: paper-ink stroke. Active: paper-ink fill, graphite letter and a view cone in white at 13%. Bovenaf shows a round cone. Pins are focusable buttons.
- **Camerarit:** a square pin at its start with its letter, and a line along its route ending in a small dot. Idle routes are dashed at 35%; the active or selected route is solid.
- **Camera markers:** a red marker with a short cone shows where the Ether camera is and where it looks. During Oefenen or a try-out a green marker shows the Voorbeeld camera. While recording, a green trail draws the route.
- **Vrije camera:** the Vrij key gets Selection Fill. During Vrij or Oefenen a dashed amber line shows the invisible space the camera may fly in. "Bewaar als Standpunt" then adds a pin and a key with the next free letter.

### Standpunt Keys
Under the floor plan, a grid of three columns, 4px gaps. Each key is 30px, Graphite Raised, 3px corners: a key cap with the letter, then the name with an ellipsis. The current one has Selection Fill and a steel-blue border. Vrij, Oefenen and Opnemen follow as keys of the same shape; Oefenen turns amber when on, Opnemen shows a red dot while recording.

### Camera Werkbank
The Camera section, chosen as design A in Proef: Camerarit opnemen en bijschaven in het dashboard. Three columns that fill the work area without scrolling the page:
- **List (250px, scrolls):** a table grouped by Vaste Standpunten, Eigen Standpunten and Cameraritten. Each row: the letter key cap, the name, and a Slate line with its timing (for example "2,0 s · schuift" or "16,0 s · herhaalt"). A red dot marks the one in the Ether. The selected row gets Selection Fill and the steel-blue left line.
- **Middle (1fr):** a button row (Vrij, Oefenen, Opnemen, Bewaar als Standpunt) above the large floor plan. Here a click on a pin or row selects it for editing; picking for the Ether happens with the Standpunt keys in the multiviewer row or with the letter.
- **Settings panel (370px, scrolls).** Empty: a short how-to for recording and the gamepad table. While recording: the recording time, a Stop button and the gamepad table.
  - Standpunt: name (fixed ones show a Vast stamp and cannot be renamed or wiped), Kies nu and Probeer in Voorbeeld, then Overgang hierheen (Bewegen or Knippen; sliders for Duur 0,5 to 6 s, Zacht starten, Zacht stoppen; a small curve of the movement), Na aankomst (Stil or Langzaam schuiven with a direction), Positie (Vervang door huidige camera, only with Vrij or Oefenen), the letter, and Wis.
  - Camerarit: name, Kies nu and Speel af in Voorbeeld, a bar of the recording with the cut parts hatched and a green play head, sliders for Gladheid, Duur, Begin eraf, Eind eraf, Zacht starten and Zacht stoppen, Begin (Eerst overgang or Knippen), Eind (Stoppen or Herhalen; Herhalen is disabled with a reason when start and end lie apart), Opnieuw opnemen, the letter, and Wis.
  - Wis is disabled while a Sjabloon or Preset uses the item, with a line that names them.
- **Live feedback:** every slider change plays the result at once in the Voorbeeld. There is no dragging of points; another route means recording again.
- **After a recording** the new Camerarit is saved at once with the next free letter, selected, and played in the Voorbeeld.

### Thumbnails
Voorbeeld, Ether and Preset thumbnails are 320x180 SVG sketches of the Renderer output: the Preview Canvas, the Zaal seats, and the Graphic laid on top in the programme style. They show the broadcast, not the dashboard, so dashboard tokens do not apply inside them.

### Icons
Lucide 0.460.0, stroke 2, round caps and joins, inlined as SVG in currentColor. Used sparingly: in the search box, the lock button, the microphone of the Voorzitter sentence, the tablet toggle and the star of a motie. Tabs, TOON and table rows carry no icons. Sizes 14 to 16px.

## Do's and Don'ts

### Do:
- **Do** keep the tab strip and the multiviewer row (Voorbeeld, TOON, Ether, camera) on screen in every section; only the work area scrolls.
- **Do** use red only for on air and green only for ready, everywhere.
- **Do** show selection as Selection Fill with a steel-blue line, and keep it quiet.
- **Do** show ready and on-air rows as a light tint with a 3px left line, plus a stamp.
- **Do** set lists as tables with a sticky head, one row per item, and put the key in the first column.
- **Do** give every Preset a fixed number and every Standpunt a fixed letter, and print every shortcut on its control as a key cap.
- **Do** let a digit put a Preset ready and let only Space or the TOON button show it.
- **Do** keep "Vastleggen en klaarzetten" in a fixed action bar, always visible.
- **Do** stamp state changes ("was live 20.58", "Vergrendeld", "Nieuwere cijfers") instead of removing them; dim unavailable actions to 40% instead of hiding them.
- **Do** put party logos on white tiles, and use a party-colour stripe with the short name on input keys.
- **Do** order parties by the Uitslagindeling (SP on the left to FvD on the right) in seat bars, Cijfers rows, Fractie keys and the Coalitiebouwer arc.
- **Do** set all text in Inter with tabular figures, and all icons in Lucide 0.460.0 at stroke 2.
- **Do** use the GLOSSARY words in the Dutch UI: Regie, Ether, Voorbeeld, Preset, Standpunt, Fractie, Zetel, Stoel, Zaal.

### Don't:
- **Don't** let Enter show a Graphic.
- **Don't** fill large areas with colour: no coloured navigation, no red or green cards, no coloured section headers.
- **Don't** use red or green for selection, hover or decoration.
- **Don't** bring back tiles or cards for lists that are tables: Presets, Stemmingen, sets, Standpunten.
- **Don't** use pills, fully round chips or large corner radii.
- **Don't** add shadows or lift animations.
- **Don't** put large logos on input controls; Fractie keys use the stripe and the short name.
- **Don't** add icons to tabs, TOON or table rows.
- **Don't** carry devices from the earlier designs: no pictogram rail, no Kamerblauw, no maple, no page numbers, fastext keys or split-flap text.
