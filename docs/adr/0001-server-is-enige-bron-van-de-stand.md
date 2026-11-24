# De server is de enige bron van de Stand

Eén server op een eigen regie-pc houdt de Stand bij. Elke Opdracht (dashboard, tablet, Stream Deck, Videowand) gaat naar die server. De Renderers (Ether, Reserve, Videowand, Voorbeeld) houden zelf geen stand bij: bij elke wijziging krijgen ze de hele Stand met een volgnummer, en ze kiezen hun animatie door de nieuwe Stand met de vorige te vergelijken. Zo krijgt de Reserve precies wat de Ether krijgt, start een Renderer na een crash meteen met het juiste eindbeeld, en verliest een herstart van het beeld nooit gegevens.

## Considered Options

- **De Ether-Renderer is zelf de server.** Afgewezen: valt het beeld uit, dan valt ook de bediening weg, en de Reserve moet dan via de Ether meelopen.
- **Losse opdrachten naar de Renderers** ("speel animatie X") in plaats van de hele Stand. Afgewezen: één gemist bericht zet het beeld scheef, en een herstarte Renderer weet niet wat er in beeld hoort.

## Consequences

- Elke Renderer moet uit twee opeenvolgende Standen zelf afleiden welke animatie hoort. Die logica zit in de Renderer, niet in de server.
- Ether en Reserve lopen niet beeld-voor-beeld gelijk (geen genlock); een verschil van een of twee beelden is geaccepteerd.
- Valt de server weg, dan houden de Renderers het laatste beeld vast tot hij terug is. De server zet de Stand na elke wijziging op schijf.
