#!/usr/bin/env bash
# Beeldenreeksen van de Coalitiebouwer in de Zaal (voorbeeldgegevens): server starten met de
# partijlijst en Zetels van de mockup `docs/vormgeving/3-zetelbalk.jpg`, per Sjabloon (Coalitie in de
# Zaal en Coalitiebalk) de coalitie D66, CDA, VVD en JA21 (75 Zetels, 1 tekort) tonen, en na een
# paar seconden SGP erbij zetten (78, een meerderheid) zonder Bijwerken. De Renderer (rol Ether)
# slaat de hele reeks op; daarvan maakt ffmpeg een MP4 en een GIF.
#
#   scripts/coalitie-beelden.sh <zaalmap> <uitmap>
#
# Uit: <uitmap>/<sjabloon>-reeks/*.png, <sjabloon>.mp4, <sjabloon>.gif, <sjabloon>-voor.png (75) en
# <sjabloon>-na.png (78). Er ligt geen programmalogo bij de server: op die plek staat de tekst.
# Bouw eerst: cargo build --release -p server -p renderer (of zet BIN op de map met de programma's).
set -euo pipefail

zaal=$1
uit=$2
bin=${BIN:-${CARGO_TARGET_DIR:-target}/release}
poort=${POORT:-7404}
export UITSLAGAVOND_SLEUTEL="proef-$RANDOM$RANDOM"

map=$(mktemp -d)
mkdir -p "$uit"
# VOORBEELDGEGEVENS: de partijkleuren en Zetels van de mockups in docs/vormgeving.
cat >"$map/invoer.json" <<'JSON'
{
  "partijen": [
    { "sleutel": "d66", "naam": "D66", "kleur": "#2DBE4F" },
    { "sleutel": "vvd", "naam": "VVD", "kleur": "#FF7A1A" },
    { "sleutel": "gl-pvda", "naam": "Progressief Nederland", "kort": "PRO", "kleur": "#D0213A" },
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
      "zetels": { "d66": 26, "vvd": 22, "gl-pvda": 20, "pvv": 26, "cda": 18, "ja21": 9, "fvd": 7, "bbb": 4,
                  "sp": 3, "denk": 3, "pvdd": 3, "sgp": 3, "cu": 3, "50plus": 2, "volt": 1 } }
  ]
}
JSON

"$bin/uitslagavond-server" --adres "127.0.0.1:$poort" --map "$map" --dashboard "${DASHBOARD:-$map/geen-dashboard}" \
  --invoer "$map/invoer.json" --zonder-open-data >"$map/server.log" 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true; rm -rf "$map"' EXIT

