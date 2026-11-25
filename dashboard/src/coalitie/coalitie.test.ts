// De Coalitiebouwer in het dashboard: `npm test` (node --test). Alle cijfers hier zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { HUIDIGE_KAMER, boog, bouwer, coalitieInBeeld, meerderheidSlaatOm, nuVanLive, regel, telling, wissel, zetelsVan } from "./coalitie.ts";
import type { Graphic } from "../typen/Graphic";
import type { Stand } from "../typen/Stand";
import { STANDAARD_STIJL } from "../typen/standaardStijl.ts";

const stand = (extra: Partial<Stand> = {}): Stand =>
  ({
    cijfers: {
      verkiezing: null,
      partijen: [],
      sets: [{ id: "exit", soort: "exitpoll", label: "Exitpoll", tijd: null, bron: null, zetels: { d66: 26, cda: 18, vvd: 22, ja21: 9 }, stemmen: null, vergrendeld: false, versie: 1 }],
      vergelijk_met: null,
      laatste_invoer: null,
    },
    kamer: {
      peildatum: "2026-11-22",
      kamerleden: [],
      fracties: [
        { id: "f1", afkorting: "D66", naam: "D66", zetels: 26 },
        { id: "f2", afkorting: "VVD", naam: "VVD", zetels: 22 },
        { id: "f3", afkorting: "Groep X", naam: "Groep X", zetels: 1 },
      ],
      partij_van: { f1: "d66", f2: "vvd", f3: "vvd" },
      laatste_ophaal: null,
      vertrokken: [],
    },
    klaar: null,
    ether: null,
    ...extra,
  }) as unknown as Stand;

test("de teller telt de Zetels van de gekozen Partijen samen", () => {
  const s = stand();
  assert.equal(telling(zetelsVan(s, "exit"), ["d66", "cda", "vvd", "ja21"]), 75);
  assert.equal(telling(zetelsVan(s, "exit"), []), 0);
  assert.equal(telling(zetelsVan(s, "exit"), ["d66", "d66"]), 26, "dubbel telt één keer");
});

test("de huidige Kamer telt de Zetels van elke Fractie bij haar Partij", () => {
  assert.deepEqual(zetelsVan(stand(), HUIDIGE_KAMER), { d66: 26, vvd: 23 });
});

test("onder de teller: nog N Zetels tot een meerderheid, of meerderheid", () => {
  assert.equal(regel(75), "nog 1 Zetel tot een meerderheid");
  assert.equal(regel(0), "nog 76 Zetels tot een meerderheid");
  assert.equal(regel(76), "meerderheid");
  assert.equal(regel(120), "meerderheid");
});

test("een Partij aantikken zet haar erbij of haalt haar eraf", () => {
  assert.deepEqual(wissel(["d66"], "vvd"), ["d66", "vvd"]);
  assert.deepEqual(wissel(["d66", "vvd"], "d66"), ["vvd"]);
});

test("de coalitie in de Ether gaat voor die die klaarstaat", () => {
  const coalitie = (partijen: string[]): Graphic => ({ sjabloon: "Coalitie", set: "exit", partijen });
  const inEther = { id: 1, versie: 1, graphic: coalitie(["d66"]), getoond: 1, sinds_ms: 0, getoond_door: "Regie", stijl: STANDAARD_STIJL };
  const klaar = { id: 2, versie: 2, graphic: coalitie(["vvd"]), klaargezet_door: "Regie", gewijzigd_door: "Regie" };
  assert.equal(coalitieInBeeld(stand()), null);
  assert.deepEqual(coalitieInBeeld(stand({ klaar })), coalitie(["vvd"]));
  assert.deepEqual(coalitieInBeeld(stand({ klaar, ether: inEther })), coalitie(["d66"]));
  const titel = { ...klaar, graphic: { sjabloon: "Titelkaart", kop: "x", regel: "" } as Graphic };
  assert.equal(coalitieInBeeld(stand({ klaar: titel })), null, "een Titelkaart is geen coalitie");
});

