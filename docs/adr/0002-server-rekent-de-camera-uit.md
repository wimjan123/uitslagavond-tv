# De server rekent de camera uit

De server rekent de camerapositie 100 keer per seconde zelf uit en stuurt die naar de Renderers. Dat geldt voor de Vrije camera, voor de overgang naar een Standpunt en voor een Camerarit. De Renderers vullen alleen de beelden daartussen vloeiend in. Zo tonen Ether en Reserve dezelfde camera, op een of twee beelden na. Dit is een uitzondering op [0001](0001-server-is-enige-bron-van-de-stand.md): voor de rest van de Stand kiezen de Renderers zelf hun animatie.

## Considered Options

- **Elke Renderer rekent zelf met de stickbewegingen van de gamepad.** Zo stond het eerst in Architectuur en stuurkoppeling. Afgewezen: kleine verschillen in beeldtijd tellen op, waardoor Ether en Reserve na een paar minuten een andere camera tonen.

## Consequences

- Tussen de stick en het beeld zit ongeveer 60 ms vertraging. Dat is geaccepteerd.
- De server heeft de onzichtbare ruimte nodig waarin de camera mag vliegen, zodat hij zelf kan voorkomen dat de camera door een muur of tussen de Stoelen gaat.
- Opslaan als Standpunt en opnemen als Camerarit lezen de positie bij de server. De Ether hoeft niets terug te melden.
