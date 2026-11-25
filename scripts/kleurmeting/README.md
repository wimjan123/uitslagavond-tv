# Kleurmeting van de Stoelen

Meet of een gekleurde Stoel in beeld de kleur van zijn Partij heeft, en stelt daar de kalibratie van het leer mee bij (`Kalibratie` in `crates/renderer/src/zaal.rs`). Het gebakken licht, de belichting en de toonweergave maken een kleur donkerder en grijzer; zonder kalibratie werd VVD-oranje zalm en waren SP, GL-PvdA en FvD hetzelfde koraalrood (council-ronde 3).

Nodig: een release-build (`cargo build --release -p server -p renderer`), een Zaal, een Stand met de partijlijst en Python met numpy en Pillow, bijvoorbeeld `uv venv /tmp/km && uv pip install --python /tmp/km/bin/python numpy pillow`.

```sh
# de eindstand van de Zetelverdeling renderen en per Partij meten (ΔE2000, lager is beter)
W=$PWD ZAAL=/pad/naar/zaal scripts/kleurmeting/render.sh /tmp/km-uit
/tmp/km/bin/python scripts/kleurmeting/meet.py crates/gedeeld/src/zaalkaart.rs /tmp/km-uit/stand.json /tmp/km-uit/reeks/007.png

# de kalibratie per kleur bijstellen (24 kleuren verspreid over alle Stoelen), en er een lijn door leggen
W=$PWD ZAAL=/pad/naar/zaal /tmp/km/bin/python scripts/kleurmeting/itereer.py 5
/tmp/km/bin/python scripts/kleurmeting/pas.py /tmp/kleurmeting/ronde4/factoren.json
```

`render.sh` start een eigen server op `POORT` (standaard 7432) met een kopie van de Stand. Een andere kalibratie probeer je zonder te bouwen met `UITSLAGAVOND_KAL` (de zeven getallen van `Kalibratie`) en per kleur met `UITSLAGAVOND_KAL_PER`.

Gemeten in council-ronde 4 (Standpunt A, Kleurkracht 50, 15 Partijen van 2025): zonder kalibratie gemiddeld ΔE 21,8 en hoogstens 31,8; met de rechte lijn van nu gemiddeld 5,8 en hoogstens 9,3. Het verschil dat overblijft zit vooral in fel oranje en geel, die de toonweergave niet helemaal haalt.
