"""Tekent de onzichtbare ruimte van de camera naast het model en schrijft hem voor de server.

De ruimte is waar de camera mag vliegen: vloer net boven de banken, wanden binnen de muren,
plafond onder de lampen (#23, #44). Losse voorwerpen zoals microfoons en schermen tellen niet mee.
Het script meet de Zaal op met stralen in Blender, tekent de ruimte als object
`Onzichtbare_ruimte` naast het model en schrijft:

- `<json>`: de maten (`crates/gedeeld/src/ruimte.json`). De server rekent ermee, het dashboard
  tekent de wanden op de plattegrond;
- `<uitmap>/ruimte.blend`: het model met de ruimte erin, om te bekijken in Blender;
- `<uitmap>/ruimte-boven.png` en `ruimte-zij.png`: de ruimte over het model, van boven en opzij.

Draaien op de uitvoer van `maak.sh`:

    blender -b --factory-startup --python-exit-code 1 --python zaal/ruimte.py -- \\
        /pad/naar/zaal/zaal.glb crates/gedeeld/src/ruimte.json /pad/naar/uitmap

Assen van Bevy: y omhoog, de Voorzitter aan de kant van +z, vak 1 aan de kant van +x. Blender
heeft z omhoog: Bevy (x, y, z) is Blender (x, -z, y).
"""
import json
import math
import os
import re
import sys

import bmesh
import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

glb, uit_json, uitmap = sys.argv[sys.argv.index("--") + 1:]
os.makedirs(uitmap, exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=glb)

# Afstand tot de muren, en hoeveel ruimte er boven de banken en onder de lampen blijft.
WANDMARGE = 0.1
VRIJ_BOVEN_BANKEN = 0.3
VRIJ_ONDER_LAMPEN = 0.5
# Zo ver naast een rugleuning of tafel blijft de vloer nog hoog, in meter.
OPZIJ = 0.6
# Stap van het hoogteprofiel van de vloer, in meter vanaf het midden.
STAP = 0.5
# Objecten die geen wand, vloer of plafond zijn: daar botst de camera niet mee.
LOS = re.compile(r".*(Mic|Plaatje|Scherm|Hanglamp|Spot).*")
# De katheder zit in Zaal_Vast, maar is ook een los voorwerp: midden (x, z) en straal in meter.
KATHEDER = (3.19, 2.21, 0.8)


def bevy(p):
    return (p.x, p.z, -p.y)


def blender(x, y, z):
    return Vector((x, -z, y))


def boom(objecten):
    """Eén BVH over de driehoeken van deze objecten, in wereldassen."""
    punten, vlakken = [], []
    dg = bpy.context.evaluated_depsgraph_get()
    for o in objecten:
        m = o.evaluated_get(dg).to_mesh()
        begin = len(punten)
        punten += [o.matrix_world @ v.co for v in m.vertices]
        vlakken += [[begin + i for i in p.vertices] for p in m.polygons]
        o.evaluated_get(dg).to_mesh_clear()
    return BVHTree.FromPolygons(punten, vlakken)


meshes = [o for o in bpy.data.objects if o.type == "MESH"]
vast = boom([o for o in meshes if not LOS.fullmatch(o.name)])


def straal(van, richting, max_afstand=60.0):
    """Afstand tot de eerste raakplek, of None."""
    hit = vast.ray_cast(blender(*van), blender(*richting).normalized(), max_afstand)
    return None if hit[0] is None else hit[3]


# Wanden: de halve cirkel om het midden (tribune en balustrade) en de rechthoek voor de Voorzitter.
cirkel = []
for h in (1.8, 2.6, 3.4, 4.2):
    for i in range(181):
        a = math.radians(i)
        d = straal((0.0, h, 0.0), (math.cos(a), 0.0, -math.sin(a)))
        if d is not None:
            cirkel.append(d)
zijwand = []
for h in (1.8, 2.6, 3.4, 4.2):
    for z in (0.25, 1.0, 2.0, 3.0, 4.0):
        for kant in (1.0, -1.0):
            d = straal((0.0, h, z), (kant, 0.0, 0.0))
            if d is not None:
                zijwand.append(d)
