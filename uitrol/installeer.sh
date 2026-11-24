#!/usr/bin/env bash
# Installeert uitslagavond-tv voor de huidige gebruiker, zonder sudo, vanuit een uitgepakt
# pakket van maak-pakket.sh. Zie INSTALLEREN.md.
#
#   ./installeer.sh regie      # regie-pc: server en Renderer Voorbeeld
#   ./installeer.sh ether      # beeldmachine: Renderer Ether op volledig scherm
#   ./installeer.sh reserve    # beeldmachine: Renderer Reserve op volledig scherm
#
# Een nieuwe versie installeer je op dezelfde manier; de vorige blijft staan in
# ~/uitslagavond/programma.vorige. De Stand (~/uitslagavond/data), de Zaal en de instellingen
# blijven staan.
set -euo pipefail

rol=${1:-}
case "$rol" in
  regie) units=(uitslagavond-server.service uitslagavond-voorbeeld.service) ;;
  ether|reserve) units=("uitslagavond-renderer@$rol.service") ;;
  *) echo "gebruik: $0 regie|ether|reserve" >&2; exit 2 ;;
esac

pakket=$(cd "$(dirname "$0")" && pwd)
thuis=$HOME/uitslagavond
instellingen=${XDG_CONFIG_HOME:-$HOME/.config}/uitslagavond
units_map=${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user

for nodig in bin/uitslagavond-server bin/uitslagavond-renderer dashboard/index.html linux/omgeving.voorbeeld; do
  [[ -e "$pakket/$nodig" ]] || { echo "$nodig ontbreekt in $pakket: is dit een pakket van maak-pakket.sh?" >&2; exit 1; }
done

# 1. Het programma: eerst naast het oude zetten, dan in één keer omwisselen.
mkdir -p "$thuis/data" "$thuis/zaal"
rm -rf "$thuis/programma.nieuw"
cp -a "$pakket" "$thuis/programma.nieuw"
if [[ -d "$thuis/programma" ]]; then
  rm -rf "$thuis/programma.vorige"
  mv "$thuis/programma" "$thuis/programma.vorige"
fi
mv "$thuis/programma.nieuw" "$thuis/programma"
echo "programma: $thuis/programma ($(cat "$thuis/programma/VERSIE" 2>/dev/null || echo 'versie onbekend'))"

# 2. De instellingen, alleen als ze er nog niet zijn. Op de regie-pc komt er een nieuwe
#    sleutel in; geef UITSLAGAVOND_SLEUTEL mee om de sleutel van de regie-pc over te nemen.
if [[ ! -e "$instellingen/omgeving" ]]; then
  mkdir -p "$instellingen"
  sleutel=${UITSLAGAVOND_SLEUTEL:-}
  if [[ -z "$sleutel" && "$rol" == regie ]]; then
    sleutel=$(head -c 24 /dev/urandom | base64 | tr '+/' '-_')
  fi
  ( umask 077
    sed -e "s#/home/uitslagavond/#$HOME/#g" "$pakket/linux/omgeving.voorbeeld" > "$instellingen/omgeving" )
  if [[ -n "$sleutel" ]]; then
    ( umask 077
      SLEUTEL=$sleutel awk '/^UITSLAGAVOND_SLEUTEL=/ { print "UITSLAGAVOND_SLEUTEL=" ENVIRON["SLEUTEL"]; next } 1' \
        "$instellingen/omgeving" > "$instellingen/omgeving.nieuw" )
    mv "$instellingen/omgeving.nieuw" "$instellingen/omgeving"
  else
    echo "LET OP: zet de sleutel van de regie-pc in $instellingen/omgeving" >&2
  fi
  echo "instellingen: $instellingen/omgeving (nieuw)"
else
  echo "instellingen: $instellingen/omgeving (bestond al, niet aangepast)"
fi

# 3. De opstartbestanden.
mkdir -p "$units_map"
cp "$pakket"/linux/*.service "$units_map/"
systemctl --user daemon-reload
systemctl --user enable "${units[@]}"
if [[ "$rol" == regie ]]; then
  systemctl --user restart "${units[@]}"
  loginctl enable-linger "$USER" 2>/dev/null \
    || echo "LET OP: zet 'sudo loginctl enable-linger $USER' aan, zodat de server ook zonder inlog start" >&2
elif systemctl --user -q is-active graphical-session.target; then
  systemctl --user restart "${units[@]}"
else
  echo "de Renderer start bij de volgende inlog in GNOME (automatisch inloggen, zie INSTALLEREN.md)"
fi
systemctl --user --no-pager --lines=0 status "${units[@]}" || true

if [[ "$rol" == regie && ! -e /etc/udev/rules.d/71-uitslagavond-gamepad.rules ]]; then
  echo "gamepad: sudo cp $thuis/programma/linux/71-uitslagavond-gamepad.rules /etc/udev/rules.d/ (zie INSTALLEREN.md)"
fi
