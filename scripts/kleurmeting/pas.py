"""Legt een rechte lijn door de factoren per kleur van itereer.py: licht en kleur als functie van de
lichtheid L, de kleurigheid C en de tint van de kleur (Oklab). Zo komen de getallen van Kalibratie in
crates/renderer/src/zaal.rs tot stand.
gebruik: pas.py <factoren.json> ..."""
import json, numpy as np, math, sys
f = {}
for p in sys.argv[1:]:
    f.update(json.load(open(p)))
def oklab(h):
    c = np.array([int(h[i:i+2],16) for i in (1,3,5)])/255
    l = np.where(c<=0.04045, c/12.92, ((c+0.055)/1.055)**2.4)
    M1 = np.array([[0.4122214708,0.5363325363,0.0514459929],[0.2119034982,0.6806995451,0.1073969566],[0.0883024619,0.2817188376,0.6299787005]])
    lms = np.cbrt(M1@l)
    M2 = np.array([[0.2104542553,0.7936177850,-0.0040720468],[1.9779984951,-2.4285922050,0.4505937099],[0.0259040371,0.7827717662,-0.8086757660]])
    L,a,b = M2@lms; return L, math.hypot(a,b), math.atan2(b,a)
rows = []
for k,(l,c,de) in f.items():
    L,C,H = oklab(k); rows.append((k,L,C,H,l,c,de))
def feat(L,C,H): return [1, L, C, math.cos(H)*C, math.sin(H)*C]
X = np.array([feat(*r[1:4]) for r in rows]); W = np.array([1.0 if (r[6] or 0) < 6 else 0.5 for r in rows])
for naam, idx in (('licht',4),('kleur',5)):
    y = np.array([r[idx] for r in rows])
    b = np.linalg.lstsq(X*W[:,None], y*W, rcond=None)[0]; pred = X@b
    print(naam, 'coef', [round(v,3) for v in b], 'rms %.3f max %.3f' % (np.sqrt(np.mean((pred-y)**2)), np.abs(pred-y).max()))
    for r,p in zip(rows,pred): print('   ', r[0], 'nodig %.2f model %.2f' % (r[idx], p))
