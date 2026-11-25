// Occlusie uit de diepte (zie occlusie.rs): hoeken, plooien en de vloer naast een bureau worden
// iets donkerder. Vast patroon, geen ruis: stil beeld blijft stil, ook na compressie.

struct Instellingen {
    // near van de camera (omgekeerde, oneindige diepte: diepte = near / afstand)
    near: f32,
    // tan van de halve beeldhoek, horizontaal en verticaal
    tan_x: f32,
    tan_y: f32,
    // straal in meter waarbinnen iets afdekt
    straal: f32,
    // 0 geen occlusie, 1 volle occlusie
    sterkte: f32,
    _a: f32,
    _b: f32,
    _c: f32,
}

@group(0) @binding(0) var bron: texture_2d<f32>;
@group(0) @binding(1) var bron_sampler: sampler;
@group(0) @binding(2) var<uniform> s: Instellingen;
@group(1) @binding(0) var diepte: texture_depth_multisampled_2d;

const AANTAL: i32 = 16;
const GULDEN_HOEK: f32 = 2.39996323;

// De plek in de ruimte van de camera bij een pixel; z negatief, voor de camera.
fn plek(px: vec2<i32>, maat: vec2<f32>) -> vec3<f32> {
    let p = clamp(px, vec2<i32>(0), vec2<i32>(maat) - 1);
    let d = max(textureLoad(diepte, p, 0), 1e-7);
    let z = s.near / d;
    let uv = (vec2<f32>(p) + 0.5) / maat;
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    return vec3<f32>(ndc.x * s.tan_x * z, ndc.y * s.tan_y * z, -z);
}

@fragment
fn fragment(@builtin(position) positie: vec4<f32>, @location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let kleur = textureSample(bron, bron_sampler, uv);
    let maat = vec2<f32>(textureDimensions(diepte));
    let px = vec2<i32>(positie.xy);
    let d = textureLoad(diepte, px, 0);
    // niets getekend (achtergrond) of te ver weg
    if (d <= 0.0 || s.sterkte <= 0.0) {
        return kleur;
    }
    let p = plek(px, maat);

    // de normaal uit de buren, aan de kant die het minst verspringt (randen van voorwerpen)
    let l = plek(px - vec2<i32>(1, 0), maat);
    let r = plek(px + vec2<i32>(1, 0), maat);
    let o = plek(px + vec2<i32>(0, 1), maat);
    let b = plek(px - vec2<i32>(0, 1), maat);
    let dx = select(p - l, r - p, abs(r.z - p.z) < abs(p.z - l.z));
    let dy = select(p - b, o - p, abs(o.z - p.z) < abs(p.z - b.z));
    var n = normalize(cross(dx, dy));
    if (dot(n, -p) < 0.0) {
        n = -n;
    }

    // de straal in pixels: zoveel meter op deze afstand
    let afstand = -p.z;
    let straal_px = clamp(s.straal / (afstand * s.tan_y) * maat.y * 0.5, 3.0, 90.0);

    var dicht = 0.0;
    for (var i = 0; i < AANTAL; i++) {
        let t = (f32(i) + 0.5) / f32(AANTAL);
        let hoek = f32(i) * GULDEN_HOEK;
        let verschuiving = vec2<f32>(cos(hoek), sin(hoek)) * sqrt(t) * straal_px;
        let q = plek(px + vec2<i32>(round(verschuiving)), maat);
        let v = q - p;
        let lengte = length(v);
        // alleen wat dichtbij is dekt af; verder weg loopt het zacht uit
        let bereik = 1.0 - smoothstep(0.5, 1.0, lengte / s.straal);
        // hoe ver het punt boven het vlak uitsteekt, als hoek: een vlak dekt zichzelf niet af
        dicht += max(dot(v, n) / max(lengte, 1e-4) - 0.15, 0.0) * bereik;
    }
    let ao = clamp(1.0 - dicht / f32(AANTAL) * 1.5, 0.0, 1.0);
    let factor = mix(1.0, ao, s.sterkte);
    return vec4<f32>(kleur.rgb * factor, kleur.a);
}
