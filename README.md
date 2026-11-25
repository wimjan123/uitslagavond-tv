<img src="assets/icon.svg" width="64" height="64" alt="">

# uitslagavond-tv

Live tv-graphics van de Tweede Kamer. Woorden en keuzes: [GLOSSARY.md](GLOSSARY.md), [PRODUCT.md](PRODUCT.md), [docs/adr/](docs/adr/).

## Onderdelen

| Map | Wat |
| --- | --- |
| `crates/gedeeld` | De Stand, de Opdrachten en de berichten. De TypeScript-typen van het dashboard komen hieruit. Met feature `volger`: de client die de Stand volgt. |
| `crates/server` | De server (`uitslagavond-server`): de enige bron van de Stand. WebSocket op `/ws`, HTTP op `/api/opdracht` en `/api/stand`, het dashboard op `/`. |
| `crates/renderer` | De Renderer (`uitslagavond-renderer`, Bevy 0.20.1): tekent één beeld met een rol (Ether, Reserve of Voorbeeld). |
| `uitrol/` | Installeren op de studiomachines: pakket, opstartbestanden voor Ubuntu 26.04 en de gamepad. Zie [uitrol/INSTALLEREN.md](uitrol/INSTALLEREN.md). |
| `zaal/` | De exportstap van de Zaal: model van de Tweede Kamerzaal (niet openbaar), licht gebakken in Blender. Zie [zaal/README.md](zaal/README.md). |
| `dashboard` | Het dashboard van de Regie (TypeScript en React), uitgeleverd door de server. `src/typen` is gegenereerd. |

## Bouwen

Nodig: Rust (stable), Node 24, en op Linux `libwayland-dev` en `libxkbcommon-dev` voor de Renderer en `libudev-dev` voor de gamepad van de server.

```sh
cargo build -p server -p renderer
(cd dashboard && npm ci && npm run build)
```

Verander je een type in `crates/gedeeld`, draai dan `cargo test -p gedeeld`. Dat schrijft de TypeScript-typen opnieuw naar `dashboard/src/typen`; commit ze mee (CI controleert dat ze kloppen).

## Draaien

```sh
export UITSLAGAVOND_SLEUTEL='een-lange-gedeelde-sleutel'
target/debug/uitslagavond-server --map data            # luistert op 0.0.0.0:7400
ZAAL_MODEL=/pad/naar/model zaal/maak.sh /pad/naar/zaal  # eenmalig, zie zaal/README.md
target/debug/uitslagavond-renderer --rol ether --server 127.0.0.1:7400 --zaal /pad/naar/zaal
target/debug/uitslagavond-renderer --rol voorbeeld --server 127.0.0.1:7400 --zaal /pad/naar/zaal   # op de regie-pc
```

- Zonder `--zaal` (of `UITSLAGAVOND_ZAAL`) tekent de Renderer alleen de graphics op een effen achtergrond.
- Open `http://<server>:7400/` en kies bij de eerste keer een naam voor de Werkplek (zonder inlog) en de sleutel. Onder Maken zet je een Titelkaart klaar; spatie of TOON zet hem in de Ether, B werkt bij, X haalt uit beeld, Enter toont nooit iets.
- De Renderer met rol Voorbeeld draait zonder venster en stuurt kleine beelden van het Voorbeeld en de Ether naar het dashboard (de multiviewer): 640 bij 360, tot 50 per seconde, en alleen als het beeld verandert. Een bewegend beeld kost ongeveer 3 MB per seconde per beeld en per Werkplek.
- Zonder sleutel start de server niet, en zonder de juiste sleutel weigert hij elke verbinding (401).
- De server bewaart de Stand in `data/stand.json`, elke Opdracht met tijd en afzender in `data/logboek.jsonl` en de portretfoto's in `data/fotos/`.
- Een knop of Stream Deck stuurt dezelfde Opdrachten over HTTP:

  ```sh
  curl -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" -H "X-Afzender: Stream Deck" \
       -H "Content-Type: application/json" -d '{"soort":"Toon"}' http://127.0.0.1:7400/api/opdracht
  ```

  Andere Opdrachten: `ZetKlaar` (met een `graphic`, zoals `{"sjabloon":"Titelkaart","kop":"...","regel":"..."}`), `WijzigKlaar`, `Bijwerken` en `UitBeeld`. Stuur bij `Toon` de `versie` mee die de knop liet zien; is de klaargezette Graphic intussen gewijzigd, dan weigert de server met de naam van wie hem wijzigde.

  Voor de Stream Deck met Bitfocus Companion: de knoppen voor Presets, Standpunten en de Vrije camera, en de status op de knoppen (`GET /api/knoppen`) staan in [docs/stream-deck.md](docs/stream-deck.md).

