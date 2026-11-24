"""Maakt van het Zaalmodel (zie zaal/README.md) de bestanden die de Renderer laadt. Start via `zaal/maak.sh`.

    blender -b --factory-startup <tweede-kamer>/blend/zaal.blend --python-exit-code 1 \
        --python zaal/export.py -- --uit <map> [--bak 8192] [--samples 512] [--stoelbak 3072] [--geen-bak]
            [--omgeving 1024] [--omgeving-samples 256] [--alleen-omgeving] [--beelden zaal-3,... [--beelden-samples 64] [--beelden-map <map>]]

`--alleen-omgeving` maakt in een bestaande map alleen het omgevingsbeeld opnieuw (en werkt het manifest bij), zonder
opnieuw te bakken. `--beelden` rendert daarna Cycles-beelden vanuit die Standpunten, om naast de Renderer te leggen.

Schrijft in <map>:
    zaal.glb                 het model zonder Draco; de 150 Stoelen zijn kopieën van één mesh
    lichtkaart.bin           het gebakken licht (RGB9E5, zie `schrijf_lichtkaart`)
    stoelkaart.bin           het gebakken licht van de 150 Stoelen, elk in een eigen vak (zelfde formaat)
    stoelkaart.json          per Stoel het vak in `stoelkaart.bin`, in UV van glTF
    standpunten.json         de camera's uit het model, in de assen van Bevy
    licht.json               de lampen, met hun sterkte in Bevy-eenheden
    omgeving.bin             het omgevingsbeeld voor reflecties: zes vlakken van een kubus (RGB9E5, zie `omgeving`)
    omgeving.json            waar het omgevingsbeeld is opgenomen en de doos van de zaal voor de reflecties
    manifest.json            driehoeken, Stoelen, omgedraaide delen, bak, sha256 van elk bestand
    lichtkaart-voorbeeld.png de lichtkaart om te bekijken (niet nodig voor de Renderer)

Werkwijze uit de proef (#8): eerst per los onderdeel de normalen naar de ruimte draaien, dan alle vaste delen samenvoegen
tot `Zaal_Vast` met een tweede UV-laag voor de lichtkaart, dan het licht van de echte lampen en het daglicht door het
glas bakken met Cycles, en de ruis eruit halen (OIDN). Metaal en glas zijn tijdens het bakken mat. De Stoelen werpen
schaduw in de bak en krijgen hun eigen lichtkaart (`stoelkaart.bin`): de bak bewaart alleen licht, geen kleur, dus ze
kunnen nog steeds van kleur wisselen. Hun leer is tijdens het bakken grijs, zodat er geen blauw op de omgeving valt.
Voor de reflecties van metaal en glas neemt Cycles de zaal op vanuit het midden, met alle lampen en het daglicht
(#65); de Renderer filtert dat beeld zelf voor ruwe en gladde vlakken.
"""
import hashlib
import json
import math
import os
import re
import sys
import time

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Vector

TK = os.path.dirname(os.path.dirname(bpy.data.filepath))
sys.path.insert(0, TK)
import webexport  # noqa: E402  (uit de modelmap: gescande materialen in een vorm die glTF kent)


def optie(naam, standaard=None):
    a = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    if naam in a:
        i = a.index(naam)
        return a[i + 1] if i + 1 < len(a) and not a[i + 1].startswith("--") else True
    return standaard


UIT = os.path.abspath(optie("--uit", "uit"))
BAK_PX = int(optie("--bak", 8192))
SAMPLES = int(optie("--samples", 512))
STOEL_PX = int(optie("--stoelbak", 3072))
# Vorm van de eilanden bij het inpakken van de lichtkaart-UV. CONCAVE vult de kaart veel beter dan AABB (de ronde
# delen van de zaal lieten met AABB het grootste deel van de kaart leeg), zodat elk vlak meer pixels krijgt.
PAK = optie("--pak", "CONCAVE")
# Grijs voor het leer van de Stoelen tijdens het bakken: ongeveer even licht als het leer en de partijkleuren.
LEER_GRIJS = 0.08
OMGEVING_PX = int(optie("--omgeving", 1024))
OMGEVING_SAMPLES = int(optie("--omgeving-samples", 256))
CYCLES_SAMPLES = int(optie("--beelden-samples", 64))
STOEL = re.compile(r"Stoel_(\d+)_(\d+)_(\d+)$")
WARM = (1.0, 0.86, 0.68)

# Omdraaien met een ray cast: zo ver kijkt een vlak voor zich (een paar centimeter) en zo ver achter zich.
VOOR_M = 0.05
ACHTER_M = 0.05
NABIJ_M = 0.003
# Zoveel vlakken per los onderdeel doen mee (de grootste); genoeg voor een stem, snel genoeg voor 116.000 vlakken.
STEMMEN = 24
# Waar de ray cast niets beslist: kijk naar het midden van de zaal (zoals de proef), in Blender-coördinaten.
MIDDEN = Vector((0.0, 2.0, 2.5))


def log(*a):
    print("[export]", *a, flush=True)


# --- UV: TEXCOORD_0 voor texturen, TEXCOORD_1 voor de lichtkaart ------------------------------

