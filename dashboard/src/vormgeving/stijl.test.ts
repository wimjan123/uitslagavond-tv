// De Stijl in de vormgevingstab: `npm test` (node --test). De Stijlen hier zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { fout, gewijzigd, letterKeuzes, letterNaam } from "./stijl.ts";
import { STANDAARD_STIJL } from "../typen/standaardStijl.ts";
import type { Stijl } from "../typen/Stijl";

const met = (wijziging: (s: Stijl) => void): Stijl => {
  const s: Stijl = structuredClone(STANDAARD_STIJL);
  wijziging(s);
  return s;
};

test("een bestandsnaam wordt een leesbare naam, zonder de hash van de server", () => {
  assert.equal(letterNaam("PlayfairDisplay-Black.ttf"), "Playfair Display Black");
  assert.equal(letterNaam("NunitoSans-ExtraBold.ttf"), "Nunito Sans Extra Bold");
  assert.equal(letterNaam("Huisletter_Bold-0123456789abcdef.otf"), "Huisletter Bold");
});

test("de keuzes voor een lettertype: de ingebouwde, plus een eigen lettertype dat de Stijl al noemt", () => {
  const eigen = met((s) => (s.lettertypes.kop = "Huisletter-0123456789abcdef.ttf"));
  const keuzes = letterKeuzes(eigen);
  assert.ok(keuzes.includes("NunitoSans-SemiBold.ttf"));
  assert.equal(keuzes.at(-1), "Huisletter-0123456789abcdef.ttf");
  assert.equal(letterKeuzes(STANDAARD_STIJL).length, new Set(letterKeuzes(STANDAARD_STIJL)).size);
});

test("Opslaan kan alleen met een geldige snelheid, kleur en logotekst", () => {
  assert.equal(fout(STANDAARD_STIJL), null);
  assert.match(fout(met((s) => (s.snelheid = 10))) ?? "", /snelheid/i);
  assert.match(fout(met((s) => (s.kleuren.hoofd = "lichtblauw"))) ?? "", /Hoofd/);
  assert.match(fout(met((s) => (s.programmalogo.tekst = "x".repeat(41)))) ?? "", /logo/i);
});

test("gewijzigd vergelijkt de inhoud, ook hoofdletters in een kleur", () => {
  assert.equal(gewijzigd(structuredClone(STANDAARD_STIJL), STANDAARD_STIJL), false);
  assert.equal(gewijzigd(met((s) => (s.kleuren.hoofd = s.kleuren.hoofd.toUpperCase())), STANDAARD_STIJL), false);
  assert.equal(gewijzigd(met((s) => (s.snelheid = 140)), STANDAARD_STIJL), true);
});
