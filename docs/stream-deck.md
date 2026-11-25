# Stream Deck met Bitfocus Companion

Een Stream Deck stuurt dezelfde Opdrachten als het dashboard, over HTTP. Daar zit [Bitfocus Companion](https://bitfocus.io/companion) tussen, met de module **Generic HTTP**. Geen OSC.

Wat een knop kan: een Preset klaarzetten, Toon, Bijwerken, Uit beeld, Kleur uit, een Standpunt of Camerarit kiezen en de Vrije camera aan- of uitzetten. Elke knopdruk staat in het logboek van de server, met de naam van de knop.

## Installeren

1. Installeer Companion op een computer in het studionetwerk die de server kan bereiken (poort 7400). Download: [bitfocus.io/companion](https://bitfocus.io/companion). Sluit de Stream Deck met USB aan op die computer.
2. Open de beheerpagina van Companion (het programma toont het adres bij het starten) en voeg bij **Connections** de module **Generic HTTP** toe.
3. Vul in de instellingen van die verbinding bij **Base URL** het adres van de server in, met een `/` aan het eind:

   ```text
   http://regie-pc:7400/
   ```

   Laat **Authentication** op **None** staan: de server wil de sleutel als `Bearer`, en dat zet je per knop in de Header (hieronder).

## De sleutel en de naam van de knop

Zet bij elke actie in het veld **Header** de gedeelde sleutel en een naam voor de knop. De naam komt in het logboek en bij "klaargezet door" op het dashboard:

```json
{"Authorization": "Bearer een-lange-gedeelde-sleutel", "X-Afzender": "Stream Deck"}
```

- Zonder of met een verkeerde sleutel weigert de server elke knop (401) en verandert er niets.
- Wil je de sleutel niet in elke knop typen, maak dan in Companion een eigen variabele `sleutel` en schrijf `"Bearer $(custom:sleutel)"`. Let op: een export van de Companion-instellingen bevat die variabele.
- De sleutel mag ook in het adres, als `?sleutel=...` achter de URI. Gebruik liever de Header, want het adres komt in logbestanden terecht.

## Knoppen

Elke knop is een **POST** naar `api/opdracht` met **Content Type** `application/json` en een vaste **Body**. Alleen de wisselknop voor de Vrije camera heeft een eigen adres.

| Knop | URI | Body |
| --- | --- | --- |
| Preset 3 klaarzetten | `api/opdracht` | `{"soort":"ZetPresetKlaar","nummer":3}` |
| Toon | `api/opdracht` | `{"soort":"Toon"}` |
| Bijwerken | `api/opdracht` | `{"soort":"Bijwerken"}` |
| Uit beeld | `api/opdracht` | `{"soort":"UitBeeld"}` |
| Kleur uit (zoals Z) | `api/opdracht` | `{"soort":"KleurUit"}` |
| Standpunt D (bewegen) | `api/opdracht` | `{"soort":"KiesStandpunt","letter":"D"}` |
| Standpunt D (knippen, zoals Shift+D) | `api/opdracht` | `{"soort":"KiesStandpunt","letter":"D","knip":true}` |
| Vrije camera om (zoals V) | `api/vrije-camera/wissel` | leeg |
| Vrije camera aan | `api/opdracht` | `{"soort":"ZetVrijeCamera","aan":true}` |
| Vrije camera uit | `api/opdracht` | `{"soort":"ZetVrijeCamera","aan":false}` |

- **Preset**: het nummer is hetzelfde als de cijfertoets op het dashboard en staat vooraan in de rij in het onderdeel Presets. Via HTTP werken ook nummers boven de 9. Klaarzetten zet hem in het Voorbeeld; pas Toon zet hem in de Ether.
- **Standpunt**: de letter is dezelfde als op het dashboard: A, S, D, F, G en H voor de vaste Standpunten, de letter van een eigen Standpunt of Camerarit (J, K en L als die toets is gekozen). Een Camerarit kies je ook met zijn letter.
- **Toon** zonder meer toont wat klaarstaat. Zie hieronder hoe een knop alleen toont wat hij liet zien.
- Het antwoord is `{"soort":"Uitgevoerd","volgnummer":12}`, of bij een weigering status 422 met de reden, zoals `{"soort":"Geweigerd","reden":"er staat geen Graphic klaar"}`. Companion zet de verbinding dan op een fout en zet de reden in zijn log. Wil je de status op de knop zien, zet hem dan met **Response Status Code Variable** in een variabele.

## Status op de knoppen

De server geeft per veld een korte tekst, zodat Companion een knop kan kleuren of een tekst kan tonen:

| Veld | Waarde |
| --- | --- |
| `kan_tonen` | `ja` als er iets klaarstaat dat nog niet in de Ether staat |
| `in_beeld` | `ja` als er een Graphic in de Ether staat |
| `kan_bijwerken` | `ja` als de Graphic in de Ether een nieuwere versie heeft |
| `klaar_versie` | de versie van de klaargezette Graphic, leeg als niets klaarstaat |
| `klaar_door` | wie de klaargezette Graphic het laatst zette of wijzigde |
| `preset_klaar` | het nummer van de Preset die klaarstaat, anders leeg |
| `preset_live` | het nummer van de Preset in de Ether, anders leeg |
| `standpunt` | de letter van het laatst gekozen Standpunt of de laatste Camerarit |
| `vrije_camera` | `aan` of `uit` |
| `kleur` | `aan` als de Stoelen in de Ether gekleurd zijn, `uit` bij hun eigen blauwe leer |

- Eén veld: **GET** `api/knoppen/kan_tonen` (antwoord: `ja`, platte tekst). Alle velden tegelijk: **GET** `api/knoppen` (JSON).
- Zo werkt het in Companion:
  1. Maak eigen variabelen, zoals `kan_tonen` en `vrije_camera`.
  2. Maak een **Trigger** die elke seconde loopt, met per veld een GET-actie met de Header van hierboven. Zet bij **JSON Response Data Variable** de variabele en laat **JSON Stringify Result** aan, zodat de tekst er gewoon in komt.
  3. Geef de knop een feedback die de variabele vergelijkt (Companion's eigen feedback voor een variabele met een waarde), bijvoorbeeld rood bij `$(custom:in_beeld)` gelijk aan `ja`. Een variabele mag ook in de tekst op de knop staan, zoals `Preset $(custom:preset_klaar)`.
- De module Generic HTTP heeft zelf geen statusfeedback; alleen een plaatje van een adres. Daarom gaat het via variabelen.

## Toon alleen wat de knop liet zien

Het dashboard stuurt bij Toon mee welke versie het scherm liet zien. Een knop kan dat ook: zet `klaar_versie` (zie hierboven) in een variabele, toon `klaar_door` of het Presetnummer op de knop, en stuur

```json
{"soort":"Toon","versie":$(custom:klaar_versie)}
```

- Wijzigt iemand de klaargezette Graphic tussen het ophalen en de knopdruk, dan weigert de server met "gewijzigd door" en de naam, net als op het dashboard. Na de volgende ronde van de Trigger laat de knop de nieuwe versie zien en lukt Toon.
- Staat er niets klaar, dan is de variabele leeg, is de body geen geldige JSON en weigert de server ook.
- Een knop zonder `versie` toont altijd wat klaarstaat. Dat is handig voor één bediener die het Voorbeeld zelf in beeld heeft.

## Proberen zonder Stream Deck

Met `curl` vanaf elke computer in het netwerk:

```sh
curl -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" -H "X-Afzender: Stream Deck" \
     -H "Content-Type: application/json" -d '{"soort":"ZetPresetKlaar","nummer":1}' \
     http://regie-pc:7400/api/opdracht
curl -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" http://regie-pc:7400/api/knoppen
curl -X POST -H "Authorization: Bearer $UITSLAGAVOND_SLEUTEL" http://regie-pc:7400/api/vrije-camera/wissel
```