def een_uv_laag():
    """Bevy leest texturen van UV0 en de lichtkaart van UV1. Alles wat een textuur nodig heeft gaat dus in één laag
    `UVMap`: vlakken met een gescand materiaal nemen de coördinaten van `uv_scan` over. Daarna is `uv_scan` weg."""
    scan_mats = set()
    for m in bpy.data.materials:
        if m.use_nodes:
            for n in m.node_tree.nodes:
                if n.type == "UVMAP" and n.uv_map == webexport.UVSET:
                    n.uv_map = "UVMap"
                    scan_mats.add(m.name)
    for me in bpy.data.meshes:
        if not me.users:
            continue
        lagen = me.uv_layers
        if "UVMap" not in lagen:
            if len(lagen):
                lagen[0].name = "UVMap"
            else:
                lagen.new(name="UVMap")
        scan = lagen.get(webexport.UVSET)
        if scan is not None:
            P, L = len(me.polygons), len(me.loops)
            mi = np.empty(P, np.int32)
            me.polygons.foreach_get("material_index", mi)
            start = np.empty(P, np.int32)
            me.polygons.foreach_get("loop_start", start)
            per_vlak = np.diff(np.append(start, L))
            vlak_van_loop = np.repeat(np.arange(P), per_vlak)
            heeft = np.array([bool(m and m.name in scan_mats) for m in me.materials] + [False])
            kies = heeft[np.minimum(mi, len(me.materials))][vlak_van_loop]
            a = np.empty(L * 2, np.float32)
            b = np.empty(L * 2, np.float32)
            lagen["UVMap"].data.foreach_get("uv", a)
            scan.data.foreach_get("uv", b)
            a, b = a.reshape(L, 2), b.reshape(L, 2)
            a[kies] = b[kies]
            lagen["UVMap"].data.foreach_set("uv", a.ravel())
        for laag in [l for l in lagen if l.name != "UVMap"]:
            lagen.remove(laag)
        lagen["UVMap"].active_render = True


# --- normalen naar de ruimte draaien -----------------------------------------------------------

def eilanden(bm):
    """Losse onderdelen van een mesh: groepen vlakken die via randen aan elkaar zitten."""
    gezien = set()
    for f in bm.faces:
        if f.index in gezien:
            continue
        stapel, deel = [f], []
        gezien.add(f.index)
        while stapel:
            g = stapel.pop()
            deel.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h.index not in gezien:
                        gezien.add(h.index)
                        stapel.append(h)
        yield deel


def materiaal_van(obj, vlak):
    me = obj.data
    if vlak < 0 or vlak >= len(me.polygons):
        return None
    i = me.polygons[vlak].material_index
    return obj.material_slots[i].material if i < len(obj.material_slots) else None


class Kijker:
    """Ray casts door de hele scène, in wereldcoördinaten."""

    def __init__(self):
        self.dg = bpy.context.evaluated_depsgraph_get()
        self.scene = bpy.context.scene

    def raakt(self, obj, eigen, eigen_mat, punt, richting, afstand):
        """Raakt een straal vanaf `punt` binnen `afstand` iets dat niet bij dit losse onderdeel hoort?
        Geeft (geraakt, ander_materiaal)."""
        # Wat binnen een paar millimeter ligt telt niet: dat is een vlak dat ertegenaan of erop ligt, zoals het
        # houten blad rond een leren inleg. Een holte achter een paneel (14 mm bij de interruptiebalk) telt wel.
        start = punt + richting * NABIJ_M
        over = afstand - NABIJ_M
        while over > 0:
            hit, plek, _n, vlak, raak, _m = self.scene.ray_cast(self.dg, start, richting, distance=over)
            if not hit:
                return False, False
            raak = raak.original
            if raak == obj and vlak in eigen:
                # zichzelf (een gebogen deel): verder kijken
                stap = (plek - start).length + 1e-4
                start = plek + richting * 1e-4
                over -= stap
                continue
            return True, materiaal_van(raak, vlak) != eigen_mat
        return False, False


def oordeel(kijker, obj, deel, mw, nw):
    """Wijst dit losse onderdeel de verkeerde kant op?

    Een gesloten vorm (elke rand hoort bij twee vlakken) draait om als zijn inhoud negatief is: binnenstebuiten.
    Voor een open vlak eerst de ray cast (besluit #8): raakt de voorkant binnen een paar centimeter ander materiaal en
    is de achterkant vrij, dan omdraaien; andersom, dan zo laten. Beslist de ray cast niet (beide vrij of beide dicht),
    dan de regel uit de proef: omdraaien als het vlak van het midden van de zaal af wijst.
    Geeft ("draai" | "laat", "inhoud" | "ray" | "midden")."""
    if all(len(e.link_faces) == 2 for f in deel for e in f.edges):
        v = 0.0
        for f in deel:
            p = [l.vert.co for l in f.loops]
            for i in range(1, len(p) - 1):
                v += p[0].dot(p[i].cross(p[i + 1]))
        return ("draai" if v < 0 else "laat"), "inhoud"
    eigen = {f.index for f in deel}
    grootste = sorted(deel, key=lambda f: f.calc_area(), reverse=True)[:STEMMEN]
    draai = laat = totaal = 0.0
    for f in grootste:
        a = f.calc_area()
        if a <= 0:
            continue
        mat = obj.material_slots[f.material_index].material if f.material_index < len(obj.material_slots) else None
        c = mw @ f.calc_center_median()
        n = (nw @ f.normal).normalized()
        voor, voor_ander = kijker.raakt(obj, eigen, mat, c, n, VOOR_M)
        achter, achter_ander = kijker.raakt(obj, eigen, mat, c, -n, ACHTER_M)
        totaal += a
        if voor and voor_ander and not achter:
            draai += a
        elif achter and achter_ander and not voor:
            laat += a
    if totaal > 0 and draai / totaal > 0.5:
        return "draai", "ray"
    if totaal > 0 and laat / totaal > 0.5:
        return "laat", "ray"
    som = opp = 0.0
    for f in deel:
        a = f.calc_area()
        c = mw @ f.calc_center_median()
        n = (nw @ f.normal).normalized()
        som += a * n.dot((MIDDEN - c).normalized())
        opp += a
    return ("draai" if opp > 0 and som / opp < -0.2 else "laat"), "midden"


