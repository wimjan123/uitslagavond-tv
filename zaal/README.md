# De Zaal voor de Renderer

`maak.sh` maakt de bestanden waarmee de Renderer de Zaal tekent, uit een Blender-model van de plenaire zaal van de Tweede Kamer.

**Het model is niet openbaar.** Zonder Zaal werkt alles gewoon: de Renderer tekent de graphics dan op een effen achtergrond (start zonder `--zaal`), en de server en het dashboard merken er niets van. Heb je het model wel, of een eigen model met dezelfde opbouw, geef de map dan mee met `ZAAL_MODEL`. Die map bevat:

- `build.py`, dat `blend/zaal.blend` bouwt (of al een gebouwde `blend/zaal.blend`);
- `webexport.py`, dat de gescande materialen omzet in een vorm die glTF kent;
- `tools/fetch_polyhaven.py`, dat de gescande materialen van Poly Haven (CC0) in `tex/polyhaven` zet.

De exportstap verwacht de namen uit dat model: Stoelen als `Stoel_<vak>_<rij>_<nummer>` met materiaal `stoel_leer`, en camera's die de Standpunten worden.

```sh
ZAAL_MODEL=/pad/naar/model zaal/maak.sh /pad/naar/zaal                     # bouwen, normalen draaien, licht bakken (8192 px, 512 samples, Stoelen 3072 px)
ZAAL_MODEL=/pad/naar/model zaal/maak.sh /pad/naar/zaal --alleen-omgeving   # alleen het omgevingsbeeld voor reflecties, in een bestaande Zaal
uitslagavond-renderer --rol ether --zaal /pad/naar/zaal ...
```

Nodig: `python3`, `blender` (5.2 of nieuwer, zonder scherm met `-b`) en een Nvidia-kaart voor de bak. De hele run duurde op een RTX PRO 6000 bijna 6 minuten (OptiX): de zaal bakken 2 minuten 51, de Stoelen 48 seconden, ontruisen 8 seconden, het omgevingsbeeld 38 seconden. Op de CPU duurt het naar schatting uren.

## Waarom niet in Git

De uitvoer is ongeveer 380 MB: `zaal.glb` 17 MB, `lichtkaart.bin` 268 MB (8192 bij 8192 pixels, 4 bytes per pixel) en `stoelkaart.bin` 38 MB. Dat past niet in de gewone Git-geschiedenis, en Git LFS van GitHub heeft een quotum van 1 GB opslag en verkeer per maand, dus elke CI-run of verse checkout zou dat snel opmaken. Daarom staat alleen de exportstap in Git. `manifest.json` in de uitvoer noemt de sha256 van elk bestand, zodat je twee Zalen kunt vergelijken.

CI heeft geen GPU en geen Zaal. De check **Renderer-beeld** tekent daarom de graphics zonder Zaal, en de proef met de echte Zaal staat in `cargo test` uit (`proef_gpu`, zie de README in de root).

## Wat de exportstap doet

