# uitslagavond-tv

Live tv-graphics van de Tweede Kamer: een 3D-plenaire zaal waarin de zetels per partij kleuren, bediend door regie en presentator tijdens een uitzending.

## Zaal en zetels

**Zaal**:
De 3D-weergave van de plenaire zaal van de Tweede Kamer.
_Avoid_: Kamer (dubbelzinnig met het instituut), plenair model

**Stoel**:
Een van de 150 fysieke Kamerstoelen in de Zaal, met een vaste plek (vak, rij, nummer).
_Avoid_: zitplaats

**Zetel**:
Een van de 150 politieke posities in de Tweede Kamer, toegekend aan een partij. Een Zetel wordt getoond op een Stoel.
_Avoid_: seat, stoel (als politieke eenheid)

**Kleurovergang**:
De manier waarop de Stoelen van kleur wisselen: als Golf, van links naar rechts, door Vervagen, alle Stoelen tegelijk, of als Opbouw.
_Avoid_: fade, animatie (voor de Stoelen)

**Opbouw**:
Een Kleurovergang die één Partij tegelijk laat oplichten, de grootste eerst, terwijl een groot label op de vloer haar Zetels en naam toont. Partijen die al aan de beurt waren, dimmen naar grijs zolang de volgende oplicht en komen na de laatste stap allemaal tegelijk terug. Daarna staan alle Partijen als kaartjes op één rij, in dezelfde volgorde als hun Stoelen van links naar rechts. Alleen bij Toon van een Sjabloon dat hem kiest, zoals de Zetelverdeling; Bijwerken gaat als Golf. Opbouw overslaan kleurt de rest meteen.
_Avoid_: onthulling (dat is de vaste Camerarit), aftellen, Opbouwen (dat is een animatie van een Blok)

**Kleur uit**:
Een Opdracht die de Stoelen in de Ether direct hun eigen blauwe leer teruggeeft, los van de Graphic. De volgende Graphic die de Stoelen kleurt, zet de kleuren weer aan.
_Avoid_: stoelen uit, reset

**Kleurkracht**:
Hoe sterk gekleurde Stoelen opvallen: hoe hoger, hoe voller de kleur van het leer en hoe donkerder de rest van de Zaal eromheen.
_Avoid_: intensiteit, contrast

**Zaalindeling**:
De koppeling van Kamerleden aan Stoelen zoals de Kamer zelf de plekken in de echte zaal toewijst. Een Stoel krijgt de kleur van de Fractie van het Kamerlid dat er zit. Toont alleen de Kamer die nu zit: na een verkiezing is dat de oude Kamer tot de nieuwe beëdigd is.
_Avoid_: plattegrond, zitplan

**Overig**:
Alle Zetels van een set die een Zetelbalk of Uitslag per Partij niet per Partij toont, samen in één deel achteraan. De Regie kiest of een Graphic alle Partijen toont of de grootste zoveel plus overig.
_Avoid_: rest, anderen

**Partij**:
Een deelnemer aan een verkiezing met een eigen lijst. Verkiezingscijfers gaan per Partij.
_Avoid_: lijst, Fractie (als het om de verkiezing gaat)

**Voorganger**:
Een Partij uit een eerdere verkiezing waarmee de winst en het verlies van een latere Partij worden vergeleken, zoals bij een nieuwe naam of een fusie.
_Avoid_: oude partij, rechtsvoorganger

**Vergelijkingsset**:
De set verkiezingscijfers waartegen winst en verlies wordt berekend, standaard de vorige verkiezing. De Regie kan in een Graphic over winst en verlies een andere set kiezen. Een Partij die in de vergelijkingsset Zetels had en nu niet meer, staat daar met "-N" en "nu 0"; in de Zaal en de zetelgraphics niet.
_Avoid_: referentieset, vorige uitslag

**Fractie**:
De groep Kamerleden in de Tweede Kamer die bij een Stemming meestal als geheel stemt. Meestal hoort een Fractie bij één Partij. Een Fractie zonder lijst is afgesplitst en heeft geen eigen Partij.
_Avoid_: Partij (als het om de groep in de Kamer gaat)