def draai_naar_ruimte(objecten):
    """Het model rekent op tweezijdige vlakken: sommige panelen wijzen met hun voorkant een holte of de muur in. Cycles
    tekent ze toch, maar bij het bakken kijkt zo'n vlak de verkeerde kant op en wordt het zwart. Daarom per los
    onderdeel beslissen en zo nodig omdraaien. Geeft een telling per reden en de lijst omgedraaide onderdelen."""
    kijker = Kijker()
    telling = {}
    omgedraaid = []
    for o in objecten:
        mw = o.matrix_world
        nw = mw.to_3x3().inverted().transposed()
        bm = bmesh.new()
        bm.from_mesh(o.data)
        bm.faces.ensure_lookup_table()
        draaien = []
        for deel in eilanden(bm):
            besluit, reden = oordeel(kijker, o, deel, mw, nw)
            telling[besluit + "/" + reden] = telling.get(besluit + "/" + reden, 0) + 1
            if besluit == "draai":
                draaien.extend(deel)
                mats = sorted({o.material_slots[f.material_index].material.name for f in deel
                               if f.material_index < len(o.material_slots) and o.material_slots[f.material_index].material})
                omgedraaid.append(dict(object=o.name, vlakken=len(deel), reden=reden, materialen=mats))
        if draaien:
            bmesh.ops.reverse_faces(bm, faces=draaien)
            bm.to_mesh(o.data)
            o.data.update()
        bm.free()
    return telling, omgedraaid


# --- vaste delen samenvoegen en een lichtkaart-UV geven -----------------------------------------

def vaste_objecten():
    """Elk zichtbaar object met een eigen mesh. Kopieën (Stoelen, lampen, microfoons) blijven los en komen niet in de bak."""
    return [o for o in bpy.context.scene.objects
            if o.type == "MESH" and o.data.users == 1 and not o.hide_render and not STOEL.match(o.name)]


def vast_samenvoegen(vast):
    for o in bpy.context.scene.objects:
        o.select_set(False)
    vl = bpy.context.view_layer
    for o in vast:
        o.hide_set(False)
        o.select_set(True)
    vl.objects.active = vast[0]
    bpy.ops.object.join()
    z = vl.objects.active
    z.name = z.data.name = "Zaal_Vast"
    # de bak moet dezelfde punten zien als de export
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    lm = z.data.uv_layers.new(name="Lichtkaart")
    z.data.uv_layers.active = lm
    t = time.time()
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=math.radians(60), island_margin=0.0, area_weight=0.0,
                             correct_aspect=True, scale_to_bounds=False)
    bpy.ops.uv.pack_islands(udim_source="CLOSEST_UDIM", rotate=True, margin_method="FRACTION",
                            margin=6.0 / BAK_PX, shape_method=PAK)
    bpy.ops.object.mode_set(mode="OBJECT")
    z.data.calc_loop_triangles()
    log("Zaal_Vast", len(z.data.loop_triangles), "driehoeken, lichtkaart-UV in", round(time.time() - t, 1), "s")
    return z


# --- licht voor de bak (zoals foto.py bij het model) ----------------------------------------------

def lamp(naam, loc, watt, kegel, zacht, blend):
    ld = bpy.data.lights.new(naam, "SPOT")
    ld.energy, ld.color = watt, WARM
    ld.spot_size, ld.spot_blend, ld.shadow_soft_size = kegel, blend, zacht
    o = bpy.data.objects.new(naam, ld)
    o.location = loc
    bpy.context.scene.collection.objects.link(o)
    return o


def onderkant(o):
    return o.location.x, o.location.y, o.location.z + min(c[2] for c in o.bound_box)


def licht_voor_bak():
    """Dezelfde lichtbronnen en sterktes als de Cycles-foto's, zodat de bak lijkt op die foto's."""
    schaal = {"LichtVul": 0.0, "LichtPlafond": 0.6, "LichtVoor": 1.0, "LichtDak": 1.0, "LichtTribune": 0.5,
              "LichtSpotTribune": 0.6}
    for o in list(bpy.data.objects):
        if o.type != "LIGHT":
            continue
        for naam, f in schaal.items():
            if o.name.startswith(naam):
                o.data.energy *= f
        if o.name.startswith("LichtSpotTribune"):
            o.data.spot_size = 1.5
        if o.name == "LichtDak":
            o.data.spread = 1.0
        if o.name == "LichtVoor":
            o.data.spread = 1.2
    n = 0
    for o in list(bpy.data.objects):
        if re.fullmatch(r"Spot\d+", o.name):
            x, y, z = onderkant(o)
            lamp("Bak" + o.name, (x, y, z - 0.04), 260.0, 1.15, 0.05, 0.5)
            n += 1
        elif re.fullmatch(r"Hanglamp_\d+", o.name):
            x, y, z = onderkant(o)
            lamp("Bak" + o.name, (x, y, z - 0.05), 100.0, 1.5, 0.12, 0.6)
            n += 1
    hemel = bpy.data.materials.get("daklicht_hemel")
    if hemel:
        b = hemel.node_tree.nodes["Principled BSDF"]
        b.inputs["Emission Color"].default_value = (0.92, 0.95, 1.0, 1.0)
        b.inputs["Emission Strength"].default_value = 7.0
    # Het daglicht door het glas zit in de bak: het licht de lichtschacht en valt zacht over de zaal.
    log("lampen voor de bak", n)


def naar_bevy(v):
    """Blender (x, y, z) met z omhoog naar glTF/Bevy (x, z, -y) met y omhoog."""
    return [v[0], v[2], -v[1]]


def lichtlijst():
    """Alle lampen voor de Renderer. Ze zitten allemaal in de bak (`live` is onwaar; een oudere Zaal had het daglicht
    live) en verlichten in de Renderer alleen wat niet gebakken is (kopieën zoals lampen en microfoons), plus glans. Eenheid: Bevy-lumen = Blender-watt voor een puntbron; een vlak
    licht (AREA) straalt naar één kant, dus vier keer zo sterk recht onder zich."""
    uit = []
    for o in bpy.data.objects:
        if o.type != "LIGHT" or o.data.energy <= 0:
            continue
        d = o.data
        r = o.matrix_world.to_3x3() @ Vector((0, 0, -1))
        kegel = d.spot_size if d.type == "SPOT" else (getattr(d, "spread", math.pi) if d.type == "AREA" else math.pi)
        uit.append(dict(naam=o.name, soort=d.type, positie=naar_bevy(o.matrix_world.translation), richting=naar_bevy(r),
                        lumen=d.energy * (4.0 if d.type == "AREA" else 1.0), kleur=list(d.color), kegel=kegel,
                        zacht=getattr(d, "spot_blend", 0.0), live=bool(o.get("live"))))
    return uit


