// Controle na de uitzending: `npm test` (node --test). De Stemmingen zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { controleLijst, minutenNaHamerslag, tellingen } from "./controle.ts";
import type { Controlestatus } from "../typen/Controlestatus";
import type { Keuze } from "../typen/Keuze";
import type { Stemming } from "../typen/Stemming";

function stemming(id: string, status: Controlestatus | null, extra: { in_ether_ms?: number; afgehandeld?: Keuze } = {}): Stemming {
  return {
    punt: { id, soort: "Motie", nummer: null, titel: id, indieners: null, agendapunt: null },
    met_de_hand: false,
    voor_uitzending: true,
    stemgedrag: { getikt: [], overige: null, algemene_stemmen: false, afwijkend: [], hoofdelijk: false },
    uitkomst: { voor: 0, tegen: 0, open: 150, besluit: null },
    vastgelegd: false,
    vergrendeld: false,
    in_ether_ms: extra.in_ether_ms,
    controle: {
      officieel: null,
      debat_direct_gezien: null,
      open_data_gezien: null,
      beoordeling: null,
      status,
      verschillen: [],
      officiele_uitkomst: null,
      afgehandeld: extra.afgehandeld ?? null,
    },
  };
}

test("de controle toont alleen ingevoerde Stemmingen, de ernstigste bovenaan en die in de Ether eerst", () => {
  const lijst = [
    stemming("gelijk", "Gelijk"),
    stemming("niet-ingevoerd", null),
    stemming("wacht", "WachtOpOfficieel"),
    stemming("anders", "UitkomstAnders"),
    stemming("anders-in-ether", "UitkomstAnders", { in_ether_ms: 1 }),
    stemming("fractie", "VerschilPerFractie"),
    stemming("afgehandeld", "UitkomstAnders", { afgehandeld: "OnzeInvoerKlopt" }),
  ];
  assert.deepEqual(
    controleLijst(lijst).map((s) => s.punt.id),
    ["anders-in-ether", "anders", "fractie", "wacht", "gelijk", "afgehandeld"],
  );
});

test("de tellingen per status slaan afgehandelde verschillen over", () => {
  const lijst = [
    stemming("a", "UitkomstAnders"),
    stemming("b", "UitkomstAnders", { afgehandeld: "OnzeInvoerKlopt" }),
    stemming("c", "Gelijk"),
    stemming("d", "Gelijk", { afgehandeld: "OfficieelOvergenomen" }),
    stemming("e", null),
  ];
  assert.deepEqual(tellingen(lijst), { UitkomstAnders: 1, VerschilPerFractie: 0, WachtOpOfficieel: 0, Gelijk: 2, afgehandeld: 1 });
});

test("de vertraging van Debat Direct in minuten na de hamerslag", () => {
  const s = stemming("a", "Gelijk");
  assert.equal(minutenNaHamerslag(s.controle), null);
  s.controle.debat_direct_gezien = "2026-11-24T17:14:30+01:00";
  s.controle.officieel = {
    bron: "DebatDirect",
    besluit: "Verworpen",
    hoofdelijk: false,
    fracties: [],
    afwijkend: [],
    hamerslag: "2026-11-24T15:06:44+01:00",
  };
  assert.equal(minutenNaHamerslag(s.controle), 127);
});