**Kamerlid**:
Een persoon die een Zetel bezet, met naam en foto.
_Avoid_: parlementariër, lid

**Uitslagindeling**:
De koppeling van Zetels aan Stoelen waarin elke partij één blok van voor tot achter vormt, van links naar rechts in een vaste volgorde. Geldt voor elke set verkiezingscijfers en voor de Coalitiebouwer. Tussen de beëdiging en een nieuwe Zaalindeling ook voor de Stemmingen, met elke Fractie als één blok; bij een hoofdelijke stemming staan de Kamerleden dan voorlopig in hun blok op volgorde van de kieslijst.
_Avoid_: halfrond, hemicycle

## Uitzending

**Regie**:
De bedieners in de regiekamer die het beeld sturen en de cijfers invoeren.
_Avoid_: operator, control room

**Presentator**:
De persoon in beeld die tijdens de uitzending eenvoudige acties start.
_Avoid_: host, anchor

**Werkplek**:
Een apparaat met een eigen naam waarop iemand het systeem bedient: een laptop van de Regie of de tablet van de Presentator. Op een Werkplek van de Regie kan iemand alles; wie wat doet is een afspraak tussen de mensen.
_Avoid_: client, station, rol

**Uitslagavond**:
Een uitzending rond een Tweede Kamerverkiezing, van prognose tot uitslag.
_Avoid_: verkiezingsavond

**Stand**:
Alles wat op een moment vastligt over de uitzending: de cijfers, welke Graphic klaarstaat, welke Graphic live is en waar de camera staat.
_Avoid_: status, state, toestand

**Opdracht**:
Eén handeling van de Regie, de Presentator, een knop of de Videowand die de Stand verandert, zoals een Graphic klaarzetten of tonen.
In de UI van het dashboard en de tablet heten de handelingen Toon, Bijwerken en Uit beeld met de vaktermen Take, Update en Take out; in de code en deze woordenlijst blijven Toon, Bijwerken en Uit beeld.
_Avoid_: commando, actie, event

**Ether**:
Het beeld dat de uitzending in gaat.
In de UI van het dashboard en de tablet heet dit Live (de vakterm), en de Renderer ervan Live-renderer; in de code en deze woordenlijst blijft het Ether.
_Avoid_: program, PGM, uitvoer

**Reserve**:
Een tweede beeld dat precies de Stand van de Ether volgt, zodat de Regie bij een storing kan overschakelen.
In de UI heet de Renderer ervan Reserve-renderer.
_Avoid_: backup, fallback

**Renderer**:
Het programma dat één beeld tekent en daarbij de Stand volgt: de Ether, de Reserve, de Videowand of het Voorbeeld.
_Avoid_: renderprogramma, engine, beeldmotor

**Videowand**:
Een groot aanraakscherm in de studio met een eigen beeld, waarop de Presentator partijen en Kamerleden aantikt.
_Avoid_: touchscreen, touchwall

**Voorbeeld**:
Het kleine beeld in het regie-dashboard dat laat zien hoe de klaargezette Graphic eruitziet, voordat die live gaat.
In de UI van het dashboard en de tablet heet dit Preview (de vakterm), en de Renderer ervan Preview-renderer; in de code en deze woordenlijst blijft het Voorbeeld.
_Avoid_: preview, PVW (buiten de UI)

**Coalitiebouwer**:
Een onderdeel waarin partijen worden samengevoegd tot een mogelijke coalitie, met een teller naar een meerderheid van 76 Zetels. Twee Sjablonen: Coalitie in de Zaal (de coalitie als één blok links in de Uitslagindeling, de rest donker, met een lijn bij Zetel 76) en de Coalitiebalk (een gestapelde balk met de coalitie vooraan en een streep bij 76). Werkt met elke set, ook met de Zetels van de huidige Kamer. Uitzondering op Bijwerken: elke Partij die erbij komt of eraf gaat, werkt direct bij, ook in de Ether.
_Avoid_: coalitietool, formatiespel