# --- bakken ------------------------------------------------------------------------------------

def gpu():
    prefs = bpy.context.preferences.addons["cycles"].preferences
    for soort in ("OPTIX", "CUDA"):
        try:
            prefs.compute_device_type = soort
        except TypeError:
            continue
        prefs.get_devices()
        gpus = [d for d in prefs.devices if d.type == soort]
        if gpus:
            for d in prefs.devices:
                d.use = d.type == soort
            log("bak op", soort, [d.name for d in gpus])
            return soort
    log("LET OP: geen GPU gevonden, bak op de CPU")
    return "CPU"


def stoelen():
    return sorted((o for o in bpy.context.scene.objects if o.type == "MESH" and STOEL.match(o.name)),
                  key=lambda o: o.name)


def stoel_uv(stoel_objecten):
    """Een lichtkaart-UV voor de mesh die alle Stoelen delen, in het hele vlak van 0 tot 1. Elke Stoel krijgt bij het
    bakken een eigen vak in `stoelkaart.bin`; de Renderer zet dat vak per Stoel (`uv_rect` van Bevy)."""
    me = stoel_objecten[0].data
    assert all(o.data == me for o in stoel_objecten), "de Stoelen delen niet één mesh"
    vl = bpy.context.view_layer
    for o in bpy.context.scene.objects:
        o.select_set(False)
    o = stoel_objecten[0]
    o.select_set(True)
    vl.objects.active = o
    lm = me.uv_layers.new(name="Lichtkaart")
    me.uv_layers.active = lm
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.uv.smart_project(angle_limit=math.radians(60), island_margin=0.0, area_weight=0.0,
                             correct_aspect=True, scale_to_bounds=False)
    vak = STOEL_PX / math.ceil(math.sqrt(len(stoel_objecten)))
    bpy.ops.uv.pack_islands(udim_source="CLOSEST_UDIM", rotate=True, margin_method="FRACTION",
                            margin=4.0 / vak, shape_method=PAK)
    bpy.ops.object.mode_set(mode="OBJECT")
    me.uv_layers["UVMap"].active_render = True
    return me


def stoel_vakken(namen):
    """Per Stoel een vak in een raster op de stoelkaart. Geeft (u0, v0, u1, v1) in UV van Blender (v omhoog) en het vak
    voor de Renderer in UV van glTF (v omlaag, zoals de rijen in het bestand): min (u0, 1 - v1), max (u1, 1 - v0)."""
    kol = math.ceil(math.sqrt(len(namen)))
    rij = math.ceil(len(namen) / kol)
    blender, gltf = {}, {}
    for i, naam in enumerate(namen):
        c, r = i % kol, i // kol
        u0, u1 = c / kol, (c + 1) / kol
        v0, v1 = 1 - (r + 1) / rij, 1 - r / rij
        blender[naam] = (u0, v0, u1, v1)
        gltf[naam] = [u0, 1 - v1, u1, 1 - v0]
    return blender, gltf


def bak_naar(objecten, px, materialen):
    """Bakt het licht (direct en indirect, zonder kleur) van `objecten` naar een nieuw beeld van px bij px, via de actieve
    UV-laag van elk object. Geeft de pixels (onderste rij eerst, zoals Blender) en de duur."""
    img = bpy.data.images.new("Bak", px, px, alpha=False, float_buffer=True)
    img.colorspace_settings.name = "Non-Color"
    for m in materialen:
        n = m.node_tree.nodes.new("ShaderNodeTexImage")
        n.name = "bak_doel"
        n.image = img
        m.node_tree.nodes.active = n
    vl = bpy.context.view_layer
    for o in bpy.context.scene.objects:
        o.select_set(False)
    for o in objecten:
        o.select_set(True)
    vl.objects.active = objecten[0]
    t = time.time()
    bpy.ops.object.bake(type="DIFFUSE", pass_filter={"DIRECT", "INDIRECT"}, use_clear=True, margin=6)
    duur = time.time() - t
    for m in materialen:
        m.node_tree.nodes.remove(m.node_tree.nodes["bak_doel"])
    pixels = np.empty(px * px * 4, np.float32)
    img.pixels.foreach_get(pixels)
    bpy.data.images.remove(img)
    return pixels.reshape(px, px, 4)[..., :3].copy(), duur


