#!/usr/bin/env bash
# Rendert de Zaal vanaf Standpunt A, met gekleurde Stoelen, als reeks beelden in <uitmap>/reeks en de
# Stand in <uitmap>/stand.json. Zonder meer: de eindstand van de Zetelverdeling (Opbouw overgeslagen).
# Met KLEUREN='[...150 kleuren...]' die Stoelkleuren, met GRAPHIC='{...}' die Graphic.
# gebruik: W=<repo> ZAAL=<zaalmap> STAND=<stand.json> POORT=<poort> render.sh <uitmap>
# De kalibratie van het leer verander je zonder bouwen met UITSLAGAVOND_KAL en UITSLAGAVOND_KAL_PER
# (zie Kalibratie in crates/renderer/src/zaal.rs).
set -euo pipefail
W=${W:?}; ZAAL=${ZAAL:-$HOME/.local/share/uitslagavond-tv/zaal}; POORT=${POORT:-7432}; BIN=${BIN:-$W/target/release}
UIT=$1; rm -rf "$UIT"; mkdir -p "$UIT/reeks"
export UITSLAGAVOND_SLEUTEL=proef-sleutel
D=$(mktemp -d); cp "${STAND:-$HOME/.local/share/uitslagavond-tv/data/stand.json}" "$D/stand.json"
"$BIN/uitslagavond-server" --adres 127.0.0.1:$POORT --map "$D" --dashboard "$D/geen" --zonder-open-data --invoer "$W/cijfers/uitslag-tk2025.json" > "$UIT/server.log" 2>&1 &
SP=$!; trap 'kill $SP 2>/dev/null; rm -rf "$D"' EXIT; sleep 2
op() { curl -s -H "Authorization: Bearer proef-sleutel" -H "Content-Type: application/json" -d "$1" http://127.0.0.1:$POORT/api/opdracht >/dev/null; }
st() { curl -s -H "Authorization: Bearer proef-sleutel" http://127.0.0.1:$POORT/api/stand; }
op '{"soort":"UitBeeld"}'; op '{"soort":"KleurUit"}'; op '{"soort":"KiesStandpunt","letter":"A","knip":true}'
N=$(( $(st | python3 -c "import json,sys; print(json.load(sys.stdin)['volgnummer'])") + 1 ))
timeout 120 "$BIN/uitslagavond-renderer" --rol ether --server 127.0.0.1:$POORT --zaal "$ZAAL" --zonder-venster --reeks "$UIT/reeks" --volgnummer "$N" --reeks-elke-ms 1000 --reeks-seconden ${SEC:-8} > "$UIT/renderer.log" 2>&1 &
RP=$!
sleep 12
if [ -n "${KLEUREN:-}" ]; then
  op "{\"soort\":\"ZetStoelkleuren\",\"kleuren\":$KLEUREN}"
elif [ -n "${GRAPHIC:-}" ]; then
  op "{\"soort\":\"ZetKlaar\",\"graphic\":$GRAPHIC}"; op '{"soort":"Toon"}'
else
  op '{"soort":"ZetKlaar","graphic":{"sjabloon":"Zetelverdeling","set":"tk2025","namen":true,"zetels":true,"indeling":"Uitslagindeling"}}'; op '{"soort":"Toon"}'
  sleep 1; op '{"soort":"SlaOpbouwOver"}'
fi
wait $RP || true
st > "$UIT/stand.json"
ls "$UIT/reeks" | tail -1
