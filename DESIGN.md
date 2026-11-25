---
name: uitslagavond-tv Regie dashboard
description: The Regie's live control surface for the Tweede Kamer graphics, in the calm, precise style of modern product software, with a multiviewer on top and lists below.
colors:
  graphite-field: "#08090a"
  graphite-ground: "#0b0c0e"
  graphite-panel: "#0f1012"
  graphite-raised: "#17181b"
  graphite-divider: "#18191c"
  graphite-line: "#232428"
  selection-fill: "#1b1c20"
  surface: "#ffffff09"
  surface-hover: "#ffffff11"
  surface-on: "#ffffff1a"
  edge: "#ffffff13"
  edge-strong: "#ffffff21"
  paper-ink: "#eeeff1"
  slate-mute: "#9fa2ab"
  tekst-3: "#8e919a"
  steel-blue: "#8d95f2"
  steel-ink: "#0b0c0e"
  ether-red: "#e5484d"
  ether-vul: "#c93a40"
  ether-vul-hover: "#d6444a"
  ether-red-ink: "#ffffff"
  ether-red-text: "#ff6b6f"
  klaar-green: "#2f9d68"
  klaar-vul: "#1f7a4f"
  klaar-vul-hover: "#258a5a"
  klaar-green-ink: "#ffffff"
  klaar-green-text: "#4ccf8e"
  warn-amber: "#e3a23b"
  warn-amber-ink: "#1a1203"
  take-warn-text: "#ffd27a"
  monitor-black: "#000000"
  logo-tile: "#ffffff"
  preview-canvas: "#0d1220"
  seat-empty: "#3a404c"
typography:
  display:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "56px"
    fontWeight: 800
    lineHeight: 1
    fontFeature: "\"tnum\", \"ss01\""
  headline:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.012em"
  action:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1
    letterSpacing: "-0.01em"
  outcome:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  body:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.45
    letterSpacing: "-0.006em"
    fontFeature: "\"tnum\", \"ss01\""
  title:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 500
    lineHeight: 1.45
  control:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 500
    lineHeight: 1.45
  small:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.45
  label:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 500
    lineHeight: 1.45
    letterSpacing: "0"
  stamp:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 500
    lineHeight: 1.45
    letterSpacing: "0"
  keycap:
    fontFamily: "Geist Variable, Geist, system-ui, sans-serif"
    fontSize: "11px"
    fontWeight: 500
    lineHeight: "15px"
rounded:
  key: "4px"
  chip: "5px"
  control: "6px"
  segment: "7px"
  monitor: "8px"
  panel: "10px"
  arc: "12px"
  dot: "50%"
spacing:
  hair: "4px"
  tight: "6px"
  sm: "8px"
  md: "10px"
  gutter: "12px"
  lg: "14px"
  xl: "16px"
