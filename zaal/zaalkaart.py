"""Schrijft de plattegrond van de Zaal als Rust-code: waar de Stoelen en de hanglampen staan.

De server rekent er de camera mee door (onder de hanglampen door, alle Stoelen in beeld) en het
dashboard tekent er de plattegrond mee. Draaien op de uitvoer van `maak.sh`:

    blender -b --factory-startup --python-exit-code 1 --python zaal/zaalkaart.py -- \
        /pad/naar/zaal/zaal.glb crates/gedeeld/src/zaalkaart.rs && rustfmt --edition 2024 crates/gedeeld/src/zaalkaart.rs

Assen van Bevy: y omhoog, de Voorzitter aan de kant van +z, vak 1 aan de kant van +x.
"""
import re
import sys

import bpy
from mathutils import Vector

glb, uit = sys.argv[sys.argv.index("--") + 1:]
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=glb)


def kader(o):
    """Kleinste en grootste hoek van een object, in de assen van Bevy."""
    punten = [o.matrix_world @ Vector(c) for c in o.bound_box]
    bevy = [(p.x, p.z, -p.y) for p in punten]
    return [min(p[i] for p in bevy) for i in range(3)], [max(p[i] for p in bevy) for i in range(3)]


STOELEN_PER_RIJ = [2, 3, 4, 5, 5, 6]
stoelen = []
for vak in range(1, 7):
    for rij, n in enumerate(STOELEN_PER_RIJ, start=1):
        for nummer in range(1, n + 1):
            mn, mx = kader(bpy.data.objects[f"Stoel_{vak}_{rij}_{nummer}"])
            # het midden van de bovenrand van de rugleuning, waar de kleur het best te zien is
            stoelen.append(((mn[0] + mx[0]) / 2, mx[1] - 0.15, (mn[2] + mx[2]) / 2))

lampen = []
for o in sorted(bpy.data.objects, key=lambda o: int(re.sub(r"\D", "", o.name) or 0)):
    if re.fullmatch(r"Hanglamp_\d+", o.name):
        mn, mx = kader(o)
        straal = max(mx[0] - mn[0], mx[2] - mn[2]) / 2
        lampen.append(((mn[0] + mx[0]) / 2, mn[1], (mn[2] + mx[2]) / 2, straal))
assert len(stoelen) == 150 and len(lampen) == 18, (len(stoelen), len(lampen))


def f(x):
    return f"{x:.3f}"


regels = [
    "//! Gemaakt door `zaal/zaalkaart.py` uit `zaal.glb`; niet met de hand wijzigen.",
    "//!",
    "//! Assen van Bevy in meter: y omhoog, de Voorzitter aan de kant van +z, vak 1 aan de kant van +x.",
    "",
    "/// Per Stoel, in de volgorde van [`crate::Stoel::alle`]: het midden van de bovenrand van de",
    "/// rugleuning.",
    "pub const STOELPLEKKEN: [[f32; 3]; 150] = [",
]
regels += [f"    [{f(x)}, {f(y)}, {f(z)}]," for x, y, z in stoelen]
regels += [
    "];",
    "",
    "/// De 18 hanglampen onder het daklicht: midden (x, z), onderkant (y) en straal.",
    "pub const HANGLAMPEN: [Hanglamp; 18] = [",
]
regels += [
    f"    Hanglamp {{ x: {f(x)}, onderkant: {f(y)}, z: {f(z)}, straal: {f(r)} }},"
    for x, y, z, r in lampen
]
regels += [
    "];",
    "",
    "#[derive(Clone, Copy, Debug, PartialEq)]",
    "pub struct Hanglamp {",
    "    pub x: f32,",
    "    pub onderkant: f32,",
    "    pub z: f32,",
    "    pub straal: f32,",
    "}",
    "",
]
open(uit, "w").write("\n".join(regels))
print(f"{uit}: {len(stoelen)} Stoelen, {len(lampen)} hanglampen")
