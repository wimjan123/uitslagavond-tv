//! Portretfoto's vrijstaand: de witte studio-achtergrond van de foto's van de Tweede Kamer wordt
//! doorzichtig, zodat het paneel van de Persoonskaart achter het hoofd doorloopt in plaats van een
//! wit vierkant dat het helderste vlak in beeld is (council ronde 4).
//!
//! Alleen bijna-wit en grijsloos licht dat vanaf de bovenrand of de zijranden (bovenste 80
//! procent) aan elkaar vastzit hoort bij de achtergrond: een wit overhemd onderaan, wit haar of de
//! ogen zitten daar niet aan vast of zijn niet zo wit. Een foto met een donkere achtergrond blijft
//! zoals hij is. De rand van het masker wordt één pixel opgerekt en zacht gemaakt, zodat er geen
//! witte zoom om het haar blijft.

use std::collections::VecDeque;

use image::{RgbImage, RgbaImage};

/// Breder hoeft een foto niet: de Persoonskaart toont hem op 400 tot 480 pixels.
pub const MAX_BREEDTE: u32 = 800;
/// Zo licht moet het donkerste kanaal zijn, en zo dicht de kanalen bij elkaar.
const WIT: u8 = 222;
const GRIJSLOOS: u8 = 22;
/// Tot zoveel van de hoogte zoekt het masker langs de zijranden naar achtergrond.
const ZIJ_TOT: f32 = 0.8;
/// Is meer dan dit deel achtergrond, dan klopt er iets niet en blijft de foto zoals hij is.
const HOOGSTENS: f32 = 0.85;

/// Welke pixels achtergrond zijn (rij voor rij).
pub fn achtergrond(foto: &RgbImage) -> Vec<bool> {
    let (w, h) = (foto.width() as usize, foto.height() as usize);
    let kan = |x: usize, y: usize| {
        let p = foto.get_pixel(x as u32, y as u32).0;
        let (lo, hi) = (p.iter().min().copied().unwrap_or(0), p.iter().max().copied().unwrap_or(0));
        lo > WIT && hi - lo < GRIJSLOOS
    };
    let mut bg = vec![false; w * h];
    let mut rij = VecDeque::new();
    let zaai = |x: usize, y: usize, bg: &mut Vec<bool>, rij: &mut VecDeque<(usize, usize)>| {
        if !bg[y * w + x] && kan(x, y) {
            bg[y * w + x] = true;
            rij.push_back((x, y));
        }
    };
    for x in 0..w {
        zaai(x, 0, &mut bg, &mut rij);
    }
    for y in 0..((h as f32 * ZIJ_TOT) as usize).min(h) {
        zaai(0, y, &mut bg, &mut rij);
        zaai(w - 1, y, &mut bg, &mut rij);
    }
    while let Some((x, y)) = rij.pop_front() {
        let buren = [(x.wrapping_sub(1), y), (x + 1, y), (x, y.wrapping_sub(1)), (x, y + 1)];
        for (nx, ny) in buren {
            if nx < w && ny < h {
                zaai(nx, ny, &mut bg, &mut rij);
            }
        }
    }
    bg
}

