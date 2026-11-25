//! Portretfoto's vrijstaand: de witte studio-achtergrond van de foto's van de Tweede Kamer wordt
//! doorzichtig, zodat het paneel van de Persoonskaart achter het hoofd doorloopt in plaats van een
//! wit vierkant dat het helderste vlak in beeld is (council ronde 4).
//!
//! Alleen bijna-wit en grijsloos licht dat vanaf de bovenrand of de zijranden (bovenste 80
//! procent) aan elkaar vastzit hoort bij de achtergrond: een wit overhemd onderaan, wit haar of de
//! ogen zitten daar niet aan vast of zijn niet zo wit. Een foto met een donkere achtergrond blijft
//! zoals hij is. Ook de lichte zoom langs de achtergrond gaat weg, de rand van het masker wordt
//! één pixel opgerekt en zacht gemaakt, half doorzichtige randpixels worden donkerder, en de
//! zijkanten van de foto lopen zacht weg: zo blijft er geen lichte rand om haar en schouders.

use std::collections::VecDeque;

use image::{RgbImage, RgbaImage};

/// Breder hoeft een foto niet: de Persoonskaart toont hem op 400 tot 480 pixels.
pub const MAX_BREEDTE: u32 = 800;
/// Zo licht moet het donkerste kanaal zijn, en zo dicht de kanalen bij elkaar.
const WIT: u8 = 222;
const GRIJSLOOS: u8 = 22;
/// Tot zoveel van de hoogte zoekt het masker langs de zijranden naar achtergrond.
const ZIJ_TOT: f32 = 0.8;
/// De zoom langs de achtergrond: zo licht en grijsloos.
const ZOOM_LICHT: u8 = 175;
const ZOOM_GRIJSLOOS: u8 = 32;
/// Aan de linker- en rechterkant van de foto vloeit hij over zoveel van zijn breedte weg, zodat
/// de schouders niet in een rechte lijn ophouden.
const ZIJ_VERLOOP: f32 = 0.07;
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
    // De zoom: de lichte, half witte pixels direct langs de achtergrond (waar haar of schouder
    // in het wit overloopt) horen er ook bij, anders blijft er een lichte rand om het portret
    // (council ronde 5). Twee pixels diep, en alleen licht en grijsloos.
    let zoom = |x: usize, y: usize| {
        let p = foto.get_pixel(x as u32, y as u32).0;
        let (lo, hi) = (p.iter().min().copied().unwrap_or(0), p.iter().max().copied().unwrap_or(0));
        lo > ZOOM_LICHT && hi - lo < ZOOM_GRIJSLOOS
    };
    for _ in 0..2 {
        let vorig = bg.clone();
        for y in 0..h {
            for x in 0..w {
                if vorig[y * w + x] || !zoom(x, y) {
                    continue;
                }
                let raakt = (x > 0 && vorig[y * w + x - 1])
                    || (x + 1 < w && vorig[y * w + x + 1])
                    || (y > 0 && vorig[(y - 1) * w + x])
                    || (y + 1 < h && vorig[(y + 1) * w + x]);
                bg[y * w + x] = raakt;
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
        // de zijkanten zacht laten weglopen
        let rand = (w as f32 * ZIJ_VERLOOP).max(1.0);
        for y in 0..h {
            for x in 0..w {
                let t = ((x.min(w - 1 - x) as f32) / rand).min(1.0);
                let f = t * t * (3.0 - 2.0 * t);
                let d = &mut dekking[y * w + x];
                *d = (f32::from(*d) * f).round() as u8;
            }
        }
    }
    RgbaImage::from_fn(w as u32, h as u32, |x, y| {
        let p = foto.get_pixel(x, y).0;
        let d = dekking[y as usize * w + x as usize];
        // Een half doorzichtige randpixel heeft nog wat wit van de achtergrond in zich: maak hem
        // donkerder naarmate hij doorzichtiger is, zodat hij op het donkere paneel niet oplicht.
        let f = 0.5 + 0.5 * f32::from(d) / 255.0;
        let donker = |c: u8| (f32::from(c) * f).round() as u8;
        image::Rgba([donker(p[0]), donker(p[1]), donker(p[2]), d])
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
    fn de_lichte_zoom_om_het_hoofd_gaat_mee_met_de_achtergrond() {
        // een hoofd met een rand van een pixel lichtgrijs, zoals haar dat in het wit overloopt
        let foto = RgbImage::from_fn(40, 50, |x, y| {
            let hoofd = (10..30).contains(&x) && y >= 10;
            let rand = hoofd && (x == 10 || x == 29 || y == 10);
            if rand {
                image::Rgb([205, 203, 200])
            } else if hoofd {
                image::Rgb([90, 70, 60])
            } else {
                image::Rgb([250, 250, 250])
            }
        });
        let uit = maak_vrijstaand(&foto);
        assert!(uit.get_pixel(20, 10).0[3] < 128, "de lichte rand bovenop het hoofd is half weg");
        assert_eq!(uit.get_pixel(20, 25).0[3], 255, "het gezicht blijft");
        // een randpixel die half doorzichtig is, is donkerder dan in de foto
        let p = uit.get_pixel(20, 11).0;
        assert!(p[3] < 255 && p[0] < 90, "{p:?}");
    }

    #[test]
    fn een_grote_foto_wordt_kleiner() {
        let groot = RgbImage::from_pixel(2000, 2600, image::Rgb([90, 70, 60]));
        let uit = maak_vrijstaand(&groot);
        assert_eq!((uit.width(), uit.height()), (MAX_BREEDTE, 1040));
    }
}
