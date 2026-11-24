// Invoer van Stemmingen: `npm test` (node --test). De Fracties zijn voorbeeldgegevens.
import { test } from "node:test";
import assert from "node:assert/strict";
import { LEEG, afwijkend, algemeen, overige, tik, volgendeOpen, voorzitterZin, wis } from "./invoer.ts";
import type { Stemgedrag } from "../typen/Stemgedrag";
import type { Fractie } from "../typen/Fractie";
import type { Stemming } from "../typen/Stemming";

const fracties: Fractie[] = [
  { id: "f-pvv", afkorting: "PVV", naam: "PVV", zetels: 40 },
  { id: "f-sp", afkorting: "SP", naam: "SP", zetels: 20 },
  { id: "f-pvdd", afkorting: "PvdD", naam: "PvdD", zetels: 3 },
  { id: "f-vvd", afkorting: "VVD", naam: "VVD", zetels: 30 },
];
const namen = { "k-1": "Dieke van Groningen (VVD)" };
const tekst = (sg: Stemgedrag) => voorzitterZin(sg, fracties, namen).map((d) => d.tekst).join("");

test("tikken zet Fracties in de volgorde van de Voorzitter; nog eens tikken haalt weg of wisselt", () => {
  let sg = tik(LEEG, "f-sp", "Voor");
  sg = tik(sg, "f-pvdd", "Voor");
  assert.deepEqual(sg.getikt, [
    { fractie: "f-sp", stem: "Voor" },
    { fractie: "f-pvdd", stem: "Voor" },
  ]);
  assert.deepEqual(tik(sg, "f-sp", "Voor").getikt, [{ fractie: "f-pvdd", stem: "Voor" }]);
  assert.deepEqual(tik(sg, "f-sp", "Tegen").getikt[0], { fractie: "f-sp", stem: "Tegen" });
});

test("tikken na algemene stemmen begint opnieuw", () => {
  const sg = tik(algemeen(LEEG), "f-sp", "Tegen");
  assert.equal(sg.algemene_stemmen, false);
  assert.deepEqual(sg.getikt, [{ fractie: "f-sp", stem: "Tegen" }]);
});

test("Overige vult de andere kant dan waarop je tikt, en nog eens drukken haalt het weg", () => {
  const sg = overige(tik(LEEG, "f-sp", "Voor"), "Voor");
  assert.equal(sg.overige, "Tegen");
  assert.equal(overige(sg, "Voor").overige, null);
  assert.equal(overige(LEEG, "Tegen").overige, "Voor");
});

test("met algemene stemmen wist de rest, maar houdt hoofdelijk", () => {
  const sg = algemeen({ ...tik(LEEG, "f-sp", "Voor"), hoofdelijk: true });
  assert.deepEqual(sg, { ...LEEG, algemene_stemmen: true, hoofdelijk: true });
  assert.equal(algemeen(sg).algemene_stemmen, false);
  assert.deepEqual(wis({ ...sg }), { ...LEEG, hoofdelijk: true });
});

test("een afwijkende stem: toevoegen, wisselen, weghalen", () => {
  let sg = afwijkend(LEEG, "k-1", "Voor");
  assert.deepEqual(sg.afwijkend, [{ kamerlid: "k-1", stem: "Voor" }]);
  sg = afwijkend(sg, "k-1", "Tegen");
  assert.deepEqual(sg.afwijkend, [{ kamerlid: "k-1", stem: "Tegen" }]);
  assert.deepEqual(afwijkend(sg, "k-1", null).afwijkend, []);
});

test("de zin herhaalt de invoer in de woorden van de Voorzitter", () => {
  assert.equal(tekst(LEEG), "");
  let sg = overige(tik(tik(LEEG, "f-sp", "Voor"), "f-pvdd", "Voor"), "Voor");
  assert.equal(tekst(sg), "de leden van de fracties van SP en PvdD voor en de overige fracties ertegen");
  sg = tik(tik(LEEG, "f-pvv", "Voor"), "f-vvd", "Tegen");
  assert.equal(tekst(sg), "de leden van de fractie van PVV voor en de leden van de fractie van VVD ertegen");
  sg = overige(tik(tik(tik(LEEG, "f-sp", "Tegen"), "f-pvv", "Tegen"), "f-pvdd", "Tegen"), "Tegen");
  assert.equal(tekst(sg), "de leden van de fracties van SP, PVV en PvdD ertegen en de overige fracties ervoor");
  assert.equal(tekst(algemeen(LEEG)), "met algemene stemmen aangenomen");
  sg = afwijkend(overige(tik(LEEG, "f-sp", "Voor"), "Voor"), "k-1", "Voor");
  assert.equal(
    tekst(sg),
    "de leden van de fractie van SP voor en de overige fracties ertegen, met uitzondering van het lid Dieke van Groningen (VVD), dat voor stemde",
  );
});

test("in de zin dragen de Fractienamen hun stem, voor de kleur", () => {
  const delen = voorzitterZin(overige(tik(LEEG, "f-sp", "Voor"), "Voor"), fracties, namen);
  assert.deepEqual(delen.filter((d) => d.stem).map((d) => [d.tekst, d.stem]), [
    ["SP", "Voor"],
    ["voor", "Voor"],
    ["ertegen", "Tegen"],
  ]);
});

function stemming(id: string, besluit: boolean, vastgelegd = false): Stemming {
  return {
    punt: { id, soort: "Motie", nummer: null, titel: id, indieners: null, agendapunt: null },
    met_de_hand: false,
    voor_uitzending: true,
    stemgedrag: LEEG,
    uitkomst: { voor: 0, tegen: 0, open: besluit ? 0 : 150, besluit: besluit ? "Aangenomen" : null },
    vastgelegd,
    vergrendeld: false,
    controle: {
      officieel: null,
      debat_direct_gezien: null,
      open_data_gezien: null,
      beoordeling: null,
      status: null,
      verschillen: [],
      officiele_uitkomst: null,
      afgehandeld: null,
    },
  };
}

test("volgende open motie: de eerste na deze zonder vastgelegde uitslag, en anders vooraan verder", () => {
  const lijst = [stemming("a", false), stemming("b", true, true), stemming("c", true), stemming("d", false)];
  assert.equal(volgendeOpen(lijst, "a"), "c");
  assert.equal(volgendeOpen(lijst, "c"), "d");
  assert.equal(volgendeOpen(lijst, "d"), "a");
  assert.equal(volgendeOpen([stemming("a", true, true)], "a"), null);
});
