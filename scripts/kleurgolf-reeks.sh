#!/usr/bin/env bash
# Een beeldenreeks van de kleurgolf in de Zaal tussen twee sets (voorbeeldgegevens): server
# starten met een eigen invoerbestand, de Zetelverdeling van de eerste set tonen, de Renderer
# (rol Ether) de Zaal laten laden, dan de Zetelverdeling van de tweede set tonen en een paar
# seconden lang beelden opslaan.
#
#   scripts/kleurgolf-reeks.sh <zaalmap> <uitmap>
#
# Standaard vanaf Standpunt A (achter de Voorzitter). Een ander Standpunt: STANDPUNT=G, of een
# eigen plek binnen de onzichtbare ruimte (plafond 4,59 m), zoals een overzicht van alle Stoelen:
# STAND='{"positie":[0,4.5,2.0],"draai":0,"helling":-0.55,"beeldhoek":1.75}'.
#
# Bouw eerst: cargo build --release -p server -p renderer (of zet BIN op de map met de programma's).
set -euo pipefail

zaal=$1
uit=$2
bin=${BIN:-${CARGO_TARGET_DIR:-target}/release}
poort=${POORT:-7402}
export UITSLAGAVOND_SLEUTEL="proef-$RANDOM$RANDOM"

map=$(mktemp -d)
mkdir -p "$uit"
# VOORBEELDGEGEVENS: verzonnen Zetels, de kleuren benaderen die van de Partijen.
cat >"$map/invoer.json" <<'JSON'
{
  "partijen": [
    { "sleutel": "sp", "naam": "SP", "kleur": "#E3001B" },
    { "sleutel": "pvdd", "naam": "Partij voor de Dieren", "kort": "PvdD", "kleur": "#006B2D" },
    { "sleutel": "gl-pvda", "naam": "GroenLinks-PvdA", "kort": "GL-PvdA", "kleur": "#C8102E" },
    { "sleutel": "denk", "naam": "DENK", "kleur": "#00B7B2" },
    { "sleutel": "volt", "naam": "Volt", "kleur": "#502379" },
    { "sleutel": "d66", "naam": "D66", "kleur": "#00AE41" },
    { "sleutel": "cu", "naam": "ChristenUnie", "kort": "CU", "kleur": "#00A7EB" },
    { "sleutel": "50plus", "naam": "50PLUS", "kleur": "#92278F" },
    { "sleutel": "cda", "naam": "CDA", "kleur": "#007B5F" },
    { "sleutel": "bbb", "naam": "BBB", "kleur": "#95C11F" },
    { "sleutel": "vvd", "naam": "VVD", "kleur": "#FF7709" },
    { "sleutel": "sgp", "naam": "SGP", "kleur": "#EA5B0C" },
    { "sleutel": "ja21", "naam": "JA21", "kleur": "#242B57" },
    { "sleutel": "pvv", "naam": "PVV", "kleur": "#0A2D6E" },
    { "sleutel": "fvd", "naam": "Forum voor Democratie", "kort": "FvD", "kleur": "#841818" }
  ],
  "sets": [
    { "id": "voorbeeld-tussenstand", "soort": "tussenstand", "label": "Voorbeeldgegevens: tussenstand",
      "zetels": { "pvv": 22, "gl-pvda": 16, "vvd": 18, "d66": 20, "cda": 14, "ja21": 7, "fvd": 5, "bbb": 3,
                  "sp": 3, "denk": 3, "pvdd": 3, "sgp": 2, "cu": 2, "volt": 1, "50plus": 1 } },
    { "id": "voorbeeld-uitslag", "soort": "uitslag", "label": "Voorbeeldgegevens: uitslag",
      "zetels": { "pvv": 24, "gl-pvda": 21, "vvd": 23, "d66": 27, "cda": 17, "ja21": 9, "fvd": 6, "bbb": 4,
                  "sp": 3, "denk": 3, "pvdd": 3, "sgp": 3, "cu": 3, "volt": 2, "50plus": 2 } }
  ]
}
JSON

"$bin/uitslagavond-server" --adres "127.0.0.1:$poort" --map "$map" --dashboard "$map/geen-dashboard" \
  --invoer "$map/invoer.json" --zonder-open-data &
server=$!
trap 'kill "$server" 2>/dev/null || true; rm -rf "$map"' EXIT

api() { curl -sSf -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" "$@"; }
for _ in $(seq 100); do
  api "http://127.0.0.1:$poort/api/stand" >/dev/null 2>&1 && break
  sleep 0.1
done
opdracht() {
  api -H "X-Afzender: kleurgolf-reeks.sh" -H "Content-Type: application/json" \
    --data "$1" "http://127.0.0.1:$poort/api/opdracht"
  echo
}
toon_set() {
  local standpunt=""
  [ -n "${STANDPUNT:-}" ] && standpunt=",\"standpunt\":\"$STANDPUNT\""
  opdracht "{\"soort\":\"ZetKlaar\",\"graphic\":{\"sjabloon\":\"Zetelverdeling\",\"set\":\"$1\",\"namen\":true,\"zetels\":true}$standpunt}"
  opdracht '{"soort":"Toon"}'
}
# STAND: een eigen Standpunt voor deze reeks (een Camerastand als JSON), dat E1 wordt
if [ -n "${STAND:-}" ]; then
  opdracht "{\"soort\":\"BewaarStandpunt\",\"naam\":\"Kleurgolf\",\"stand\":$STAND}"
  STANDPUNT=E1
fi
# wachten tot het invoerbestand binnen is
for _ in $(seq 100); do
  [ "$(api "http://127.0.0.1:$poort/api/stand" | jq '.stand.cijfers.sets | length')" = 2 ] && break
  sleep 0.1
done
toon_set voorbeeld-tussenstand
# met STANDPUNT staat de camera meteen daar (de eerste Toon beweegt hem erheen)
if [ -n "${STANDPUNT:-}" ]; then
  opdracht "{\"soort\":\"KiesStandpunt\",\"letter\":\"$STANDPUNT\",\"knip\":true}"
fi
nu=$(api "http://127.0.0.1:$poort/api/stand" | jq .volgnummer)

"$bin/uitslagavond-renderer" --rol ether --server "127.0.0.1:$poort" --zaal "$zaal" \
  --zonder-venster --reeks "$uit" --volgnummer "$((nu + 2))" --reeks-elke-ms "${ELKE_MS:-50}" \
  --reeks-seconden "${SECONDEN:-3}" >"$map/renderer.log" 2>&1 &
renderer=$!
# wachten tot de Zaal er staat en de reflecties gefilterd zijn
for _ in $(seq 600); do
  grep -q "Zaal geladen" "$map/renderer.log" && break
  sleep 0.1
done
sleep "${OPWARMEN:-4}"
toon_set voorbeeld-uitslag
wait "$renderer" || true
grep -E "beelden in|ERROR|panicked" "$map/renderer.log" || tail -20 "$map/renderer.log"
