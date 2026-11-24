"""Zet de PDF "Zetelverdeling plenaire zaal" van de Tweede Kamer over naar een Zaalindeling.

Zo is `tk2025.json` gemaakt (#53). Daarna past de Regie het bestand met de hand aan; draai dit
script alleen opnieuw voor een nieuwe PDF.

    python3 -m venv /tmp/pdf && /tmp/pdf/bin/pip install pymupdf numpy scipy pillow
    /tmp/pdf/bin/python -I zaalindeling/overzetten.py <pdf> <peildatum JJJJ-MM-DD> \
        crates/gedeeld/src/zaalkaart.rs zaalindeling/tk2025.json [controlebeeld.png]

Stap 1, de PDF: elke Stoel is een vlak in de tekening. De vlakken gaan in zes groepen (de vakken)
op hun hoek vanaf het midden van de waaier. Per vak legt het script de 25 Stoelen van het model
(`STOELPLEKKEN`) met een affiene passing en een één-op-één-koppeling op de 25 vlakken. De kleur
van het vlak geeft de Fractie; de Stoelen van GroenLinks/PvdA zijn gestreept en hebben alleen
een rand, dus daar telt de kleur eronder. De telling per Fractie moet gelijk zijn aan de legenda.

Stap 2, de Kamerleden: de Kamer op de peildatum uit de open data (dezelfde vraag als de server).
Elke Fractie van nu vult de Stoelen van haar Fractie in de PDF; een afgesplitste Fractie zit in
het blok van de Fractie waar ze uit kwam (besluit #14). De PDF noemt geen namen: binnen een blok
krijgt de fractievoorzitter de voorste Stoel, daarna de rest op volgorde van de FractieZetel
(`Gewicht`), en een afgesplitste Fractie de achterste Stoelen van het blok.
"""

import collections
import json
import re
import sys
import urllib.parse
import urllib.request

import numpy as np
import pymupdf
from scipy.optimize import linear_sum_assignment

PDF = "https://www.tweedekamer.nl/sites/default/files/2025-11/202511%20Zetelverdeling%20plenaire%20zaal.pdf"
OPEN_DATA = "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0"

# Vulkleur van een Stoel in de PDF -> de Fractie zoals de legenda haar noemt.
VULLING = {
    (0.044, 0.695, 0.295): "D66", (0.094, 0.307, 0.465): "PVV", (0.951, 0.562, 0.124): "VVD",
    (0.0, 0.503, 0.4): "CDA", (0.129, 0.227, 0.423): "JA21", (0.564, 0.218, 0.209): "FVD",
    (0.546, 0.743, 0.286): "BBB", (0.047, 0.617, 0.852): "ChristenUnie",
    (0.949, 0.397, 0.131): "SGP", (0.934, 0.19, 0.166): "SP", (0.0, 0.448, 0.212): "PvdD",
    (0.218, 0.751, 0.759): "DENK", (0.571, 0.151, 0.562): "50Plus", (0.192, 0.145, 0.398): "VOLT",
}
# Eén PVV-Stoel staat alleen als vlak zonder rand in de onderste laag van de tekening.
PVV_ZONDER_RAND = (0.087, 0.203, 0.379)
LEGENDA = {
    "D66": 26, "PVV": 26, "VVD": 22, "GroenLinks/PvdA": 20, "CDA": 18, "JA21": 9, "FVD": 7,
    "BBB": 4, "SP": 3, "PvdD": 3, "SGP": 3, "ChristenUnie": 3, "DENK": 3, "50Plus": 2, "VOLT": 1,
}
# Fractie van nu (afkorting in de open data) -> het blok in de PDF: een andere naam ...
ANDERE_NAAM = {"PRO": "GroenLinks/PvdA", "50PLUS": "50Plus", "Volt": "VOLT"}
# ... of afgesplitst sinds november 2025.
AFGESPLITST = {"Groep Markuszower": "PVV", "Lid Keijzer": "BBB"}
STOELEN_PER_RIJ = [2, 3, 4, 5, 5, 6]
STOELEN = [(v, r, n) for v in range(1, 7) for r, k in enumerate(STOELEN_PER_RIJ, 1) for n in range(1, k + 1)]
MIDDEN_WAAIER = (255.0, 280.0)  # in punten van de PDF, ongeveer bij de interruptiemicrofoons


