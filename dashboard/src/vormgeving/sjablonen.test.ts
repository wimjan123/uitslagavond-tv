// Sjablonen in de vormgevingstab: `npm test` (node --test). De Sjablonen hier zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { benodigd, fout, gewijzigdSjabloon, lijst, nieuwBlok, nieuwSjabloon, standaardVan, verplaatsBlok, wisBlok } from "./sjablonen.ts";
import type { BewerktSjabloon } from "../typen/BewerktSjabloon";
import { VINGERAFDRUKKEN } from "../typen/standaardSjablonen.ts";

const titelkaart = () => structuredClone(standaardVan("titelkaart")!);

test("een aangepast vast Sjabloon met een oudere basis meldt een nieuwe standaard", () => {
  const aangepast = titelkaart();
  aangepast.snelheid = 120;
  const oud: BewerktSjabloon = { sjabloon: aangepast, standaard: "0123456789abcdef" };
  const nu: BewerktSjabloon = { sjabloon: aangepast, standaard: VINGERAFDRUKKEN["titelkaart"] };
  const rijOud = lijst([oud]).find((r) => r.sjabloon.id === "titelkaart")!;
  assert.equal(rijOud.soort, "aangepast");
  assert.equal(rijOud.nieuweStandaard, true);
  assert.equal(rijOud.sjabloon.snelheid, 120);
  assert.equal(lijst([nu]).find((r) => r.sjabloon.id === "titelkaart")!.nieuweStandaard, false);
});

test("de lijst heeft eerst alle vaste Sjablonen, dan de eigen", () => {
  const eigen = nieuwSjabloon([]);
  const rijen = lijst([{ sjabloon: eigen }]);
  assert.equal(rijen[0].sjabloon.id, "titelkaart");
  assert.equal(rijen[0].soort, "vast");
  assert.equal(rijen.at(-1)!.sjabloon.id, eigen.id);
  assert.equal(rijen.at(-1)!.soort, "eigen");
});

test("een nieuw eigen Sjabloon krijgt een vrij id dat de server accepteert", () => {
  const eerste = nieuwSjabloon([]);
  const tweede = nieuwSjabloon([eerste.id]);
  assert.match(eerste.id, /^eigen-[a-z0-9-]+$/);
  assert.notEqual(eerste.id, tweede.id);
  assert.equal(fout(eerste), null);
});

test("een Sjabloon zonder naam, met een vreemde snelheid of een leeg invulveld kan niet", () => {
  const s = nieuwSjabloon([]);
  assert.match(fout({ ...s, naam: "  " }) ?? "", /naam/);
  assert.match(fout({ ...s, snelheid: 5 }) ?? "", /snelheid/);
  const blok = nieuwBlok("Tekst");
  if (blok.soort.blok === "Tekst") blok.soort.bron = { Veld: "" };
  assert.match(fout({ ...s, blokken: [blok] }) ?? "", /invulveld/);
});

test("een vers geladen Sjabloon met null in plaats van een leeg veld telt niet als gewijzigd", () => {
  const a = titelkaart();
  const b = JSON.parse(JSON.stringify(a)) as typeof a;
  for (const blok of b.blokken) if (blok.soort.blok === "Vlak" && blok.soort.rand == null) delete blok.soort.rand;
  assert.equal(gewijzigdSjabloon(a, b), false);
  b.blokken[0].plek.x += 10;
  assert.equal(gewijzigdSjabloon(a, b), true);
});

test("een eigen Sjabloon vraagt de invulvelden van zijn Blokken, en een set of Partij als een Blok die nodig heeft", () => {
  const s = nieuwSjabloon([]);
  const kop = nieuwBlok("Tekst");
  if (kop.soort.blok === "Tekst") kop.soort.bron = { Veld: "kop" };
  const getal = nieuwBlok("Getal");
  if (getal.soort.blok === "Getal") getal.soort.bron = { Veld: "opkomst" };
  assert.deepEqual(benodigd({ ...s, blokken: [kop, getal, structuredClone(kop)] }), { velden: ["kop", "opkomst"], set: false, partij: false });
  const zetels = nieuwBlok("Getal");
  if (zetels.soort.blok === "Getal") zetels.soort.bron = "Zetels";
  assert.deepEqual(benodigd({ ...s, blokken: [zetels] }), { velden: [], set: true, partij: true });
  assert.deepEqual(benodigd({ ...s, blokken: [nieuwBlok("Zetelbalk")] }), { velden: [], set: true, partij: false });
});

test("een Blok wissen of verplaatsen houdt de naambalk om de goede teksten", () => {
  // De Titelkaart: Blok 1 is de naambalk om de teksten 2 en 3.
  const s = titelkaart();
  const om = (x: typeof s, i: number) => (x.blokken[i].soort.blok === "Vlak" ? x.blokken[i].soort.om_tekst?.blokken : undefined);
  assert.deepEqual(om(s, 1), [2, 3]);
  // De waas (0) weg: de naambalk is 0 en past om 1 en 2.
  assert.deepEqual(om(wisBlok(s, 0), 0), [1, 2]);
  // De kop (2) weg: de naambalk past alleen om de regel, die nu 2 is.
  assert.deepEqual(om(wisBlok(s, 2), 1), [2]);
  // De regel (3) naar achteren, naar plaats 0: de naambalk schuift naar 2 en past om 3 en 0.
  const verplaatst = verplaatsBlok(s, 3, 0);
  assert.deepEqual(om(verplaatst, 2), [3, 0]);
  assert.equal(verplaatst.blokken[0], s.blokken[3]);
});
