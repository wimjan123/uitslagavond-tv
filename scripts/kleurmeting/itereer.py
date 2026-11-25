"""Stelt de kalibratie per kleur bij tot de render klopt: 24 kleuren (de partijkleuren van 2025 en negen
andere rond de kleurencirkel) verspreid over alle Stoelen, dan per kleur de lichtheid en kleurigheid bij
naar wat de meting zegt. Schrijft per ronde <uit>/factoren.json; pas.py legt er een lijn door.
gebruik: W=<repo> itereer.py <rondes> [factoren.json om mee te beginnen]   (ALLEEN_METEN=1: één meting)"""
import json, os, subprocess, math, sys
G = os.path.dirname(os.path.abspath(__file__)); W = os.environ['W']; PY = sys.executable
basis = os.environ.get('KAL', '0.87,0.88,0.2,3.1,-1.5,0.6,0.15'); bl, bc = 1.4, 1.6
partij = ['#0a8a74','#1e9be6','#22b5c2','#2b63d9','#2dbe4f','#3a4fb8','#5a2d91','#5e8f23','#9b2d8f','#a3243a','#a8c81e','#d0213a','#e6a11b','#f0412a','#ff7a1a']
extra = ['#e84a8a','#f5d300','#7ac143','#00b7eb','#1d2d6b','#8b5a2b','#9ac8eb','#00594f','#f6a5c0']
alle = partij + extra
kleuren = [alle[(i * 7) % len(alle)] for i in range(150)]
f = json.load(open(sys.argv[2])) if len(sys.argv) > 2 else {}
f = {k: tuple(v[:2]) for k, v in f.items()}
for it in range(int(sys.argv[1])):
    per = ';'.join(f'{k[1:]}:{v[0]:.4f}:{v[1]:.4f}' for k, v in f.items())
    uit = os.path.join(os.environ.get('UIT', '/tmp/kleurmeting'), f'ronde{it}')
    env = dict(os.environ, UITSLAGAVOND_KAL=basis, UITSLAGAVOND_KAL_PER=per, W=W, KLEUREN=json.dumps(kleuren))
    subprocess.run([f'{G}/render.sh', uit], env=env, capture_output=True)
    beeld = sorted(os.listdir(f'{uit}/reeks'))[-1]
    env['KLEUR_JSON'] = f'{uit}/kleur.json'
    r = subprocess.run([PY, f'{G}/meet.py', f'{W}/crates/gedeeld/src/zaalkaart.rs', f'{uit}/stand.json', f'{uit}/reeks/{beeld}'], env=env, capture_output=True, text=True)
    print('iteratie', it, r.stdout.strip().splitlines()[-1], flush=True)
    m = json.load(open(f'{uit}/kleur.json'))
    if os.environ.get('ALLEEN_METEN'): break
    for k, v in m.items():
        l, c = f.get(k, (bl, bc))
        Lt, Lm = v['doel'][0], v['gemeten'][0]
        Ct, Cm = math.hypot(*v['doel'][1:]), math.hypot(*v['gemeten'][1:])
        f[k] = (min(l * (Lt / max(Lm, 1)) ** 0.7, 4.0), min(c * (Ct / max(Cm, 1)) ** 0.5, 3.0))
    json.dump({k: [v[0], v[1], m.get(k, {}).get('de')] for k, v in f.items()}, open(f'{uit}/factoren.json', 'w'))
for k, v in sorted(m.items(), key=lambda x: -x[1]['de']): print(k, f.get(k), 'ΔE %.1f' % v['de'])
