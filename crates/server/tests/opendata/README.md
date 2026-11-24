# Vastgelegde antwoorden van de OData API van de Tweede Kamer

Echte antwoorden, op 2026-11-22 één keer opgehaald van
`https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0` (open data, CC0). De tests gebruiken
ze zonder netwerk. Er staan geen foto's bij: die horen niet in Git.

- `kamerleden-2026-11-15.json`: de vraag van de server met peildatum 2026-11-15. Daarin staat
  Bente Becker (VVD, tot en met 2026-11-16) en al Dieke van Groningen, die vanaf 2026-11-17 op
  dezelfde FractieZetel zit.
- `kamerleden-2026-11-22.json`: dezelfde vraag met peildatum 2026-11-22: 150 Kamerleden in
  17 Fracties, en één Kamerlid zonder foto (Iem Al Biyati).

De vraag zelf staat in `crates/server/src/opendata.rs` (`vraag`). Opnieuw vastleggen: haal het
adres van `vraag` met een andere peildatum op (bijvoorbeeld met `curl -G` en `--data-urlencode`
per parameter) en bewaar het antwoord zoals het binnenkomt.

## Stemmingslijst

Ook op 2026-11-22 opgehaald, met de vraag uit `crates/server/src/stemmingslijst.rs` (`vraag`):

- `stemmingslijst-2026-11-24.json`: dinsdag 24 november, status "Gepland": 129 moties in acht
  agendapunten, waaronder een nader gewijzigde motie zonder Kamerstukdossier.
- `stemmingslijst-2026-11-12.json`: donderdag 12 november (Belastingplan): twee Activiteiten,
  met amendementen, moties en vijf wetsvoorstellen.
- `stemmingslijst-2026-11-22.json`: zondag 22 november, geen Stemmingen.

## Controle na de uitzending

Op 2026-11-22 opgehaald, voor de officiële uitslag van de Stemmingen (`tests/controle.rs`):

- `stemmingslijst-2026-11-19.json`: donderdag 19 november, met de vraag uit
  `crates/server/src/stemmingslijst.rs`: zeven moties, waaronder een aangehouden motie.
- `stemming-2026-11-19.json`: dezelfde dag met de vraag uit
  `crates/server/src/controle/open_data.rs` (`vraag`): per Besluit de Zaak en de
  `Stemming`-regels per Fractie. 50PLUS heeft daarin een ander `Fractie_Id` dan in de Kamer.
- `debatdirect-debat-2026-11-19.json`: het debat "Stemmingen" van die dag uit Debat Direct
  (`https://cdn.debatdirect.tweedekamer.nl/api/agenda/2026-11-19/debates/1e7725ce-...`), zoals
  het binnenkwam: zes rondes per Fractie.
- `debatdirect-debat-2026-11-12.json`: het tweede Stemmingen-debat van 12 november, met de
  hoofdelijke stemming over het Belastingplan 2027 (stemmen per Kamerlid).
- `debatdirect-agenda-2026-11-19.json`: de agenda van 19 november
  (`/api/agenda/2026-11-19`), ingekort tot `id`, `name`, `debateType`, `startsAt` en `endsAt`
  per debat.
- `debatdirect-actors-2026-11-22.json`: `/api/actors`, ingekort tot de partijen en per persoon
  `id`, `firstName`, `lastName`, `name` en `partyId` (zonder teksten en links).

Debat Direct is de interne API van de app van de Kamer, zonder documentatie of voorwaarden.
