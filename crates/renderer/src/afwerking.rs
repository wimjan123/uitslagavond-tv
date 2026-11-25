//! De afwerking van de Zaal na het laden, zodat hij leest als een studio en niet als een game:
//! matter en minder oranje hout, matte stof op de Stoelen, een rustiger tapijt, een donkerder en
//! koeler plafond, en mipmaps voor alle texturen (zonder flikkert een fijn patroon in de verte
//! bij compressie). Het model zelf blijft zoals `zaal/maak.sh` het bakte.
//!
//! Dit gebeurt in het eerste beeld waarin een materiaal in de wereld komt, vóór de Stoelen en de
//! dimmer van de Zaal (`zaal.rs`) de kleur ervan onthouden.

use std::collections::HashSet;

use bevy::gltf::GltfMaterialName;
use bevy::image::{ImageFilterMode, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

/// Wat de Renderer per soort materiaal bijstelt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Afwerking {
    /// Hoeveel van de verzadiging van de kleur overblijft (1 is zoals het model).
    pub verzadiging: f32,
    /// Factor op de kleur per kanaal (lineair): donkerder of koeler.
    pub kleur: [f32; 3],
    /// Vaste ruwheid in plaats van de ruwheidstextuur; `None` laat het model staan.
    pub ruwheid: Option<f32>,
    /// Hoe sterk het oppervlak spiegelt (Bevy: 0,5 is 4 procent, het standaardgeval).
    pub glans: Option<f32>,
    /// Contrast van de kleurtextuur rond haar gemiddelde (1 is zoals het model).
    pub contrast: f32,
}

const GEWOON: Afwerking = Afwerking {
    verzadiging: 1.0,
    kleur: [1.0, 1.0, 1.0],
    ruwheid: None,
    glans: None,
    contrast: 1.0,
};

/// Fineer en perenhout: een kwart minder oranje, iets donkerder en mat gelakt in plaats van
/// plastic glans.
pub const HOUT: Afwerking = Afwerking {
    verzadiging: 0.62,
    kleur: [0.86, 0.86, 0.86],
    ruwheid: Some(0.62),
    glans: Some(0.3),
    ..GEWOON
};

/// De stof van de Stoelen: mat, zonder witte glimlichten in een close-up.
pub const STOF: Afwerking = Afwerking {
    ruwheid: Some(0.82),
    glans: Some(0.22),
    ..GEWOON
};

/// Het tapijt: het vlekkenpatroon met minder dan de helft van het contrast, en donkerder.
pub const TAPIJT: Afwerking = Afwerking {
    kleur: [0.74, 0.76, 0.78],
    contrast: 0.4,
    ..GEWOON
};

/// Het plafond: donkerder en koeler, een rustige achtergrond achter de Graphics bovenin.
pub const PLAFOND: Afwerking = Afwerking {
    verzadiging: 0.7,
    kleur: [0.6, 0.62, 0.66],
    ..GEWOON
};

/// De afwerking bij een materiaal uit het model.
pub fn afwerking(materiaal: &str) -> Option<Afwerking> {
    Some(match materiaal {
        "perenhout" | "perenhout_blad" | "middenvak_blad" | "middenvak_paneel" | "middenvak_katheder"
        | "desk_rand" | "desk_leer" | "balies_kastje" | "balies_kader" | "balies_kap" | "balies_bladrand" => HOUT,
        m if m.starts_with("balies_fineer_") => HOUT,
        "stoel_leer" | "stoel_bies" | "stoel_naad" | "balies_leer" => STOF,
        "tapijt" => TAPIJT,
        "plafond" | "daklicht_plafond" => PLAFOND,
        _ => return None,
    })
}

impl Afwerking {
    /// De nieuwe kleur (lineair, alfa blijft).
    pub fn kleur(&self, k: LinearRgba) -> LinearRgba {
        let grijs = 0.2126 * k.red + 0.7152 * k.green + 0.0722 * k.blue;
        let v = self.verzadiging;
        let [r, g, b] = self.kleur;
        LinearRgba::new(
            (grijs + (k.red - grijs) * v) * r,
            (grijs + (k.green - grijs) * v) * g,
            (grijs + (k.blue - grijs) * v) * b,
            k.alpha,
        )
    }

