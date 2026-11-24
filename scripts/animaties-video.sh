#!/usr/bin/env bash
# Beeldenreeksen van de animaties van de Blokken: van elke animatie in en uit een MP4 (50 beelden
# per seconde) en een overzichtsblad met twaalf beelden, uit een Renderer zonder venster.
#
#   scripts/animaties-video.sh <map>
#
# Draait eerst de proef `animaties_in_beeld` (Vulkan nodig: een GPU of lavapipe) met
# UITSLAGAVOND_BEWIJS=<map>, en maakt dan met ffmpeg <map>/animaties/<reeks>.mp4 en
# <map>/animaties/<reeks>-overzicht.png. Alle cijfers en namen zijn voorbeeldgegevens.
set -euo pipefail

map=$(realpath "${1:?geef een map}")
UITSLAGAVOND_BEWIJS="$map" cargo test -p renderer --release -- --ignored proef_beeld::animaties_in_beeld

for reeks in "$map"/animaties/*/; do
  naam=$(basename "$reeks")
  aantal=$(ls "$reeks" | wc -l)
  ffmpeg -loglevel error -y -framerate 50 -i "$reeks/%04d.png" \
    -c:v libx264 -pix_fmt yuv420p -crf 18 "$map/animaties/$naam.mp4"
  # twaalf beelden, gelijk verdeeld over de reeks, in een raster van 4 bij 3
  stap=$(( (aantal + 11) / 12 ))
  ffmpeg -loglevel error -y -i "$reeks/%04d.png" \
    -vf "select='not(mod(n\,$stap))',scale=640:-1,tile=4x3:padding=4" -frames:v 1 \
    "$map/animaties/$naam-overzicht.png"
  echo "$naam: $aantal beelden"
done
