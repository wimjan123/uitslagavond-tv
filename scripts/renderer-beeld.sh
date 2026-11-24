#!/usr/bin/env bash
# Een beeld uit een Renderer zonder venster, van begin tot eind:
# server starten, via HTTP een Titelkaart klaarzetten en tonen, de Renderer (rol Ether) laten tekenen
# en het beeld opslaan.
#
#   scripts/renderer-beeld.sh [uitmap]        (bouw eerst: cargo build -p server -p renderer)
#
# Zonder GPU (zoals in CI) werkt het met software-Vulkan (lavapipe uit mesa-vulkan-drivers).
set -euo pipefail

uit=${1:-renderer-beeld}
bin=${BIN:-${CARGO_TARGET_DIR:-target}/debug}
poort=${POORT:-7400}
kop=${KOP:-"De Uitslagavond"}
regel=${REGEL:-"Voorbeeldgegevens: live vanuit Den Haag"}
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

opdracht() {
  api -H "X-Afzender: renderer-beeld.sh" -H "Content-Type: application/json" \
    --data "$1" "http://127.0.0.1:$poort/api/opdracht"
  echo
}
opdracht "$(jq -cn --arg k "$kop" --arg r "$regel" \
  '{soort: "ZetKlaar", graphic: {sjabloon: "Titelkaart", kop: $k, regel: $r}}')"
opdracht '{"soort":"Toon"}'

"$bin/uitslagavond-renderer" --rol ether --server "127.0.0.1:$poort" \
  --zonder-venster --info --beeld "$uit/ether.png" --volgnummer 2
cp "$map/logboek.jsonl" "$uit/logboek.jsonl"
echo "beeld: $uit/ether.png"