def stoelplekken(zaalkaart):
    tekst = open(zaalkaart, encoding="utf-8").read().split("STOELPLEKKEN")[1].split("= [", 1)[1].split("];")[0]
    plek = [(float(x), float(z)) for x, _, z in re.findall(r"\[(-?[\d.]+), (-?[\d.]+), (-?[\d.]+)\]", tekst)]
    assert len(plek) == 150, len(plek)
    return np.array(plek)


def vlakken(pagina):
    """Elk Stoel-vlak van de bovenste laag: (u, v, Fractie of None bij alleen een rand)."""
    teken = pagina.get_drawings()
    uit = []
    for i, x in enumerate(teken):
        r = x["rect"]
        f = tuple(round(v, 3) for v in x["fill"]) if x.get("fill") else None
        zonder_rand = x["type"] == "f" and f == PVV_ZONDER_RAND
        if i < len(teken) // 2 - 30 and not zonder_rand:
            continue  # de onderste laag (een schaduw van de bovenste)
        if not (8 < r.width < 40 and 8 < r.height < 40 and 250 < r.y0 < 480):
            continue
        if x["type"] == "fs" and f in VULLING:
            fractie = VULLING[f]
        elif zonder_rand:
            fractie = "PVV"
        elif x["type"] == "s":
            fractie = None
        else:
            continue
        c = ((r.x0 + r.x1) / 2, (r.y0 + r.y1) / 2)
        if not any(abs(c[0] - a) < 6 and abs(c[1] - b) < 6 for a, b, _ in uit):
            uit.append((c[0], c[1], fractie))
    assert len(uit) == 150, f"{len(uit)} vlakken in plaats van 150"
    return uit


def koppel(plek, vlak):
    """Per Stoel (plaats in STOELEN) het vlak in de PDF."""
    q = np.array([(a, b) for a, b, _ in vlak])
    hoek = np.degrees(np.arctan2(q[:, 1] - MIDDEN_WAAIER[1], q[:, 0] - MIDDEN_WAAIER[0]))
    volg = np.argsort(-hoek)  # vak 1 staat links in de PDF
    gaten = sorted(np.argsort(np.diff(-hoek[volg]))[-5:].tolist())
    groepen = np.split(volg, [g + 1 for g in gaten])
    assert [len(g) for g in groepen] == [25] * 6, [len(g) for g in groepen]
    x = np.c_[plek, np.ones(150)]
    uit = {}
    for vak, groep in enumerate(groepen, 1):
        ix = [i for i, s in enumerate(STOELEN) if s[0] == vak]
        xv, qv = x[ix], q[groep]
        # model naar PDF: de Voorzitter boven, vak 1 links (180 graden gedraaid)
        a = np.array([[-15.0, 0.2], [-0.4, -15.6], [255.0, 265.0]])
        for _ in range(30):
            kost = (((xv @ a)[:, None, :] - qv[None, :, :]) ** 2).sum(2)
            ri, ci = linear_sum_assignment(kost)
            a, *_ = np.linalg.lstsq(xv[ri], qv[ci], rcond=None)
        rest = np.sqrt(kost[ri, ci])
        print(f"vak {vak}: afwijking gemiddeld {rest.mean():.1f} pt, hoogstens {rest.max():.1f} pt")
        for r, c in zip(ri, ci):
            uit[ix[r]] = int(groep[c])
    return uit


def fracties_per_stoel(pdf, zaalkaart, beeld=None):
    pagina = pymupdf.open(pdf)[0]
    vlak = vlakken(pagina)
    bij = koppel(stoelplekken(zaalkaart), vlak)
    s = 6
    pix = pagina.get_pixmap(matrix=pymupdf.Matrix(s, s))

    def kleur(u, v):
        c = pix.pixel(int(u * s), int(v * s))
        return VULLING[min(VULLING, key=lambda k: sum((a - b / 255) ** 2 for a, b in zip(k, c)))]

    uit = []
    for i in range(150):
        u, v, fractie = vlak[bij[i]]
        if fractie is None:
            monster = collections.Counter(kleur(u + du, v + dv) for du in (-3, -1.5, 0, 1.5, 3) for dv in (-3, -1.5, 0, 1.5, 3))
            # gestreept groen en rood: GroenLinks/PvdA
            fractie = "GroenLinks/PvdA" if monster["BBB"] >= 3 and monster["SP"] >= 3 else monster.most_common(1)[0][0]
        uit.append((fractie, u, v))
    telling = collections.Counter(f for f, _, _ in uit)
    assert dict(telling) == LEGENDA, telling
    if beeld:
        from PIL import Image, ImageDraw
        img = Image.frombytes("RGB", (pix.width, pix.height), pix.samples)
        d = ImageDraw.Draw(img)
        for (vak, rij, nr), (_, u, v) in zip(STOELEN, uit):
            d.ellipse([(u - 2) * s, (v - 2) * s, (u + 2) * s, (v + 2) * s], outline="white", width=3)
            d.text(((u - 4) * s, (v - 6) * s), f"{vak}{rij}{nr}", fill="black")
        img.crop((40 * s, 260 * s, 470 * s, 480 * s)).save(beeld)
    return [f for f, _, _ in uit]