components:
  tab-strip:
    backgroundColor: "{colors.graphite-ground}"
    height: "40px"
    padding: "0 8px"
  tab:
    textColor: "{colors.slate-mute}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    height: "28px"
    padding: "0 10px"
  tab-on:
    backgroundColor: "{colors.surface-on}"
    textColor: "{colors.paper-ink}"
  tab-count:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.slate-mute}"
    rounded: "{rounded.key}"
    height: "17px"
  multiviewer:
    backgroundColor: "{colors.graphite-ground}"
    padding: "12px"
  monitor:
    backgroundColor: "{colors.graphite-panel}"
    rounded: "{rounded.monitor}"
  monitor-label-strip:
    backgroundColor: "{colors.graphite-panel}"
    textColor: "{colors.slate-mute}"
    typography: "{typography.small}"
    padding: "7px 10px 8px"
  monitor-label-preview:
    backgroundColor: "{colors.klaar-vul}"
    textColor: "{colors.klaar-green-ink}"
    rounded: "{rounded.key}"
    padding: "2px 7px"
  monitor-label-live:
    backgroundColor: "{colors.ether-vul}"
    textColor: "{colors.ether-red-ink}"
    rounded: "{rounded.key}"
    padding: "2px 7px"
  take-button:
    backgroundColor: "{colors.ether-vul}"
    textColor: "{colors.ether-red-ink}"
    typography: "{typography.action}"
    rounded: "{rounded.monitor}"
    height: "58px"
    padding: "0 12px"
  take-column-button:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    height: "28px"
  button:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    padding: "0 10px"
    height: "28px"
  button-primary:
    backgroundColor: "{colors.paper-ink}"
    textColor: "{colors.graphite-ground}"
    rounded: "{rounded.control}"
  button-commit:
    backgroundColor: "{colors.klaar-vul}"
    textColor: "{colors.klaar-green-ink}"
    rounded: "{rounded.control}"
    height: "28px"
  button-danger:
    textColor: "{colors.ether-red-text}"
    rounded: "{rounded.control}"
  undo-box:
    backgroundColor: "{colors.graphite-raised}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.monitor}"
    padding: "7px 8px 9px"
  undo-button:
    backgroundColor: "{colors.paper-ink}"
    textColor: "{colors.graphite-field}"
    rounded: "{rounded.control}"
    height: "26px"
  amber-chip:
    textColor: "{colors.warn-amber}"
    rounded: "{rounded.chip}"
    height: "20px"
    padding: "0 7px"
  filter-toggle:
    textColor: "{colors.slate-mute}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    height: "26px"
    padding: "0 9px"
  filter-toggle-on:
    backgroundColor: "{colors.surface-on}"
    textColor: "{colors.paper-ink}"
  segment:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.slate-mute}"
    rounded: "{rounded.segment}"
    padding: "2px"
  segment-on:
    backgroundColor: "{colors.surface-on}"
    textColor: "{colors.paper-ink}"
  segment-voor-on:
    backgroundColor: "{colors.klaar-vul}"
    textColor: "{colors.klaar-green-ink}"
  segment-tegen-on:
    backgroundColor: "{colors.ether-vul}"
    textColor: "{colors.ether-red-ink}"
  input-field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.control}"
    height: "30px"
    padding: "0 9px"
  table-head:
    backgroundColor: "{colors.graphite-ground}"
    textColor: "{colors.tekst-3}"
    typography: "{typography.label}"
    height: "32px"
    padding: "0 10px"
  table-row:
    textColor: "{colors.paper-ink}"
    typography: "{typography.body}"
  table-row-selected:
    backgroundColor: "{colors.surface-on}"
  stamp-live:
    textColor: "{colors.ether-red-text}"
    typography: "{typography.stamp}"
  stamp-klaar:
    textColor: "{colors.klaar-green-text}"
    typography: "{typography.stamp}"
  stamp-was:
    textColor: "{colors.slate-mute}"
    typography: "{typography.stamp}"
  keycap:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.slate-mute}"
    typography: "{typography.keycap}"
    rounded: "{rounded.key}"
    padding: "1px 5px"
  preset-key:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.slate-mute}"
    rounded: "{rounded.chip}"
    width: "22px"
    height: "22px"
  standpunt-key:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.small}"
    rounded: "{rounded.control}"
    height: "30px"
  standpunt-key-on:
    backgroundColor: "{colors.surface-on}"
  fractie-key:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.segment}"
    padding: "5px 8px 6px 10px"
  arc-party:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.monitor}"
    width: "56px"
  arc-party-on:
    backgroundColor: "{colors.surface-on}"
  floor-plan:
    backgroundColor: "{colors.graphite-panel}"
    rounded: "{rounded.monitor}"
  thumbnail:
    backgroundColor: "{colors.preview-canvas}"
    rounded: "{rounded.key}"
---

# Design System: uitslagavond-tv Regie dashboard

Chosen by the owner on 2026-11-25: design "Studio", on the layout of "Omroepsoftware" (layout 1, multiviewer on top, chosen 2026-11-22). Studio replaces "Mengpaneel", which the owner still found too AI-made and childish. His brief for Studio: "more like Linear", professional, and less text, with icons and tooltips where the picture is clear enough. Earlier designs that are gone: "Plenaire Zaal" (childish for a control room), the Inter look of Omroepsoftware, and the warm black with Barlow of Mengpaneel.

The design lives in `dashboard/src/ontwerp.css`, which `main.tsx` loads last, so it sits on top of `stijl.css` and the CSS of each part. Token names are older than Studio (steel-blue is no longer blue-grey, graphite-* are cool near-blacks); the names stay so the component CSS keeps working.

## Overview

**Creative North Star: "A quiet studio tool"**

The Regie dashboard looks like careful modern product software: a near-black, cool grey surface, thin light edges, 6 to 8px corners, one typeface at 13px, and small, neat text. Nothing is playful and nothing explains itself in a sentence when an icon, a word and a tooltip already say it. Colour is rare and always means something about the broadcast: red is what goes on air, green is what is ready, amber is "you must do something here". Everything else is grey.

The shell never moves. A strip of text tabs runs along the top, and under it the multiviewer row holds Preview, the Take column, Live and the camera block in every section. Only the work area below changes and scrolls. The work is set as quiet lists and compact panels: one row per Preset, one row per Stemming.

