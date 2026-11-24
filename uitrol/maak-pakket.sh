#!/usr/bin/env bash
# Bouwt één pakket voor alle studiomachines: server, Renderer en dashboard uit dezelfde commit,
# want de server weigert een Renderer met een andere versie. Zie INSTALLEREN.md.
#
#   uitrol/maak-pakket.sh /pad/naar/uit     # maakt uitslagavond-tv-<versie>-linux-x86_64.tar.gz
set -euo pipefail

uit=${1:?gebruik: $0 <uitmap>}
repo=$(cd "$(dirname "$0")/.." && pwd)
doel=${CARGO_TARGET_DIR:-$repo/target}
versie=$(git -C "$repo" describe --always --dirty --tags)
naam=uitslagavond-tv-$versie-linux-$(uname -m)

cargo build --release --locked -j "${JOBS:-8}" -p server -p renderer --manifest-path "$repo/Cargo.toml"
(cd "$repo/dashboard" && npm ci --no-audit --no-fund && npm run build)

mkdir -p "$uit"
werk=$(mktemp -d "$uit/.$naam.XXXX")
trap 'rm -rf "$werk"' EXIT
p=$werk/$naam
mkdir -p "$p/bin" "$p/linux"
cp "$doel/release/uitslagavond-server" "$doel/release/uitslagavond-renderer" "$p/bin/"
cp -r "$repo/dashboard/dist" "$p/dashboard"
# Startbestanden voor de server: de uitslag van 2025 en de Zaalindeling (kopieer ze naar ~/uitslagavond).
mkdir -p "$p/start"
cp "$repo"/cijfers/*.json "$repo"/zaalindeling/*.json "$p/start/"
cp "$repo"/uitrol/linux/* "$p/linux/"
cp "$repo/uitrol/installeer.sh" "$repo/uitrol/controleer-zaal.sh" "$repo/uitrol/INSTALLEREN.md" "$p/"
echo "$versie" > "$p/VERSIE"
tar -C "$werk" -czf "$uit/$naam.tar.gz" "$naam"
echo "$uit/$naam.tar.gz"
