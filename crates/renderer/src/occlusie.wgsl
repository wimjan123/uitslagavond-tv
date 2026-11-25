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
    // wat dichterbij is dan dit (meter) wordt donkerder, tot voorgrond_donker
    voorgrond: f32,
    voorgrond_donker: f32,
    _c: f32,
    // omhoog in de ruimte van de camera
    omhoog: vec4<f32>,
    // het midden van de Zaal op de vloer (de oorsprong van de wereld) in de ruimte van de camera
    oorsprong: vec4<f32>,
    // lichtkringen: midden (xyz) en straal (w), en het extra licht per kanaal
    kring_a: vec4<f32>,
    kring_a_licht: vec4<f32>,
    kring_b: vec4<f32>,
    kring_b_licht: vec4<f32>,
    // lichtstrook: begin met halve breedte (w), eind met de halve breedte van de kern (w), en
    // het extra licht met dat van de kern (w)
    strook_van: vec4<f32>,
    strook_naar: vec4<f32>,
    strook_licht: vec4<f32>,
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

// Wat er van v overblijft als je de hoogte eruit haalt.
fn vlak(v: vec3<f32>) -> vec3<f32> {
    return v - s.omhoog.xyz * dot(v, s.omhoog.xyz);
}

// De vloer van de Zaal: de rijen Stoelen staan op treden rond het midden, elke rij 1,325 m verder
// en 0,119 m hoger; de eerste rij en alles ervoor op 0 (zie gedeeld::zaalkaart). 1 op de vloer
// (tapijt en treden), 0 op een bureau, zitting of de tribune.
fn op_de_vloer(p: vec3<f32>) -> f32 {
    let v = p - s.oorsprong.xyz;
    let r = length(vlak(v));
    let rij = clamp(ceil((r - 5.81) / 1.325), 0.0, 5.0);
    let boven = dot(v, s.omhoog.xyz) - 0.119 * rij;
    return 1.0 - smoothstep(0.06, 0.16, abs(boven));
}

// Hoeveel van een lichtkring op punt p valt: 1 in het midden, zacht naar 0 aan de rand.
fn kring(p: vec3<f32>, k: vec4<f32>) -> f32 {
    if (k.w <= 0.0) {
        return 0.0;
    }
    let r = length(vlak(p - k.xyz)) / k.w;
    return 1.0 - smoothstep(0.65, 1.0, r);
}

// Hoeveel van de lichtstrook op punt p valt: zacht over de breedte, met zachte uiteinden.
fn strook(p: vec3<f32>) -> f32 {
    let half = s.strook_van.w;
    if (half <= 0.0) {
        return 0.0;
    }
    let ab = vlak(s.strook_naar.xyz - s.strook_van.xyz);
    let lengte = max(length(ab), 1e-4);
    let langs = ab / lengte;
    let v = vlak(p - s.strook_van.xyz);
    let t = dot(v, langs);
    let opzij = length(v - langs * t);
    let breed = 1.0 - smoothstep(0.15, 1.0, opzij / half);
    let kern = 1.0 - smoothstep(0.6, 1.4, opzij / max(s.strook_naar.w, 1e-3));
    let einden = smoothstep(-0.4, 0.1, t) * (1.0 - smoothstep(lengte - 0.1, lengte + 0.4, t));
    return (breed * breed + kern * s.strook_licht.w) * einden;
}

@fragment
fn fragment(@builtin(position) positie: vec4<f32>, @location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let kleur = textureSample(bron, bron_sampler, uv);
    let maat = vec2<f32>(textureDimensions(diepte));
    let px = vec2<i32>(positie.xy);
    let d = textureLoad(diepte, px, 0);
    // niets getekend (achtergrond)
    if (d <= 0.0) {
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

    // licht op de vloer: alleen wat naar boven kijkt en op de vloer ligt
    let boven = smoothstep(0.55, 0.9, dot(n, s.omhoog.xyz));
    var licht = vec3<f32>(0.0);
    if (boven > 0.0) {
        licht = s.kring_a_licht.rgb * kring(p, s.kring_a) + s.kring_b_licht.rgb * kring(p, s.kring_b)
            + s.strook_licht.rgb * strook(p);
        licht *= boven * op_de_vloer(p);
    }
    // de voorgrond donkerder (de Persoonskaart): het bureau vooraan trekt dan niet het oog
    let voor = s.voorgrond_donker * (1.0 - smoothstep(0.6, 0.92, -p.z / max(s.voorgrond, 1e-3)));
    let verlicht = kleur.rgb * (1.0 + licht) * (1.0 - voor);
    if (s.sterkte <= 0.0) {
        return vec4<f32>(verlicht, kleur.a);
    }

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
    return vec4<f32>(verlicht * factor, kleur.a);
}
