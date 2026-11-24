#!/usr/bin/env bash
# Maakt de Zaal voor de Renderer: het model bouwen, normalen draaien, licht bakken, GLB schrijven.
#
#   ZAAL_MODEL=<modelmap> zaal/maak.sh <uitmap> [extra opties voor export.py, zoals --geen-bak of --samples 128]
#
# Nodig: python3, blender (5.2 of nieuwer) en bij voorkeur een Nvidia-kaart (OptiX); op de CPU duurt de bak uren.
# De uitvoer (ongeveer 380 MB) staat niet in Git. Start de Renderer daarna met `--zaal <uitmap>`.
#
# Het model van de Tweede Kamerzaal is niet openbaar. De modelmap bevat `build.py` (bouwt `blend/zaal.blend`),
# `webexport.py` en `tools/fetch_polyhaven.py`, of al een gebouwde `blend/zaal.blend`. Zie zaal/README.md.
set -euo pipefail

uit=${1:?gebruik: ZAAL_MODEL=<modelmap> zaal/maak.sh <uitmap> [opties]}
shift
zaal=${ZAAL_MODEL:?zet ZAAL_MODEL op de map met het model van de Zaal, zie zaal/README.md}
hier=$(cd "$(dirname "$0")" && pwd)

# gescande materialen (CC0, 44 MB, niet in het model zelf)
[ -d "$zaal/tex/polyhaven" ] || python3 "$zaal/tools/fetch_polyhaven.py"
# de Zaal bouwen
[ -f "$zaal/blend/zaal.blend" ] || (cd "$zaal" && blender -b --python build.py)

mkdir -p "$uit"
blender -b --factory-startup "$zaal/blend/zaal.blend" --python-exit-code 1 \
  --python "$hier/export.py" -- --uit "$(cd "$uit" && pwd)" "$@"
echo "Zaal klaar in $uit"