- Graphics bestaan uit Blokken (Vlak, Tekst, Getal, Zetelbalk, Partijlogo, Foto Kamerlid, Lijn en Programmalogo), elk met een eigen animatie in en uit (ADR 0003). Een Sjabloon is data: zie `crates/gedeeld/src/sjabloon.rs`; de vaste Sjablonen, zoals de Titelkaart, staan daar ook. Kleuren, lettertypes, programmalogo en snelheid komen uit de Stijl in de Stand.
- De standaard-Stijl heet Grafiet: vijf kleuren, het lettertype Nunito Sans (ingebouwd in de Renderer) en een Beweging met de tijden van de animaties, zoals Openen vanuit lijn. Kleuren, maten, de Beweging en per Sjabloon de Blokken staan in [docs/vormgeving.md](docs/vormgeving.md). Een omroep zet zijn eigen vormgeving in de Stijl en de Sjablonen, met het dashboard of met de Opdrachten `ZetStijl` en `ZetSjabloon`.
- **Programmalogo**: leg het logo (wit op doorzichtig, PNG) neer als `data/bestanden/programmalogo.png`. De Renderers halen het op en tonen het rechtsboven in de Graphics. Het logo zit niet in de software; zonder dat bestand staat er "Uitslagavond" in de kopletter.
- **Vormgeving > Stijl** in het dashboard: de vijf kleuren, de drie lettertypes (ingebouwd of een eigen TTF of OTF), het programmalogo (een eigen PNG of JPEG, of alleen tekst) en de snelheid. De Beweging (de tijden en maten van de animaties) staat ook in de Stijl en verander je met `ZetStijl`. Een wijziging geldt na **Opslaan**: het Voorbeeld toont hem meteen, de Graphic in de Ether houdt haar vorm tot de volgende Toon. De klaargezette Graphic telt dan als net veranderd, dus een Toon van een ander scherm dat nog de oude vorm liet zien, weigert. **Standaard-Stijl** zet de standaard-Stijl terug in het formulier. Een geplaatst logo of lettertype krijgt een eigen naam bij zijn inhoud in `data/bestanden/` (`POST /api/bestand`), zodat een Renderer nooit een oude kopie toont. Een `stand.json` met de eerste standaardstijl (van vóór 22 november 2026) krijgt bij het starten de huidige standaard; een zelf aangepaste Stijl blijft.
- **Vormgeving > Sjablonen** in het dashboard: één blokkeneditor voor vaste en eigen Sjablonen. Per Blok plek, grootte, kleur, lettertype en animatie in en uit met "start na"; per Sjabloon de achtergrond, wat de Stoelen doen, het standaard Standpunt en de snelheid. Een vast Sjabloon heeft **Terug naar standaard**; komt een nieuwe versie met een andere standaard, dan blijft de aanpassing en meldt de tab dat. Een eigen Sjabloon zet je klaar onder Maken. Net als bij de Stijl geldt een wijziging na Opslaan en in de Ether pas bij de volgende Toon. Zie [docs/vormgeving.md](docs/vormgeving.md#sjablonen-bewerken).
- Lettertypes, logo's en foto's legt de omroep in `data/bestanden/` (bijvoorbeeld `data/bestanden/logos/pvv.png`; de Stijl en het invoerbestand noemen ze met dat pad). De Renderers halen ze bij de server en bewaren een kopie in `~/.cache/uitslagavond-tv/bestanden` (andere map: `--kopie`). Ontbreekt een partijlogo, dan staat de korte naam op die plek.
- Renderer zonder venster: `--zonder-venster --beeld ether.png` slaat het beeld op zodra de Stand binnen is. `scripts/renderer-beeld.sh` doet de hele keten (server, Opdracht, Renderer) en werkt ook met software-Vulkan zonder GPU.
- De Stoelen krijgen hun kleur met de Opdracht `ZetStoelkleuren`: 150 hexcodes in de volgorde vak, rij, nummer (zie `Stoel::alle` in `crates/gedeeld`). Een lege lijst geeft ze hun eigen blauwe leer terug.
- **Kleur uit** (knop onder Uit beeld, toets Z, Opdracht `KleurUit`): de Stoelen in de Ether krijgen direct hun eigen blauwe leer terug; de Graphic blijft staan. De volgende Graphic die de Stoelen kleurt, zet de kleuren weer aan. Een Sjabloon kan dat ook zelf: Stoelen "Uit".
- **Kleurovergang**: de Stoelen wisselen van kleur als **Golf** van links naar rechts (standaard 1 seconde) of door **Vervagen**, alle tegelijk (standaard 1,5 seconde). Je kiest hem onder Vormgeving > Stijl; een Sjabloon kan een eigen Kleurovergang hebben. De server zet bij elke nieuwe kleur welke overgang geldt (`Stand::kleurovergang`), zodat Ether en Reserve hetzelfde doen.
- **Kleurkracht** (Vormgeving > Stijl, 0 tot 100): hoe hoger, hoe voller de kleur van de Stoelen en hoe donkerder de rest van de Zaal zolang er Stoelen gekleurd zijn. Bij 100 houdt de Zaal een tiende van zijn licht over, bij 50 ongeveer een derde; het licht gaat mee met de Kleurovergang.
- Tijd per beeld meten: `scripts/meet-zaal.sh /pad/naar/zaal` (release-build, standaard 600 seconden).
- Dashboard tijdens het ontwikkelen: `npm run dev` in `dashboard/` stuurt `/ws` en `/api` door naar een server op `127.0.0.1:7400`.

## Reserve en storingen

- Ether en Reserve verbinden met dezelfde server en starten hun animaties zodra de Stand binnenkomt; een verschil van een of twee beelden is normaal. Valt de Ether uit, dan schakel je op de **mixer** over naar de Reserve.
- **Versie**: elk programma heeft de versie uit `Cargo.toml` plus de commit, zoals `0.1.0+4a46080` (`uitslagavond-server --version`). De server weigert een Renderer met een andere versie; onder Systeem zie je welke machine en welke versie dat is. Installeer dan op die machine hetzelfde pakket; de Renderer probeert het elke seconde opnieuw en verbindt daarna vanzelf.
- **Server weg**: de Renderers houden het laatste beeld vast en verbinden elke seconde opnieuw. Het dashboard toont een rode balk en de status van de Renderers is dan onbekend.
- **Renderer herstart** (of start halverwege de uitzending): hij krijgt meteen de hele Stand en toont het eindbeeld, zonder animatie, Kleurovergang of camera-overgang.
- **Server herstart**: hij leest de Stand uit `data/stand.json`, dus er verandert niets in beeld.

## Cijfers: het invoerbestand

Start de server met `--invoer <pad>` en hij leest dat JSON-bestand in zodra het verandert (binnen een halve seconde). Het dashboard meldt wat binnenkwam en welke sets zijn overgeslagen, met de reden.

```json
{
  "verkiezing": { "naam": "Tweede Kamer 2025", "datum": "2025-10-29" },
  "partijen": [
    { "sleutel": "pro", "naam": "PRO", "kort": "PRO", "kleur": "#C8102E", "logo": "logos/pro.png", "voorgangers": ["gl-pvda"] },
    { "sleutel": "groep-x", "naam": "Groep X", "kort": "X", "fractie_zonder_lijst": true }
  ],
  "sets": [
    { "id": "exit", "soort": "exitpoll", "label": "Exitpoll 21:00", "tijd": "21:00", "bron": "Ipsos I&O",
      "zetels": { "pro": 22, "overig": 128 } }
  ],
  "vergelijk_met": "tk2025"
}
```

(Voorbeeldgegevens.) Regels:

- `soort` is `peiling`, `exitpoll`, `prognose`, `tussenstand` of `uitslag`. Een Partij die in `zetels` ontbreekt heeft nul Zetels; `overig` telt mee voor wat de set niet apart noemt.
- Partijen en sets met een bekende `sleutel` of `id` worden bijgewerkt, nieuwe komen erbij. Wat niet in het bestand staat, blijft staan.
- Een set met een Partij die niet in de partijlijst staat, of een fout veld, wordt overgeslagen. Een set die niet op 150 Zetels uitkomt komt wel binnen, met een waarschuwing.
- Een vergrendelde set (Opdracht `VergrendelSet`) blijft gelijk tot `OntgrendelSet`, ook na een herstart.
- Zonder geldige `kleur` (`#RRGGBB`) krijgt een Partij een tijdelijke kleur; zonder `logo` staat de korte naam op die plek.
- Winst en verlies rekent het systeem uit tegen `vergelijk_met`, met de som van de Voorgangers. Een Voorganger hoort bij hooguit één Partij.

`cijfers/uitslag-tk2025.json` is de uitslag van 29 oktober 2025 (Kiesraad, open data, CC0), zonder kleuren en logo's: die levert de omroep. Opnieuw maken uit "Verkiezingsuitslag Tweede Kamer 2025 (Deel 1)" op [data.overheid.nl](https://data.overheid.nl/dataset/verkiezingsuitslag-tweede-kamer-2025):

```sh
python3 -I scripts/kiesraad-naar-invoer.py Resultaat_TK2025.eml.xml Totaaltelling_TK2025.eml.xml > cijfers/uitslag-tk2025.json
```

## Kamerleden uit de open data

De server haalt de Kamer die zit uit de [OData API van de Tweede Kamer](https://opendata.tweedekamer.nl/documentatie/odata-api) (CC0): bij het starten en daarna elke 10 minuten (`--open-data-elke <minuten>`, uitzetten met `--zonder-open-data`). Hij bewaart alles lokaal, dus tijdens de uitzending is de API niet nodig.

- Een nieuw Kamerlid telt mee vanaf de dag van de beëdiging (`Van` van de `FractieZetelPersoon`). Een Graphic in beeld verandert niet vanzelf: die heeft zijn eigen gegevens.
- Het dashboard meldt wissels (wie wie opvolgt op dezelfde FractieZetel), Fracties die niet in de partijlijst stonden en Kamerleden zonder foto.
- Een Fractie die niet in de partijlijst staat, komt erbij als Fractie zonder lijst, met een tijdelijke kleur en de korte naam op de plek van het logo. Komt de Partij later in het invoerbestand, dan hoort de Fractie daarna bij die Partij.
- Foto's (ongeveer 210 MB voor 149 Kamerleden) staan in `<map>/fotos/open-data/`. Een eigen foto van de omroep zet je met `PUT /api/foto/<id>` (JPEG of PNG als body) en haal je weg met `DELETE`; die staat in `<map>/fotos/eigen/` en gaat voor. `GET /api/foto/<id>` geeft de foto die geldt. De rechten op foto's liggen bij de omroep.
- Zonder foto toont de Persoonskaart een vlak in de Fractiekleur met de voorletters (`Kamerlid::voorletters`, zoals "DG" voor Dieke van Groningen).
- Van elk Kamerlid komt de hele geschiedenis van `FractieZetelPersoon` mee. De vroegste `Van` is de eerste dag in de Kamer, ook als die in een eerdere Kamerperiode lag.

## Persoonskaart

Bij **Maken** kies je de Persoonskaart: zoek een Kamerlid op naam of Partij, of klik op zijn Stoel in de plattegrond bovenin (dat kan zolang het formulier open is). De kaart toont naam, foto en Partij, de functie alleen bij een fractievoorzitter, een vrije regel die leeg mag blijven en met een vinkje leeftijd, woonplaats, geboorteplaats en het jaar van de eerste dag in de Kamer. **Maak standaard** zet de vinkjes voor een nieuwe kaart; dat verhuist later naar de vormgevingstab.

- **De Stoel licht op** en de camera gaat naar een shot dat de server per Stoel uitrekent: schuin van voren, met de Stoel links in beeld (de kaart staat rechts), binnen de onzichtbare ruimte. De overgang is die van een Standpunt (2 seconden, zachte start en stop); een eigen Standpunt voor de Graphic gaat voor.
- **Zonder Stoel** licht niets op en gaat de camera naar het Standpunt van het Sjabloon (S, Laag over de banken). Bij een Kamerlid zonder Stoel en bij een persoon zonder foto waarschuwt het formulier.
- **Eigen personen**: iemand die geen Kamerlid is, zoals een lijsttrekker, met naam, Partij, vrije regel, geboortedatum, woonplaats en geboorteplaats. Ze blijven bewaard in de Stand, dus ook in een Preset. Een eigen foto (van een Kamerlid of een eigen persoon) plaats je met **Eigen foto** in het formulier; die van een eigen persoon staat in `<map>/fotos/personen/`.
- De Graphic houdt de gegevens van het moment van klaarzetten, ook de Stoel. Een Renderer haalt een foto op als `GET /api/bestand/fotos/<open-data|eigen|personen>/<id>`.

## Controle na de uitzending

Na de Stemmingen haalt de server zelf de officiële uitslag op van elke Stemming op de lijst van de dag, en zet die naast wat de Regie invoerde (Stemmingen, knop **Controle**):

- **Debat Direct** (`--debat-direct <adres>`): elke minuut, tot elke Stemming een uitslag heeft. De Kamer laat die daar ongeveer 2 uur na de Stemming zien. Het is de interne API van de app van de Kamer, zonder documentatie; hij kan zonder aankondiging veranderen.
- **De open data** (`Stemming`-regels): elke 10 minuten (`--open-data-elke`), ongeveer 18 uur na de Stemming. Die gaan voor Debat Direct.
- Het kijkt tot twee dagen na de dag van de lijst. Een amendement of een motie die de Regie met de hand typte, komt pas met de open data (of nooit) binnen.

Per Stemming: **Uitkomst anders**, **Verschil per Fractie**, **Wacht op officieel** of **Gelijk**, de ernstigste bovenaan en met de tijd als de Stemming in de Ether was. De server verandert de invoer nooit zelf. Bij een verschil kiest de Regie **Neem officieel over** of **Onze invoer klopt**; een Preset en de Graphic in de Ether houden hun eigen gegevens.

De vertraging van Debat Direct meten: de server schrijft de eerste keer dat Debat Direct een uitslag heeft in zijn log hoeveel minuten na de hamerslag dat was (`Debat Direct heeft de uitslag van <nummer>: <n> minuten na de hamerslag (<hamerslag>)`). Dezelfde tijden staan per Stemming in `stand.json` (`controle.debat_direct_gezien` en `controle.officieel.hamerslag`) en in de tabel.

## Zaalindeling: Kamerleden op hun Stoel

Start de server met `--zaalindeling zaalindeling/tk2025.json`. Dat bestand zegt per Stoel (vak, rij, nummer) welk Kamerlid er zit, met het id uit de open data; de naam staat er alleen bij om het te kunnen lezen. De Fractie volgt uit het Kamerlid, dus na een overstap kleurt de Stoel vanzelf naar de nieuwe Fractie.

- `tk2025.json` is overgezet uit de PDF [Zetelverdeling plenaire zaal, november 2025](https://www.tweedekamer.nl/sites/default/files/2025-11/202511%20Zetelverdeling%20plenaire%20zaal.pdf) met `zaalindeling/overzetten.py`, met de Kamerleden van 22 november 2026. De PDF toont alleen blokken per Fractie, geen namen: binnen een blok staat de fractievoorzitter voorop en de rest op volgorde van de FractieZetel. Een afgesplitste Fractie (Groep Markuszower, Lid Keijzer) zit achterin het blok waar ze uit kwam.
- Stoelen verschuiven: pas het bestand aan. De server leest het binnen een halve seconde in. De controle (ontbrekende of dubbele Stoelen, een Kamerlid op twee Stoelen) staat in het dashboard bij Systeem, onder de Kamer.
- Bij een wissel stelt het dashboard voor dat het nieuwe Kamerlid de Stoel van de vertrokken persoon krijgt (eerst op dezelfde FractieZetel, anders in dezelfde Fractie). **Geef Stoel** bevestigt dat, en de server schrijft het terug in het bestand. Tot dan heeft het nieuwe Kamerlid geen Stoel.
- De Zetelverdeling in de Zaal kiest bij Maken de Uitslagindeling (een set, per Partij één blok) of de Zaalindeling (de Kamer die zit, elk Kamerlid op zijn eigen Stoel).

## Camera: Vrije camera, eigen Standpunten en Cameraritten

De server rekent de camera uit (ADR 0002) en leest zelf de gamepad: een Xbox-controller met snoer aan de pc waar de server draait. Zonder controller start de server gewoon (in het log staat dan "nog geen gamepad aangesloten"); een controller die je later insteekt, telt meteen mee.

| Gamepad | Wat |
| --- | --- |
| Linker stick | Vooruit, achteruit en opzij (horizontaal, in de kijkrichting) |
| Rechter stick | Rondkijken |
| Rechter trigger / linker trigger | Omhoog / omlaag |
| Rechter bumper / linker bumper | Inzoomen / uitzoomen |
| A | Langzaam of snel |
| B | Opname van een Camerarit starten of stoppen, net als R (alleen tijdens Oefenen) |
| Start | Vrije camera aan of uit, net als V op het dashboard |

- De stick doet alleen iets met de Vrije camera aan. Na loslaten glijdt de camera in een halve seconde tot stilstand; hij kantelt nooit opzij en kijkt hooguit 85 graden omhoog of omlaag.
- De camera blijft in de onzichtbare ruimte (`zaal/ruimte.py`, zie [zaal/README.md](zaal/README.md)). Bij de rand remt hij af in de richting van de rand en glijdt hij erlangs.
- Een Standpunt kiezen (een letter, een pin, of Toon met een Graphic) zet de Vrije camera uit en beweegt de camera vanaf waar hij is.
- **Bewaar als Standpunt** (in het camerablok of onder Camera, met de Vrije camera aan) bewaart waar de camera nu staat. Eigen Standpunten hebben geen maximum; de eerste drie krijgen J, K en L, de rest kies je in de lijst onder Camera of als pin op de plattegrond. Onder Camera geef je een naam, verplaats je een toets, vervang je de plek door de camera van nu, en wis je.
- Wissen kan niet zolang iets het Standpunt gebruikt: de klaargezette Graphic, de Graphic in de Ether of de camera van de Ether. Het dashboard zegt dan waar. Sjablonen en Presets komen erbij in `Stand::gebruik_van_standpunt` als ze een Standpunt kunnen kiezen.
- Dezelfde Opdrachten over HTTP: `ZetVrijeCamera`, `BewaarStandpunt` (zonder `stand` neemt de server de camera van nu), `OverschrijfStandpunt`, `HernoemStandpunt`, `ZetStandpuntToets` en `WisStandpunt`.
- **Oefenen** (O, of de knop naast Vrij): de gamepad stuurt alleen de camera van het Voorbeeld; de Ether blijft staan. Oefenen en Vrij gaan niet samen. Het Voorbeeld krijgt een amber lijn en het label Oefenen.
- **Opnemen** (R, of B op de gamepad) kan alleen tijdens Oefenen. De server bewaart 100 standen per seconde van het Voorbeeld, tot 60 seconden. Na Stop staat de Camerarit op de eerste vrije letter van J, K en L (gedeeld met de eigen Standpunten) en speelt hij af in het Voorbeeld: eerst de gewone overgang vanaf de Ether naar het beginpunt, dan de rit.
- Een Camerarit kies je overal waar je een Standpunt kiest: met zijn letter, als vierkante pin op de plattegrond, onder Maken of in een Preset. Onder Camera schaaf je hem bij met schuifjes (gladheid, duur, begin eraf, eind eraf, zacht starten en stoppen) en kies je het begin (eerst overgang of knippen) en het eind (stoppen of herhalen; herhalen kan alleen als begin en eind bijna gelijk zijn). Elke wijziging speelt meteen af in het Voorbeeld. Onder Camera stel je ook de overgang naar een Standpunt in (duur, zacht starten en stoppen, bewegen of knippen, stil of langzaam schuiven).
- Opdrachten over HTTP: `ZetOefenen`, `ZetOpnemen` (bij stoppen zonder `opname` vult de server in wat hij opnam), `ProbeerInVoorbeeld`, `ZetStandpuntOvergang` en `BijschaafCamerarit`. `HernoemStandpunt`, `ZetStandpuntToets`, `WisStandpunt` en `KiesStandpunt` werken ook met de sleutel van een Camerarit (`R1`, `R2`, ...).

### Testen met een echte controller

1. Steek de Xbox-controller met snoer in de pc en start de server en een Renderer met rol Ether (zie Draaien). In het log van de server staat "gamepad aangesloten: <naam>".
2. Open het dashboard. Druk op Start: de Vrij-toets gaat aan, de camera heet "Vrije camera" en op de plattegrond staat de ruimte gestippeld.
3. Rijd met de linker stick, kijk rond met de rechter, ga omhoog en omlaag met de triggers en zoom met de bumpers. Laat los: de camera glijdt kort uit. Druk op A en rijd opnieuw: nu sneller.
4. Rijd tegen een wand, het plafond en omlaag naar de banken: de camera blijft binnen en schuift langs.
5. Druk op Bewaar als Standpunt: er komt een pin en een J-toets bij. Druk op A (het Standpunt) en daarna op J: de camera gaat terug naar je eigen plek.
6. Druk nog eens op Start (of V): de stick doet niets meer.
7. Druk op O (Oefenen): het Voorbeeld wordt amber. Rijd met de stick: alleen het Voorbeeld beweegt, de Ether niet. Druk op B, rijd een stuk, en druk nog eens op B: onder Camera staat een nieuwe Camerarit op J, K of L en het Voorbeeld speelt hem af.

Werkt de controller niet: op Linux moet de gebruiker van de server `/dev/input/event*` kunnen lezen (groep `input`).

## Testen

```sh
cargo test --workspace                    # Rust-tests, ook de server van buiten via WebSocket en HTTP
(cd dashboard && npm run typecheck && npm test)
scripts/renderer-beeld.sh renderer-beeld  # beeld uit de Renderer zonder venster

# met Vulkan (GPU of lavapipe), zonder Zaal: de Titelkaart in een andere Stijl, en tekst die
# scherp blijft tijdens het invliegen. Beeldenreeksen van alle animaties: scripts/animaties-video.sh
cargo test -p renderer -- --ignored proef_beeld

# met een GPU en de Zaal uit zaal/maak.sh: alle 150 Stoelen wisselen in het volgende beeld, en
# drie Standpunten als PNG om naast de Cycles-beelden te leggen (zie zaal/README.md)
UITSLAGAVOND_ZAAL=/pad/naar/zaal UITSLAGAVOND_BEWIJS=bewijs \
  cargo test -p renderer -- --ignored --test-threads 1
```

CI ([.github/workflows/ci.yml](.github/workflows/ci.yml)) draait drie checks: **Rust-tests**, **Dashboard-typecheck** en **Renderer-beeld**. Het beeld staat als artifact `renderer-beeld` bij elke run. CI heeft geen GPU en geen Zaal: **Renderer-beeld** toont de graphics zonder Zaal (software-Vulkan), draait de beeldproeven zonder Zaal (`proef_beeld`), en de proef met de echte Zaal (`proef_gpu`) staat daar uit. Die draai je op een machine met een GPU.

## Licentie

De code valt onder MIT of Apache-2.0, naar keuze ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE)). De meegeleverde lettertypes hebben hun eigen licentie (zie `crates/renderer/assets/fonts/`). De 3D-Zaal en het programmalogo staan niet in deze repo.
