// De Persoonskaart in het dashboard: `npm test` (node --test). Alle personen hier zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { opStoel, plaatsVan, standpuntVoor, stoelOp, waarschuwingen, zoek } from "./persoonskaart.ts";
import type { Kamerlid } from "../typen/Kamerlid";
import type { Stand } from "../typen/Stand";

const lid = (id: string, roepnaam: string, achternaam: string, extra: Partial<Kamerlid> = {}): Kamerlid => ({
  id,
  roepnaam,
  initialen: "",
  tussenvoegsel: null,
  achternaam,
  fractie: "f-rood",
  zetel: `z-${id}`,
  fractievoorzitter: false,
  van: "2025-11-12",
  geboortedatum: null,
  geboorteplaats: null,
  woonplaats: null,
  foto: "OpenData",
  ...extra,
});

/** Drie Kamerleden en een eigen persoon; Noëlle zit op Stoel 2-3-1, Bram heeft geen Stoel en geen foto. */
const stand = {
  kamer: {
    kamerleden: [lid("noelle", "Noëlle", "Bénard"), lid("bram", "Bram", "Bes", { foto: "Geen" }), lid("dirk", "Dirk", "Dijk", { tussenvoegsel: "van" })],
    partij_van: { "f-rood": "rood" },
  },
  cijfers: { partijen: [{ sleutel: "rood", kort: "ROOD" }] },
  zaalindeling: {
    stoelen: [
      { vak: 2, rij: 3, nummer: 1, kamerlid: "noelle", naam: "" },
      { vak: 1, rij: 1, nummer: 1, kamerlid: "dirk", naam: "" },
    ],
  },
  personen: [{ id: "p7", naam: "Mila Verzonnen", foto: false, regel: "" }],
} as unknown as Stand;

test("de Stoelen hebben dezelfde volgorde als in de server", () => {
  assert.deepEqual(stoelOp(0), { vak: 1, rij: 1, nummer: 1 });
  assert.deepEqual(stoelOp(25), { vak: 2, rij: 1, nummer: 1 });
  assert.deepEqual(stoelOp(149), { vak: 6, rij: 6, nummer: 6 });
  assert.equal(stoelOp(150), null);
  for (let i = 0; i < 150; i++) assert.equal(plaatsVan(stoelOp(i)!), i);
});

test("zoeken vindt op een deel van de naam of de Partij, zonder op accenten te letten", () => {
  assert.deepEqual(zoek(stand, "noelle").map((k) => k.naam), ["Noëlle Bénard"]);
  assert.deepEqual(zoek(stand, "dijk").map((k) => k.naam), ["Dirk van Dijk"]);
  assert.deepEqual(zoek(stand, "rood b").map((k) => k.naam), ["Noëlle Bénard", "Bram Bes"]);
  assert.deepEqual(zoek(stand, "mila").map((k) => k.wie), [{ soort: "Eigen", id: "p7" }]);
  assert.deepEqual(zoek(stand, "  "), []);
});

test("een klik op een Stoel kiest het Kamerlid dat er zit", () => {
  assert.equal(opStoel(stand, plaatsVan({ vak: 2, rij: 3, nummer: 1 }))?.naam, "Noëlle Bénard");
  assert.equal(opStoel(stand, plaatsVan({ vak: 3, rij: 1, nummer: 1 })), null, "een lege Stoel");
});

test("het dashboard waarschuwt bij een persoon zonder foto of een Kamerlid zonder Stoel", () => {
  assert.deepEqual(waarschuwingen(stand, { soort: "Kamerlid", id: "noelle" }), []);
  const bram = waarschuwingen(stand, { soort: "Kamerlid", id: "bram" });
  assert.equal(bram.length, 2);
  assert.match(bram[0], /Geen foto/);
  assert.match(bram[1], /Geen Stoel/);
  // een eigen persoon heeft nooit een Stoel: alleen de foto telt
  assert.deepEqual(waarschuwingen(stand, { soort: "Eigen", id: "p7" }).length, 1);
});

test("de camera gaat naar het shot van de Stoel, of zonder Stoel naar het Standpunt van het Sjabloon", () => {
  assert.equal(standpuntVoor(stand, { soort: "Kamerlid", id: "noelle" }), "stoel-2-3-1");
  assert.equal(standpuntVoor(stand, { soort: "Kamerlid", id: "bram" }), "S");
  assert.equal(standpuntVoor(stand, { soort: "Eigen", id: "p7" }), "S");
});