/// De foto met een doorzichtige achtergrond, hooguit [`MAX_BREEDTE`] breed.
pub fn maak_vrijstaand(foto: &RgbImage) -> RgbaImage {
    let foto = if foto.width() > MAX_BREEDTE {
        let h = (foto.height() as f32 * MAX_BREEDTE as f32 / foto.width() as f32).round() as u32;
        image::imageops::resize(foto, MAX_BREEDTE, h.max(1), image::imageops::FilterType::Triangle)
    } else {
        foto.clone()
    };
    let (w, h) = (foto.width() as usize, foto.height() as usize);
    let bg = achtergrond(&foto);
    let aantal = bg.iter().filter(|&&b| b).count();
    let mut dekking = vec![255u8; w * h];
    if aantal > 0 && (aantal as f32) < HOOGSTENS * (w * h) as f32 {
        // één pixel oprekken (geen witte zoom), dan zacht over een vak van 5 bij 5
        let op = |x: isize, y: isize| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && bg[y as usize * w + x as usize];
        let mut ruim = vec![0f32; w * h];
        for y in 0..h as isize {
            for x in 0..w as isize {
                let raak = (-1..=1).any(|dy| (-1..=1).any(|dx| op(x + dx, y + dy)));
                ruim[y as usize * w + x as usize] = if raak { 1.0 } else { 0.0 };
            }
        }
        let zacht = vervaag(&vervaag(&ruim, w, h, true), w, h, false);
        for (d, a) in dekking.iter_mut().zip(zacht) {
            *d = ((1.0 - a) * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
    RgbaImage::from_fn(w as u32, h as u32, |x, y| {
        let p = foto.get_pixel(x, y).0;
        image::Rgba([p[0], p[1], p[2], dekking[y as usize * w + x as usize]])
    })
}

/// Gemiddelde over vijf pixels in een richting.
fn vervaag(v: &[f32], w: usize, h: usize, liggend: bool) -> Vec<f32> {
    let mut uit = vec![0f32; v.len()];
    for y in 0..h {
        for x in 0..w {
            let mut som = 0.0;
            let mut n = 0.0;
            for d in -2isize..=2 {
                let (nx, ny) = if liggend { (x as isize + d, y as isize) } else { (x as isize, y as isize + d) };
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    som += v[ny as usize * w + nx as usize];
                    n += 1.0;
                }
            }
            uit[y * w + x] = som / n;
        }
    }
    uit
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Een portret van 40 bij 50: witte achtergrond, een donker hoofd in het midden met wit haar
    /// erin, en onderaan een wit overhemd dat niet aan de achtergrond vastzit.
    fn portret() -> RgbImage {
        RgbImage::from_fn(40, 50, |x, y| {
            let hoofd = (10..30).contains(&x) && y >= 10;
            let haar = hoofd && (14..26).contains(&x) && (12..16).contains(&y);
            let hemd = hoofd && (16..24).contains(&x) && y >= 40;
            if haar || hemd {
                image::Rgb([246, 246, 244])
            } else if hoofd {
                image::Rgb([90, 70, 60])
            } else {
                image::Rgb([250, 250, 250])
            }
        })
    }

    #[test]
    fn de_witte_achtergrond_wordt_doorzichtig_en_wit_in_het_portret_blijft() {
        let uit = maak_vrijstaand(&portret());
        assert_eq!(uit.get_pixel(2, 2).0[3], 0, "hoek is achtergrond");
        assert_eq!(uit.get_pixel(2, 45).0[3], 0, "zijkant onderaan hangt aan de achtergrond");
        assert_eq!(uit.get_pixel(20, 13).0[3], 255, "wit haar binnen het hoofd blijft");
        assert_eq!(uit.get_pixel(20, 45).0[3], 255, "het overhemd blijft");
        assert_eq!(uit.get_pixel(20, 25).0[3], 255, "het gezicht blijft");
    }

    #[test]
    fn een_foto_met_een_donkere_achtergrond_blijft_zoals_hij_is() {
        let donker = RgbImage::from_fn(30, 30, |x, _| if x < 15 { image::Rgb([20, 20, 30]) } else { image::Rgb([200, 120, 90]) });
        assert!(maak_vrijstaand(&donker).pixels().all(|p| p.0[3] == 255));
        // helemaal wit: geen portret, niets weg
        let wit = RgbImage::from_pixel(30, 30, image::Rgb([255, 255, 255]));
        assert!(maak_vrijstaand(&wit).pixels().all(|p| p.0[3] == 255));
    }

    #[test]
    fn een_grote_foto_wordt_kleiner() {
        let groot = RgbImage::from_pixel(2000, 2600, image::Rgb([90, 70, 60]));
        let uit = maak_vrijstaand(&groot);
        assert_eq!((uit.width(), uit.height()), (MAX_BREEDTE, 1040));
    }
}
