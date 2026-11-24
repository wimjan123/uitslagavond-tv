// De status van Server, Ether, Reserve en Voorbeeld in het onderdeel Systeem: `npm test`.
import { test } from "node:test";
import assert from "node:assert/strict";
import { statusVan } from "./systeem.ts";
import type { Renderermelding } from "./systeem.ts";

const melding = (m: Partial<Renderermelding> = {}): Renderermelding => ({
  versie: "0.1.0+4a46080",
  verbonden: [],
  geweigerd: [],
  ...m,
});

test("een Renderer met een andere versie staat er als geweigerd, met zijn machine en versie", () => {
  const regels = statusVan("verbonden", melding({ geweigerd: [{ rol: "Reserve", machine: "studio-reserve", versie: "0.1.0+0000000" }] }));
  const reserve = regels.find((r) => r.naam === "Reserve")!;
  assert.equal(reserve.toon, "let-op");
  assert.match(reserve.status, /andere versie/i);
  assert.deepEqual(reserve.machines, [{ machine: "studio-reserve", versie: "0.1.0+0000000", geweigerd: true }]);
});

test("verbonden Renderers zijn goed; een rol zonder Renderer vraagt aandacht", () => {
  const regels = statusVan("verbonden", melding({ verbonden: [{ rol: "Ether", machine: "studio-ether", versie: "0.1.0+4a46080" }] }));
  const naar = Object.fromEntries(regels.map((r) => [r.naam, r]));
  assert.deepEqual(Object.keys(naar), ["Server", "Ether", "Reserve", "Voorbeeld"]);
  assert.equal(naar.Server.toon, "goed");
  assert.equal(naar.Server.machines[0].versie, "0.1.0+4a46080");
  assert.equal(naar.Ether.toon, "goed");
  assert.equal(naar.Ether.machines[0].machine, "studio-ether");
  assert.equal(naar.Reserve.toon, "let-op");
  assert.equal(naar.Reserve.status, "Niet verbonden");
});

test("een tweede machine met een andere versie naast een goede Ether blijft zichtbaar", () => {
  const regels = statusVan(
    "verbonden",
    melding({
      verbonden: [{ rol: "Ether", machine: "studio-ether", versie: "0.1.0+4a46080" }],
      geweigerd: [{ rol: "Ether", machine: "studio-oud", versie: "0.1.0+0000000" }],
    }),
  );
  const ether = regels.find((r) => r.naam === "Ether")!;
  assert.equal(ether.toon, "let-op");
  assert.equal(ether.machines.length, 2);
});

test("zonder server is de status van de Renderers onbekend", () => {
  const regels = statusVan("geen verbinding", melding({ verbonden: [{ rol: "Ether", machine: "studio-ether", versie: "0.1.0+4a46080" }] }));
  const naar = Object.fromEntries(regels.map((r) => [r.naam, r]));
  assert.equal(naar.Server.toon, "fout");
  assert.equal(naar.Ether.toon, "onbekend");
  assert.deepEqual(naar.Ether.machines, []);
});
