// Presets in het dashboard: `npm test` (node --test). De Presets hier zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { bewaardAls, filterPresets, statusVan } from "./presets.ts";
import type { Graphic } from "../typen/Graphic";
import type { Preset } from "../typen/Preset";
import type { Stand } from "../typen/Stand";

const titelkaart = (kop: string): Graphic => ({ sjabloon: "Titelkaart", kop, regel: "" });
const stemming: Graphic = {
  sjabloon: "UitslagStemming",
  uitslag: { stemming: "m1", titel: "Motie", nummer: null, indieners: null, voor: 80, tegen: 70, besluit: "Aangenomen", algemene_stemmen: false, fracties: [], afwijkend: [], indeling: "Zaalindeling", voorlopig: false, stoelen: [], kamerleden: [] },
};

const preset = (nummer: number, graphic: Graphic, extra: Partial<Preset> = {}): Preset => ({
  nummer,
  graphic,
  op_tablet: false,
  bewaard_ms: 0,
  bewaard_door: "Regie links",
  ...extra,
});

/** Een Stand met alleen wat de Presets nodig hebben: wat klaarstaat en wat in de Ether staat. */
const stand = (klaarId: number | null, etherId: number | null, klaarVersie = klaarId): Stand =>
  ({
    klaar: klaarId === null ? null : { id: klaarId, versie: klaarVersie, graphic: titelkaart("x"), klaargezet_door: "Regie links", gewijzigd_door: "Regie links" },
    ether: etherId === null ? null : { id: etherId, versie: etherId, graphic: titelkaart("x"), getoond: etherId, sinds_ms: 0, getoond_door: "Regie links" },
  }) as Stand;

test("Klaar en Live volgen de Graphic die uit de Preset kwam", () => {
  const p = preset(1, titelkaart("Een"), { klaargezet_als: 7, getoond_als: 5, laatst_live_ms: 1000 });
  assert.deepEqual(statusVan(p, stand(7, null)), { klaar: true, live: false, wasLiveOm: 1000 });
  assert.deepEqual(statusVan(p, stand(7, 5)), { klaar: true, live: true, wasLiveOm: 1000 });
  assert.deepEqual(statusVan(p, stand(8, 6)), { klaar: false, live: false, wasLiveOm: 1000 });
  assert.deepEqual(statusVan(preset(2, titelkaart("Twee")), stand(7, 5)), { klaar: false, live: false, wasLiveOm: null });
  // Dezelfde Graphic, maar gewijzigd na het klaarzetten: dat is de Preset niet meer.
  assert.deepEqual(statusVan(p, stand(7, null, 9)), { klaar: false, live: false, wasLiveOm: 1000 });
});

test("filteren op groep en op de tablet", () => {
  const lijst = [preset(1, titelkaart("Een"), { op_tablet: true }), preset(2, stemming), preset(3, stemming, { op_tablet: true })];
  const nummers = (l: Preset[]) => l.map((p) => p.nummer);
  assert.deepEqual(nummers(filterPresets(lijst, null, false)), [1, 2, 3]);
  assert.deepEqual(nummers(filterPresets(lijst, "Stemmingen", false)), [2, 3]);
  assert.deepEqual(nummers(filterPresets(lijst, "Algemeen", false)), [1]);
  assert.deepEqual(nummers(filterPresets(lijst, null, true)), [1, 3]);
  assert.deepEqual(nummers(filterPresets(lijst, "Stemmingen", true)), [3]);
  assert.deepEqual(nummers(filterPresets(lijst, "Uitslagavond", false)), []);
});

test("een Graphic die al zo bewaard is, noemt de Preset", () => {
  const lijst = [preset(1, titelkaart("Een")), preset(2, titelkaart("Een"), { standpunt: "D" })];
  assert.equal(bewaardAls(lijst, titelkaart("Een"), undefined), 1);
  assert.equal(bewaardAls(lijst, titelkaart("Een"), "D"), 2);
  assert.equal(bewaardAls(lijst, titelkaart("Een"), "S"), null, "een ander Standpunt is een andere Preset");
  assert.equal(bewaardAls(lijst, titelkaart("Twee"), undefined), null);
});
