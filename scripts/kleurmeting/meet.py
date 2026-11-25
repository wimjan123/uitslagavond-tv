"""Meet per kleur de kleur op de Stoelen in een beeld vanaf Standpunt A (ΔE2000 tegen de kleur uit de Stand):
per Stoel twee plekjes van 3x3 pixels op de rugleuning, links en rechts van het wapen, en daarvan per kleur
de mediaan in Lab. Met KLEUR_JSON=<pad> ook als JSON.
gebruik: meet.py <crates/gedeeld/src/zaalkaart.rs> <stand.json> <beeld.png> [camerastand-json]"""
import sys, json, math, re
import numpy as np
from PIL import Image
zk, standpad, beeld = sys.argv[1:4]
src = open(zk).read()
blok = src.split("pub const STOELPLEKKEN")[1].split("= [",1)[1].split("];")[0]
P = [tuple(map(float, m)) for m in re.findall(r"\[([-\d.]+), ([-\d.]+), ([-\d.]+)\]", blok)]
stand = json.load(open(standpad))['stand']
A = next(p['stand'] for p in stand['standpunten'] if p['letter']=='A')
if len(sys.argv) > 4: A = json.loads(sys.argv[4])
kleuren = stand['stoelkleuren']
partijen = {}
for p in stand['cijfers']['partijen'] if isinstance(stand['cijfers'].get('partijen'), list) else []:
    partijen[p['kleur'].lower()] = p.get('kort') or p.get('naam')
def in_beeld(c, p):
    pos, draai, hel, fov = c['positie'], c['draai'], c['helling'], c['beeldhoek']
    sd, cd = math.sin(draai), math.cos(draai); sh, ch = math.sin(hel), math.cos(hel)
    v = (-sd*ch, sh, -cd*ch); r = (cd, 0, -sd)
    b = (r[1]*v[2]-r[2]*v[1], r[2]*v[0]-r[0]*v[2], r[0]*v[1]-r[1]*v[0])
    d = [p[i]-pos[i] for i in range(3)]; dot = lambda a, e: sum(a[i]*e[i] for i in range(3))
    z = dot(d, v); th = math.tan(fov/2); tw = th*16/9
    return ((dot(d, r)/z/tw+1)/2*1920, (1-dot(d, b)/z/th)/2*1080)
def srgb_naar_lab(c):
    c = np.asarray(c, dtype=float)/255.0
    l = np.where(c <= 0.04045, c/12.92, ((c+0.055)/1.055)**2.4)
    M = np.array([[0.4124,0.3576,0.1805],[0.2126,0.7152,0.0722],[0.0193,0.1192,0.9505]])
    xyz = l @ M.T / np.array([0.95047,1.0,1.08883])
    f = np.where(xyz > (6/29)**3, np.cbrt(xyz), xyz/(3*(6/29)**2)+4/29)
    return np.stack([116*f[...,1]-16, 500*(f[...,0]-f[...,1]), 200*(f[...,1]-f[...,2])], -1)
def de2000(a, b):
    L1,a1,b1 = a; L2,a2,b2 = b
    C1, C2 = math.hypot(a1,b1), math.hypot(a2,b2); Cm = (C1+C2)/2
    G = 0.5*(1-math.sqrt(Cm**7/(Cm**7+25**7)))
    a1p, a2p = a1*(1+G), a2*(1+G)
    C1p, C2p = math.hypot(a1p,b1), math.hypot(a2p,b2)
    h1 = math.degrees(math.atan2(b1,a1p))%360; h2 = math.degrees(math.atan2(b2,a2p))%360
    dL = L2-L1; dC = C2p-C1p
    dh = h2-h1
    if C1p*C2p == 0: dh = 0
    elif dh > 180: dh -= 360
    elif dh < -180: dh += 360
    dH = 2*math.sqrt(C1p*C2p)*math.sin(math.radians(dh/2))
    Lm = (L1+L2)/2; Cmp = (C1p+C2p)/2
    hm = h1+h2
    if C1p*C2p != 0:
        hm = (h1+h2+360)/2 if abs(h1-h2) > 180 else (h1+h2)/2
    T = 1-0.17*math.cos(math.radians(hm-30))+0.24*math.cos(math.radians(2*hm))+0.32*math.cos(math.radians(3*hm+6))-0.20*math.cos(math.radians(4*hm-63))
    dth = 30*math.exp(-((hm-275)/25)**2)
    Rc = 2*math.sqrt(Cmp**7/(Cmp**7+25**7))
    Sl = 1+0.015*(Lm-50)**2/math.sqrt(20+(Lm-50)**2); Sc = 1+0.045*Cmp; Sh = 1+0.015*Cmp*T
    Rt = -math.sin(math.radians(2*dth))*Rc
    return math.sqrt((dL/Sl)**2+(dC/Sc)**2+(dH/Sh)**2+Rt*(dC/Sc)*(dH/Sh))
img = np.asarray(Image.open(beeld).convert('RGB')).astype(float)
groep = {}
for i, p in enumerate(P):
    if i >= len(kleuren): break
    k = kleuren[i].lower()
    # opzij van het wapen, een stukje onder de bovenrand van de rugleuning
    r = math.hypot(p[0], p[2]); t = (-p[2]/r, p[0]/r)
    for zij in (-0.11, 0.11):
        q = (p[0]+t[0]*zij, p[1]+0.05, p[2]+t[1]*zij)
        x, y = in_beeld(A, q); x, y = int(round(x)), int(round(y))
        if 2 <= x < 1918 and 362 <= y < 1078:
            groep.setdefault(k, []).append(img[y-1:y+2, x-1:x+2].reshape(-1,3))
rijen = []
for k, stukken in sorted(groep.items()):
    px = np.concatenate(stukken)
    lab = srgb_naar_lab(px); med = np.median(lab, 0)
    doel = srgb_naar_lab([int(k[1:3],16), int(k[3:5],16), int(k[5:7],16)])
    gemeten_srgb = np.median(px, 0)
    d = de2000(tuple(doel), tuple(med))
    rijen.append((d, partijen.get(k, k), k, '#%02x%02x%02x' % tuple(int(v) for v in gemeten_srgb), tuple(round(v,1) for v in doel), tuple(round(v,1) for v in med)))
for r in sorted(rijen, reverse=True): print('%5.1f  %-10s doel %s gemeten %s  Lab doel %s gemeten %s' % r)
print('gemiddeld ΔE2000 %.1f, max %.1f' % (sum(r[0] for r in rijen)/len(rijen), max(r[0] for r in rijen)))
import os
if os.environ.get('KLEUR_JSON'):
    json.dump({r[2]: {'naam': r[1], 'de': r[0], 'doel': [float(v) for v in r[4]], 'gemeten': [float(v) for v in r[5]]} for r in rijen}, open(os.environ['KLEUR_JSON'], 'w'))