def ontruis(rgb):
    """Haalt de ruis uit een gebakken lichtkaart met OIDN (de Denoise-node van de compositor). Een lichtkaart heeft
    geen textuur, alleen licht, dus het filter mag alles glad maken wat geen rand van licht en schaduw is."""
    h, b = rgb.shape[:2]
    sc = bpy.context.scene
    img = bpy.data.images.new("Ruis", b, h, alpha=False, float_buffer=True)
    img.colorspace_settings.name = "Non-Color"
    p = np.ones((h, b, 4), np.float32)
    p[..., :3] = rgb
    img.pixels.foreach_set(p.ravel())
    oud = (sc.compositing_node_group, sc.render.resolution_x, sc.render.resolution_y, sc.render.resolution_percentage,
           sc.render.image_settings.file_format, sc.render.image_settings.color_depth, sc.render.filepath,
           sc.view_settings.view_transform)
    boom = bpy.data.node_groups.new("Ontruis", "CompositorNodeTree")
    boom.interface.new_socket("Image", in_out="OUTPUT", socket_type="NodeSocketColor")
    bron = boom.nodes.new("CompositorNodeImage")
    bron.image = img
    dn = boom.nodes.new("CompositorNodeDenoise")
    dn.inputs["HDR"].default_value = True
    uit = boom.nodes.new("NodeGroupOutput")
    boom.links.new(bron.outputs["Image"], dn.inputs["Image"])
    boom.links.new(dn.outputs["Image"], uit.inputs[0])
    sc.compositing_node_group = boom
    sc.render.resolution_x, sc.render.resolution_y, sc.render.resolution_percentage = b, h, 100
    sc.render.image_settings.file_format = "OPEN_EXR"
    sc.render.image_settings.color_depth = "32"
    sc.view_settings.view_transform = "Standard"
    pad = os.path.join(UIT, "ontruis.exr")
    sc.render.filepath = pad
    t = time.time()
    bpy.ops.render.render(write_still=True)
    r = bpy.data.images.load(pad)
    q = np.empty(h * b * 4, np.float32)
    r.pixels.foreach_get(q)
    bpy.data.images.remove(r)
    bpy.data.images.remove(img)
    bpy.data.node_groups.remove(boom)
    os.remove(pad)
    (sc.compositing_node_group, sc.render.resolution_x, sc.render.resolution_y, sc.render.resolution_percentage,
     sc.render.image_settings.file_format, sc.render.image_settings.color_depth, sc.render.filepath,
     sc.view_settings.view_transform) = oud
    log("ontruisd", b, "x", h, "in", round(time.time() - t, 1), "s")
    uit_rgb = q.reshape(h, b, 4)[..., :3]
    # Buiten de eilanden (zwart in de bak) blijft het zwart; zo lekt er niets naar de lege ruimte.
    return np.where(rgb.max(-1, keepdims=True) > 0, np.maximum(uit_rgb, 0), 0)


def bak(z):
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    apparaat = gpu()
    sc.cycles.device = "GPU" if apparaat != "CPU" else "CPU"
    sc.cycles.samples = SAMPLES
    sc.cycles.max_bounces = 8
    sc.cycles.diffuse_bounces = 6
    sc.render.bake.margin = 6
    sc.render.bake.margin_type = "EXTEND"
    stoel_objecten = stoelen()
    gedeeld = stoel_uv(stoel_objecten)
    materialen = {m for o in [z, *stoel_objecten] for m in o.data.materials if m is not None and m.use_nodes}
    # Metaal heeft geen diffuse kleur, dus de bak gaf daar nul: tijdens het bakken mat maken. Glas wordt tijdens het
    # bakken helemaal doorzichtig, zodat het daglicht de lichtschacht in valt (mat glas hield het tegen); het glas zelf
    # tekent de Renderer toch doorzichtig.
    terug = []
    for m in materialen:
        b = m.node_tree.nodes.get("Principled BSDF")
        if b is None:
            continue
        glas = b.inputs["Transmission Weight"]
        if not glas.links and glas.default_value > 0.5:
            for invoer, waarde in (("Transmission Weight", 0.0), ("Alpha", 0.0)):
                sok = b.inputs[invoer]
                if not sok.links:
                    terug.append((sok, sok.default_value))
                    sok.default_value = waarde
        for invoer in ("Metallic", "Transmission Weight"):
            sok = b.inputs[invoer]
            if not sok.links and sok.default_value > 0.0:
                terug.append((sok, sok.default_value))
                sok.default_value = 0.0
    log("mat gemaakt voor de bak:", len(terug), "instellingen")
    # Het leer grijs, zodat het blauw niet op de banken en de Stoelen ernaast valt (de kleur wisselt in de uitzending).
    leer = bpy.data.materials.get("stoel_leer")
    leer_terug = None
    if leer and leer.use_nodes and leer.node_tree.nodes.get("Principled BSDF"):
        kleur = leer.node_tree.nodes["Principled BSDF"].inputs["Base Color"]
        leer_terug = (kleur, [l.from_socket for l in kleur.links], tuple(kleur.default_value))
        for l in list(kleur.links):
            leer.node_tree.links.remove(l)
        kleur.default_value = (LEER_GRIJS, LEER_GRIJS, LEER_GRIJS, 1.0)

    # 1. de vaste delen, met de Stoelen als schaduw
    zaal_mats = [m for m in z.data.materials if m is not None and m.use_nodes]
    rgb, duur = bak_naar([z], BAK_PX, zaal_mats)
    log("bak klaar in", round(duur, 1), "s")
    rgb = ontruis(rgb)

    # 2. de Stoelen: per Stoel een kopie met de lichtkaart-UV in zijn eigen vak, samengevoegd tot één object (één bak
    # per object kost elke keer een nieuwe opbouw van de scène; 150 losse bakken duurden minuten)
    vakken, vakken_gltf = stoel_vakken([o.name for o in stoel_objecten])
    kopieen = []
    for o in stoel_objecten:
        me = gedeeld.copy()
        uv = np.empty(len(me.loops) * 2, np.float32)
        me.uv_layers["Lichtkaart"].data.foreach_get("uv", uv)
        u0, v0, u1, v1 = vakken[o.name]
        uv = uv.reshape(-1, 2) * (u1 - u0, v1 - v0) + (u0, v0)
        me.uv_layers["Lichtkaart"].data.foreach_set("uv", uv.ravel())
        k = bpy.data.objects.new("Bak_" + o.name, me)
        k.matrix_world = o.matrix_world.copy()
        bpy.context.scene.collection.objects.link(k)
        kopieen.append(k)
        o.hide_render = True
    vl = bpy.context.view_layer
    for o in bpy.context.scene.objects:
        o.select_set(False)
    for k in kopieen:
        k.select_set(True)
    vl.objects.active = kopieen[0]
    bpy.ops.object.join()
    samen = vl.objects.active
    samen.data.uv_layers.active = samen.data.uv_layers["Lichtkaart"]
    stoel_mats = [m for m in gedeeld.materials if m is not None and m.use_nodes]
    stoel_rgb, stoel_duur = bak_naar([samen], STOEL_PX, stoel_mats)
    log("Stoelen gebakken in", round(stoel_duur, 1), "s")
    stoel_rgb = ontruis(stoel_rgb)
    me = samen.data
    bpy.data.objects.remove(samen)
    bpy.data.meshes.remove(me)
    for o in stoel_objecten:
        o.hide_render = False

    for sok, w in terug:
        sok.default_value = w
    if leer_terug:
        kleur, van, waarde = leer_terug
        kleur.default_value = waarde
        for v in van:
            leer.node_tree.links.new(v, kleur)

    rgb = rgb[::-1]     # Blender begint onderaan, de GPU bovenaan
    schrijf_lichtkaart(os.path.join(UIT, "lichtkaart.bin"), rgb)
    schrijf_lichtkaart(os.path.join(UIT, "stoelkaart.bin"), stoel_rgb[::-1])
    json.dump(dict(px=STOEL_PX, stoelen=vakken_gltf), open(os.path.join(UIT, "stoelkaart.json"), "w"), indent=1)
    zwart = zwart_per_materiaal(z, rgb)
    for naam, beeld in (("lichtkaart-voorbeeld.png", rgb), ("stoelkaart-voorbeeld.png", stoel_rgb[::-1])):
        n = beeld.shape[0]
        voorbeeld = bpy.data.images.new("voorbeeld", n, n, alpha=False)
        p = np.ones((n, n, 4), np.float32)
        p[..., :3] = np.clip(beeld[::-1] / (1 + beeld[::-1]), 0, 1)
        voorbeeld.pixels.foreach_set(p.ravel())
        voorbeeld.filepath_raw = os.path.join(UIT, naam)
        voorbeeld.file_format = "PNG"
        voorbeeld.save()
        bpy.data.images.remove(voorbeeld)
    return dict(px=BAK_PX, samples=SAMPLES, apparaat=apparaat, seconden=round(duur, 1), pak=PAK, ontruisd=True,
                gemiddeld=float(rgb.mean()), max=float(rgb.max()), zwart_per_materiaal=zwart,
                stoelen=dict(px=STOEL_PX, seconden=round(stoel_duur, 1), aantal=len(stoel_objecten)))


