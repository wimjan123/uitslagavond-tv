#!/usr/bin/env bash
# Meet de tijd per beeld van de Renderer met de Zaal, zonder venster, op 1920x1080.
# Start de server, zet een tekst en Stoelkleuren (voorbeeldgegevens), en wisselt elke 5 seconden
# alle 150 Stoelen van kleur zolang de meting loopt. Het rapport komt in <uitmap>/meting.json.
#
#   cargo build --release -p server -p renderer
#   scripts/meet-zaal.sh <zaalmap> [uitmap]          SECONDEN=600 (standaard), BIN=target/release
#
# Met twee GPU's kies je de kaart met DRI_PRIME=pci-0000_33_00_0 (Mesa-laag voor Vulkan).
set -euo pipefail

zaal=${1:?gebruik: scripts/meet-zaal.sh <zaalmap> [uitmap]}
uit=${2:-meting}
seconden=${SECONDEN:-600}
bin=${BIN:-${CARGO_TARGET_DIR:-target}/release}
poort=${POORT:-7400}
export UITSLAGAVOND_SLEUTEL="meet-$RANDOM$RANDOM"

map=$(mktemp -d)
mkdir -p "$uit"
"$bin/uitslagavond-server" --adres "127.0.0.1:$poort" --map "$map" --dashboard "$map/geen-dashboard" &
server=$!
trap 'kill "$server" 2>/dev/null || true; rm -rf "$map"' EXIT

api() { curl -sSf -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" -H "X-Afzender: meet-zaal.sh" "$@"; }
for _ in $(seq 100); do
  api "http://127.0.0.1:$poort/api/stand" >/dev/null 2>&1 && break
  sleep 0.1
done
opdracht() { api -H "Content-Type: application/json" --data "$1" "http://127.0.0.1:$poort/api/opdracht" >/dev/null; }

# VOORBEELDGEGEVENS: twee verzonnen verdelingen van 150 Stoelen over vijf kleuren
kleuren() { jq -cn --argjson n "$1" '[["#d4202c","#00ae41","#0a2cca","#0e2c5a","#95c11f"] as $k
  | range(150) as $i | $k[[($i / $n | floor), 4] | min]] | {soort: "ZetStoelkleuren", kleuren: .}'; }
opdracht '{"soort":"ZetKlaar","graphic":{"sjabloon":"Titelkaart","kop":"VOORBEELDGEGEVENS","regel":"meting tijd per beeld"}}'
opdracht '{"soort":"Toon"}'
opdracht "$(kleuren 30)"

"$bin/uitslagavond-renderer" --rol ether --server "127.0.0.1:$poort" --zaal "$zaal" \
  --zonder-venster --meet "$seconden" --uitslag "$uit/meting.json" &
renderer=$!
wissel=0
while kill -0 "$renderer" 2>/dev/null; do
  sleep 5
  wissel=$((1 - wissel))
  opdracht "$(kleuren $((wissel ? 37 : 30)))" || true
done
wait "$renderer"