def kamer_op(peildatum):
    geldt = f"Verwijderd eq false and (TotEnMet eq null or TotEnMet ge {peildatum})"
    vraag = {
        "$filter": "Verwijderd eq false and FractieZetelPersoon/any(a:a/Verwijderd eq false and "
        f"(a/TotEnMet eq null or a/TotEnMet ge {peildatum}))",
        "$select": "Id,Roepnaam,Voornamen,Tussenvoegsel,Achternaam",
        "$expand": f"FractieZetelPersoon($filter={geldt};$select=Functie,Van,TotEnMet;"
        "$expand=FractieZetel($select=Gewicht;$expand=Fractie($select=Afkorting)))",
    }
    adres = OPEN_DATA + "/Persoon?" + urllib.parse.urlencode(vraag)
    personen = []
    while adres:
        with urllib.request.urlopen(adres, timeout=60) as r:
            d = json.load(r)
        personen += d["value"]
        adres = d.get("@odata.nextLink")
    kamer = []
    for p in personen:
        for z in p["FractieZetelPersoon"]:
            van, tot = (z["Van"] or "")[:10], (z["TotEnMet"] or "")[:10]
            fz = z.get("FractieZetel") or {}
            if van <= peildatum and (not tot or peildatum <= tot) and fz.get("Fractie"):
                roep = p["Roepnaam"] or (p["Voornamen"] or "").split(" ")[0]
                naam = " ".join(t.strip() for t in (roep, p["Tussenvoegsel"] or "", p["Achternaam"]) if t and t.strip())
                kamer.append({"id": p["Id"], "naam": naam, "fractie": fz["Fractie"]["Afkorting"],
                              "voorzitter": z["Functie"] == "Fractievoorzitter", "gewicht": fz.get("Gewicht") or 0})
                break
    assert len(kamer) == 150, f"{len(kamer)} Kamerleden op {peildatum}"
    return kamer


def main():
    pdf, peildatum, zaalkaart, uit = sys.argv[1:5]
    beeld = sys.argv[5] if len(sys.argv) > 5 else None
    fractie = fracties_per_stoel(pdf, zaalkaart, beeld)
    kamer = kamer_op(peildatum)
    blok = collections.defaultdict(list)  # Fractie in de PDF -> Stoelen, voorste eerst
    for i in sorted(range(150), key=lambda i: (STOELEN[i][1], i)):
        blok[fractie[i]].append(i)
    leden = collections.defaultdict(list)
    for k in kamer:
        f = k["fractie"]
        leden[AFGESPLITST.get(f) or ANDERE_NAAM.get(f) or f].append(k)
    kamerlid = [None] * 150
    for naam, stoelen in blok.items():
        wie = sorted(leden[naam], key=lambda k: (k["fractie"] in AFGESPLITST, k["fractie"], not k["voorzitter"], k["gewicht"], k["naam"]))
        assert len(wie) == len(stoelen), f"{naam}: {len(wie)} Kamerleden voor {len(stoelen)} Stoelen"
        for i, k in zip(stoelen, wie):
            kamerlid[i] = k
    regels = [
        f'    {{ "vak": {v}, "rij": {r}, "nummer": {n}, "kamerlid": "{k["id"]}", "naam": {json.dumps(k["naam"], ensure_ascii=False)} }}'
        for (v, r, n), k in zip(STOELEN, kamerlid)
    ]
    kop = {
        "kamerperiode": "2025",
        "bron": f"Zetelverdeling plenaire zaal, november 2025 ({PDF}); Kamerleden uit de open data op {peildatum}",
    }
    with open(uit, "w", encoding="utf-8") as f:
        f.write("{\n")
        for k, v in kop.items():
            f.write(f'  "{k}": {json.dumps(v, ensure_ascii=False)},\n')
        f.write('  "stoelen": [\n' + ",\n".join(regels) + "\n  ]\n}\n")
    print(f"{uit}: 150 Stoelen")


if __name__ == "__main__":
    main()