def zwart_per_materiaal(z, rgb):
    """Per materiaal: welk deel van het oppervlak zwart gebakken is (lichtkaart < 0,02 in het midden van het vlak).
    Laat zien of een paneel de verkeerde kant op wijst, zoals het front van de interruptiebalk."""
    me = z.data
    h, b = rgb.shape[:2]
    helder = rgb.max(-1)
    P, L = len(me.polygons), len(me.loops)
    st = np.empty(P, np.int32)
    me.polygons.foreach_get("loop_start", st)
    per_vlak = np.diff(np.append(st, L))
    uv = np.empty(L * 2, np.float32)
    me.uv_layers["Lichtkaart"].data.foreach_get("uv", uv)
    uv = uv.reshape(L, 2)
    vlak_van_loop = np.repeat(np.arange(P), per_vlak)
    midden = np.zeros((P, 2))
    np.add.at(midden, vlak_van_loop, uv)
    midden /= per_vlak[:, None]
    waarde = helder[np.clip(((1 - midden[:, 1]) * h).astype(int), 0, h - 1),
                    np.clip((midden[:, 0] * b).astype(int), 0, b - 1)]
    opp = np.empty(P, np.float32)
    me.polygons.foreach_get("area", opp)
    mi = np.empty(P, np.int32)
    me.polygons.foreach_get("material_index", mi)
    uit = {}
    for i, m in enumerate(me.materials):
        kies = mi == i
        if m is None or not kies.any() or opp[kies].sum() <= 0:
            continue
        uit[m.name] = round(float((opp[kies] * (waarde[kies] < 0.02)).sum() / opp[kies].sum()), 4)
    for naam, deel in sorted(uit.items(), key=lambda x: -x[1])[:15]:
        log(f"zwart {naam:28s} {deel:.0%}")
    log("zwart middenvak_paneel (front interruptiebalk):", uit.get("middenvak_paneel"))
    return uit


def schrijf_lichtkaart(pad, rgb):
    """RGB9E5: drie getallen met één gedeelde exponent in 32 bits, door de GPU te filteren.
    Kop: magic b'LK95', breedte en hoogte als u32 little-endian; daarna rij voor rij de u32-pixels, bovenste rij eerst."""
    rgb = np.clip(np.nan_to_num(rgb.astype(np.float64)), 0, 65408.0)
    mx = rgb.max(-1)
    exp = np.maximum(-16, np.floor(np.log2(np.maximum(mx, 1e-30)))) + 1 + 15
    exp = np.clip(exp, 0, 31)
    schaal = 2.0 ** (exp - 15 - 9)
    m = np.floor(mx / schaal + 0.5)
    exp = np.where(m == 512, exp + 1, exp)
    schaal = 2.0 ** (exp - 15 - 9)
    q = np.clip(np.floor(rgb / schaal[..., None] + 0.5), 0, 511).astype(np.uint32)
    woord = q[..., 0] | (q[..., 1] << 9) | (q[..., 2] << 18) | (exp.astype(np.uint32) << 27)
    h, b = woord.shape
    with open(pad, "wb") as f:
        f.write(b"LK95")
        f.write(np.array([b, h], "<u4").tobytes())
        f.write(woord.astype("<u4").tobytes())
    log("lichtkaart", pad, round(os.path.getsize(pad) / 1e6, 1), "MB")


# --- omgevingsbeeld voor reflecties (#65) -------------------------------------------------------

# De zes vlakken in de volgorde en assen van een kubustextuur van de GPU (+X, -X, +Y, -Y, +Z, -Z). Per vlak de richting
# bij pixel (a, b), met a van links (-1) naar rechts (+1) en b van boven (-1) naar onder (+1), in de assen van de textuur:
# die van Bevy met z omgekeerd (de shader van Bevy keert z om voordat hij de kubus leest).
KUBUS = (
    lambda a, b: (1.0, -b, -a),
    lambda a, b: (-1.0, -b, a),
    lambda a, b: (a, 1.0, b),
    lambda a, b: (a, -1.0, -b),
    lambda a, b: (a, -b, 1.0),
    lambda a, b: (-a, -b, -1.0),
)


