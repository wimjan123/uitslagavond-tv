//! Lettertypes, logo's en foto's: de Renderer haalt ze bij de server en bewaart een eigen kopie.
//!
//! Vraagt een Blok een bestand, dan laadt de Renderer eerst zijn eigen kopie (als die er is)
//! en haalt daarna het bestand bij de server (`GET /api/bestand/<naam>`). Is dat anders dan
//! de kopie, dan schrijft hij de kopie opnieuw en gebruikt de nieuwe versie. Zo tekent een
//! Renderer die opnieuw start zonder server toch met de goede letters en logo's.
//! De ingebouwde lettertypes ([`gedeeld::sjabloon::INGEBOUWD`]) heeft hij altijd.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use gedeeld::sjabloon::{
    LETTER_CIJFERS, LETTER_GEWOON, LETTER_TEKST, LETTER_VET, geldige_bestandsnaam,
};

/// Hoe lang de Renderer wacht voordat hij het opnieuw bij de server probeert.
const OPNIEUW: Duration = Duration::from_secs(2);
/// Groter haalt de Renderer niet op.
const MAX_GROOTTE: usize = 32 << 20;

/// Waar de server staat.
#[derive(Clone, Debug)]
pub struct Bron {
    pub adres: String,
    pub sleutel: String,
}

/// Een bestand dat binnen is: uit de eigen kopie of van de server.
#[derive(Debug, PartialEq, Eq)]
pub struct Binnen {
    pub naam: String,
    pub inhoud: Vec<u8>,
}

/// Haalt bestanden op in de achtergrond. Los van Bevy, zodat het zonder beeld te testen is.
pub struct Ophaler {
    kopie: PathBuf,
    bron: Option<Bron>,
    zend: Sender<Binnen>,
    ontvang: Mutex<Receiver<Binnen>>,
}

impl Ophaler {
    pub fn new(kopie: PathBuf, bron: Option<Bron>) -> Self {
        let (zend, ontvang) = channel();
        Ophaler {
            kopie,
            bron,
            zend,
            ontvang: Mutex::new(ontvang),
        }
    }

    /// Haalt `naam` op: eerst de eigen kopie, dan de server. Wat binnenkomt staat daarna in
    /// [`Ophaler::binnen`].
    pub fn vraag(&self, naam: &str) {
        if !geldige_bestandsnaam(naam) {
            warn!("ongeldige bestandsnaam: {naam:?}");
            return;
        }
        let (naam, pad, bron, zend) = (
            naam.to_string(),
            self.kopie.join(naam),
            self.bron.clone(),
            self.zend.clone(),
        );
        std::thread::spawn(move || {
            let kopie = std::fs::read(&pad).ok();
            if let Some(k) = &kopie {
                let _ = zend.send(Binnen {
                    naam: naam.clone(),
                    inhoud: k.clone(),
                });
            }
            let Some(bron) = bron else { return };
            loop {
                match haal(&bron, &naam) {
                    Ok(Some(inhoud)) => {
                        if kopie.as_ref() != Some(&inhoud) {
                            if let Err(e) = bewaar(&pad, &inhoud) {
                                warn!("geen kopie van {naam}: {e}");
                            }
                            let _ = zend.send(Binnen { naam, inhoud });
                        }
                        return;
                    }
                    Ok(None) => {
                        warn!("de server heeft geen bestand {naam}");
                        return;
                    }
                    Err(e) => {
                        debug!("{naam} niet opgehaald: {e}");
                        std::thread::sleep(OPNIEUW);
                    }
                }
            }
        });
    }

    /// Alles wat sinds de vorige keer binnen is.
    pub fn binnen(&self) -> Vec<Binnen> {
        self.ontvang.lock().expect("ophaler").try_iter().collect()
    }
}

/// Eén GET bij de server, in HTTP/1.0: de server stuurt dan het hele bestand en sluit.
/// `Ok(None)`: de server heeft dit bestand niet.
fn haal(bron: &Bron, naam: &str) -> std::io::Result<Option<Vec<u8>>> {
    let fout = |m: String| std::io::Error::other(m);
    let mut s = TcpStream::connect(&bron.adres)?;
    s.set_read_timeout(Some(Duration::from_secs(20)))?;
    write!(
        s,
        "GET /api/bestand/{naam} HTTP/1.0\r\nHost: {}\r\nAuthorization: Bearer {}\r\n\r\n",
        bron.adres, bron.sleutel
    )?;
    let mut antwoord = Vec::new();
    s.take(MAX_GROOTTE as u64 + 64 * 1024)
        .read_to_end(&mut antwoord)?;
    let kop_eind = antwoord
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| fout("geen volledig antwoord".into()))?;
    let kop = String::from_utf8_lossy(&antwoord[..kop_eind]);
    let status = kop.split_whitespace().nth(1).unwrap_or("");
    match status {
        "200" => Ok(Some(antwoord[kop_eind + 4..].to_vec())),
        "404" => Ok(None),
        s => Err(fout(format!("de server antwoordt {s}"))),
    }
}