voorwand = []
for h in (1.8, 2.6, 3.4, 4.2):
    for x in range(-8, 9):
        d = straal((float(x), h, 0.0), (0.0, 0.0, 1.0))
        if d is not None:
            voorwand.append(d)
r_wand = min(cirkel) - WANDMARGE
w_voor = min(zijwand) - WANDMARGE
d_voor = min(voorwand) - WANDMARGE

# Plafond: onder de laagste hanglamp en onder het plafond zelf.
lampen = [o for o in meshes if re.fullmatch(r"Hanglamp_\d+", o.name)]
lamp_onder = min(min(bevy(o.matrix_world @ Vector(c))[1] for c in o.bound_box) for o in lampen)
plafond_zelf = min(
    d for d in (straal((x, 2.0, z), (0.0, 1.0, 0.0)) for x in range(-8, 9, 2) for z in (-6, -3, 0, 2))
    if d is not None
) + 2.0
plafond = min(lamp_onder, plafond_zelf) - VRIJ_ONDER_LAMPEN


def hoogte_onder(x, z):
    """De hoogte van het eerste wat een straal recht omlaag vanaf het plafond raakt."""
    d = straal((x, plafond, z), (0.0, -1.0, 0.0), plafond + 1.0)
    return plafond - d if d is not None else 0.0


# Vloer achter (z < 0): per ring om het midden het hoogste punt van banken en tafels.
ringen = []
r = 0.0
while r <= r_wand:
    n = max(8, int(math.pi * r / 0.1))
    hoogst = max(hoogte_onder(r * math.cos(math.pi * i / n), -r * math.sin(math.pi * i / n)) for i in range(n + 1))
    ringen.append((r, hoogst))
    r += 0.1
# Breed genoeg naast elke bank, en een gat smaller dan GAT meter tussen twee hoge plekken telt niet:
# zo springt de vloer niet op en neer tussen de rijen.
GAT = 1.0


def hoogst_rond(van, tot):
    return max((h for rr, h in ringen if van <= rr <= tot), default=0.0)


breed = [(r, hoogst_rond(r - OPZIJ, r + OPZIJ)) for r, _ in ringen]
dicht = [
    (r, max(h, min(max(hh for rr, hh in breed if r - GAT <= rr <= r), max(hh for rr, hh in breed if r <= rr <= r + GAT))))
    for r, h in breed
]
vloer_rond = []
r = 0.0
while r <= r_wand + 1e-6:
    h = max(hh for rr, hh in dicht if abs(rr - r) <= 0.05)
    vloer_rond.append([round(r, 2), round(h + VRIJ_BOVEN_BANKEN, 3)])
    r += STAP
if vloer_rond[-1][0] < r_wand:
    vloer_rond.append([round(r_wand, 2), vloer_rond[-1][1]])

# Vloer voor de Voorzitter (z > 0): het hoogste punt van balie, tafels en stoelen.
hoogst_voor = 0.0
for i in range(int(2 * w_voor / 0.2) + 1):
    for j in range(int(d_voor / 0.2) + 1):
        x, z = -w_voor + 0.2 * i, 0.2 * j
        if math.hypot(x - KATHEDER[0], z - KATHEDER[1]) < KATHEDER[2]:
            continue
        h = hoogte_onder(x, z)
        if h > hoogst_voor:
            hoogst_voor, waar_voor = h, (round(x, 1), round(z, 1))
print("hoogste punt voor de Voorzitter", round(hoogst_voor, 3), "op", waar_voor)
vloer_voor = round(hoogst_voor + VRIJ_BOVEN_BANKEN, 3)

ruimte = {
    "bron": "zaal/ruimte.py, gemeten in zaal.glb van zaal/maak.sh",
    "straal": round(r_wand, 3),
    "voor_halve_breedte": round(w_voor, 3),
    "voor_diepte": round(d_voor, 3),
    "vloer_rond": vloer_rond,
    "vloer_voor": vloer_voor,
    "plafond": round(plafond, 3),
}
with open(uit_json, "w") as f:
    json.dump(ruimte, f, indent=1)
    f.write("\n")
print(json.dumps({k: v for k, v in ruimte.items() if k != "vloer_rond"}))
print("vloer_rond", vloer_rond)