1. De Zaal bouwen in Blender (`build.py` uit de modelmap) met de gescande materialen van Poly Haven.
2. **Normalen draaien per los onderdeel.** Een gesloten vorm die binnenstebuiten staat draait om. Voor een open vlak kijkt een ray cast een paar centimeter voor en achter het vlak: raakt de voorkant ander materiaal en is de achterkant vrij, dan draait het vlak om. Zo wijst het front van de interruptiebalk niet meer de holte van 14 mm in, en bakt het niet meer zwart. Beslist de ray cast niets, dan geldt de oude regel: van het midden van de zaal af is verkeerd om.
3. Alle vaste delen samenvoegen tot `Zaal_Vast`, met een tweede UV-laag voor de lichtkaart. De eilanden worden ingepakt op hun echte vorm (`--pak CONCAVE`), niet als rechthoek.
4. Licht bakken met Cycles: de plafondspots, hanglampen, tribunelampen en het daglicht door het glas. Metaal is tijdens de bak mat; glas is tijdens de bak helemaal doorzichtig, zodat het daglicht de lichtschacht in valt. De Stoelen werpen schaduw op banken en vloer.
5. **De Stoelen** krijgen hun eigen lichtkaart, `stoelkaart.bin`: alle Stoelen delen één mesh met één lichtkaart-UV, en bij het bakken krijgt elke Stoel een eigen vak (`stoelkaart.json`). De kaart bewaart alleen licht, geen kleur, dus de Stoelen kunnen nog van kleur wisselen. Hun leer is tijdens de bak grijs, zodat er geen blauw op de omgeving valt. De Renderer geeft alleen het leer en de houten kuip dit licht: Bevy maakt per deel met een lichtkaart een eigen groep van vier, en alle zes delen van 150 Stoelen kostten 6 ms per beeld.
6. **Ontruisen**: beide kaarten gaan na de bak door OIDN (de Denoise-node van de compositor). Een lichtkaart heeft geen textuur, dus het filter maakt het plafond glad zonder iets weg te poetsen.
7. **Omgevingsbeeld voor reflecties** (#65): Cycles neemt de zaal op als kubus van zes vlakken van 1024 px (256 samples, 49 seconden op een RTX PRO 6000), vanuit het midden van de doos om alles wat vast staat. Hier zijn metaal en glas echt, en het daglicht en de Stoelen staan erin. De Renderer zet die kubus in een reflectieprobe met de doos van de zaal als grens, filtert hem één keer op de GPU en geeft er alleen glans mee: licht en schaduw blijven uit de bak.
8. `zaal.glb` schrijven zonder Draco, met de 150 Stoelen als kopieën van één mesh.

## Uitvoer

| Bestand | Inhoud |
|---|---|
| `zaal.glb` | Het model. Elke Stoel heet `Stoel_<vak>_<rij>_<nummer>`; het leer heeft materiaal `stoel_leer`. |
| `lichtkaart.bin` | Het gebakken licht als RGB9E5 (kop `LK95`, breedte, hoogte). |
| `stoelkaart.bin` | Het gebakken licht van de Stoelen, in hetzelfde formaat; 3072 px, 13 bij 12 vakken. |
| `stoelkaart.json` | Per Stoel (`Stoel_<vak>_<rij>_<nummer>`) zijn vak in `stoelkaart.bin`, in UV van glTF (v omlaag). |
| `standpunten.json` | De camera's uit het model in de assen van Bevy. De hoofdcamera achter de Voorzitter is `zaal-3`. |
| `licht.json` | De lampen met hun sterkte in Bevy-eenheden. Alles zit in de bak; de Renderer gebruikt ze live alleen voor glans en voor wat niet gebakken is (lampen, microfoons, klapstoelen). Een oudere Zaal had het daglicht `live`. |
| `omgeving.bin` | Het omgevingsbeeld voor reflecties: zes vlakken onder elkaar (+X, -X, +Y, -Y, +Z, -Z, in de assen van de kubustextuur), in hetzelfde formaat als de lichtkaart. 25 MB. |
| `omgeving.json` | Waar de kubus is opgenomen en de halve maten van de zaal, in de assen van Bevy. |
| `omgeving-voorbeeld.png` | De zes vlakken naast elkaar om te bekijken; de Renderer gebruikt hem niet. |
| `manifest.json` | Driehoeken, Stoelen, omgedraaide onderdelen, gegevens van de bak en de sha256 van elk bestand. |
| `lichtkaart-voorbeeld.png`, `stoelkaart-voorbeeld.png` | De lichtkaarten om te bekijken; de Renderer gebruikt ze niet. |

Zonder `omgeving.bin` start de Renderer gewoon, met een waarschuwing, en zijn metaal en glas mat. Een oudere Zaal zonder `stoelkaart.bin` werkt ook: dan krijgen de Stoelen live licht en is het daglicht live.

## Licht en beeld in de Renderer

Alle instelwaarden staan in `BEELD` in `crates/renderer/src/beeld.rs`: belichting (EV100 0,3), toonweergave (TonyMcMapface), een zachte gloed om spots en daklicht, koeler en iets verzadigder, een lichte vignet (boven en onder sterker dan opzij, waar de Graphics staan), occlusie uit de diepte (`occlusie.rs`: hoeken en de vloer naast bureaus iets donkerder, ook op het gebakken licht, ongeveer 0,5 ms per beeld) en hoe ver de lampen live reiken (12 m). Na het laden werkt `afwerking.rs` de materialen af: hout een kwart minder oranje en mat, de stof van de Stoelen mat, het tapijt met minder contrast en donkerder, het plafond donkerder en koeler, en mipmaps voor alle texturen, zodat fijne patronen in de verte niet flikkeren. Gekozen naast foto's van de plenaire zaal aan de Bezuidenhoutseweg. Gloed, toonweergave en kleurcorrectie werken alleen op de Zaal: de Graphics komen er daarna overheen en veranderen niet. De kom van elke plafondspot straalt in de Renderer, zoals de lamp in Cycles; de lens van het model zit erachter.

## Vergelijken met Cycles

`--beelden` rendert na het omgevingsbeeld Cycles-beelden vanuit Standpunten uit het model, met hetzelfde licht als de Renderer en dezelfde belichting (EV100 0,2, AgX als toonweergave; zonder de kleurcorrectie en gloed van de Renderer). De proef `drie_standpunten_naast_cycles` tekent dezelfde Standpunten in de Renderer.

```sh
ZAAL_MODEL=/pad/naar/model zaal/maak.sh /pad/naar/zaal --alleen-omgeving --beelden zaal-3,overzicht-zaal,dassenman-zaal --beelden-map cycles
UITSLAGAVOND_ZAAL=/pad/naar/zaal UITSLAGAVOND_BEWIJS=cycles \
  cargo test --release -p renderer drie_standpunten -- --ignored
```

Met twee kaarten kies je de kaart voor Blender met `CUDA_VISIBLE_DEVICES=1` en voor de Renderer met `DRI_PRIME=pci-0000_33_00_0`.

`zwart_per_materiaal` in het manifest telt het hele oppervlak van een materiaal, ook onderkanten die tegen de vloer liggen. Het is alleen zinvol voor een open paneel, zoals `middenvak_paneel` (het front van de interruptiebalk).

## Plattegrond voor server en dashboard

`zaalkaart.py` haalt uit `zaal.glb` waar de 150 Stoelen en de 18 hanglampen staan en schrijft dat als Rust-code naar `crates/gedeeld/src/zaalkaart.rs`. De server gebruikt het voor de camera (onder de hanglampen door, alle Stoelen in beeld vanaf Totaal), het dashboard voor de plattegrond (`cargo test -p gedeeld` zet het in `dashboard/src/typen/zaalkaart.ts`). Draai het opnieuw als het model verandert:

```sh
blender -b --factory-startup --python-exit-code 1 --python zaal/zaalkaart.py -- /pad/naar/zaal/zaal.glb crates/gedeeld/src/zaalkaart.rs
rustfmt --edition 2024 crates/gedeeld/src/zaalkaart.rs
```

Een beeldenreeks van een camera-overgang met de echte Zaal maakt `scripts/camera-reeks.sh <zaalmap> <uitmap> A G`.

Een beeldenreeks van de kleurgolf tussen twee sets (voorbeeldgegevens) maakt `scripts/kleurgolf-reeks.sh <zaalmap> <uitmap>`. Een overzicht van alle Stoelen binnen de onzichtbare ruimte: `STAND='{"positie":[0,4.5,2.0],"draai":0,"helling":-0.55,"beeldhoek":1.75}'`.

## De onzichtbare ruimte van de camera

`ruimte.py` tekent de ruimte waarin de camera mag vliegen in Blender naast het model: vloer net boven de banken (30 cm boven het hoogste van banken en tafels, ook 60 cm opzij), wanden 10 cm binnen de muren, plafond 50 cm onder de laagste hanglamp. Losse voorwerpen tellen niet mee (microfoons, schermen, de katheder). Het meet de Zaal met stralen, schrijft de maten naar `crates/gedeeld/src/ruimte.json` (de server rekent ermee, het dashboard tekent de wanden) en bewaart `ruimte.blend` met het model en de ruimte, plus twee beelden van boven en opzij. Draai het opnieuw als het model verandert:

```sh
blender -b --factory-startup --python-exit-code 1 --python zaal/ruimte.py -- \
    /pad/naar/zaal/zaal.glb crates/gedeeld/src/ruimte.json /pad/naar/uitmap
```

De tests van de server controleren dat de ruimte onder alle hanglampen ligt, dat de vloer boven elke Stoel ligt en dat de vaste Standpunten erin staan.

### De studio en de schil

Totaal vanaf voorzitter (Standpunt A) staat niet in de Zaal maar erboven en erachter, op 10 m hoogte en 9 m achter het midden, met een lens van 58 graden. Van daar oogt het halfrond minder plat dan vanaf de wand achter de Voorzitter, zodat de Stoelen groter in beeld staan (een rugleuning achterin 26 in plaats van 18 pixels breed) en het plafond uit beeld is. De verhouding met de Graphics: alle 150 Stoelen staan tussen de kaartjes van de Zetelverdeling (tot y 360) en het Opbouwlabel (vanaf y 760); de panelen van Zetelbalk en Coalitiebalk vallen over de voorste Stoelen opzij.

Dat werkt als in een virtuele studio. De server laat de camera daar vliegen: boven de Voorzitter loopt het plafond van de ruimte zacht op naar de studio, en achter de voorwand mag de camera boven 5 m (`Ruimte` in `crates/server/src/camera.rs`). De Renderer splitst bij het laden de schil af (`crates/renderer/src/schil.rs`): alles boven 5 m (plafond, daklicht, hanglampen en spots) en alles achter z 4,55 (de nis achter de Voorzitter met de zuil, het lage plafond en de voorwand). Die schil staat op een eigen laag, en elke camera ziet hem alleen zolang hij in de Zaal staat. Een camera die van A de Zaal in vliegt, ziet het plafond dus pas als hij eronder is.