/// Schrijft de kopie in één keer weg, zodat een halve kopie nooit blijft staan.
fn bewaar(pad: &Path, inhoud: &[u8]) -> std::io::Result<()> {
    if let Some(map) = pad.parent() {
        std::fs::create_dir_all(map)?;
    }
    let tijdelijk = pad.with_extension("deel");
    std::fs::write(&tijdelijk, inhoud)?;
    std::fs::rename(tijdelijk, pad)
}

/// De standaardmap voor de kopie: `$XDG_CACHE_HOME/uitslagavond-tv/bestanden` of
/// `~/.cache/uitslagavond-tv/bestanden`.
pub fn standaard_kopie() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(std::env::temp_dir)
        .join("uitslagavond-tv/bestanden")
}

pub struct BestandenPlugin {
    pub kopie: PathBuf,
    pub bron: Option<Bron>,
}

impl Plugin for BestandenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Bestanden {
            ophaler: Ophaler::new(self.kopie.clone(), self.bron.clone()),
            letters: HashMap::new(),
            beelden: HashMap::new(),
            gevraagd: HashSet::new(),
        })
        .add_systems(Startup, laad_ingebouwd)
        .add_systems(PreUpdate, ontvang);
    }
}

/// De lettertypes en beelden die er zijn, op bestandsnaam.
#[derive(Resource)]
pub struct Bestanden {
    ophaler: Ophaler,
    letters: HashMap<String, Handle<Font>>,
    beelden: HashMap<String, Handle<Image>>,
    gevraagd: HashSet<String>,
}

impl Bestanden {
    /// Vraagt een bestand op als dat nog niet gebeurd is.
    pub fn vraag(&mut self, naam: &str) {
        if !self.letters.contains_key(naam) && self.gevraagd.insert(naam.to_string()) {
            self.ophaler.vraag(naam);
        }
    }

    /// Het lettertype met deze naam, als het binnen is.
    pub fn letter(&self, naam: &str) -> Option<Handle<Font>> {
        self.letters.get(naam).cloned()
    }

    /// Het ingebouwde vette lettertype: zolang een ander lettertype nog niet binnen is.
    pub fn reserve(&self) -> Handle<Font> {
        self.letters[LETTER_VET].clone()
    }

    pub fn beeld(&self, naam: &str) -> Option<Handle<Image>> {
        self.beelden.get(naam).cloned()
    }
}

fn laad_ingebouwd(mut bestanden: ResMut<Bestanden>, mut fonts: ResMut<Assets<Font>>) {
    for (naam, inhoud) in [
        (
            LETTER_VET,
            &include_bytes!("../assets/fonts/DejaVuSans-Bold.ttf")[..],
        ),
        (
            LETTER_GEWOON,
            &include_bytes!("../assets/fonts/DejaVuSans.ttf")[..],
        ),
        (
            LETTER_TEKST,
            &include_bytes!("../assets/fonts/NunitoSans-SemiBold.ttf")[..],
        ),
        (
            LETTER_CIJFERS,
            &include_bytes!("../assets/fonts/NunitoSans-ExtraBold.ttf")[..],
        ),
    ] {
        let h = fonts.add(Font::from_bytes(inhoud.to_vec()));
        bestanden.letters.insert(naam.into(), h);
    }
}

/// Het soort beeld: JPEG of PNG aan de eerste bytes (zoals een portretfoto `fotos/eigen/<id>`,
/// die geen extensie heeft), anders de extensie van de naam.
fn beeldsoort(naam: &str, b: &[u8]) -> String {
    if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg".into()
    } else if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        "png".into()
    } else {
        Path::new(naam)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
    }
}

/// Een portretfoto als beeld met een doorzichtige achtergrond ([`crate::vrijstaand`]).
fn vrijstaande_foto(inhoud: &[u8]) -> Option<Image> {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let foto = image::load_from_memory(inhoud).ok()?.to_rgb8();
    let uit = crate::vrijstaand::maak_vrijstaand(&foto);
    let (w, h) = uit.dimensions();
    let mut b = Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        uit.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    b.sampler = ImageSampler::linear();
    Some(b)
}

/// Of de bytes een lettertype zijn (TrueType, OpenType of een collectie).
fn is_lettertype(b: &[u8]) -> bool {
    matches!(b.get(..4), Some([0, 1, 0, 0] | b"OTTO" | b"true" | b"ttcf"))
}

