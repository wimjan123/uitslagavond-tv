#!/usr/bin/env bash
# Beelden van de drie Sjablonen voor Stemmingen in de Zaal (voorbeeldgegevens): server starten met
# een partijlijst, de Kamer van 22 november 2026 uit de vastgelegde open data
# (`crates/server/tests/opendata/kamerleden-2026-11-22.json`) en de Zaalindeling, twee verzonnen
# moties invoeren, en per Sjabloon de Renderer (rol Ether) een reeks laten opslaan. Het laatste
# beeld van elke reeks is het eindbeeld: `<uitmap>/<sjabloon>.png`.
#
#   scripts/stemming-beelden.sh <zaalmap> <uitmap>
#
# INDELING=Uitslagindeling zet de Stemmingen in de Uitslagindeling (de hoofdelijke stemming is
# dan voorlopig). Er ligt geen programmalogo bij de server: op die plek staat de tekst.
# Bouw eerst: cargo build --release -p server -p renderer (of zet BIN op de map met de programma's).
set -euo pipefail

zaal=$1
uit=$2
bin=${BIN:-${CARGO_TARGET_DIR:-target}/release}
poort=${POORT:-7403}
wortel=$(cd "$(dirname "$0")/.." && pwd)
export UITSLAGAVOND_SLEUTEL="proef-$RANDOM$RANDOM"

map=$(mktemp -d)
mkdir -p "$uit"
# VOORBEELDGEGEVENS: de partijkleuren van de mockup `docs/vormgeving/5-stemming.jpg`.
cat >"$map/invoer.json" <<'JSON'
{
  "partijen": [
    { "sleutel": "d66", "naam": "D66", "kleur": "#2DBE4F" },
    { "sleutel": "vvd", "naam": "VVD", "kleur": "#FF7A1A" },
    { "sleutel": "pro", "naam": "Progressief Nederland", "kort": "PRO", "kleur": "#D0213A" },
    { "sleutel": "cda", "naam": "CDA", "kleur": "#0A8A74" },
    { "sleutel": "denk", "naam": "DENK", "kleur": "#22B5C2" },
    { "sleutel": "pvdd", "naam": "Partij voor de Dieren", "kort": "PvdD", "kleur": "#5E8F23" },
    { "sleutel": "cu", "naam": "ChristenUnie", "kort": "CU", "kleur": "#1E9BE6" },
    { "sleutel": "sp", "naam": "SP", "kleur": "#F0412A" },
    { "sleutel": "50plus", "naam": "50PLUS", "kleur": "#9B2D8F" },
    { "sleutel": "volt", "naam": "Volt", "kleur": "#5A2D91" },
    { "sleutel": "pvv", "naam": "PVV", "kleur": "#2B63D9" },
    { "sleutel": "ja21", "naam": "JA21", "kleur": "#3A4FB8" },
    { "sleutel": "fvd", "naam": "Forum voor Democratie", "kort": "FvD", "kleur": "#A3243A" },
    { "sleutel": "bbb", "naam": "BBB", "kleur": "#A8C81E" },
    { "sleutel": "sgp", "naam": "SGP", "kleur": "#E6A11B" }
  ],
  "sets": [
    { "id": "voorbeeld-uitslag", "soort": "uitslag", "label": "Voorbeeldgegevens: uitslag",
      "zetels": { "d66": 26, "vvd": 22, "pro": 20, "pvv": 26, "cda": 18, "ja21": 9, "fvd": 7, "bbb": 4,
                  "sp": 3, "denk": 3, "pvdd": 3, "sgp": 3, "cu": 3, "50plus": 2, "volt": 1 } }
  ]
}
JSON
cp "$wortel/zaalindeling/tk2025.json" "$map/zaalindeling.json"

"$bin/uitslagavond-server" --adres "127.0.0.1:$poort" --map "$map" --dashboard "$map/geen-dashboard" \
  --invoer "$map/invoer.json" --zaalindeling "$map/zaalindeling.json" --zonder-open-data \
  >"$map/server.log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true; rm -rf "$map"' EXIT