**Sjabloon**:
Een vorm voor een graphic, opgebouwd uit Blokken, met invulvelden, zoals de Zetelbalk. Een vast Sjabloon komt mee met het systeem; een eigen Sjabloon maakt de omroep zelf in de vormgevingstab. Beide zijn op dezelfde manier aan te passen.
_Avoid_: template, layout

**Blok**:
Een onderdeel van een Sjabloon met een eigen plek, grootte, bron en kant-en-klare animatie, zoals Tekst, Getal, Partijlogo of Zetelbalk.
_Avoid_: element, laag, widget

**Stijl**:
De kleuren, lettertypes, het programmalogo en de snelheid van het tv-programma. Blokken nemen die over, tenzij een Blok een eigen kleur of lettertype krijgt.
_Avoid_: thema, huisstijl, look

**Naambalk**:
Een Vlak dat zo breed is als de tekst erin, met een kop en een regel eronder, zoals de naam en functie van een gast in een nieuwsprogramma. De Titelkaart is een naambalk.
_Avoid_: lower third, ondertitel, banner

**Graphic**:
Een Sjabloon dat met gegevens is ingevuld en klaar is om te tonen.
_Avoid_: item, element

**Persoonskaart**:
Een Sjabloon dat één persoon toont: een Kamerlid, of iemand die de Regie zelf toevoegt, zoals een lijsttrekker die nog geen Kamerlid is. Alleen bij een Kamerlid met een Stoel licht die Stoel op.
_Avoid_: Kamerlid-kaart, naamkaart

**Preset**:
Een opgeslagen Graphic die de Regie met één klik klaarzet, met de gegevens zoals ze waren bij het opslaan.
_Avoid_: knop, favoriet (en nooit voor een camerapositie: dat is een Standpunt)

**Standpunt**:
Een vooraf vastgelegde camerapositie in de Zaal, waar de camera vloeiend naartoe beweegt.
_Avoid_: preset, shot

**Totaal boven het paneel**:
Een vast shot van alle 150 Stoelen van hoog achter de Voorzitter, zoals Standpunt A maar steiler, zodat de Stoelen boven een paneel onderin staan. Sjablonen met zo'n paneel kiezen het; je kiest het overal waar je een Standpunt kiest.
_Avoid_: paneelstandpunt

**Camerarit**:
Een opgenomen camerabeweging door de Zaal met een begin en een eind, die je overal kiest waar je ook een Standpunt kiest. Een vaste Camerarit komt mee met het systeem, net als de vaste Standpunten, en kan terug naar zijn standaard; een eigen Camerarit neemt de Regie zelf op.
_Avoid_: camerapad, animatie, move

**Aanschuiven**:
Een camerabeweging die langzaam naar het blok Stoelen van de gekozen Partij van de Graphic schuift. Anders dan een Camerarit is hij niet opgenomen: waar hij heen gaat, hangt af van de uitslag.
_Avoid_: inzoomen, push-in

**Vrije camera**:
Een camera die de Regie live met de hand door de Zaal stuurt.
_Avoid_: free cam, fly-through

**Oefenen**:
Een stand van de Vrije camera waarin de gamepad alleen de camera van het Voorbeeld stuurt en de Ether blijft staan.
_Avoid_: repetitie, rehearsal, proefmodus

**Stemming**:
Een stemming in de Tweede Kamer over een motie, amendement of wetsvoorstel, getoond als voor en tegen per Zetel.
_Avoid_: vote, verkiezing

**Stemmingslijst**:
De lijst van alle Stemmingen van één dag, die het systeem 's ochtends inleest. De Regie vinkt erin aan welke Stemmingen in de uitzending komen.
_Avoid_: agenda, draaiboek

**Afwijkende stem**:
De stem van een Kamerlid dat anders stemt dan zijn Fractie. Alleen de Stoel van dat Kamerlid krijgt de andere kleur.
_Avoid_: dissident, verdeelde fractie
