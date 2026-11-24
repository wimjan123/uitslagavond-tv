// Cameraritten in het dashboard: `npm test` (node --test).
import { test } from "node:test";
import assert from "node:assert/strict";
import { cameraKeuzes, kanHerhalen, opnameTijd, ritTiming, routeVan, stukS } from "./camerarit.ts";
import type { Camerarit } from "../typen/Camerarit";
import type { Standpunt } from "../typen/Standpunt";

/** Een opname van `sec` seconden, 100 standen per seconde (voorbeeldgegevens). */
function rit(sec: number, plek: (t: number) => [number, number, number], extra: Partial<Camerarit> = {}): Camerarit {
  const opname: Camerarit["opname"] = [];
  for (let i = 0; i <= Math.round(sec * 100); i++) opname.push([...plek(i / 100), 0, -0.1, 0.9]);
  return {
    letter: "R1",
    naam: "Camerarit 1",
    opname,
    bijschaving: {
      gladheid: 0.2,
      duur_s: sec,
      begin_eraf_s: 0,
      eind_eraf_s: 0,
      zacht_starten: 0,
      zacht_stoppen: 0,
      begin: "Overgang",
      eind: "Stoppen",
    },
    ...extra,
  };
}

const rechtdoor = () => rit(10, (t) => [t, 3, -4]);
const rond = () =>
  rit(8, (t) => {
    const a = (t / 8) * 2 * Math.PI;
    return [2 * Math.sin(a), 3, -4 + 2 * (1 - Math.cos(a))];
  });

test("herhalen staat uit met dezelfde reden als in de server", () => {
  assert.equal(kanHerhalen(rechtdoor()), "begin en eind liggen 10,0 m uit elkaar");
  assert.equal(kanHerhalen(rond()), null);
  const half = rond();
  half.bijschaving.eind_eraf_s = 4;
  assert.equal(kanHerhalen(half), "begin en eind liggen 4,0 m uit elkaar");
  const gedraaid = rond();
  gedraaid.opname[gedraaid.opname.length - 1][3] = 0.9;
  assert.equal(kanHerhalen(gedraaid), "begin en eind kijken 52° anders");
});

test("de route op de plattegrond loopt van begin eraf tot eind eraf", () => {
  const r = rechtdoor();
  r.bijschaving.begin_eraf_s = 2;
  r.bijschaving.eind_eraf_s = 3;
  assert.equal(stukS(r), 5);
  const route = routeVan(r, 20);
  assert.ok(route.length <= 21, `${route.length} punten`);
  assert.deepEqual(route[0], [2, -4]);
  assert.deepEqual(route[route.length - 1], [7, -4]);
});

test("de lijst toont de duur en wat de rit aan het eind doet", () => {
  const r = rond();
  r.bijschaving.duur_s = 16;
  r.bijschaving.eind = "Herhalen";
  assert.equal(ritTiming(r), "16,0 s · herhaalt");
  assert.equal(ritTiming(rechtdoor()), "10,0 s");
  assert.equal(opnameTijd(1234), "12,3 s");
});

test("waar je een Standpunt kiest, kies je ook een Camerarit, met toets en naam", () => {
  const vast: Standpunt = {
    letter: "A",
    naam: "Totaal vanaf voorzitter",
    vast: true,
    stand: { positie: [0, 4.5, 4.45], draai: 0, helling: -0.4, beeldhoek: 1.7 },
    overgang: { soort: "Bewegen", duur_s: 2, zacht_starten: 1, zacht_stoppen: 1 },
    na_aankomst: { soort: "Stil" },
  };
  const eigen: Standpunt = { ...vast, letter: "E1", naam: "Close-up", vast: false, toets: "J" };
  const zonderToets: Standpunt = { ...vast, letter: "E2", naam: "Achter", vast: false };
  const keuzes = cameraKeuzes([vast, eigen, zonderToets], [rit(4, (t) => [t, 3, -4], { toets: "K" })]);
  assert.deepEqual(
    keuzes.map((k) => [k.letter, k.toets, k.tekst, k.soort]),
    [
      ["A", "A", "A · Totaal vanaf voorzitter", "Standpunt"],
      ["E1", "J", "J · Close-up", "Standpunt"],
      ["E2", null, "Achter", "Standpunt"],
      ["R1", "K", "K · Camerarit 1", "Camerarit"],
    ],
  );
});
