# Vaste Sjablonen bestaan uit Blokken

Elk vast Sjabloon is opgebouwd uit dezelfde Blokken als een eigen Sjabloon. De vormgevingstab opent een vast Sjabloon in dezelfde blokkeneditor, met een knop "terug naar standaard". Er is dus één soort Sjabloon, en de Renderer tekent alleen Blokken. Een animatie die een vast Sjabloon nodig heeft, zit daarom in een Blok. Voor ingewikkelde graphics zijn er grote kant-en-klare Blokken, zoals Zetelbalk, Winst-en-verliesgrafiek, Fractielijst en Namenlijst. Besloten in Proef: vormgevingstab met blokken en animaties.

## Considered Options

- **Vaste Sjablonen in code, alleen in te stellen** (variant B in de proef). Een vast Sjabloon zou dan alleen knoppen hebben voor plek, grootte, kleur, letters en beweging. Voor meer moest je een kopie maken als eigen Sjabloon. Afgewezen: de omroep moet elk vast Sjabloon helemaal kunnen aanpassen, en twee soorten Sjablonen betekent twee manieren van tekenen.
- **Eén algemeen Lijst-Blok** dat kleine Blokken herhaalt per Partij, Fractie of Kamerlid. Afgewezen: de editor wordt veel ingewikkelder en mooie animaties zijn lastiger te maken.

## Consequences

- Een vast Sjabloon dat de omroep heeft aangepast, blijft zo bij een nieuwe versie van het systeem. De vormgevingstab meldt dat er een nieuwe standaard is, met een knop om die over te nemen.
- Een wijziging aan een Sjabloon geldt pas na Opslaan. Een Graphic in de Ether houdt haar vorm tot de volgende Toon. Een klaargezette Graphic krijgt de nieuwe vorm en telt dan als net veranderd, dus Toon weigert even.
- Vaste Sjablonen gebruiken alleen kleuren en lettertypes uit de Stijl. Zo verandert alles mee als de Stijl verandert.