# De tekening: dezelfde vorm als `Ruimte::vloer_op` in crates/server/src/camera.rs.
def vloer_op(x, z):
    def rond(r):
        if r <= vloer_rond[0][0]:
            return vloer_rond[0][1]
        for (r0, y0), (r1, y1) in zip(vloer_rond, vloer_rond[1:]):
            if r <= r1:
                return y0 + (y1 - y0) * (r - r0) / (r1 - r0)
        return vloer_rond[-1][1]

    def glad(t):
        t = min(max(t, 0.0), 1.0)
        return t * t * (3 - 2 * t)

    if z > 0:
        laag = rond(abs(x))
        return max(vloer_voor, laag + (vloer_voor - laag) * (1 - glad(z)))
    laag = rond(math.hypot(x, z))
    return max(laag, laag + (vloer_voor - laag) * (1 - glad(-z)))


def rand(n=90):
    """De wanden van bovenaf: de rechthoek voor de Voorzitter, dan de halve cirkel achterom."""
    boog = [(r_wand * math.cos(math.pi * (1 - i / n)), -r_wand * math.sin(math.pi * (1 - i / n))) for i in range(n + 1)]
    return [(-w_voor, d_voor), (-w_voor, 0.0)] + boog + [(w_voor, 0.0), (w_voor, d_voor)]


omtrek = rand()
bm = bmesh.new()
# de wanden van de vloer tot het plafond, zonder deksel, zodat je van boven de vloer ziet
onder = [bm.verts.new(blender(x, vloer_op(x, z), z)) for x, z in omtrek]
boven = [bm.verts.new(blender(x, plafond, z)) for x, z in omtrek]
for i in range(len(omtrek) - 1):
    bm.faces.new((onder[i], onder[i + 1], boven[i + 1], boven[i]))
bm.faces.new((onder[-1], onder[0], boven[0], boven[-1]))
# de vloer als raster van 25 cm, zodat het profiel over de banken te zien is
raster = {}
n = int(r_wand / 0.25) + 1
for i in range(-n, n + 1):
    for j in range(-n, int(d_voor / 0.25) + 1):
        x, z = 0.25 * i, 0.25 * j
        if (z <= 0 and math.hypot(x, z) <= r_wand) or (0 <= z <= d_voor and abs(x) <= w_voor):
            raster[(i, j)] = bm.verts.new(blender(x, vloer_op(x, z), z))
for (i, j), v in raster.items():
    hoek = [raster.get(k) for k in ((i + 1, j), (i + 1, j + 1), (i, j + 1))]
    if all(hoek):
        bm.faces.new((v, *hoek))
mesh = bpy.data.meshes.new("Onzichtbare_ruimte")
bm.to_mesh(mesh)
obj = bpy.data.objects.new("Onzichtbare_ruimte", mesh)
bpy.context.scene.collection.objects.link(obj)
obj.color = (1.0, 0.55, 0.05, 1.0)
bpy.ops.wm.save_as_mainfile(filepath=os.path.join(uitmap, "ruimte.blend"))


# Twee beelden met Workbench: de ruimte oranje in het model. Van boven vanaf net onder het
# plafond, en opzij als doorsnede door het midden (x = 0).
scene = bpy.context.scene
scene.render.engine = "BLENDER_WORKBENCH"
scene.display.shading.light = "STUDIO"
scene.display.shading.color_type = "OBJECT"
for o in meshes:
    o.color = (0.75, 0.75, 0.75, 1.0)
scene.render.resolution_x, scene.render.resolution_y = 1600, 1000
cam = bpy.data.objects.new("camera", bpy.data.cameras.new("camera"))
scene.collection.objects.link(cam)
scene.camera = cam
cam.data.type = "ORTHO"
for naam, plek, draai, schaal, begin in (
    ("ruimte-boven.png", blender(0, plafond + 0.3, -4), (0, 0, 0), 28, 0.0),
    ("ruimte-zij.png", blender(30, 2.5, -4), (math.radians(90), 0, math.radians(90)), 22, 30.0),
):
    cam.location = plek
    cam.rotation_euler = draai
    cam.data.ortho_scale = schaal
    cam.data.clip_start = begin
    cam.data.clip_end = 200
    scene.render.filepath = os.path.join(uitmap, naam)
    bpy.ops.render.render(write_still=True)
print(f"{uit_json}, {uitmap}/ruimte.blend en twee beelden geschreven")
