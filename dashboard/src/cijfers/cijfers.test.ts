// Het onderdeel Cijfers: `npm test` (node --test). Alle cijfers hier zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { kopVergelijking, rijen, ruimOp, verdeel, zichtbaar } from "./cijfers.ts";
import type { Cijfers } from "../typen/Cijfers";
import type { Cijferset } from "../typen/Cijferset";
import type { Partij } from "../typen/Partij";

const partij = (sleutel: string, extra: Partial<Partij> = {}): Partij => ({
  sleutel,
  naam: sleutel,
  kort: sleutel.toUpperCase(),
  kleur: "#808080",
  kleur_tijdelijk: false,
  logo: null,
  fractie_zonder_lijst: false,
  voorgangers: [],
  ...extra,
});
const set = (id: string, label: string, zetels: Record<string, number>): Cijferset => ({
  id, soort: "exitpoll", label, tijd: null, bron: null, zetels, stemmen: null, vergrendeld: false, versie: 1,
});

// Vorige keer Alfa 60, Beta 50, Gamma 30, Rood 6 en Groen 4. Nu is Groenrood de opvolger van Rood
// en Groen, is Delta nieuw, doet Gamma niet meer mee en is Lid Eps een Fractie zonder lijst.
const cijfers: Cijfers = {
  verkiezing: null,
  partijen: [
    partij("alfa"), partij("beta"), partij("gamma"), partij("rood"), partij("groen"),
    partij("groenrood", { voorgangers: ["rood", "groen"] }), partij("delta"),
    partij("eps", { fractie_zonder_lijst: true }),
  ],
  sets: [
    set("vorige", "Uitslag 2025", { alfa: 60, beta: 50, gamma: 30, rood: 6, groen: 4 }),
    set("exit", "Exitpoll", { alfa: 62, beta: 47, groenrood: 15, delta: 26 }),
  ],
  vergelijk_met: "vorige",
  laatste_invoer: null,
};

test("rijen: een opgegane Voorganger en een Fractie zonder lijst zonder Zetels staan er niet; de opvolger telt de Voorgangers op", () => {
  const r = rijen(cijfers, "exit").map((x) => [x.partij.sleutel, x.nu, x.eerder, x.verschil, x.nieuw]);
  assert.deepEqual(r, [
    ["alfa", 62, 60, 2, false],
    ["beta", 47, 50, -3, false],
    ["gamma", 0, 30, -30, false],
    ["groenrood", 15, 10, 5, false],
    ["delta", 26, 0, 26, true],
  ]);
});

test("rijen zonder vergelijkingsset: geen 2025 en geen verschil", () => {
  const r = rijen({ ...cijfers, vergelijk_met: null }, "exit");
  assert.deepEqual(r.map((x) => [x.eerder, x.verschil]), r.map(() => [null, null]));
});

test("kop van de vergelijkingskolom: het jaartal uit de naam, anders de naam", () => {
  assert.equal(kopVergelijking(cijfers), "2025");
  assert.equal(kopVergelijking({ ...cijfers, vergelijk_met: "exit" }), "Exitpoll");
  assert.equal(kopVergelijking({ ...cijfers, vergelijk_met: null }), null);
});

test("verdeel: de eerste helft links, de rest rechts, in dezelfde volgorde", () => {
  assert.deepEqual(verdeel([1, 2, 3, 4, 5]), [[1, 2, 3], [4, 5]]);
  assert.deepEqual(verdeel([]), [[], []]);
  assert.deepEqual(verdeel([1, 2, 3, 4, 5, 6, 7], 3), [[1, 2, 3], [4, 5, 6], [7]]);
});

test("eigen wijziging: het vakje toont de eigen waarde tot de Stand van het antwoord er is", () => {
  let concept = { "exit/alfa": { zetels: 63, tot: null as number | null } };
  assert.equal(zichtbaar(concept, "exit", "alfa", 62), 63, "nog geen antwoord: eigen waarde");
  concept = { "exit/alfa": { zetels: 63, tot: 41 } };
  assert.equal(zichtbaar(ruimOp(concept, 40), "exit", "alfa", 62), 63, "Stand van voor het antwoord");
  // In Stand 41 kan een andere Werkplek het vakje al hebben overschreven: dan telt die.
  assert.equal(zichtbaar(ruimOp(concept, 41), "exit", "alfa", 64), 64);
});

test("de rijen volgen de volgorde van de Uitslagindeling; een onbekende Partij staat achteraan", () => {
  const volgorde = { partijen: ["delta", "groenrood", "beta", "alfa"], niet_op_lijst: [] };
  const r = rijen(cijfers, "exit", volgorde).map((x) => x.partij.sleutel);
  assert.deepEqual(r, ["delta", "groenrood", "beta", "alfa", "gamma"]);
});