    fn stel_bij(&self, m: &mut StandardMaterial) {
        m.base_color = self.kleur(LinearRgba::from(m.base_color)).into();
        if let Some(r) = self.ruwheid {
            // de textuur draagt ook metaal, maar deze materialen zijn geen metaal
            m.metallic_roughness_texture = None;
            m.perceptual_roughness = r;
            m.metallic = 0.0;
        }
        if let Some(g) = self.glans {
            m.reflectance = g;
        }
    }
}

pub struct AfwerkingPlugin;

impl Plugin for AfwerkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, werk_af.in_set(Afgewerkt));
    }
}

/// Na deze set zijn de materialen in de wereld afgewerkt.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Afgewerkt;

/// Werkt elk materiaal één keer af zodra het in de wereld komt, en geeft elke textuur van de Zaal
/// mipmaps.
fn werk_af(
    nieuw: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>), Added<GltfMaterialName>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut beelden: ResMut<Assets<Image>>,
    mut klaar: Local<HashSet<AssetId<StandardMaterial>>>,
    mut klare_beelden: Local<HashSet<AssetId<Image>>>,
) {
    for (naam, materiaal) in &nieuw {
        if !klaar.insert(materiaal.0.id()) {
            continue;
        }
        let Some(mut m) = materialen.get_mut(&materiaal.0) else {
            klaar.remove(&materiaal.0.id());
            continue;
        };
        let a = afwerking(&naam.0).unwrap_or(GEWOON);
        a.stel_bij(&mut m);
        let texturen = [
            (m.base_color_texture.clone(), a.contrast),
            (m.normal_map_texture.clone(), 1.0),
            (m.metallic_roughness_texture.clone(), 1.0),
            (m.emissive_texture.clone(), 1.0),
            (m.occlusion_texture.clone(), 1.0),
        ];
        for (h, contrast) in texturen.into_iter().filter_map(|(h, c)| Some((h?, c))) {
            if !klare_beelden.insert(h.id()) {
                continue;
            }
            let Some(mut beeld) = beelden.get_mut(&h) else {
                continue;
            };
            if contrast != 1.0 {
                verlaag_contrast(&mut beeld, contrast);
            }
            maak_mipmaps(&mut beeld);
        }
    }
}

fn srgb_naar_lineair(b: u8) -> f32 {
    let c = f32::from(b) / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn lineair_naar_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0).round() as u8
}

/// Haalt het contrast van een kleurtextuur (RGBA8) terug rond haar gemiddelde, in lineair licht.
fn verlaag_contrast(beeld: &mut Image, contrast: f32) {
    let srgb = beeld.texture_descriptor.format == TextureFormat::Rgba8UnormSrgb;
    if !srgb && beeld.texture_descriptor.format != TextureFormat::Rgba8Unorm {
        return;
    }
    let Some(data) = beeld.data.as_mut() else {
        warn!("textuur zonder pixels in het geheugen: contrast blijft");
        return;
    };
    let naar = |b: u8| if srgb { srgb_naar_lineair(b) } else { f32::from(b) / 255.0 };
    let terug = |c: f32| if srgb { lineair_naar_srgb(c) } else { (c.clamp(0.0, 1.0) * 255.0).round() as u8 };
    let pixels = data.len() / 4;
    let mut som = [0.0f64; 3];
    for p in data.chunks_exact(4) {
        for k in 0..3 {
            som[k] += f64::from(naar(p[k]));
        }
    }
    let gem = som.map(|s| (s / pixels.max(1) as f64) as f32);
    for p in data.chunks_exact_mut(4) {
        for k in 0..3 {
            p[k] = terug(gem[k] + (naar(p[k]) - gem[k]) * contrast);
        }
    }
}