test("de Partijen liggen van links naar rechts op een halve cirkel", () => {
  const plekken = boog(3, 100);
  assert.equal(plekken.length, 3);
  assert.deepEqual(plekken.map((p) => Math.round(p.x)), [-100, 0, 100]);
  assert.deepEqual(plekken.map((p) => Math.round(p.y)), [0, -100, 0]);
  assert.deepEqual(boog(1, 100).map((p) => [Math.round(p.x), Math.round(p.y)]), [[0, -100]]);
});

test("de bouwer toont de coalitie in beeld met haar eigen cijfers, in de volgorde van de Zaal", () => {
  const partij = (sleutel: string) => ({ sleutel, naam: sleutel, kort: sleutel.toUpperCase(), kleur: "#888888" });
  const basis = { uitslagindeling: { partijen: ["ja21", "vvd", "cda", "d66"] }, cijfers: { ...stand().cijfers, partijen: ["d66", "cda", "vvd", "ja21"].map(partij) } };
  // zonder coalitie in beeld: de eigen keuze met de cijfers van de gekozen set
  const zelf = bouwer(stand(basis as Partial<Stand>), "exit", ["d66"]);
  assert.equal(zelf.inBeeld, null);
  assert.deepEqual(zelf.volgorde.map((p) => p.sleutel), ["ja21", "vvd", "cda", "d66"]);
  assert.equal(zelf.totaal, 26);
  // in beeld: de Partijen en de kopie van de cijfers uit de Graphic; een Partij zonder Zetels valt weg
  const cijfers = { set: { ...stand().cijfers.sets[0], zetels: { d66: 30, vvd: 50, cda: 0 } }, partijen: [] };
  const klaar = { id: 2, versie: 2, graphic: { sjabloon: "Coalitie", set: "exit", partijen: ["vvd", "d66"], cijfers } as Graphic, klaargezet_door: "Regie", gewijzigd_door: "Regie" };
  const b = bouwer(stand({ ...(basis as Partial<Stand>), klaar }), "exit", ["cda"]);
  assert.deepEqual(b.partijen, ["vvd", "d66"]);
  assert.deepEqual(b.volgorde.map((p) => p.sleutel), ["vvd", "d66"]);
  assert.equal(b.totaal, 80);
});

test("een live coalitie: de cijfers van nu komen uit haar eigen set, en een omslag van de meerderheid telt", () => {
  const oud = { set: { id: "exit", zetels: { d66: 26, cda: 18, vvd: 22, ja21: 6 } } };
  const g = { sjabloon: "Coalitie", set: "exit", partijen: ["d66", "cda", "vvd", "ja21"], cijfers: oud } as unknown as Graphic;
  const s = stand({ ether: { id: 1, versie: 1, graphic: g, sinds_ms: 0 } } as unknown as Partial<Stand>);
  assert.deepEqual(nuVanLive(s), { live: 72, nu: 75 });
  assert.equal(meerderheidSlaatOm(s), false, "72 en 75 zijn allebei geen meerderheid");
  const s2 = stand({ ether: { id: 1, versie: 1, graphic: { ...g, partijen: ["d66", "cda", "vvd", "ja21", "vvd"] }, sinds_ms: 0 } } as unknown as Partial<Stand>);
  assert.equal(meerderheidSlaatOm(s2), false);
  const s3 = stand({
    ether: { id: 1, versie: 1, graphic: { ...g, cijfers: { set: { id: "exit", zetels: { d66: 26, cda: 18, vvd: 22, ja21: 6 } } } }, sinds_ms: 0 },
    cijfers: { ...stand().cijfers, sets: [{ ...stand().cijfers.sets[0], zetels: { d66: 26, cda: 18, vvd: 22, ja21: 10 } }] },
  } as unknown as Partial<Stand>);
  assert.deepEqual(nuVanLive(s3), { live: 72, nu: 76 });
  assert.equal(meerderheidSlaatOm(s3), true);
  assert.equal(nuVanLive(stand()), null, "zonder coalitie in Live");
});