def kubus_naar_blender(d):
    """Textuur (x, y, z) = Bevy (x, y, -z); Bevy (x, y, z) = Blender (x, -z, y). Samen: Blender (x, z, y)."""
    return Vector((d[0], d[2], d[1]))


def kubuscamera(vlak):
    """Draaiing van een Blender-camera (kijkt langs -z, x rechts, y omhoog) die precies dit vlak van de kubus ziet.
    Een pixel (a, b) kijkt in de camera langs (a, -b, -1); in het vlak langs midden + a * A + b * B."""
    f = KUBUS[vlak]
    midden = kubus_naar_blender(f(0.0, 0.0))
    A = kubus_naar_blender(f(1.0, 0.0)) - midden
    B = kubus_naar_blender(f(0.0, 1.0)) - midden
    m = Matrix((A, -B, -midden)).transposed()
    assert abs(m.determinant() - 1.0) < 1e-6, ("vlak", vlak, "is gespiegeld")
    return m


def zaaldoos():
    """Midden en halve maten van alles wat vast staat, in Blender-coördinaten."""
    lo = Vector((1e9, 1e9, 1e9))
    hi = -lo
    for o in vaste_objecten():
        for c in o.bound_box:
            p = o.matrix_world @ Vector(c)
            lo = Vector(map(min, lo, p))
            hi = Vector(map(max, hi, p))
    return (lo + hi) / 2, (hi - lo) / 2


def cycles_instellen(samples):
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    apparaat = gpu()
    sc.cycles.device = "GPU" if apparaat != "CPU" else "CPU"
    sc.cycles.samples = samples
    sc.cycles.use_denoising = True
    sc.cycles.max_bounces = 8
    sc.cycles.diffuse_bounces = 6
    sc.render.film_transparent = False
    sc.render.resolution_percentage = 100
    sc.render.pixel_aspect_x = sc.render.pixel_aspect_y = 1.0
    return apparaat