fn ontvang(
    mut bestanden: ResMut<Bestanden>,
    mut fonts: ResMut<Assets<Font>>,
    mut beelden: ResMut<Assets<Image>>,
) {
    let binnen = bestanden.ophaler.binnen();
    if binnen.is_empty() {
        return;
    }
    for Binnen { naam, inhoud } in binnen {
        if is_lettertype(&inhoud) {
            let h = fonts.add(Font::from_bytes(inhoud));
            bestanden.letters.insert(naam.clone(), h);
            info!("lettertype binnen: {naam}");
            continue;
        }
        let soort = beeldsoort(&naam, &inhoud);
        // een portretfoto vrijstaand, zonder zijn witte studio-achtergrond
        if naam.starts_with("fotos/") {
            if let Some(b) = vrijstaande_foto(&inhoud) {
                let h = beelden.add(b);
                bestanden.beelden.insert(naam.clone(), h);
                info!("foto binnen: {naam}");
                continue;
            }
        }
        match Image::from_buffer(
            &inhoud,
            ImageType::Extension(&soort),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::linear(),
            RenderAssetUsages::RENDER_WORLD,
        ) {
            Ok(b) => {
                let h = beelden.add(b);
                bestanden.beelden.insert(naam.clone(), h);
                info!("beeld binnen: {naam}");
            }
            Err(e) => warn!("{naam} is geen lettertype en geen beeld dat de Renderer kent: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn een_foto_zonder_extensie_herkent_de_renderer_aan_de_eerste_bytes() {
        assert_eq!(beeldsoort("fotos/eigen/abc", b"\x89PNG\r\n\x1a\n rest"), "png");
        assert_eq!(beeldsoort("fotos/open-data/abc", b"\xFF\xD8\xFF rest"), "jpg");
        // een PNG met de naam .jpg is toch een PNG
        assert_eq!(beeldsoort("logos/a.jpg", b"\x89PNG\r\n\x1a\n"), "png");
        assert_eq!(beeldsoort("logos/a.WEBP", b"RIFF"), "webp");
    }
    use std::time::Instant;

    /// Een server die één keer antwoordt met `inhoud`, en de aanvraag teruggeeft.
    fn server(inhoud: &'static [u8]) -> (String, std::thread::JoinHandle<String>) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let adres = l.local_addr().unwrap().to_string();
        let t = std::thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            // Lees de hele aanvraag: sluiten met ongelezen bytes geeft een reset.
            let mut aanvraag = Vec::new();
            let mut stuk = [0u8; 256];
            while !aanvraag.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = s.read(&mut stuk).unwrap();
                aanvraag.extend_from_slice(&stuk[..n]);
            }
            write!(
                s,
                "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n",
                inhoud.len()
            )
            .unwrap();
            s.write_all(inhoud).unwrap();
            String::from_utf8_lossy(&aanvraag).into_owned()
        });
        (adres, t)
    }

    fn wacht(o: &Ophaler, aantal: usize) -> Vec<Binnen> {
        let tot = Instant::now() + Duration::from_secs(10);
        let mut alles = Vec::new();
        while alles.len() < aantal && Instant::now() < tot {
            alles.extend(o.binnen());
            std::thread::yield_now();
        }
        alles
    }

    #[test]
    fn een_bestand_komt_van_de_server_met_de_sleutel_en_blijft_als_kopie_zonder_server() {
        let kopie = std::env::temp_dir().join(format!("uitslagavond-kopie-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kopie);
        let (adres, server) = server(b"logo-inhoud");
        let o = Ophaler::new(
            kopie.clone(),
            Some(Bron {
                adres,
                sleutel: "geheim".into(),
            }),
        );
        o.vraag("logos/a.png");
        let binnen = wacht(&o, 1);
        assert_eq!(
            binnen,
            [Binnen {
                naam: "logos/a.png".into(),
                inhoud: b"logo-inhoud".to_vec()
            }]
        );
        let aanvraag = server.join().unwrap();
        assert!(
            aanvraag.starts_with("GET /api/bestand/logos/a.png "),
            "{aanvraag}"
        );
        assert!(
            aanvraag.contains("Authorization: Bearer geheim\r\n"),
            "{aanvraag}"
        );
        assert_eq!(
            std::fs::read(kopie.join("logos/a.png")).unwrap(),
            b"logo-inhoud"
        );

        // Opnieuw gestart, en de server is weg: de kopie is er meteen.
        let weg = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .to_string();
        let o = Ophaler::new(
            kopie.clone(),
            Some(Bron {
                adres: weg,
                sleutel: "geheim".into(),
            }),
        );
        o.vraag("logos/a.png");
        assert_eq!(
            wacht(&o, 1),
            [Binnen {
                naam: "logos/a.png".into(),
                inhoud: b"logo-inhoud".to_vec()
            }]
        );
        let _ = std::fs::remove_dir_all(&kopie);
    }

    #[test]
    fn een_naam_buiten_de_map_wordt_niet_gevraagd() {
        let o = Ophaler::new(std::env::temp_dir(), None);
        o.vraag("../etc/passwd");
        std::thread::sleep(Duration::from_millis(50));
        assert!(o.binnen().is_empty());
    }
}
