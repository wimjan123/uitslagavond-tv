# Installeren op de studiomachines

Drie machines op het studionetwerk, zonder Docker en zonder Tailscale (zie de uitzondering in [AGENTS.md](../AGENTS.md)):

| Machine | Wat draait er | Opstartbestand |
|---|---|---|
| **Regie-pc** | de server, de Renderer Voorbeeld, de gamepad | `uitslagavond-server.service`, `uitslagavond-voorbeeld.service` |
| **Beeldmachine Ether** | de Renderer Ether, randloos op volledig scherm via HDMI | `uitslagavond-renderer@ether.service` |
| **Beeldmachine Reserve** | de Renderer Reserve, net zo | `uitslagavond-renderer@reserve.service` |

De laptops van de Regie openen het dashboard op `http://<regie-pc>:7400/`. Alle programma's zijn systemd user units: ze draaien als de gewone gebruiker `uitslagavond`, starten met de pc en starten opnieuw als ze stoppen. Het logboek lees je met `journalctl --user -u 'uitslagavond-*' -f`.

## 1. Eén pakket maken

Op een bouwmachine met Rust (stable), Node 24 en `libwayland-dev`, `libxkbcommon-dev` en `libudev-dev`:

```sh
uitrol/maak-pakket.sh ~/pakketten     # geeft uitslagavond-tv-<versie>-linux-x86_64.tar.gz
```

Gebruik **hetzelfde pakket op alle drie de machines**: de server weigert een Renderer met een andere versie. Bouw op Ubuntu 24.04 of 26.04, niet op een nieuwer systeem, anders vraagt het programma een nieuwere glibc dan Ubuntu 26.04 heeft.

## 2. Ubuntu 26.04 klaarzetten (elke machine)

1. Installeer **Ubuntu 26.04 LTS Desktop** (GNOME op Wayland) met een gebruiker `uitslagavond`.
2. Videokaart: `sudo ubuntu-drivers install`, herstart, en kijk met `nvidia-smi` of de kaart er is.
3. **Automatisch inloggen**: Instellingen, Systeem, Gebruikers, Automatisch inloggen aan. De Renderers op de beeldmachines hebben de GNOME-sessie nodig.
4. Geen zwart scherm, slot of meldingen tijdens de uitzending (als `uitslagavond`):

   ```sh
   gsettings set org.gnome.desktop.session idle-delay 0
   gsettings set org.gnome.desktop.screensaver lock-enabled false
   gsettings set org.gnome.desktop.notifications show-banners false
   gsettings set org.gnome.settings-daemon.plugins.power sleep-inactive-ac-type nothing
   ```

5. Geen updates op de dag van de uitzending: `sudo systemctl disable --now unattended-upgrades apt-daily.timer apt-daily-upgrade.timer` en `sudo snap refresh --hold`. Werk de machines een paar dagen vooraf bij en zet het daarna weer zo.
6. Geef de regie-pc een vast adres op het studionetwerk (Instellingen, Netwerk). Staat de firewall aan, open dan poort 7400: `sudo ufw allow 7400/tcp`.

## 3. Regie-pc

```sh
tar -xzf uitslagavond-tv-<versie>-linux-x86_64.tar.gz
uitslagavond-tv-<versie>-linux-x86_64/installeer.sh regie
```

Dat zet het programma in `~/uitslagavond/programma`, maakt `~/uitslagavond/data` (de Stand en het logboek) en `~/uitslagavond/zaal`, schrijft de instellingen naar `~/.config/uitslagavond/omgeving` met een **nieuwe willekeurige sleutel**, en start de server en de Renderer Voorbeeld. Daarna:

- **Starten zonder inlog**: het script zet `loginctl enable-linger` aan; lukt dat niet, dan zegt het welk `sudo`-commando nodig is. Dan start de server ook als niemand is ingelogd.
- **Gamepad** (Xbox-controller met snoer, zie #44): de server leest hem, niet de browser.

  ```sh
  sudo cp ~/uitslagavond/programma/linux/71-uitslagavond-gamepad.rules /etc/udev/rules.d/
  sudo udevadm control --reload && sudo udevadm trigger
  sudo usermod -aG plugdev uitslagavond     # daarna opnieuw opstarten
  ```

  Ubuntu geeft een gamepad al aan wie is ingelogd. De regel geeft hem ook aan de groep `plugdev`, zodat de server hem leest als niemand is ingelogd.
- **Cijfers en Zaalindeling**: het pakket heeft in `start/` de uitslag van 2025 en de Zaalindeling. De server leest ze en schrijft in de Zaalindeling, dus kopieer ze eenmalig naast de data, zodat een nieuwe versie ze niet overschrijft:

  ```sh
  cp ~/uitslagavond/programma/start/uitslag-tk2025.json ~/uitslagavond/cijfers.json
  cp ~/uitslagavond/programma/start/tk2025.json ~/uitslagavond/zaalindeling.json
  ```

  Zet dan in de instellingen `UITSLAGAVOND_SERVER_OPTIES=--invoer /home/uitslagavond/uitslagavond/cijfers.json --zaalindeling /home/uitslagavond/uitslagavond/zaalindeling.json` en herstart met `systemctl --user restart uitslagavond-server`.
- **De Zaal** voor het Voorbeeld: zie stap 5.
- Controle: `systemctl --user status uitslagavond-server`, en open `http://<regie-pc>:7400/` op een laptop. De sleutel voor het dashboard staat in `~/.config/uitslagavond/omgeving`.

## 4. Beeldmachine (Ether of Reserve)

1. Sluit de HDMI-uitgang aan op de omzetter naar SDI (of de mixer). Zet in Instellingen, Beeldschermen die uitgang op **1920 x 1080, 50 Hz, schaal 100%**. Gebruik bij voorkeur alleen die uitgang.
2. Installeer met de sleutel van de regie-pc, zodat hij op alle machines gelijk is:

   ```sh
   tar -xzf uitslagavond-tv-<versie>-linux-x86_64.tar.gz
   UITSLAGAVOND_SLEUTEL='<sleutel van de regie-pc>' uitslagavond-tv-<versie>-linux-x86_64/installeer.sh ether   # of reserve
   ```

3. Zet in `~/.config/uitslagavond/omgeving`:
   - `UITSLAGAVOND_SERVER=<adres van de regie-pc>:7400`
   - `UITSLAGAVOND_MONITOR`: `hoofdscherm`, een nummer of de naam van de aansluiting, zoals `HDMI-A-1`. De Renderer noemt bij de start alle monitors in het logboek (`monitor 0: HDMI-A-1, 1920x1080, 50 Hz`). Is die monitor er nog niet (de omzetter staat uit), dan komt de Renderer op het hoofdscherm en gaat hij over zodra de monitor er is.
   - `UITSLAGAVOND_BEELDFORMAAT`: standaard `1080p50`. Vraagt de mixer iets anders, zoals `720p50` of `1080p59.94`, zet het hier en in de scherminstellingen.
4. Herstart met `systemctl --user restart 'uitslagavond-renderer@*'`, of log opnieuw in.

De Renderer draait randloos op volledig scherm, met vsync (`Fifo`), zonder muispijl. Staat het scherm niet in het Beeldformaat (een andere maat of 60 Hz in plaats van 50 Hz), dan staat dat als fout in het logboek: `het scherm ververst op 60 Hz, het Beeldformaat 1080p50 vraagt 50 Hz`. De Renderer tekent dan wel, maar de mixer krijgt het verkeerde beeld.

## 5. De Zaal op een machine zetten

De gebakken Zaal (ongeveer 340 MB) staat niet in Git, zie [zaal/README.md](../zaal/README.md). Bak hem één keer en zet **dezelfde Zaal** op alle drie de machines, zodat Ether, Reserve en Voorbeeld hetzelfde beeld tekenen.

1. Bak hem op een machine met een Nvidia-kaart en Blender 5.2: `ZAAL_MODEL=/pad/naar/model zaal/maak.sh /pad/naar/zaal` (het model is niet openbaar, zie [zaal/README.md](../zaal/README.md); op een RTX PRO 6000 bijna 3 minuten). Een beeldmachine met een RTX 5070 Ti kan het ook, als Blender erop staat.
2. Kopieer de map naar elke machine: `rsync -a /pad/naar/zaal/ uitslagavond@<machine>:uitslagavond/zaal/` (of met een USB-stick).
3. Controleer op elke machine of hij gelijk is aan de bak: `~/uitslagavond/programma/controleer-zaal.sh ~/uitslagavond/zaal`. Elke regel moet `goed` zijn.
4. Haal in `~/.config/uitslagavond/omgeving` het `#` weg voor `UITSLAGAVOND_ZAAL` en herstart de Renderer. Zonder Zaal tekent hij alleen de graphics op een effen achtergrond; staat de regel aan terwijl de map leeg is, dan stopt de Renderer meteen (en start steeds opnieuw).

## 6. Bijwerken en terugzetten

Installeer een nieuw pakket met hetzelfde commando, op alle machines. De Stand, de Zaal en de instellingen blijven staan; de vorige versie staat in `~/uitslagavond/programma.vorige`. Terugzetten:

```sh
mv ~/uitslagavond/programma ~/uitslagavond/programma.kapot
mv ~/uitslagavond/programma.vorige ~/uitslagavond/programma
systemctl --user restart 'uitslagavond-*'
```

## 7. Reserve: Windows met DirectX 12

Werkt Ubuntu niet op een beeldmachine, dan is Windows 11 LTSC de reserve (#19). CI bouwt de server en de Renderer voor Windows (check **Windows-build (DirectX 12)**). Op Windows kiest de Renderer zelf DirectX 12, niet Vulkan (Vulkan op Windows heeft fout bevy#18898); `WGPU_BACKEND` gaat voor.

- Bouwen op Windows: `cargo build --release -p server -p renderer`.
- Starten met de pc: een snelkoppeling in `shell:startup` naar
  `uitslagavond-renderer.exe --rol reserve --volledig-scherm --server <regie-pc>:7400`, met `UITSLAGAVOND_SLEUTEL` en `UITSLAGAVOND_ZAAL` als omgevingsvariabelen van de gebruiker.
- Zet in Windows de uitgang op 1920 x 1080, 50 Hz, en zet updates en meldingen uit.

Er is geen installatiescript voor Windows en het is nog niet op een echte Windows-pc geprobeerd.

## Wat getest is

- **Ubuntu 26.04.1 zonder scherm** (de cloud-image van Ubuntu, geboot met `systemd-nspawn` en systemd 259 als init, zonder netwerk naar buiten): `installeer.sh regie` als gewone gebruiker; de server en de Renderer Voorbeeld starten, ook na een herstart zonder inlog; de server weigert zonder sleutel (401), geeft met sleutel de Stand (200) en het dashboard; met de startbestanden leest hij de cijfers en de Zaalindeling in (logboek: `LeesInvoer`, `LeesZaalindeling`); `systemctl --user stop` stopt hem netjes; `systemd-analyze --user verify` vindt niets. Het Voorbeeld tekende daar met software-Vulkan (Mesa, zoals op Ubuntu Desktop), zonder Zaal.
- **`installeer.sh ether`** zet de Renderer aan voor de GNOME-sessie; zonder sessie blijft hij uit, zoals bedoeld.
- **Volledig scherm** in een X-server zonder scherm (Xvfb, 1920 x 1080) op een Nvidia-kaart: randloos over het hele scherm, zonder muispijl, met de lijst van monitors en de melding over de verversing in het logboek.
- **Niet getest**: GNOME op Wayland met een echt scherm, de gamepad en de udev-regel op echte hardware, de Nvidia-driver van Ubuntu 26.04, Windows. Vsync op een echte HDMI-uitgang van 50 Hz en de mixer komen in #66 en #68.