def render_exr(cam, pad):
    sc = bpy.context.scene
    sc.camera = cam
    sc.render.image_settings.file_format = "OPEN_EXR"
    sc.render.image_settings.color_depth = "32"
    sc.render.filepath = pad
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(pad)
    w, h = img.size
    px = np.empty(w * h * 4, np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    os.remove(pad)
    return px.reshape(h, w, 4)[::-1, :, :3]     # bovenste rij eerst


def omgeving():
    """Neemt de zaal op als kubus vanuit het midden van de doos om alles wat vast staat, met alle lampen, het daglicht en
    de Stoelen. Metaal en glas zijn hier echt (niet mat zoals in de bak). De Renderer zet de doos om de zaal als grens
    voor de reflecties, zodat een spiegeling op de vloer het plafond op de goede plek toont."""
    sc = bpy.context.scene
    apparaat = cycles_instellen(OMGEVING_SAMPLES)
    midden, half = zaaldoos()
    kijker = Kijker()
    vrij = {}
    for naam, r in (("+x", (1, 0, 0)), ("-x", (-1, 0, 0)), ("+y", (0, 1, 0)), ("-y", (0, -1, 0)),
                    ("omhoog", (0, 0, 1)), ("omlaag", (0, 0, -1))):
        hit, plek, *_ = kijker.scene.ray_cast(kijker.dg, midden, Vector(r), distance=100.0)
        vrij[naam] = round((plek - midden).length, 2) if hit else None
    log("omgeving opgenomen op", tuple(round(v, 2) for v in midden), "vrije ruimte", vrij)
    cd = bpy.data.cameras.new("Omgeving")
    cd.type = "PERSP"
    cd.sensor_fit = "HORIZONTAL"
    cd.angle = math.pi / 2
    cd.clip_start = 0.01
    cd.clip_end = 200.0
    cam = bpy.data.objects.new("Omgeving", cd)
    sc.collection.objects.link(cam)
    sc.render.resolution_x = sc.render.resolution_y = OMGEVING_PX
    vlakken = []
    t = time.time()
    for v in range(6):
        cam.matrix_world = Matrix.Translation(midden) @ kubuscamera(v).to_4x4()
        vlakken.append(render_exr(cam, os.path.join(UIT, f"omgeving-{v}.exr")))
    duur = time.time() - t
    log("omgeving klaar in", round(duur, 1), "s")
    bpy.data.objects.remove(cam)
    rgb = np.concatenate(vlakken, axis=0)            # zes vlakken onder elkaar: breedte N, hoogte 6N
    schrijf_lichtkaart(os.path.join(UIT, "omgeving.bin"), rgb)
    voorbeeld = bpy.data.images.new("omgeving_voorbeeld", OMGEVING_PX * 6, OMGEVING_PX, alpha=False)
    rij = np.concatenate(vlakken, axis=1)[::-1]
    p = np.ones(rij.shape[:2] + (4,), np.float32)
    p[..., :3] = np.clip(rij / (1 + rij), 0, 1) ** (1 / 2.2)
    voorbeeld.pixels.foreach_set(p.ravel())
    voorbeeld.filepath_raw = os.path.join(UIT, "omgeving-voorbeeld.png")
    voorbeeld.file_format = "PNG"
    voorbeeld.save()
    doos = dict(midden=naar_bevy(midden), half=[half.x, half.z, half.y], px=OMGEVING_PX)
    json.dump(doos, open(os.path.join(UIT, "omgeving.json"), "w"), indent=1)
    return dict(doos, samples=OMGEVING_SAMPLES, apparaat=apparaat, seconden=round(duur, 1), vrije_ruimte_m=vrij,
                gemiddeld=float(rgb.mean()), max=float(rgb.max()))


def cycles_beelden(namen, map_):
    """Cycles-beelden vanuit Standpunten uit het model, met hetzelfde licht als de Renderer en dezelfde belichting
    (EV100 0,2 zoals `BEELD` in `beeld.rs`; de kleurcorrectie en gloed van de Renderer zitten er niet in). Om naast de
    beelden van de Renderer te leggen."""
    sc = bpy.context.scene
    cycles_instellen(CYCLES_SAMPLES)
    os.makedirs(map_, exist_ok=True)
    sc.render.resolution_x, sc.render.resolution_y = 1920, 1080
    sc.view_settings.view_transform = "AgX"
    sc.view_settings.look = "None"
    # Bevy: belichting = 1 / (1,2 * 2^EV100)
    sc.view_settings.exposure = math.log2(1.0 / (1.2 * 2.0 ** 0.2))
    sc.render.image_settings.file_format = "PNG"
    sc.render.image_settings.color_depth = "8"
    for naam in namen:
        cam = bpy.data.objects["Cam_" + naam]
        cam.data.sensor_fit = "HORIZONTAL"
        sc.camera = cam
        sc.render.filepath = os.path.join(map_, f"cycles-{naam}.png")
        t = time.time()
        bpy.ops.render.render(write_still=True)
        log("Cycles", naam, "in", round(time.time() - t, 1), "s:", sc.render.filepath)


# --- Standpunten -------------------------------------------------------------------------------

def standpunten():
    uit = []
    for o in bpy.data.objects:
        if o.type != "CAMERA":
            continue
        q = o.matrix_world.to_quaternion()
        # een draaiing om Blender-as (x, y, z) wordt een draaiing om Bevy-as (x, z, -y)
        uit.append(dict(naam=o.name.removeprefix("Cam_"), positie=naar_bevy(o.matrix_world.translation),
                        rotatie_xyzw=[q.x, q.z, -q.y, q.w], lens_mm=o.data.lens, sensor_mm=o.data.sensor_width))
    return uit


# --- export ------------------------------------------------------------------------------------

def sha256(pad):
    h = hashlib.sha256()
    with open(pad, "rb") as f:
        for blok in iter(lambda: f.read(1 << 20), b""):
            h.update(blok)
    return h.hexdigest()


BESTANDEN = ("zaal.glb", "lichtkaart.bin", "stoelkaart.bin", "stoelkaart.json", "standpunten.json", "licht.json",
             "omgeving.bin", "omgeving.json")


def werk_manifest_bij(omgeving_info):
    """Na `--alleen-omgeving`: het manifest van de bestaande Zaal krijgt het nieuwe omgevingsbeeld erbij."""
    pad = os.path.join(UIT, "manifest.json")
    info = json.load(open(pad)) if os.path.exists(pad) else {}
    info["omgeving"] = omgeving_info
    info.setdefault("bestanden", {})
    for n in ("omgeving.bin", "omgeving.json"):
        info["bestanden"][n] = sha256(os.path.join(UIT, n))
    json.dump(info, open(pad, "w"), indent=1)


def main():
    os.makedirs(UIT, exist_ok=True)
    klaar = webexport.maak_materialen()
    log("gescande materialen", sorted(klaar))
    webexport.maak_uvs(klaar)
    een_uv_laag()
    if optie("--alleen-omgeving"):
        licht_voor_bak()
        werk_manifest_bij(omgeving())
        if optie("--beelden"):
            cycles_beelden(optie("--beelden").split(","), os.path.abspath(optie("--beelden-map", os.path.join(UIT, "cycles"))))
        return
    vast = vaste_objecten()
    log("vaste objecten", len(vast))
    t = time.time()
    telling, omgedraaid = draai_naar_ruimte(vast)
    log("losse onderdelen per besluit", telling, "in", round(time.time() - t, 1), "s")
    for d in omgedraaid:
        if any(m.startswith("middenvak") for m in d["materialen"]):
            log("omgedraaid", d)
    info = dict(omgedraaid=dict(telling=telling, onderdelen=omgedraaid))
    if optie("--alleen-draaien"):
        json.dump(info, open(os.path.join(UIT, "omgedraaid.json"), "w"), indent=1)
        return
    z = vast_samenvoegen(vast)
    if not optie("--geen-bak"):
        licht_voor_bak()
        info["bak"] = bak(z)
    json.dump(standpunten(), open(os.path.join(UIT, "standpunten.json"), "w"), indent=1)
    json.dump(lichtlijst(), open(os.path.join(UIT, "licht.json"), "w"), indent=1)
    info["omgeving"] = omgeving()
    if optie("--beelden"):
        cycles_beelden(optie("--beelden").split(","), os.path.abspath(optie("--beelden-map", os.path.join(UIT, "cycles"))))
    for o in list(bpy.data.objects):
        if o.type != "MESH":
            bpy.data.objects.remove(o)
    glb = os.path.join(UIT, "zaal.glb")
    bpy.ops.export_scene.gltf(filepath=glb, export_format="GLB", export_apply=True, export_yup=True,
                              export_cameras=False, export_lights=False, export_tangents=True,
                              export_image_format="JPEG", export_jpeg_quality=88,
                              export_draco_mesh_compression_enable=False)
    stoelen = sorted(o.name for o in bpy.data.objects if STOEL.match(o.name))
    driehoeken = 0
    for o in bpy.data.objects:
        o.data.calc_loop_triangles()
        driehoeken += len(o.data.loop_triangles)
    info.update(glb_mb=round(os.path.getsize(glb) / 1e6, 2), driehoeken=driehoeken, stoelen=len(stoelen),
                stoel_meshes=len({bpy.data.objects[s].data.name for s in stoelen}), objecten=len(bpy.data.objects),
                blender=bpy.app.version_string,
                bestanden={n: sha256(os.path.join(UIT, n)) for n in sorted(os.listdir(UIT))
                           if n in BESTANDEN})
    json.dump(info, open(os.path.join(UIT, "manifest.json"), "w"), indent=1)
    log("klaar", {k: v for k, v in info.items() if k not in ("omgedraaid", "bak")})


main()
