# Totaal vanaf voorzitter staat in een studio buiten de Zaal

Standpunt A (Totaal vanaf voorzitter) staat hoog boven het plafond en achter de voorwand, op 10,5 m hoogte en 12,5 m achter het midden, met een lens van 48 graden (tot ronde 4: 10 m, 9 m en 58 graden). De Renderer laat de schil van de Zaal weg zolang een camera daar staat: het plafond met alles erboven en de nis achter de Voorzitter. De server laat de camera in die studio vliegen. Zo staat het halfrond groot in beeld, met alle 150 Stoelen tussen de kaartjes en het Opbouwlabel van de Zetelverdeling, en zonder plafond. Besloten na de review van de 3D-zaal in ronde 3, waar de Stoelen op A te klein waren.

## Considered Options

- **A in de Zaal, tegen de wand achter de Voorzitter, met een heel wijde lens** (tot ronde 3). Van elke plek in de Zaal oogt het halfrond zo'n 8 keer zo breed als hoog, want de camera kan niet hoger dan het plafond. Afgewezen: de Stoelen besloegen maar een vijfde van het beeld, het plafond en de regeringstafels de rest.
- **Alleen een near clip** op de camera. Afgewezen: het plafond loopt over de hele Zaal door, dus de stralen naar de achterste rijen gaan er ver van de camera doorheen.
- **Het plafond enkelzijdig maken**, zodat je er van boven doorheen kijkt. Afgewezen: alle materialen van het model zijn dubbelzijdig, en de normalen zijn niet overal te vertrouwen.

## Consequences

- Elk deel van het model heeft twee meshes, heel en zonder schil, en tekent er één van: zo kost de schil geen extra delen met gebakken licht (elk zo'n deel kost tijd per beeld). Alleen als twee camera's in één Renderer het oneens zijn (het Voorbeeld), komt de schil los op een eigen laag. Een camera die van A de Zaal in vliegt, ziet het plafond pas als hij eronder is, dus de wissel valt niet op.
- De vliegruimte van de server heeft een studio: boven de Voorzitter loopt het plafond zacht op, achter de voorwand mag de camera boven 4 m (tot ronde 4: 5 m). Ook de Vrije camera kan daar komen.
- Een Stand van een oudere versie die A nog op een vroegere standaard had, krijgt de nieuwe A. Een A die de Regie zelf opnieuw opsloeg, blijft.
- Sjablonen met een paneel onderin (Uitslag van een Stemming, Zetelbalk en Coalitiebalk) kiezen Totaal boven het paneel: ook in de studio, maar steiler, zodat alle Stoelen boven het paneel staan (ronde 4; daarvoor lagen de panelen over de voorste Stoelen).
- Boven en onder in beeld komt een donker verloop zolang de camera in de studio staat (`crates/renderer/src/studio.rs`): bovenin valt het plafond weg en zag je de zaagtand van de afgesneden wanden en een lichte tribune, onderin de schuin afgesneden regeringsbanken. Het verloop loopt tot net boven de achterste rij en vanaf net onder de voorste Stoelen, bij elke kadrering.
- De studio is 14 m diep (was 10), voor de langere lens van A.
