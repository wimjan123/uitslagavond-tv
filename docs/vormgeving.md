# Vormgeving: de standaard-Stijl

De standaard-Stijl van de Renderer heet **Grafiet**: geel en framboos op grafiet, met Nunito Sans. Dit document is de bron voor elk Sjabloon: bouw een nieuw Sjabloon met deze kleuren, letters, maten en Blokken, zodat alles er hetzelfde uitziet. Een omroep zet zijn eigen vormgeving in de Stijl en de Sjablonen (zie [De Stijl aanpassen](#de-stijl-aanpassen)); de software hoeft daarvoor niet te veranderen.

## Kleuren

Vijf Stijlkleuren. Een vast Sjabloon gebruikt alleen deze kleuren (`Kleurkeuze::Stijl`), zodat een andere Stijl alles meekleurt.

| Stijlkleur | Hex | Waarvoor |
|---|---|---|
| Hoofd | `#FFD166` geel | koppen en namen in de kopletter, korte namen van partijen, kicker |
| Accent | `#EF476F` framboos | alleen als markering: het streepje voor een kicker, de lijn bij 76, "1 tekort" |
| Tekst | `#FFFFFF` wit | regels, getallen, partijnamen in grafieken |
| Vlak | `#1C1F26` grafiet | de naambalk (93 procent) en de onderkant van elk paneel |
| Achtergrond | `#3A3F4B` leisteen | de bovenkant van het verloop in panelen, de waas over het beeld |

Vaste afgeleide waarden: rasterlijnen in Hoofd op 22 procent, de rest van een Zetelbalk in Tekst op 14 procent (neutraal grijs). Stemkleuren (geen Stijlkleur, gelijk aan de Stoelen): voor `#29B894` licht blauwgroen, tegen `#C0283A` donkerrood. Ze verschillen ook in helderheid, zodat wie rood en groen slecht onderscheidt ze uit elkaar houdt; op voor staat donkere tekst, op tegen witte. Partijkleuren komen uit de partijlijst en zijn in de mockups voorbeeldkleuren.

## Lettertypes

Drie Stijlletters, ingebouwd in elke Renderer (SIL Open Font License, licenties in `crates/renderer/assets/fonts/`):

| Stijlletter | Bestand | Waarvoor |
|---|---|---|
| Kop | `NunitoSans-ExtraBold.ttf` | koppen 64 tot 92 px, namen 76 tot 80 px, voorletters; altijd in Hoofd |
| Tekst | `NunitoSans-SemiBold.ttf` | partijnamen 30 tot 32 px, regels 30 tot 40 px |
| Cijfers | `NunitoSans-ExtraBold.ttf` | getallen 30 tot 132 px, kicker 30 px in kapitalen, "Aangenomen"; de cijfers zijn even breed |

Een eigen lettertype (TTF of OTF) plaatst de Regie onder Vormgeving > Stijl.

## Maten

- Beeld 1920 bij 1080. Tekst blijft binnen title safe: x 96 tot 1824, y 54 tot 1026. Een waas of zijpaneel mag tot de rand van het beeld.
- **Geen ronding**, nergens (`hoek: 0`). Kleinste tekst 30 px.
- Naambalk: binnenmarge links 44, rechts 48, boven 30, onder 28 px. Kop 92 px, 14 px ruimte, regel 40 px.
- Paneel: binnenmarge 48 px. Kicker bovenaan, kop er direct onder, dan eventueel een witte lijn van 2 px (55 procent) over de breedte van het paneel.
- Kicker: een streepje van 10 bij 30 px in Accent, dan 30 px tekst in de cijferletter en Hoofd, in kapitalen.
- Grafieken: rij 44 tot 45 px, balk 32 px, labels rechts uitgelijnd voor de as, waarde 16 px achter de balk, verticale rasterlijnen van 2 px.
- Programmalogo als bug rechtsboven: 46 px van boven, rechts uitgelijnd op de title safe-rand (x 1824), op 90 procent.

## Animatie

De tijden en maten van de animaties staan in de **Beweging** van de Stijl (`Stijl::beweging`), dus een omroep kan ze aanpassen zonder de software te veranderen. De standaard:

- **In: Openen vanuit lijn** (`Erin::OpenenVanuitLijn`). Een vlak verschijnt in 80 ms als lijn van 2 px over de hele breedte, gaat in 260 ms open tot zijn volle hoogte en daarna vervaagt de tekst erin in 200 ms. Geef alle Blokken van een naambalk of paneel deze animatie: een Tekst of Getal wacht vanzelf tot het vlak open is.
- **Uit: Sluiten naar lijn** (`Eruit::SluitenNaarLijn`): tot 30 procent van het weggaan vervaagt de tekst, tot 80 procent gaat het vlak dicht tot een lijn, en dan verdwijnt de lijn.
- De andere animaties duren 500 ms; invliegen komt van 120 px ver en is na 40 procent helemaal zichtbaar. Weggaan duurt 70 procent daarvan, Bijwerken en Optellen 140 procent.
- Balken Groeien vanaf de as en getallen tellen op, tegelijk met het vlak.
- Rustig en zakelijk, geen stuiter. Alle tijden zijn bij snelheid 100 en schalen mee met de snelheid van Stijl en Sjabloon.

## Bouwstenen in het model

Alles staat in `crates/gedeeld/src/sjabloon.rs`. Een Vlak heeft naast kleur en dekking:

- `verloop`: een kleurverloop naar een tweede (Stijl)kleur met eigen dekking, met richting en begin en eind in procent, zoals `linear-gradient` in CSS. Dezelfde kleur met dekking 0 is een verloop naar doorzichtig.
- `draaiing`: graden met de klok mee, om het midden.
- `rand`: een rand binnen het vlak, met kleur, dekking en dikte.
- `om_tekst`: het vlak is zo breed als de langste tekst van de genoemde Blokken plus een marge, en nooit breder dan zijn Plek. Een Tekst in zo'n vlak blijft op één regel; past hij niet in zijn eigen Plek, dan wordt de letter kleiner.

Een Lijn en het Programmalogo hebben een dekking. Partijlabels tonen de Zetels in de cijferletter en Tekst, en de korte naam in een tweede letter (`naamletter`) en Hoofd; met `kaartjes` krijgt elke Partij een eigen kaartje. Het Opbouwlabel toont tijdens een Opbouw de Partij die in de Zaal oplicht.

Hulpfuncties voor Sjablonen in deze stijl, gebruik ze in plaats van losse getallen:

| Functie | Wat |
|---|---|
| `naambalk(plek, blokken)` | Vlak op 93 procent dat om de tekst van `blokken` past, marge 48, opent vanuit een lijn |
| `paneel(plek)` | verloop van Achtergrond naar Vlak (beide 97 procent), 120 graden, tot 62 procent |
| `kicker(x, y, breedte, bron)` | streepje en kickertekst (twee Blokken) |
| `programmabug()` | het programmalogo rechtsboven |

## De Sjablonen

### Titelkaart (gebouwd)

Een waas onderin over de volle breedte (y 560 tot 1080) in Vlak, van 85 procent onderaan naar doorzichtig bovenaan, zodat de Zaal erboven helder blijft; de naambalk linksonder (x 96, y 744) met de kop in de kopletter en Hoofd en de regel in Tekst, en het programmalogo rechtsboven.

### Zetelverdeling in de Zaal (gebouwd)

Linksboven een naambalk met de kicker "ZETELVERDELING" en de naam van de set (64 px), rechtsboven het programmalogo. De Stoelen komen als Opbouw (1,5 seconde per Partij). Intussen staat op de vloer (y 850, voor de voorzittersbank) het Opbouwlabel: een vlak (94 procent) met een streep van 18 px in de partijkleur, de Zetels (110 px, cijferletter), de korte naam (52 px, kopletter, Hoofd) en "zetels" eronder. Daarna staan in het plafond (y 250, 31 px onder de naambalk, 136 px hoog) de Partijlabels als kaartjes op één rij, in de volgorde van de Stoelen van links naar rechts, op één vlak (97 procent) zodat de Zaal er niet doorheen schijnt: per kaartje bovenaan een kleurteken van 56 bij 12 px in de partijkleur, op elk kaartje even breed, de Zetels (51 px) en de korte naam (34 px, Tekst). Elk kaartje is zo breed als zijn naam of getal nodig heeft, zodat een lange naam niet kleiner wordt. Geen ronding, ook niet in het Opbouwlabel. Tijdens de Opbouw staan de Stoelen op drie lichtheden: de Partij van het label het lichtst en vol in haar kleur (een donkere partijkleur als JA21 of PVV wordt zolang lichter in haar eigen tint), "al geweest" grijs op ongeveer 60 procent van die lichtheid (voor elke Partij even licht en bijna zonder kleur, zodat het niet leest als een pastelkleur), en leeg (nog niet aan de beurt) het donkerst, in blauwgrijs. De Partij van het label flitst kort op als ze haar kleur krijgt en gloeit daarna zacht na tot haar stap voorbij is. Onder haar blok ligt een lichtkring op de vloer, met wat van haar kleur, die met de flits opkomt en daarna blijft branden, zodat ook een klein blok achterin (BBB) meteen te vinden is. Die kring is licht op de vloer zelf, geen spot: hij maakt het tapijt en de treden lichter, niet de rugleuningen, zodat de partijkleur niet verbleekt. Geen labels in de Zaal. Na de laatste stap komen alle Partijen tegelijk terug. De camera staat stil op Standpunt A; de plekken passen daarbij en bij het eindbeeld van de Onthulling.

### Uitslag per Partij (#49, gebouwd)

`paneel` op x 96, y 150, 1728 bij 860 (de hele breedte binnen title safe, 30 px onder het programmalogo), over de Zaal donker en onscherp (zoals de Stemmingen). `kicker` "ZETELS PER PARTIJ", de naam van de set als kop (92 px in Hoofd), rechts "150 zetels · meerderheid 76" in Hoofd (30 px). Een witte Lijn van 2 px op 55 procent onder de kop. Groot Blok Uitslaggrafiek: per Partij een rij van hooguit 45 px met de korte naam rechts uitgelijnd voor de as (Tekst, 32 px), een balk van 32 px in de partijkleur, de Zetels 16 px erachter (cijferletter, 34 px), en verticale rasterlijnen van 2 px in Hoofd op 22 procent, om de 5 Zetels (om de 10 bij een as boven 50). De rijen staan van groot naar klein; bij de grootste N staat "overig" onderaan. `programmabug`. De naam van de set staat in de kop, zoals bij de Zetelverdeling: de kicker zegt wat het is, de kop welke set.

### Zetelbalk (#49, gebouwd) en Coalitie (#51, gebouwd)

`paneel` op x 96, y 676, 1728 bij 330. Bij de Zetelbalk: `kicker` "ZETELS", de naam van de set als kop (64 px) en rechts "150 zetels · meerderheid 76". Bij de Coalitie: `kicker` "COALITIE", kop 64 px met de partijen, en rechts "zetels samen" (Tekst, 34 px), "1 tekort voor 76" in Accent (cijferletter, 32 px) en het totaal (cijferletter, 132 px).

Het Blok Zetelbalk (`zetelbalk_blok`) is 52 px hoog: de Partijen in partijkleuren met 3 px tussenruimte, "overig" in Hoofd op 45 procent, Zetels die nog bij niemand horen in Tekst op 14 procent; de lijn bij 76 in Accent (8 px breed) met "76" erboven. Labels onder de balk aan het begin van elk deel: Zetels in wit (cijferletter) en de korte naam in Hoofd, 30 px. In de losse Zetelbalk staat een label in zijn deel (in een tekstkleur die leesbaar is op de partijkleur) als het erin past; anders onder de balk op een van twee rijen, onder het midden van zijn deel, met een blokje van 18 px in de partijkleur ervoor. Bij Twee sets en de Coalitiebalk staat een label onder de balk aan het begin van zijn deel en blijft het weg als het niet past (met 10 px ruimte). De balk Bouwt op vanaf de as als het paneel open is. `programmabug`.

De Coalitiebalk is dit paneel met de coalitie vooraan in partijkleuren en elke andere Partij als eigen deel in Tekst op 14 procent, zonder label; een label mag doorlopen over die delen tot het volgende label ("9 JA21"). Het totaal is de cijferletter op 132 px en krimpt als 100 of meer niet past. Coalitie in de Zaal heeft linksboven de naambalk van de Zetelverdeling (kicker "COALITIE", de Partijen als kop) en rechtsonder een `paneel` (x 1224, y 846, 600 bij 160) met dezelfde teller. De Zaal toont de coalitie als één blok links, de rest gedimd, en bij Zetel 76 een lijn van licht op de vloer van het middenpad die van voor naar achter groeit: een smalle felle kern in een brede zachte gloed in de tint van Accent. Het licht valt alleen op de treden, niet op de stootborden ertussen, zodat de lijn plat op de vloer ligt. Het getal 76 ligt plat op het open tapijt tussen de katheder en de voorzittersbank, in de richting van de lijn (1,65 m lang in de diepte, vanaf Standpunt A ongeveer 70 px hoog), in de tint van Accent maar veel lichter, met een donkere rand, zodat het minstens 3:1 tegen het tapijt staat. Alleen bij Coalitie in de Zaal: bij de Coalitiebalk staat 76 al in de balk. Haalt de coalitie bij een Partij erbij de 76, dan telt het totaal op en groeit het daarna even tot 1,2 keer zijn maat (0,6 s), en de regel wordt "meerderheid".

### Twee sets naast elkaar (#49, gebouwd)

Twee Zetelbalken in het paneel van de Zetelbalk, hoger gemaakt. `paneel` op x 96, y 412, 1728 bij 594. `kicker` "VERGELIJKING", de naam van de nieuwste set als kop (64 px), rechts "150 zetels · meerderheid 76". Boven elke balk de naam van de set (Tekst, 30 px, Hoofd): de oudste set boven, de nieuwste onder. Onder de nieuwste balk staat onder elk label een pijltje in Hoofd met het verschil ("▲3", "▼2", "=").

### Winst en verlies (#50, gebouwd)

Zoals de Uitslag per Partij, met de as in het midden: winst naar rechts, verlies naar links, "+17" en "−20" (een echt minteken) in de cijferletter, 0 in Hoofd. Een Partij die verdwijnt krijgt "nu 0" in Hoofd en een balk in Hoofd op 60 procent; een nieuwe Partij "nieuw" in Hoofd achter het getal (Tekst, 30 px). De rijen staan van de grootste winst naar het grootste verlies, bij een gelijk verschil op de Uitslagindeling. Winst en verlies rekent tegen de som van de Voorgangers en zegt dat niet in beeld.

`paneel` op x 96, y 150, 1728 bij 860, `kicker` "WINST EN VERLIES", de naam van de set als kop (92 px in Hoofd), rechts "zetels tegenover" en de naam van de vergelijkingsset (Tekst, 30 px, Hoofd), de witte Lijn van 2 px. Het Blok Winst-en-verliesgrafiek (x 148, y 360, 1624 bij 642) tekent met dezelfde code als de Uitslaggrafiek: dezelfde rijen, balk, getal en rasterlijnen, en dezelfde as (`as_van` van het grootste verschil, naar beide kanten). De korte namen staan rechts uitgelijnd voor x 200 in het Blok; de baan heeft aan beide kanten 110 px voor het getal naast de langste balk. De as zelf is wit op 70 procent. `programmabug`. Zoals bij de Uitslag per Partij staat de naam van de set in de kop en zegt de kicker wat het is.

Bij Bijwerken schuiven de balken (ook van winst naar verlies: de ene krimpt, de andere groeit), tellen de getallen, en schuift een rij die van plek wisselt naar zijn nieuwe plek. Dat geldt ook voor de Uitslag per Partij.

### Partij uitgelicht (#50, gebouwd)

Gebouwd met de bouwstenen van de Zetelverdeling, het grote getal zoals bij de Coalitie. De Zaal kleurt de Stoelen van de gekozen Partij in haar kleur, op haar plek in de Uitslagindeling, en alle andere Stoelen grijs (`#6B7280`, net als Zetels zonder Partij); de kleurgolf loopt mee. Linksboven een `naambalk` (x 96, y 54, tot 1300 breed, 212 hoog) die om de kicker en de naam past: links haar Zetels groot (cijferletter 112 px, Tekst, rechts uitgelijnd tot x 340) met "zetels" eronder (Tekst 30 px, Hoofd), rechts daarvan de naam van de set als kicker (x 380) en de naam van de Partij als kop (kopletter 76 px, Hoofd). Alle Blokken openen vanuit de lijn. `programmabug`. De kicker is de naam van de set zoals de Regie hem schreef, niet in kapitalen: een Tekst-Blok kent nog geen kapitalen.

### Stemmingen (gebouwd, #56)

Drie Sjablonen met dezelfde uitslag; de Regie wisselt ertussen in het scherm Stemmingen. De bovenkant is bij alle drie gelijk: `kicker`, de titel als kop (64 px, op één regel: een lange titel krijgt een kleinere letter; "Motie van het lid X over Y" wordt "Motie over Y") en rechtsboven het Blok Uitkomst ("AANGENOMEN" of "VERWORPEN", cijferletter 44 px, kapitalen) op een vlak in de stemkleur. De Zaal kleurt elke Stoel in de stem van wie er zit, in de stemkleur.

- **Uitslag van een Stemming**: `paneel` op x 96, y 512, 1728 bij 494, kicker "STEMMING". Het Blok Stemmingsbalk van 72 px: voor in groen, tegen in rood, met "101 voor" en "49 tegen" erin (cijferletter 50 px, Tekst 40 px) en de drempel in Accent: een streep bij een meerderheid van de uitgebrachte stemmen (de helft plus een; bij 150 stemmen 76), op dezelfde schaal als de balk, met het getal erboven (cijferletter 30 px), zoals bij de Zetelbalk. Daaronder het Blok Fractielijst als labels (cijferletter 30 px, wit) op een neutraal vlak (wit op 7 procent) met links een streep van 10 px in de partijkleur; voor links en tegen rechts van waar de balk wisselt. Een verdeelde Fractie staat aan beide kanten, met achter haar naam het aantal Kamerleden dat zo stemde van haar Zetels ("D66 25 van 26"), zodat het aantal niet leest als haar Zetels. Een label staat nooit onder de andere kant van de balk; tegen staat rechts uitgelijnd. Past een kant niet in drie rijen, dan worden zijn labels kleiner.
- **Fracties voor en tegen**: `paneel` op x 96, y 196, 1728 bij 814 (onder het programmalogo), over de Zaal donker en onscherp, en het grote Blok Fractielijst: twee kolommen met een kop in de stemkleur ("101 voor"), zo breed als zijn aandeel, en daaronder per Fractie een tegel van 112 px hoog, drie naast elkaar: het partijlogo of de korte naam in wit op een neutraal vlak met links een streep in de partijkleur (40 px; een naam met een spatie op twee regels van hooguit 30 px, een lang woord kleiner), met eronder het aantal stemmen (30 px). Zo trekt een Fractie met één stem niet evenveel kleur als een met 26.
- **Hoofdelijke stemming**: hetzelfde paneel met kicker "HOOFDELIJKE STEMMING" en het grote Blok Namenlijst als tabel in twee kolommen, voor links en tegen rechts, elk met een kop in de stemkleur zo breed als zijn aandeel; per Fractie een regel met haar label (neutraal vlak met een streep in de partijkleur, naam links, aantal Kamerleden dat zo stemde rechts), en alleen bij wie anders stemde dan de eigen Fractie de achternamen (Tekst 40 px). Staan er namen, dan zegt een regel onder de kortste kolom "Met naam: stemde anders dan de eigen fractie". Een verdeelde Fractie staat zo aan beide kanten. Alle 150 namen passen niet leesbaar in beeld; de Zaal toont elke stem op de Stoel. Past de lijst niet, dan wordt de letter kleiner. Standpunt G (Bovenaf).

De tekstkleur op een partijkleur of stemkleur is Tekst of Vlak, op de helderheid van die kleur (`tekst_op`): wit zolang wit minstens 3,5 keer zo licht is (relatieve luminantie tot 0,25), anders donker.

### Persoonskaart (gebouwd)

Een zijpaneel rechts tot de rand van het beeld (x 1166, 754 breed), effen: een verloop van Achtergrond naar Vlak (180 graden) dat op y 640 Vlak is, en daaronder Vlak, zodat er geen naad is. Het fotovlak (x 1214, y 130, 610 bij 510, zo breed als de tekst): de Renderer maakt een portretfoto vrijstaand (de witte studio-achtergrond van de Kamerfoto's wordt doorzichtig, `crates/renderer/src/vrijstaand.rs`), zodat het paneel achter het hoofd doorloopt, en zet hem in het midden, even hoog als het vlak tot driekwart van de foto. De lichte zoom om haar en schouders gaat mee weg en de zijkanten lopen zacht weg; onderaan zakt de foto in de laatste 36 px zacht in het onderste vlak (y 640); zonder foto de voorletters in de kopletter op de partijkleur. Daaronder het Partijlogo of de korte naam (cijferletter 30 px op de partijkleur), de functie (Tekst 34 px), de naam (kopletter 76 px, Hoofd), een regel in Hoofd (Tekst 34 px) en de vrije regel met een streepje in Accent ervoor. `programmabug`.

Gebouwd met het nieuwe Blok **Partijregel** (het Partijlogo of de korte naam op een vlak in de partijkleur dat om de naam past, met de functie erachter op één regel). De naam en de regel met extra gegevens blijven op één regel (`een_regel` op een Tekst) en krijgen een kleinere letter als ze niet passen; de vrije regel mag over twee regels lopen. Het streepje voor de vrije regel verdwijnt als die leeg is (`bij_tekst` op een Vlak). Op een donkere partijkleur staan de voorletters in wit.

## Stoelen in de Zaal

De **Kleurovergang** en de **Kleurkracht** staan in de Stijl (`Stijl::kleurovergang`, `Stijl::kleurkracht`). De standaard: een Golf van links naar rechts van 1 seconde (Vervagen: 1,5 seconde) en Kleurkracht 50. Bij Kleurkracht 50 houdt de rest van de Zaal ongeveer 40 procent van zijn licht over zolang er Stoelen gekleurd zijn, bij 100 16 procent, zodat tapijt en banken in een wijd shot ook na compressie blijven staan. In een wijd shot straalt het leer zacht in zijn eigen kleur, zodat de kleur ook vol blijft waar de bak donker is, en is de kleur ruim een derde minder vol dan de partijkleur (niet snoepkleurig tegen de donkere Zaal). Van dichtbij (Partij uitgelicht, Persoonskaart) straalt het leer niet: daar valt een zachte spot schuin van boven en opzij op de Stoelen waar de lens op scherp staat, met schaduw, en houdt de rest van de Zaal 30 procent van zijn gebakken licht. Zo loopt een Stoel van licht naar schaduw en loopt de muur erachter naar donker. Het leer heeft daar meer nerf en een zachte glans, het wapen leest als reliëf in de kleur van het leer, niet als wit plaatje, en de bies en de naad zijn een donkere tint van de stoelkleur (in het model zijn ze blauw). Aanschuiven (Partij uitgelicht) kijkt schuin op het blok, zodat de rijen in de diepte aflopen. Bij de Persoonskaart is de spot zwakker, de Zaal onscherper en is wat voor de Stoel staat (het bureau vooraan) donkerder: daar is de Zaal achtergrond voor het gezicht. Achter een Sjabloon met de achtergrond Zaal donker (de Stemmingen, Uitslag per Partij, Winst en verlies) ligt een zwart vlak op 88 procent, is de Zaal tot 40 px onscherp en verliest hij zijn kleur en 3,5 stop licht, zodat de gekleurde Stoelen geen groene en rode vlekken achter de tabel zijn. Bij de Titelkaart houdt de Zaal de helft van zijn licht en het leer 35 procent, en valt een lamp van boven op de middelste Stoelen, met een zachte lichtplek op de vloer eromheen. Een Sjabloon kan een eigen Kleurovergang hebben (`Sjabloon::kleurovergang`).

## Programmalogo

Het programmalogo is merkmateriaal en zit daarom niet in de software. De standaard-Stijl noemt het bestand `programmalogo.png` in de map `bestanden` van de server (`data/bestanden/programmalogo.png`): leg daar het logo neer, wit op doorzichtig, als PNG. De Renderers halen het op en tonen het als bug rechtsboven. Zonder dat bestand staat de tekst "Uitslagavond" in de kopletter op die plek. In het dashboard kan de Regie onder Vormgeving > Stijl ook een ander logo plaatsen; dat krijgt een eigen naam bij zijn inhoud.

## De Stijl aanpassen

Onder **Vormgeving > Stijl** in het dashboard. Een wijziging geldt pas na Opslaan (Opdracht `ZetStijl`) en dan pas bij de volgende Toon: de Graphic in de Ether bewaart de Stijl van haar Toon (`GraphicInBeeld::stijl`), ook bij Bijwerken en in een Renderer die halverwege start. De klaargezette Graphic krijgt de nieuwe vorm als nieuwe versie. **Standaard-Stijl** zet de waarden uit deze pagina terug in het formulier; het dashboard haalt ze uit `dashboard/src/typen/standaardStijl.ts`, dat `cargo test -p gedeeld` schrijft.

## Sjablonen bewerken

Onder **Vormgeving > Sjablonen** in het dashboard (#62). Eén blokkeneditor voor vaste en eigen Sjablonen (ADR 0003), zonder tijdlijn:

- **Links** de lijst: eerst de vaste Sjablonen (met de stempel Aangepast als de omroep er een aanpaste, en Nieuwe standaard als de software sindsdien met een andere standaard kwam), dan de eigen Sjablonen en de knop Nieuw Sjabloon.
- **In het midden** een schets van het beeld (1920 bij 1080) met de Blokken in de kleuren van de Stijl. Klik kiest een Blok, slepen verplaatst het, het hoekje rechtsonder maakt het groter of kleiner. Daaronder de Blokken op volgorde (het eerste ligt achteraan), met Blok toevoegen (de acht gewone Blokken), Naar voren, Naar achteren, Kopie en Wis Blok. Een naambalk die om de tekst van andere Blokken past, schuift mee.
- **Rechts** de instellingen van het gekozen Blok (plek en grootte in pixels, kleur uit de Stijl of eigen, lettertype en grootte, de inhoud, animatie in en uit en "start na" in seconden) of van **heel het Sjabloon**: naam, achtergrond, wat de Stoelen doen (kleuren volgens de gegevens, gekozen Partij uitgelicht, houden of uit), een eigen Kleurovergang of die van de Stijl, het standaard Standpunt of de standaard Camerarit, en de snelheid bovenop die van de Stijl.

De achtergrond is de Zaal, de Zaal donker en onscherp (een zwart vlak op 88 procent, dus ongeveer een derde zo licht, en vaag met scherptediepte op de camera tot 40 px; de Graphic zelf blijft scherp), een eigen kleur of een eigen afbeelding (PNG of JPEG, geplaatst via `POST /api/bestand`). De Renderer tekent hem als eerste Blok, dat vervaagt (`Sjabloon::met_achtergrond`). De Regie kan de achtergrond niet per Graphic veranderen.

De Stoelen kleuren volgens de gegevens (de set, de Stemming of de coalitie; zonder gegevens houden ze hun kleur), lichten op voor de gekozen Partij (de rest grijs), of houden hun kleur.

Een wijziging geldt pas na **Opslaan** (Opdracht `ZetSjabloon`) en in de Ether pas bij de volgende Toon: de Graphic in de Ether bewaart een kopie van haar Sjabloon (`GraphicInBeeld::sjabloon`), net als van haar Stijl. Een klaargezette Graphic van dat Sjabloon krijgt de nieuwe vorm als nieuwe versie (gewijzigd door de Werkplek die opsloeg), dus Toon van een scherm dat nog de oude vorm liet zien, weigert even.

De bewerkte Sjablonen staan in de Stand (`Stand::sjablonen`). Bij een aangepast vast Sjabloon bewaart de server de vingerafdruk van de standaard waar het op gebaseerd is (`sjabloon::vingerafdruk`). Komt een nieuwe versie van de software met een andere standaard, dan blijft de aanpassing, en meldt de tab dat er een nieuwe standaard is, met **Nieuwe standaard overnemen** (`WisSjabloon`, gelijk aan **Terug naar standaard**) en **Mijn versie houden** (`HoudSjabloon`). Opslaan wat gelijk is aan de standaard, telt niet als aangepast. Het dashboard haalt de standaarden en hun vingerafdruk uit `dashboard/src/typen/standaardSjablonen.ts`, dat `cargo test -p gedeeld` schrijft.

Een **eigen Sjabloon** heeft een `id` dat met `eigen-` begint. Onder **Maken > Eigen Sjablonen** zet de Regie er een Graphic van klaar (`Graphic::Eigen`): met de invulvelden die het Sjabloon vraagt (elke Tekst of Getal met als bron een invulveld), en een set en een Partij als een Blok die nodig heeft (Zetelbalk, Zetels, Partijlogo, partijnaam). Een eigen Sjabloon dat klaarstaat of in een Preset zit, mag niet weg.
