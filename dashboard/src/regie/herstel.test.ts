import assert from "node:assert/strict";
import { test } from "node:test";
import { biedHerstel, herstel, vergeetHerstel } from "./herstel.ts";
import type { Regie } from "./regie.ts";
import type { Stand } from "../typen/Stand.ts";

/** Een Regie met alleen de Stand; meer heeft herstel niet nodig. */
const regie = (stand: Partial<Stand>) => ({ stand: { stand } }) as unknown as Regie;

test("herstel voert alleen het laatste aanbod uit, één keer", () => {
  const gedaan: string[] = [];
  biedHerstel({ tekst: "Uit beeld", wat: "Titelkaart", knop: "Terug in beeld", geldig: () => true, doe: () => gedaan.push("eerste") });
  biedHerstel({ tekst: "Alle stemmen gewist", wat: "Motie", knop: "Zet terug", geldig: () => true, doe: () => gedaan.push("tweede") });
  assert.equal(herstel(regie({})), true);
  assert.equal(herstel(regie({})), false);
  assert.deepEqual(gedaan, ["tweede"]);
});

test("herstel doet niets als de Stand intussen veranderde, zoals een nieuwe Graphic in de Ether", () => {
  let gedaan = false;
  biedHerstel({
    tekst: "Uit beeld",
    wat: "Zetelverdeling",
    knop: "Terug in beeld",
    geldig: (s) => s.ether === null,
    doe: () => (gedaan = true),
  });
  const nuIetsInBeeld = regie({ ether: { id: 9 } } as unknown as Partial<Stand>);
  assert.equal(herstel(nuIetsInBeeld), false);
  assert.equal(gedaan, false);
});

test("na vergeten doet herstel niets", () => {
  let gedaan = false;
  biedHerstel({ tekst: "Coalitie gewist", wat: "Coalitie", knop: "Zet terug", geldig: () => true, doe: () => (gedaan = true) });
  vergeetHerstel();
  assert.equal(herstel(regie({})), false);
  assert.equal(gedaan, false);
});