api() { curl -sSf -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" "$@"; }
for _ in $(seq 100); do
  api "http://127.0.0.1:$poort/api/stand" >/dev/null 2>&1 && break
  sleep 0.1
done
opdracht() {
  api -H "X-Afzender: coalitie-beelden.sh" -H "Content-Type: application/json" \
    --data "$1" "http://127.0.0.1:$poort/api/opdracht" >/dev/null
}
for _ in $(seq 100); do
  [ "$(api "http://127.0.0.1:$poort/api/stand" | jq '.stand.cijfers.sets | length')" = 1 ] && break
  sleep 0.1
done

# Elk beeld krijgt linksboven het etiket VOORBEELDGEGEVENS, zoals de mockups (alleen in het bewijs).
wortel=$(cd "$(dirname "$0")/.." && pwd)
letter="$wortel/crates/renderer/assets/fonts/NunitoSans-SemiBold.ttf"
# y: 54 (boven) of 974 (onder, als de naambalk linksboven staat)
etiket_op() { echo "drawbox=x=96:y=$1:w=372:h=52:color=black@0.66:t=fill,drawtext=fontfile=$letter:text=VOORBEELDGEGEVENS:x=114:y=$(($1 + 12)):fontsize=28:fontcolor=white"; }

# Eén Sjabloon: de coalitie in beeld, na ERBIJ_NA seconden SGP erbij, en de Renderer slaat de reeks op.
neem() {
  local sjabloon=$1 naam=$2 etiket
  etiket=$(etiket_op "$3")
  opdracht "{\"soort\":\"ZetKlaar\",\"graphic\":{\"sjabloon\":\"$sjabloon\",\"set\":\"voorbeeld-uitslag\",\"partijen\":[\"d66\",\"cda\",\"vvd\",\"ja21\"]}}"
  local nu
  nu=$(api "http://127.0.0.1:$poort/api/stand" | jq .volgnummer)
  rm -rf "$map/reeks"
  "$bin/uitslagavond-renderer" --rol ether --server "127.0.0.1:$poort" --zaal "$zaal" \
    --zonder-venster --reeks "$map/reeks" --volgnummer "$((nu + 1))" --reeks-elke-ms "${ELKE_MS:-40}" \
    --reeks-seconden "${SECONDEN:-7}" >"$map/renderer.log" 2>&1 &
  local renderer=$!
  for _ in $(seq 600); do
    grep -q "Zaal geladen" "$map/renderer.log" && break
    sleep 0.1
  done
  sleep "${OPWARMEN:-4}"
  opdracht '{"soort":"Toon"}'
  sleep "${ERBIJ_NA:-3}"
  # de Coalitiebouwer: SGP erbij, zonder Bijwerken
  opdracht '{"soort":"ZetCoalitie","partijen":["d66","cda","vvd","ja21","sgp"]}'
  wait "$renderer" || true
  grep -E "ERROR|panicked" "$map/renderer.log" || true
  rm -rf "$uit/$naam-reeks"
  mkdir -p "$uit/$naam-reeks"
  cp "$map"/reeks/*.png "$uit/$naam-reeks/"
  local beelden=("$uit/$naam-reeks"/*.png)
  local aantal=${#beelden[@]}
  # het beeld vlak voor SGP erbij komt, en het laatste
  local voor=$(( (${ERBIJ_NA:-3} * 1000) / ${ELKE_MS:-40} - 2 ))
  ffmpeg -loglevel error -y -i "${beelden[$voor]}" -vf "$etiket" "$uit/$naam-voor.png"
  ffmpeg -loglevel error -y -i "${beelden[$((aantal - 1))]}" -vf "$etiket" "$uit/$naam-na.png"
  local fps=$(( 1000 / ${ELKE_MS:-40} ))
  ffmpeg -loglevel error -y -framerate "$fps" -pattern_type glob -i "$uit/$naam-reeks/*.png" \
    -vf "$etiket" -c:v libx264 -pix_fmt yuv420p -crf 20 "$uit/$naam.mp4"
  ffmpeg -loglevel error -y -framerate "$fps" -pattern_type glob -i "$uit/$naam-reeks/*.png" \
    -vf "$etiket,fps=20,scale=960:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse" \
    "$uit/$naam.gif"
  # twaalf beelden van vlak voor SGP erbij komt tot na de puls, in een raster van 4 bij 3
  ffmpeg -loglevel error -y -framerate "$fps" -pattern_type glob -i "$uit/$naam-reeks/*.png" \
    -vf "$etiket,select='between(n\,$voor\,$((voor + 66)))*not(mod(n-$voor\,6))',scale=640:-1,tile=4x3:padding=4" \
    -frames:v 1 "$uit/$naam-overzicht.png"
  opdracht '{"soort":"UitBeeld"}'
  echo "$naam: $aantal beelden"
}

neem Coalitie coalitie-in-de-zaal 974
neem Coalitiebalk coalitiebalk 54
if [ -n "${BLIJF:-}" ]; then
  # de server blijft draaien voor een screenshot van het dashboard
  opdracht "{\"soort\":\"ZetKlaar\",\"graphic\":{\"sjabloon\":\"Coalitie\",\"set\":\"voorbeeld-uitslag\",\"partijen\":[\"d66\",\"cda\",\"vvd\",\"ja21\",\"sgp\"]}}"
  opdracht '{"soort":"Toon"}'
  echo "server op 127.0.0.1:$poort, sleutel $UITSLAGAVOND_SLEUTEL; stop met Ctrl-C"
  wait "$server"
fi
ls "$uit"
