import assert from "node:assert/strict";
import { test } from "node:test";
import { biedHerstel, herstel, vergeetHerstel } from "./herstel.ts";

test("herstel voert het laatste aanbod één keer uit en is daarna weg", () => {
  const gedaan: string[] = [];
  biedHerstel("Uit beeld: Titelkaart", "Terug in beeld", () => gedaan.push("eerste"));
  biedHerstel("Alle stemmen gewist", "Zet stemmen terug", () => gedaan.push("tweede"));
  herstel();
  herstel();
  assert.deepEqual(gedaan, ["tweede"]);
});

test("na vergeten doet herstel niets", () => {
  let gedaan = false;
  biedHerstel("Coalitie gewist", "Zet coalitie terug", () => (gedaan = true));
  vergeetHerstel();
  herstel();
  assert.equal(gedaan, false);
});
