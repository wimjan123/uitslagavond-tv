// Sneltoetsen: `npm test` (node --test).
import { test } from "node:test";
import assert from "node:assert/strict";
import { actieVoorToets, standpuntVoorToets } from "./toetsen.ts";
import type { Toets } from "./toetsen.ts";

const toets = (key: string, extra: Partial<Toets> = {}): Toets => ({
  key,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  doel: "elders",
  ...extra,
});

test("spatie toont, B werkt bij, X haalt uit beeld, Z zet de kleur uit", () => {
  assert.equal(actieVoorToets(toets(" ")), "Toon");
  assert.equal(actieVoorToets(toets("b")), "Bijwerken");
  assert.equal(actieVoorToets(toets("B")), "Bijwerken");
  assert.equal(actieVoorToets(toets("x")), "UitBeeld");
  assert.equal(actieVoorToets(toets("z")), "KleurUit");
});

test("Enter toont nooit iets", () => {
  assert.equal(actieVoorToets(toets("Enter")), null);
  assert.equal(actieVoorToets(toets("Enter", { doel: "knop" })), null);
});

test("spatie op een knop drukt alleen die knop", () => {
  assert.equal(actieVoorToets(toets(" ", { doel: "knop" })), null);
  assert.equal(actieVoorToets(toets("x", { doel: "knop" })), "UitBeeld");
});

test("in een tekstveld typen de toetsen gewoon", () => {
  for (const key of [" ", "b", "x"]) {
    assert.equal(actieVoorToets(toets(key, { doel: "tekstveld" })), null);
  }
});

test("met Cmd, Ctrl of Alt doet een toets niets", () => {
  assert.equal(actieVoorToets(toets(" ", { ctrlKey: true })), null);
  assert.equal(actieVoorToets(toets("b", { metaKey: true })), null);
  assert.equal(actieVoorToets(toets("x", { altKey: true })), null);
});

test("A tot H kiezen een Standpunt, met Shift knipt de camera", () => {
  const letters = ["a", "s", "d", "f", "g", "h"];
  for (const l of letters) {
    assert.deepEqual(actieVoorToets(toets(l)), { soort: "Standpunt", toets: l.toUpperCase(), knip: false });
    // met Shift komt de hoofdletter binnen
    assert.deepEqual(actieVoorToets(toets(l.toUpperCase(), { shiftKey: true })), { soort: "Standpunt", toets: l.toUpperCase(), knip: true });
  }
  // Caps Lock zonder Shift knipt niet
  assert.deepEqual(actieVoorToets(toets("G")), { soort: "Standpunt", toets: "G", knip: false });
});

test("een Standpunt-toets werkt ook als de focus op een knop staat, maar niet in een tekstveld", () => {
  assert.deepEqual(actieVoorToets(toets("d", { doel: "knop" })), { soort: "Standpunt", toets: "D", knip: false });
  assert.equal(actieVoorToets(toets("d", { doel: "tekstveld" })), null);
  assert.equal(actieVoorToets(toets("g", { ctrlKey: true })), null);
});

test("J, K en L kiezen een eigen Standpunt, V zet de Vrije camera aan of uit", () => {
  assert.deepEqual(actieVoorToets(toets("j")), { soort: "Standpunt", toets: "J", knip: false });
  assert.deepEqual(actieVoorToets(toets("L", { shiftKey: true })), { soort: "Standpunt", toets: "L", knip: true });
  assert.equal(actieVoorToets(toets("v")), "VrijeCamera");
  assert.equal(actieVoorToets(toets("v", { doel: "tekstveld" })), null);
});

test("1 tot 9 zetten de Preset met dat nummer klaar, ook met de focus op een knop", () => {
  for (let n = 1; n <= 9; n++) {
    assert.deepEqual(actieVoorToets(toets(String(n))), { soort: "Preset", nummer: n });
  }
  assert.deepEqual(actieVoorToets(toets("4", { doel: "knop" })), { soort: "Preset", nummer: 4 });
  assert.equal(actieVoorToets(toets("0")), null, "0 hoort bij geen Preset");
  assert.equal(actieVoorToets(toets("3", { doel: "tekstveld" })), null);
  assert.equal(actieVoorToets(toets("3", { ctrlKey: true })), null);
});

test("O zet Oefenen aan of uit en R neemt op; B blijft Bijwerken", () => {
  assert.equal(actieVoorToets(toets("o")), "Oefenen");
  assert.equal(actieVoorToets(toets("R")), "Opnemen");
  assert.equal(actieVoorToets(toets("b")), "Bijwerken");
  assert.equal(actieVoorToets(toets("r", { doel: "tekstveld" })), null);
});

test("andere letters doen niets", () => {
  for (const key of ["i", "p", "Enter"]) assert.equal(actieVoorToets(toets(key)), null);
});

test("een toets hoort bij het Standpunt dat hem nu heeft", () => {
  const stand = { positie: [0, 2, 0] as [number, number, number], draai: 0, helling: 0, beeldhoek: 1 };
  const p = (letter: string, vast: boolean, t?: string) => ({ letter, naam: letter, vast, toets: t, stand, overgang: undefined as never, na_aankomst: undefined as never });
  const standpunten = [p("A", true), p("E1", false), p("E2", false, "J"), p("E3", false, "K")];
  assert.equal(standpuntVoorToets(standpunten, "A")?.letter, "A");
  assert.equal(standpuntVoorToets(standpunten, "J")?.letter, "E2");
  assert.equal(standpuntVoorToets(standpunten, "L"), null);
  // de sleutel van een eigen Standpunt is geen toets
  assert.equal(standpuntVoorToets(standpunten, "E1"), null);
  // een Camerarit met L kies je met L
  const rit = { letter: "R1", naam: "Rit", toets: "L" };
  assert.equal(standpuntVoorToets([...standpunten, rit], "L")?.letter, "R1");
});
