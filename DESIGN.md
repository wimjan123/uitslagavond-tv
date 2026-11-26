# DESIGN.md — uitslagavond-tv (DaVinci Deck)

Ontwerpsysteem van het Regie-dashboard en de Presentator-tablet (`/tablet`) van `uitslagavond-tv`.
Gekozen en opnieuw vanaf nul opgebouwd als **DaVinci Deck** (vervangt de oudere "Studio" / "Omroepsoftware" / "Mengpaneel" CSS-lagen).

---

## 1. Kernprincipes

1. **Drie onafhankelijke, samen te stellen lagen (`Camera · Stoelbeeld · Graphic`)** (zie `docs/adr/0005-drie-lagen-camera-stoelen-graphic.md`):
   - **Camera:** Standpunt, Camerarit of Vrije camera. Een klaargezette Camerarit wacht op zijn beginpunt in Preview (met `▶ Test rit`) en start pas op Live bij `Take` (`Spatie`).
   - **Stoelbeeld:** Losse stoelkleuring (zonder 2D-Graphic) uit 5 bronnen (`Partij(en)`, `Hele set`, `Coalitie`, `Stemming`, `Losse stoelen`) met directe `Kleurovergang` en `Kleurkracht`.
   - **Graphic:** 2D-overlay die met `Camera blijft staan` over een lopende Camerarit of actief Stoelbeeld heen start zonder die te onderbreken.
2. **OBS Studio Mode-transport & direct zichtbare status:**
   - Edge-to-edge twin **720p Preview & Live monitoren** (`1280×720`) met versleepbare hoogte, 1-klik vergroten en los pop-out venster (`/monitors`).
   - Direct onder elke monitor staan 3 statuschips (`Camera · Stoelen · Graphic`) zodat zonder uitlegteksten direct helder is wat er in Preview klaarstaat en wat er Live op zender staat.
   - Eén horizontale transportbalk: **`Beweeg | Fade (duur) | Knip`** · **`TAKE (Spatie)`** · **`Take out (X)`** · **`Kleur uit (Z)`** · **`▾ Meer`** · 20s Undo (`Ctrl+Z`) · klikbare statuspil (`Server · Live · Res · Klok` → Systeem-lade).
3. **Maken direct in Regie + intuïtieve Studio-editor:**
   - Geen losse `Maken`-tab: in `Regie` staat de **Media Pool (Presets)** links (visuele 16:9 kaarten + 1-klik lijst, drag-and-drop herordenen voor `1–9`, schone contextmenu's) direct naast de **Inspector** rechts (met privé-**Conceptbeeld** over het echte 3D-Zaalbeeld).
   - In `Studio` klappen de monitoren in tot een smalle `38px` rode `LIVE`-balk (met `Take out`-noodrem) voor volledige werkhoogte in de drag-and-snap **Sjabloon-editor** (over echt 3D-Zaalbeeld, Title-safe, magnetisch uitlijnen, anker-presets, `▶ Test animatie`, `Ctrl+Z`/`Ctrl+Shift+Z`, exacte pixels standaard ingeklapt).
4. **Rustig en gecontroleerd uitklapbaar:**
   - Maximaal **1 niveau** uitklapbaarheid (`details.deck-fold` of één dropdown).
   - Geen muren van uitlegtekst, geen dubbele rijen actieknoppen op kaarten.
5. **Live-veiligheid:**
   - `Enter` triggert **nooit** `Take`.
   - Knopklikken stelen geen focus van `Spatie = Take`.
   - Toetslabels vervagen tijdens het typen (`body.typt`).
   - Camera-paneel start standaard veilig in `Preview`; `Direct live` is een bewuste rode schakelaar.

---

## 2. Kleuren & Oppervlakken (`dashboard/src/deck.css`)

Dark-only warm-neutraal antraciet DaVinci-studiodeck:

| Token | Waarde | Gebruik |
| --- | --- | --- |
| `--bg-well` | `#17181b` | Monitor-bak, canvas-achtergrond, paginabalk |
| `--bg-deck` | `#222327` | Hoofdachtergrond werkruimtes |
| `--bg-surface` | `#282a30` | Transportbalk, Inspector, zijbalken, kaarten |
| `--bg-tile` | `#2b2d33` | Preset-tegels, knoppen, cameratezels |
| `--bg-input` | `#1b1c20` | Invoervelden, segmented control-spoor |
| `--hairline` | `rgba(255,255,255,0.085)` | Subtiele randen tussen panelen |
| `--text` | `#ebedf2` | Primaire tekst |
| `--text-2` | `#a8adba` | Labels, secundaire meta |
| `--text-3` | `#838998` | Korte meta, placeholders (`>= 4.5:1`) |
| `--amber` | `#f0b44c` | Focus, actieve tab, selectiekader, bewerk-accent |
| `--preview` / `--preview-bright` | `#1d8051` / `#2ea66d` | Preview-tally (groen), `Zet klaar` |
| `--live` / `--live-bright` | `#c9373d` / `#e5484d` | Live-tally (rood), `TAKE`, `Direct live`, opname |

---

## 3. Typografie & Iconen

- **UI-lettertype:** Self-hosted `@fontsource-variable/archivo` (`Archivo Variable`).
- **Monospace / cijfers:** Self-hosted `@fontsource-variable/jetbrains-mono` (`JetBrains Mono Variable`) met `font-variant-numeric: tabular-nums` voor klok, zetels, tijden, sneltoetsen en coördinaten.
- **Iconen:** `lucide-react` (strakke 12–15px SVG-lijnen).

---

## 4. Paginastructuur (DaVinci onderste paginabalk)

`Regie` · `Zaal & Camera` · `Stemmingen` · `Coalitie` · `Cijfers` · `Studio`
- **Systeem** opent als slide-over lade vanuit de klikbare statuspil rechts in de transportbalk.
- **`/tablet`** hergebruikt exact dezelfde DaVinci Deck-tokens met de T2 2-tabindeling (`Presets` + `Coalitiebouwer`).
- **`/monitors`** opent de twin 720p Preview & Live monitoren in een los venster voor een tweede scherm.

---

## 5. Sneltoetsen (buiten invoervelden)

| Toets | Actie |
| --- | --- |
| `Spatie` | `TAKE` (zet klaargezette Graphic / Stoelbeeld / Camera live) |
| `X` | `Take out` (alleen 2D-Graphic uit Live; 20s Undo) |
| `Z` | `Kleur uit` (alleen Stoelen terug naar blauw leer) |
| `B` | `Update` (live Graphic vloeiend bijwerken) |
| `C` | `Opbouw overslaan` |
| `N` | `Cijfers verversen` in Preview |
| `1–9` | Preset 1 t/m 9 klaarzetten in Preview |
| `A–H`, `J–L`, `Q–U` | Standpunt of Camerarit (`Shift` = knip) |
| `V` / `O` / `R` | Vrije camera / Oefenen / Opnemen |
| `Ctrl+Z` | Laatste wis- of Take out-actie terugdraaien (in Studio: canvas-undo) |