api() { curl -sSf -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" "$@"; }
for _ in $(seq 100); do
  api "http://127.0.0.1:$poort/api/stand" >/dev/null 2>&1 && break
  sleep 0.1
done
opdracht() {
  api -H "X-Afzender: stemming-beelden.sh" -H "Content-Type: application/json" \
    --data "$1" "http://127.0.0.1:$poort/api/opdracht" >/dev/null
}
for _ in $(seq 100); do
  [ "$(api "http://127.0.0.1:$poort/api/stand" | jq '.stand.cijfers.sets | length')" = 1 ] && break
  sleep 0.1
done

# De Kamer uit de vastgelegde open data, zoals `opendata::kamer_op` hem leest.
opdracht "$(jq -c '
  [.value[] | . as $p | ($p.FractieZetelPersoon[] | select(.TotEnMet == null)) as $z | {p: $p, z: $z}] as $r
  | {soort: "LeesKamer", opgehaald: "2026-11-22T12:00:00+01:00", peildatum: "2026-11-22",
     kamerleden: [$r[] | {id: .p.Id, roepnaam: (.p.Roepnaam // ""), initialen: (.p.Initialen // ""),
       tussenvoegsel: .p.Tussenvoegsel, achternaam: .p.Achternaam, fractie: .z.FractieZetel.Fractie.Id,
       zetel: .z.FractieZetel_Id, fractievoorzitter: (.z.Functie == "Fractievoorzitter"), van: .z.Van[0:10],
       geboortedatum: null, geboorteplaats: null, woonplaats: null, foto: "Geen",
       kieslijst: .z.FractieZetel.Gewicht}],
     fracties: ([$r[] | .z.FractieZetel.Fractie] | unique_by(.Id)
       | map({id: .Id, afkorting: .Afkorting, naam: .NaamNL, zetels: .AantalZetels}))}
' "$wortel/crates/server/tests/opendata/kamerleden-2026-11-22.json")"

# VOORBEELDGEGEVENS: twee verzonnen moties.
opdracht '{"soort":"LeesStemmingslijst","dag":"2026-11-24","ingelezen":"2026-11-24T08:00:00+01:00","punten":[
  {"id":"m1","soort":"Motie","nummer":"Voorbeeld, nr. 1","titel":"Motie van het lid Voorbeeld over gratis schoolfruit","indieners":"Voorbeeld","agendapunt":null},
  {"id":"m2","soort":"Motie","nummer":"Voorbeeld, nr. 2","titel":"Motie van de leden Voorbeeld en Proef over een vrije dag op de dag na de verkiezingen","indieners":"Voorbeeld en Proef","agendapunt":null}]}'
[ -n "${INDELING:-}" ] && opdracht "{\"soort\":\"ZetStemmingIndeling\",\"indeling\":\"$INDELING\"}"

stand=$(api "http://127.0.0.1:$poort/api/stand")
fractie() { jq -r --arg a "$1" '.stand.kamer.fracties[] | select(.afkorting == $a) | .id' <<<"$stand"; }
lid() { jq -r --arg n "$1" '.stand.kamer.kamerleden[] | select(.achternaam == $n) | .id' <<<"$stand" | head -1; }
getikt() {
  local uit="" a
  for a in "$@"; do
    uit+="{\"fractie\":\"$(fractie "${a%:*}")\",\"stem\":\"${a#*:}\"},"
  done
  echo "[${uit%,}]"
}
# Zoals de mockup: 101 voor en 49 tegen.
opdracht "{\"soort\":\"ZetStemgedrag\",\"stemming\":\"m1\",\"stemgedrag\":{\"getikt\":$(getikt \
  D66:Voor VVD:Voor PRO:Voor CDA:Voor DENK:Voor PvdD:Voor ChristenUnie:Voor SP:Voor 50PLUS:Voor Volt:Voor),
  \"overige\":\"Tegen\",\"algemene_stemmen\":false,\"afwijkend\":[],\"hoofdelijk\":false}}"
# Hoofdelijk, met twee afwijkende stemmen.
opdracht "{\"soort\":\"ZetStemgedrag\",\"stemming\":\"m2\",\"stemgedrag\":{\"getikt\":$(getikt \
  PVV:Voor VVD:Voor CDA:Voor JA21:Voor BBB:Voor SGP:Voor FVD:Voor),
  \"overige\":\"Tegen\",\"algemene_stemmen\":false,
  \"afwijkend\":[{\"kamerlid\":\"$(lid Ellian)\",\"stem\":\"Tegen\"},{\"kamerlid\":\"$(lid Sneller)\",\"stem\":\"Voor\"}],
  \"hoofdelijk\":true}}"

# Eén Sjabloon in beeld: de Renderer start, laadt de Zaal en slaat na de Toon een reeks op.
neem() {
  local naam=$1 graphic=$2
  opdracht "{\"soort\":\"ZetKlaar\",\"graphic\":$graphic}"
  local nu
  nu=$(api "http://127.0.0.1:$poort/api/stand" | jq .volgnummer)
  rm -rf "$map/reeks"
  "$bin/uitslagavond-renderer" --rol ether --server "127.0.0.1:$poort" --zaal "$zaal" \
    --zonder-venster --reeks "$map/reeks" --volgnummer "$((nu + 1))" --reeks-elke-ms 250 \
    --reeks-seconden "${SECONDEN:-3}" >"$map/renderer.log" 2>&1 &
  local renderer=$!
  for _ in $(seq 600); do
    grep -q "Zaal geladen" "$map/renderer.log" && break
    sleep 0.1
  done
  sleep "${OPWARMEN:-4}"
  opdracht '{"soort":"Toon"}'
  wait "$renderer" || true
  grep -E "ERROR|panicked" "$map/renderer.log" || true
  mkdir -p "$uit/$naam-reeks"
  cp "$map"/reeks/*.png "$uit/$naam-reeks/"
  cp "$(ls "$map"/reeks/*.png | tail -1)" "$uit/$naam.png"
  opdracht '{"soort":"UitBeeld"}'
}

klaar() { api "http://127.0.0.1:$poort/api/stand" | jq -c '.stand.klaar.graphic'; }
opdracht '{"soort":"LegStemmingVast","stemming":"m1"}'
uitslag=$(klaar)
neem uitslag-stemming "$uitslag"
neem fracties-voor-en-tegen "$(jq -c '.sjabloon = "FractiesVoorEnTegen"' <<<"$uitslag")"
opdracht '{"soort":"LegStemmingVast","stemming":"m2"}'
neem hoofdelijke-stemming "$(klaar)"
neem uitslag-hoofdelijk "$(klaar | jq -c '.sjabloon = "UitslagStemming"')"
ls "$uit"