/// Maakt de mipmaps van een RGBA8-textuur zonder mipmaps (zoals een JPEG uit het model): elk
/// niveau het gemiddelde van vier pixels, voor een kleurtextuur in lineair licht. Daarbij
/// filtert de GPU schuin en tot 16 keer anisotroop, zodat het tapijt in de verte scherp blijft
/// zonder te flikkeren.
pub fn maak_mipmaps(beeld: &mut Image) {
    let d = &beeld.texture_descriptor;
    let srgb = d.format == TextureFormat::Rgba8UnormSrgb;
    if (!srgb && d.format != TextureFormat::Rgba8Unorm)
        || d.mip_level_count != 1
        || d.size.depth_or_array_layers != 1
    {
        return;
    }
    let (mut w, mut h) = (d.size.width as usize, d.size.height as usize);
    let Some(data) = beeld.data.as_mut() else {
        warn!("textuur zonder pixels in het geheugen: geen mipmaps");
        return;
    };
    if data.len() != w * h * 4 {
        return;
    }
    let mut niveaus = 1;
    let mut vorig: Vec<f32> = data
        .iter()
        .enumerate()
        .map(|(i, b)| if srgb && i % 4 != 3 { srgb_naar_lineair(*b) } else { f32::from(*b) / 255.0 })
        .collect();
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut nieuw = vec![0.0f32; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                let (x0, y0) = ((2 * x).min(w - 1), (2 * y).min(h - 1));
                let (x1, y1) = ((2 * x + 1).min(w - 1), (2 * y + 1).min(h - 1));
                for k in 0..4 {
                    let p = |px: usize, py: usize| vorig[(py * w + px) * 4 + k];
                    nieuw[(y * nw + x) * 4 + k] = (p(x0, y0) + p(x1, y0) + p(x0, y1) + p(x1, y1)) / 4.0;
                }
            }
        }
        data.extend(nieuw.iter().enumerate().map(|(i, c)| {
            if srgb && i % 4 != 3 { lineair_naar_srgb(*c) } else { (c.clamp(0.0, 1.0) * 255.0).round() as u8 }
        }));
        vorig = nieuw;
        (w, h) = (nw, nh);
        niveaus += 1;
    }
    beeld.texture_descriptor.mip_level_count = niveaus;
    // de sampler uit het model houdt zijn herhaling; zonder eigen sampler blijft de standaard
    if let ImageSampler::Descriptor(s) = &mut beeld.sampler {
        s.min_filter = ImageFilterMode::Linear;
        s.mag_filter = ImageFilterMode::Linear;
        s.mipmap_filter = ImageFilterMode::Linear;
        s.anisotropy_clamp = 16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension};

    #[test]
    fn hout_wordt_een_kwart_minder_verzadigd_en_houdt_zijn_tint() {
        // het perenhout uit het model, lineair
        let hout = LinearRgba::rgb(1.0, 0.503, 0.06);
        let na = HOUT.kleur(hout);
        let verzadiging = |k: LinearRgba| {
            let (hoog, laag) = (k.red.max(k.green).max(k.blue), k.red.min(k.green).min(k.blue));
            (hoog - laag) / hoog
        };
        assert!(verzadiging(na) < verzadiging(hout) * 0.8, "{na:?}");
        assert!(na.red > na.green && na.green > na.blue, "nog steeds hout: {na:?}");
    }

    #[test]
    fn mipmaps_lopen_tot_een_pixel_en_middelen_in_lineair_licht() {
        // een dambord van zwart en wit, 4 bij 2: het kleinste niveau is middengrijs in licht
        let pixels: Vec<u8> = (0..8).flat_map(|i| if (i + i / 4) % 2 == 0 { [0, 0, 0, 255] } else { [255; 4] }).collect();
        let mut beeld = Image::new(
            Extent3d { width: 4, height: 2, depth_or_array_layers: 1 },
            TextureDimension::D2,
            pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        maak_mipmaps(&mut beeld);
        assert_eq!(beeld.texture_descriptor.mip_level_count, 3);
        let data = beeld.data.as_ref().unwrap();
        // 4x2 + 2x1 + 1x1 pixels
        assert_eq!(data.len(), (8 + 2 + 1) * 4);
        let laatste = &data[data.len() - 4..];
        // de helft van het licht is in sRGB 188, niet 128
        assert_eq!(laatste[0], 188);
        assert_eq!(laatste[3], 255);
    }

    #[test]
    fn minder_contrast_houdt_het_gemiddelde() {
        let mut beeld = Image::new(
            Extent3d { width: 2, height: 1, depth_or_array_layers: 1 },
            TextureDimension::D2,
            vec![0, 0, 0, 255, 200, 200, 200, 255],
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        verlaag_contrast(&mut beeld, 0.5);
        assert_eq!(beeld.data.unwrap(), vec![50, 50, 50, 255, 150, 150, 150, 255]);
    }
}
