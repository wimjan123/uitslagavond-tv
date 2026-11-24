//! Het gebakken licht uit `lichtkaart.bin` (zie `schrijf_lichtkaart` in `zaal/export.py`).
//!
//! Kop: `LK95`, dan breedte en hoogte als u32 (little-endian); daarna rij voor rij de pixels
//! als RGB9E5 (vier bytes per pixel, bovenste rij eerst). De GPU filtert dat formaat zelf.

use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub fn lees(pad: &Path) -> Image {
    let data = std::fs::read(pad).unwrap_or_else(|e| panic!("{}: {e}", pad.display()));
    let (breedte, hoogte) =
        kop(&data).unwrap_or_else(|| panic!("geen lichtkaart: {}", pad.display()));
    let mut beeld = Image::new(
        Extent3d {
            width: breedte,
            height: hoogte,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data[12..].to_vec(),
        TextureFormat::Rgb9e5Ufloat,
        RenderAssetUsages::RENDER_WORLD,
    );
    beeld.sampler = ImageSampler::linear();
    beeld
}

/// Breedte en hoogte, als de kop klopt en de pixels er allemaal zijn.
fn kop(data: &[u8]) -> Option<(u32, u32)> {
    if data.get(0..4)? != b"LK95" {
        return None;
    }
    let breedte = u32::from_le_bytes(data.get(4..8)?.try_into().ok()?);
    let hoogte = u32::from_le_bytes(data.get(8..12)?.try_into().ok()?);
    (data.len() as u64 == 12 + 4 * u64::from(breedte) * u64::from(hoogte))
        .then_some((breedte, hoogte))
}