The signature is the tally. Preview and Live are 16:9 pictures with a label strip under them, as on a multiviewer. A ready Preview and Live carry a 3px ring in their tally colour round the whole monitor and a filled label ("Preview" on green, "Live" on red), so the state reads from the corner of your eye while the rest of the screen stays calm.

**Key Characteristics:**
- Cool near-black ground, light grey ink, two steps of grey for secondary text.
- Surfaces are translucent white over the ground (3.5%, 6.5%, 10%) with 1px edges at 7.5% or 13% white. A control that is on is one step lighter, never a coloured block.
- Solid red only for Live and Take. Solid green only for ready: the Preview label and every "put ready" button. Amber marks something to act on.
- Text tabs on top, multiviewer row under them, one work area that scrolls.
- Lists, not cards. Every Preset 1 to 9 has a key cap with its number, every Standpunt a letter; shortcuts are printed where they are used.
- Sentence case everywhere, including Take, Live and the stamps. No uppercase labels.
- Icons from Lucide with a word and a tooltip in the Take column; the tooltip says why a control is off.

## Colors

Six near-black steps, three translucent white surfaces and two edges for everything neutral, one accent for focus, and the three state colours. Red and green each have a lighter tally/text tone and a darker fill tone, so white ink on a fill reaches 4.5:1.

