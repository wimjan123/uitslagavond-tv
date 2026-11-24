#!/usr/bin/env bash
# Een beeldenreeks van een camera-overgang met de echte Zaal: server starten, de Renderer (rol
# Ether) de Zaal laten laden, dan een Standpunt kiezen en een paar seconden lang beelden opslaan.
#
#   scripts/camera-reeks.sh <zaalmap> <uitmap> [van] [naar]     (standaard van A naar G)
#
# Bouw eerst: cargo build --release -p server -p renderer (of zet BIN op de map met de programma's).
set -euo pipefail

zaal=$1
uit=$2
van=${3:-A}
naar=${4:-G}
bin=${BIN:-${CARGO_TARGET_DIR:-target}/release}
poort=${POORT:-7401}
export UITSLAGAVOND_SLEUTEL="proef-$RANDOM$RANDOM"

map=$(mktemp -d)
mkdir -p "$uit"
"$bin/uitslagavond-server" --adres "127.0.0.1:$poort" --map "$map" --dashboard "$map/geen-dashboard" &
server=$!
trap 'kill "$server" 2>/dev/null || true; rm -rf "$map"' EXIT

api() { curl -sSf -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" "$@"; }
for _ in $(seq 100); do
  api "http://127.0.0.1:$poort/api/stand" >/dev/null 2>&1 && break
  sleep 0.1
done
kies() {
  api -H "X-Afzender: camera-reeks.sh" -H "Content-Type: application/json" \
    --data "{\"soort\":\"KiesStandpunt\",\"letter\":\"$1\",\"knip\":$2}" "http://127.0.0.1:$poort/api/opdracht"
  echo
}
kies "$van" true

"$bin/uitslagavond-renderer" --rol ether --server "127.0.0.1:$poort" --zaal "$zaal" \
  --zonder-venster --reeks "$uit" --volgnummer 2 --reeks-elke-ms "${ELKE_MS:-100}" \
  --reeks-seconden "${SECONDEN:-3}" >"$map/renderer.log" 2>&1 &
renderer=$!
# wachten tot de Zaal er staat en de reflecties gefilterd zijn
for _ in $(seq 600); do
  grep -q "Zaal geladen" "$map/renderer.log" && break
  sleep 0.1
done
sleep "${OPWARMEN:-4}"
kies "$naar" false
wait "$renderer"
grep -E "beelden in|ERROR" "$map/renderer.log" || true
