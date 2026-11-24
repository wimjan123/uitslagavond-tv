#!/usr/bin/env bash
# Controleert een gekopieerde Zaal tegen zijn manifest.json: elk bestand moet dezelfde sha256
# hebben als bij het bakken. Zo weet je dat Ether, Reserve en Voorbeeld dezelfde Zaal tekenen.
#
#   ./controleer-zaal.sh ~/uitslagavond/zaal
set -euo pipefail
map=${1:?gebruik: $0 <map met de Zaal>}
[[ -f "$map/manifest.json" ]] || { echo "geen manifest.json in $map" >&2; exit 1; }
python3 -I - "$map" <<'PY'
import hashlib, json, pathlib, sys
map = pathlib.Path(sys.argv[1])
fout = 0
for naam, verwacht in json.loads((map / "manifest.json").read_text())["bestanden"].items():
    pad = map / naam
    if not pad.is_file():
        print(f"ontbreekt: {naam}")
        fout += 1
        continue
    h = hashlib.sha256()
    with pad.open("rb") as f:
        for blok in iter(lambda: f.read(1 << 20), b""):
            h.update(blok)
    if h.hexdigest() != verwacht:
        print(f"anders:    {naam}")
        fout += 1
    else:
        print(f"goed:      {naam}")
sys.exit(1 if fout else 0)
PY