### Primary
- **Accent** (steel-blue, #8d95f2): focus and editing only. The 2px focus outline, the focus ring of an input, text selection, range sliders, the chosen sketch in Vormgeving and the chosen pin in the Camera section. It never marks a broadcast state and never fills a control.

### Secondary (broadcast states)
- **Live Red** (ether-red, #e5484d): the tally of Live. The 3px ring round the Live monitor, the dot of the Live stamp, the dot after "Camera" in the camera block, the left line of the camera key in use, the dashed frame when the Live-renderer is gone, the frame round a live coalition.
- **Live Fill** (ether-vul, #c93a40, hover ether-vul-hover): solid red with white ink. Only the Take button and the "Live" label under the Live monitor, and the pressed Tegen segment in Stemmingen. Red text on dark ground uses Live Red Text (ether-red-text, #ff6b6f).
- **Klaar Green** (klaar-green, #2f9d68): the tally of a ready Preview: the 3px ring round the monitor, the border of a Voor key in Stemmingen. Green text on dark ground uses Klaar Green Text (klaar-green-text, #4ccf8e): the Klaar stamp, Aangenomen, a coalition count from 76.
- **Klaar Fill** (klaar-vul, #1f7a4f, hover klaar-vul-hover): solid green with white ink. The "Preview" label of a ready monitor, every commit button that puts something ready (Zet klaar, Vastleggen en klaarzetten), the pressed Voor segment, the put-ready button on the tablet.
- **Attention Amber** (warn-amber, #e3a23b): do something. Old figures (an amber chip on the monitor, and inside Take a dark chip with take-warn-text #ffd27a), the undo box (its title and seconds), "typen" in Take while you type, Oefenen when on, a Stemming that is still Open, the count of differences on the Stemmingen tab, the dot before a reason next to a disabled button, the warning about current figures next to a live coalition. A full amber fill with near-black ink (warn-amber-ink) only for a server refusal under Take or Presets, and for a coalition warning when the majority flips.

### Neutral
- **Graphite Field** (#08090a): the darkest step, ink on white buttons.
- **Graphite Ground** (#0b0c0e): the page, the tab strip, the multiviewer row, toolbars, table heads and action bars.
- **Graphite Panel** (#0f1012): the monitor label strips and the floor plan.
- **Graphite Raised** (#17181b): the undo box and floating hints.
- **Graphite Divider** (#18191c): the line between table rows.
- **Surface / Surface Hover / Surface On** (white at 3.5%, 6.5%, 10%): the fill of buttons, inputs, keys, segments and tiles; hover; and a control that is on (active tab, filter, segment, Standpunt key, tablet toggle, selected row).
- **Edge / Edge Strong** (white at 7.5% and 13%): every 1px border; the strong one on hover and on a control that is on.
- **Paper Ink** (#eeeff1): primary text and icons.
- **Slate** (slate-mute, #9fa2ab): secondary text, labels, idle tabs and filters, key caps.
- **Tekst 3** (tekst-3, #8e919a): the third step: table heads, disabled text, placeholders, Preset numbers from 10.
- **Monitor Black**, **Logo White**, **Preview Canvas**, **Empty Seat**: only behind a monitor picture, under a supplied party logo, and inside the Preset sketches, which imitate the Renderer.

### Named Rules
**The Red Means Air Rule.** Solid red is only for Take and Live: the one action that sends something to air and the state of being on air. A wipe is red text, not a red block; Tegen in the Fractie keys is a red border, Verworpen is white text.

**The Tally Rule.** A monitor shows its state with a 3px ring in the tally colour (a 1px border plus a 2px ring, so the layout does not shift) and a filled label. Live always has the red tally, also with no Graphic, because the Zaal is still going to air.

**The Amber Is A Task Rule.** Amber appears only where the operator can and should act. "Op slot" is a normal safe state and stays neutral; "cijfers hh.mm" in the Preset list is calm grey, because the warning that matters sits on the monitor and in Take where the choice is made.

**The Ink Pair Rule.** Every state fill carries its own ink: white on Live Fill and Klaar Fill, near-black on amber. State colours used as text on dark ground take their Text variant.

## Typography

**Body Font:** Geist (variable, self-hosted through `@fontsource-variable/geist`), with system-ui as fallback. Base 13px, line height 1.45, letter spacing -0.006em, `tnum` and `ss01` on for the whole page.

**Character:** A neutral, precise grotesque, the type of modern product software. It reads small and clean in a dim room, and tabular figures keep counts and clocks still.

### Hierarchy
- **Display** (800, 56px, line height 1): the coalition count in the Coalitiebouwer.
- **Headline** (600, 15px, -0.012em): the title of the open Stemming, set or form.
- **Action** (600, 15px, -0.01em): the word Take.
- **Outcome** (600, 15px, -0.01em, sentence case): Aangenomen, Verworpen, Open or "Nu aangenomen".
- **Body** (400, 13px): default text and table cells.
- **Title** (500, 13px): the main line of a row (Graphic name, motie title).
- **Control** (500, 13px): tabs, filters, buttons.
- **Small** (400, 12px, Slate): second lines, hints, monitor detail text.
- **Label** (500, 12px, Slate, sentence case): field labels, the monitor label words; table heads in Tekst 3.
- **Stamp** (500, 12px): Live, Klaar, "was live hh.mm", each after a 7px dot.
- **Keycap** (500, 11px, Slate): the key in a light surface plate with a 1px edge and 4px corners.

### Named Rules
**The One Family Rule.** Geist sets every word in the dashboard, key caps and Preset numbers included. Weight (400, 500, 600) and the two grey steps make hierarchy, not size jumps.

**The Twelve Pixel Rule.** Everything you read during the broadcast is at least 12px: stamps, chips on the monitors, the "Cameraritten" heading, the tap order on Fractie keys, the 76 at the majority line, the old-figures line inside Take, the party names in the arc. Below it sit only key caps (11px), the count boxes on tabs and filters (11px) and, from the older Preset CSS, the stamps in the Preset list (11.5px).

**The Less Text Rule.** A control says what it does with an icon or one word; the tooltip says the rest, including why it is off. A sentence appears only where the picture is not enough: what an undo brings back, the reason next to a disabled commit button, the warning about current figures next to a live coalition.

## Layout

The shell is a CSS grid over the full viewport: tab strip (40px), an optional connection banner, the multiviewer row, and the work area. The page targets a laptop at 1440x900; the laptop is the measure, also on 1920 wide. The shell never scrolls; only the work area does. Below 900px wide the dashboard shows a short message with a link to the Presentator's tablet, and the shortcuts are off.

**Tab strip (40px, Graphite Ground, 1px edge below).** Text tabs from the left: Presets, Maken, Cijfers, Stemmingen, Coalitie, Camera, Vormgeving, Systeem, each 28px with 6px corners. The active tab has a Surface On fill and paper ink; idle tabs are Slate. Stemmingen shows small count boxes after its name: "2 open" in grey, and "1 verschil" in amber when the control after the broadcast found a difference. At the right: a 6px status dot with label for the server, the Live-renderer and the Reserve-renderer (green fine, amber needs a look, red broken), the status in words when one is not fine, and the clock (hh.mm.ss, 500). The Preview-renderer is listed under Systeem, not here.

**Connection banner.** When the server is not connected, a full-width red line with white text sits under the tabs ("Geen verbinding met de server..."). It is the one large red area, because without the server nothing goes to air.

**Multiviewer row (Graphite Ground, 12px padding, 12px gaps, 1px edge below).** Four columns, `1fr 156px 1fr minmax(340px, 25%)`, always visible in every section and never collapsible.
1. **Preview:** a monitor with label "Preview". Right side of the strip: "klaargezet door <Werkplek>" or "gewijzigd door <Werkplek>"; when the ready Graphic has older figures than the current ones, the amber chip-button "oude cijfers · Ververs N"; when what is ready is already on air, "staat in beeld". During Oefenen the label reads Oefenen in amber (or a red dot with the recording time), with "De gamepad stuurt alleen dit beeld, Live blijft staan" as detail. While the Regie tries a Standpunt or plays a Camerarit here, the line above the strip is dashed.
2. **Take column (156px):** see Take.
3. **Live:** a monitor with label "Live" and always the red tally; with no Graphic the name reads "Geen Graphic". Right side: "sinds hh.mm"; when Update can bring newer figures or a newer version, the amber chip is itself the button ("oude cijfers · Update B" or "nieuwere versie · Update B"); when Live has old figures and something else is ready, plain amber text "oude cijfers"; when nothing is ready, the chip-button "oude cijfers · zet opnieuw klaar". When the Live-renderer is not connected, the frame is dashed red and the right side reads "Live-renderer weg" on red.
4. **Camera:** see Camera Block.

**Work area.** A toolbar (Graphite Ground, 40px, 6px 12px padding, 1px edge below) with the section's filters as text toggles, buttons and a hint at the right; the section name lives only on the tab (and for screen readers). A section without filters gets no toolbar. Under it the section body fills the rest and scrolls.
- **Presets:** one list (see Preset Row), ending in a row "Nieuwe Graphic maken". Toolbar: "Alle n", one filter per group with its count, "Op de tablet n" with the tablet icon, and "Preview bewaren als Preset".
- **Stemmingen:** two panes, `minmax(300px, 36%) 1fr`. Left the motie list of the day as a table with a sticky head; right the input pane (see Stemming Tiles) with a fixed action bar at its foot, so "Vastleggen en klaarzetten" is always visible. A sub-view Controle compares each entered Stemming with the official result.
- **Coalitie:** the arc in the middle and a 340px side panel (see Coalition Arc).
- **Camera:** the Camera workbench in three columns (list, large floor plan, settings panel).
- **Maken, Cijfers, Vormgeving, Systeem:** a list plus a detail pane in the same quiet style. Vormgeving and Systeem are plain forms (they are not used during the broadcast); Systeem puts its data warnings at the top; the Opslaan bar of Vormgeving sticks to the foot.

**Spacing rhythm.** 4, 6, 8, 10, 12, 14, 16px. Controls sit 6 to 8px apart; detail panes pad 16px 18px with 16px gaps.

### Keyboard
The keyboard works anywhere outside a text field. Inside a field keys type, and Escape leaves the field. Combinations with Cmd, Ctrl or Alt are ignored, except Ctrl+Z (or Cmd+Z) for undo. A held key does not repeat.

| Key | Action |
| --- | --- |
| 1 to 9 | Put the Preset with that number ready in Preview. It does not go Live. |
| Space | Take: put what is in Preview Live. |
| Enter | Never takes, also not with focus on Take. It only presses another focused button. |
| Ctrl+Enter | In Stemmingen: Vastleggen en klaarzetten, also from the search field of Afwijkende stem. |
| N | Ververs: put the current figures in the ready Graphic. |
| B | Update: put a newer version or newer figures in Live without building up again. |
| X | Take out: take the Graphic out of Live. |
| Z | Kleur uit: the Stoelen in Live get their own leather back; the Graphic stays. |
| C | Opbouw over: during an Opbouw, the Partijen not yet lit colour at once. |
| Ctrl+Z | Undo the last Take out or wipe, while the undo box shows. |
| A, S, D, F, G, H | Fixed Standpunten: Totaal vanaf voorzitter, Laag over de banken, Linkerflank, Rechterflank, Bovenaf, Katheder. |
| Q, W, E, T, Y, U | Fixed Cameraritten. They start live at once. |
| J, K, L | Own Standpunten and Cameraritten. |
| Shift with a letter | Pick that Standpunt or Camerarit with a cut, without movement. |
| V | Vrije camera on or off (gamepad drives the Live camera). |
| O | Oefenen on or off: the gamepad drives only the Preview camera. |
| R | Start or stop recording a Camerarit, only during Oefenen. B on the gamepad does the same; B on the keyboard stays Update. |

A mouse click never gives a button focus, so Space after a click is always Take; a click on a button also ends typing in a field first, so its last value arrives before the click. When focus sits on a button through Tab, Space and Enter press that button and do not take. **While focus is in a text field, every key cap in the multiviewer disappears** (the letters go to the field), and Take shows an amber "typen" where "Spatie" stands; a click on Take still works. A changed Zetel count in Cijfers shows "Enter bewaart · Esc annuleert" under the field.

## Elevation & Depth

Mostly flat. Depth comes from the translucent surfaces and the 1px edges. Small shadows exist but stay low: buttons carry `0 1px 1px` black at 35%, Take `0 1px 2px` plus a faint lit top edge, a pressed segment `0 1px 1px`. Only floating things lift further: the undo box (`0 6px 18px` black at 45%) and the hint under a changed Zetel field (`0 4px 12px`).

### Motion
Buttons, tabs, filters and keys fade background, border and colour in 120ms ease-out. The bar in the undo box runs down over its seconds. On the tablet a tap gives at once a small press (scale 0.97, a little brighter) before the server answers. Nothing else moves.

## Shapes

Small, even corners: key caps and count boxes 4px, chips and the Preset number box 5px, buttons, inputs, tabs, filters and Standpunt keys 6px, segments and Fractie keys 7px, monitors, Take, the undo box, the floor plan and the Voorzitter sentence 8px, the Fractie key field 10px, the coalition arc 12px. Circles only for dots: status, stamps, the dot before a monitor label, a party dot, the check on a chosen arc party, and the pins on the floor plan.

A dashed border means "not tapped, but filled in or played on request": a Fractie key set by "Overige", a Camerarit key, the strip line while Preview plays a try-out, and the Live frame when the Live-renderer is gone.

## Components

### Buttons
- **Default:** 28px, 6px corners, Surface fill, 1px Edge, paper ink 500, 0 10px padding, Slate icon. Hover: Surface Hover and Edge Strong. On (aria-pressed): Surface On and Edge Strong.
- **Commit (put ready):** Klaar Fill with white ink, for every action that puts something ready (Zet klaar in Maken and Coalitie, Vastleggen en klaarzetten). 30px in the Stemmingen action bar.
- **Primary:** paper-ink fill with near-black ink, for confirming without putting ready (Opslaan, Verbinden).
- **Danger:** Live Red Text, for wipes (Wis alle stemmen, Wis coalitie); hover adds a faint red tint and border. A wipe that changes Live directly asks a second click: the button turns solid red with "Klik nog eens: wis uit Live", and offers undo afterwards.
- **Disabled:** stays visible: transparent, Edge border, Tekst 3 text, no shadow. A disabled commit or primary button looks the same. The reason sits next to it as Slate text after an amber dot ("Kies eerst Partijen"), or in the tooltip.
- **Lock:** a neutral stamp with the state (Op slot or Open) and next to it a button with the verb.

### Take
The Take column holds, top to bottom with 5px gaps:
- **Take:** a fixed 58px (it never grows, so the buttons under it never shift), full width, 8px corners, Live Fill with white ink, the word Take at the left and a "Spatie" key cap at the right. When the ready Graphic has old figures, a dark chip with light amber text "oude cijfers (N)" sits inside the button, under the finger. Without anything to take it turns grey (Surface, Tekst 3) with the reason "niets klaar" or "staat al live". While typing, "Spatie" becomes an amber "typen" chip.
- **Update (B), Take out (X), Kleur uit (Z), Opbouw over (C):** 28px buttons with a Lucide icon, one word and the key cap at the right. Each sits in a span that carries the tooltip, so a disabled button still says why.
- **Server refusal:** a full amber line with near-black text.
- **Undo box (see Undo).**

### Monitor (signature)
Preview and Live, as on a multiviewer.
- **Frame:** 8px corners, 1px Edge, the picture 16:9 on Monitor Black (at most 34% of the viewport height), centred, black fills the rest. Without a picture a short Slate line: "Wacht op beeld", "Geen beeld: de Preview-renderer is niet verbonden" or "Geen verbinding met de server".
- **Tally:** idle: no colour. Ready Preview: a 3px Klaar Green ring. Live: a 3px Live Red ring, always. Renderer gone: a dashed red frame.
- **Label strip (Graphite Panel, two lines):** line 1 the label (idle: a grey 7px dot and the word in Slate; ready or live: the word filled, white on Klaar Fill or Live Fill, 600, 4px corners), the Sjabloon name in paper ink 500, and at the right the state in 12px Slate or an amber chip. Line 2: what the Graphic shows, in full, in Slate.

### Undo
After Take out, Wis alle stemmen, Met algemene stemmen, Wis coalitie or a tap on a live coalition, an undo box shows for up to 20 seconds: Graphite Raised, 1px Edge Strong, 8px corners, a lifted shadow. It reads what happened in amber and what comes back in paper ink ("Take out: Zetelverdeling"), the seconds left in amber at the top right, a white primary button with the action ("Terug in beeld", "Zet stemmen terug", "Zet coalitie terug") and Ctrl+Z, and a 2px amber bar that runs down. The box sits at the foot of the Take column; for a live coalition it also sits in the Coalitie side panel, and on the tablet without the key cap. It disappears as soon as the Stand changed in between (another Graphic taken or put ready, by any Werkplek), so undo never overwrites a later choice. "Terug in beeld" puts exactly the same Graphic with the same figures back in Live and leaves Preview alone.

### Stamps
A dot and a word, no box: 12px 500, a 7px dot, 6px gap.
- **Live:** Live Red dot with a faint red halo, Live Red Text.
- **Klaar:** Klaar Green Text dot and word.
- **was live hh.mm:** grey dot, Slate, 400.
- **tussenstand, Op slot, Open:** grey, neutral.
On the tablet, Live and Klaar are filled (white on Live Fill or Klaar Fill), because the Presentator reads them under studio light.

### Preset Row
The default work area: a list without a head row, rows 44px high, Graphite Divider between rows. Columns: key (48px), status (96px), sketch (86px), Graphic, data, camera (250px), tablet (84px).
- **Key:** Presets 1 to 9 have a 22px key cap box with the number; from 10 the number is plain Tekst 3 text, because only 1 to 9 are shortcuts.
- **Status:** the Live, Klaar or "was live hh.mm" stamp.
- **Sketch:** a 66px 16:9 sketch of the Graphic, brightened so the shape (bar, arc, card) reads at a glance.
- **Graphic:** the Sjabloon name as a text-like button (so Presets from 10 can be put ready with Tab and Enter), and for a Titelkaart " · " and the heading in Slate.
- **Data:** one line with what the Graphic shows; when newer figures exist, " · cijfers hh.mm" in Slate, calm on purpose.
- **Camera:** the key cap and the Standpunt or Camerarit name over up to two lines in Slate, "Shot op de eigen Stoel" for a Persoonskaart, or amber "Standpunt bestaat niet meer".
- **Tablet:** a toggle with the tablet icon and Aan or Uit; on is neutral (Surface On), not coloured.
- **States:** hover Surface. Klaar: a 6% green tint plus the Klaar stamp. Live: a 6% red tint plus the Live stamp.
- **Click:** a click anywhere on the row puts the Preset ready. It never takes. The tablet toggle only toggles the tablet.

### Camera Block
The fourth column of the multiviewer.
- **Header:** "Camera" with a red dot and "Live" (these keys drive the Live camera), then the name of the current Standpunt or Camerarit, or "Vrije camera".
- **Floor plan:** the Zaal from above on Graphite Panel, 8px corners: the hemicycle, 150 Stoelen as dots in the colours of Live, round pins for Standpunten and square pins with a route for Cameraritten. A red marker shows the Live camera; during Oefenen or a try-out a green one shows the Preview camera. At its right Vrij (V), Oefenen (O, amber tint when on) and Opnemen (R, only during Oefenen, a red dot and "Stop" with the time while recording).
- **Standpunt keys:** three columns, 30px, Surface, 6px corners: the key cap, then the name over up to two lines. The key in use has Surface On, Edge Strong and a 2px Live Red line at its left.
- **Cameraritten:** under the heading "Cameraritten · direct live" (12px Slate), two columns of 29px keys with a dashed Edge Strong border and the duration ("16 s") at the right, so a Camerarit never reads as a Standpunt.
- With Vrij or Oefenen on, "Bewaar als Standpunt" appears under the keys.

### Camera bij Take
In every form in Maken (and per Sjabloon in Vormgeving), the field "Camera bij Take" is a segment of three options side by side: **Sjabloon** (the Standpunt of the Sjabloon, named under it in Slate, such as "A · Totaal vanaf voorzitter"), **Blijft staan** (Take changes only the Graphic and the Stoelen; the Live camera stays where it is, also free or during a ride) and **Kies**. Only Kies opens a select with the Standpunten and Cameraritten in two groups.

### Coalition Arc
- **Arc:** the Partijen on a flattened semicircle, each a 56px tile: Surface, 8px corners, a 3px line in the party colour at the top, the short name in 12px 600 paper ink, and the Zetels in 14px under it. Hover: Surface Hover. A chosen party gets Surface On, a 40% white edge, the party line at the top and the foot, and a small white circle with a check at its corner. No logos on the tiles.
- **Count:** in the centre the total in the display style, green text from 76, with "meerderheid" or the Zetels still needed below. Under the arc a seat bar with the coalition in party colours and "76" at the majority line.
- **Live:** when the coalition is in Live, a red frame sits round the arc with a line of a Live stamp and "Elke klik wijzigt de live coalitie". When the live coalition was built on older figures, the count itself turns amber with "in Live, met oudere cijfers".
- **Side panel (340px):** the heading with a Live or Klaar stamp, the Sjabloon as filters, the set, the chosen parties as a list with party dots, then the buttons. Not live: the green commit "Zet klaar · <set> · N Zetels". Live: a neutral "Kopie met cijfers van nu · N Zetels", with directly above it the amber-outlined line "Met de cijfers van nu: N Zetels", fully amber with ", wel een meerderheid" or ", geen meerderheid meer" when the majority flips. Then "Wis coalitie" (or "Wis coalitie live", two clicks) and the undo box.
- Below 560px wide (the Presentator's phone) the arc becomes a grid with one tap area per party.

### Stemming Tiles
The Fractie keys in the Stemmingen input pane.
- **Field:** a grid of 6 columns with 6px gaps inside a 10px-cornered frame whose 1px border is green or red after the side a click sets ("Een klik zet" Voor or Tegen).
- **Key:** Surface, 1px Edge, 7px corners. An 8px dot in the party colour before the short name (two lines at most, long names break with a hyphen), the Zetels in 13px 500, the tap order in 12px Tekst 3, and once set the word Voor or Tegen in 600.
- **Voor:** a 14% green tint, a green border and the word in Klaar Green Text. **Tegen:** no tint, a red border and the word in Live Red Text, so solid red stays for Live and Take. **Filled by Overige:** a dashed border.
- **Around it:** the outcome in the outcome style (Aangenomen green, Verworpen white, Open amber; while not committed "Nu aangenomen" or "Nu verworpen" in white with a grey "tussenstand" stamp), "n voor · n tegen · n open", a seat bar (Voor green, open empty, Tegen red, majority marker), the Voorzitter sentence with a microphone icon and the Fractie names tinted by their vote, and the fixed action bar with Wis alle stemmen, Overige, Met algemene stemmen, Afwijkende stem, Volgende and the commit button.

### Party Dot and Logo Tile
In lists (Cijfers rows, the coalition list) a Partij is an 8px dot in its party colour next to its short name, not a chip with tiny text. A supplied logo still sits on a white tile; the prototype has no supplied logos.

### Thumbnails
Preview and Live show real small pictures from the Renderers. The Preset sketches are 320x180 SVG sketches of the Renderer output on the Preview Canvas. Both show the broadcast, not the dashboard, so dashboard tokens do not apply inside them.

### Icons
Lucide 0.460.0, stroke 2, round caps and joins, inlined as SVG in currentColor, 14px by default. Used in the Take column (Update, Take out, Kleur uit, Opbouw over), the search box, the lock button, the microphone of the Voorzitter sentence, the tablet toggle and filter, the star of a motie, plus and check. Tabs, Take and table rows carry no icons.

## Do's and Don'ts

### Do:
- **Do** keep the tab strip and the multiviewer row (Preview, Take, Live, camera) on screen in every section; only the work area scrolls.
- **Do** keep solid red for Take and the Live label, and show tally as a 3px ring plus a filled label.
- **Do** make every "put ready" button green and every confirm-without-ready button white.
- **Do** use amber only where the operator should act, and put it where the choice is made (the monitor, Take), not on every row.
- **Do** show a control that is on as one surface step lighter with a stronger edge.
- **Do** give every Preset 1 to 9 a key cap and every Standpunt a letter, and print shortcuts on their controls; hide them while typing.
- **Do** let a digit put a Preset ready and let only Space or the Take button send it Live.
- **Do** keep "Vastleggen en klaarzetten" in a fixed action bar, always visible.
- **Do** offer undo after an action that takes something away, instead of asking first; only a wipe straight in Live asks a second click.
- **Do** keep disabled actions visible and give the reason next to them or in the tooltip.
- **Do** keep everything read during the broadcast at 12px or more.
- **Do** use the UI words Take, Live, Preview, Update, Take out, Live-renderer, Reserve-renderer and Preview-renderer, and the GLOSSARY words for everything else (Preset, Standpunt, Fractie, Zetel, Stoel, Zaal).

### Don't:
- **Don't** let Enter take a Graphic.
- **Don't** fill large areas with colour: no coloured navigation, no red or green cards, no coloured section headers.
- **Don't** use red or green for selection, hover, toggles or decoration; the tablet toggle and "Op slot" stay neutral.
- **Don't** use uppercase labels, coloured stripes at the left of rows, or a sentence of explanation under every form.
- **Don't** let Take change height: its content changes, its size does not.
- **Don't** add icons to tabs, Take or table rows, and never a unicode glyph or emoji as an icon.
- **Don't** put party logos or tiny text on colour in lists; use the party dot.
- **Don't** bring back devices of the earlier designs: no warm black, no Barlow, no lit warm-white keys, no UMD strips in tally colour, no pictogram rail.
