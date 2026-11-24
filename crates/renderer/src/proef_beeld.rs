//! Proeven met echte beelden van de Graphic-laag, zonder Zaal. Ze hebben Vulkan nodig (een
//! GPU of lavapipe) en staan daarom uit in `cargo test`; CI draait ze in de job Renderer-beeld:
//!
//!     cargo test -p renderer -- --ignored proef_beeld
//!
//! Met `UITSLAGAVOND_BEWIJS=<map>` slaan ze hun beelden op; `animaties_in_beeld` maakt dan van
//! elke animatie in en uit een beeldenreeks (zie `scripts/animaties-video.sh`).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::image::Image;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use gedeeld::cijfers::Invoer;
use gedeeld::sjabloon::{
    Animatie, Blok, Bloksoort, Erin, Eruit, Getalbron, Invulling, Kleurkeuze, Letterkeuze,
    Partijbron, Persoon, Plek, Sjabloon, Stijlkleur, Stijlletter, Tekstbron, Uitlijning,
};
use gedeeld::{GenummerdeStand, Graphic, GraphicInBeeld, Herkomst, Opdracht, Rol, Stand};

use crate::animatie::Tijden;
use crate::graphic::{HandGestuurd, Laag, Sjablonen, Wens};
use crate::proef_gpu::{neem, stap};
use crate::volging::Standen;
use crate::{Instelling, maak_app};

/// Tijd per beeld: 50 beelden per seconde, zoals de uitzending.
const BEELD: Duration = Duration::from_millis(20);

type Rgb = image::RgbImage;

/// Een kopie-map met het lettertype dat de "server" leverde, en een logo.
fn kopie(naam: &str) -> PathBuf {
    let map = std::env::temp_dir().join(format!("uitslagavond-{naam}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&map);
    std::fs::create_dir_all(map.join("logos")).unwrap();
    std::fs::write(
        map.join("DejaVuSerif-Bold.ttf"),
        include_bytes!("../assets/fonts/DejaVuSerif-Bold.ttf"),
    )
    .unwrap();
    // VOORBEELDGEGEVENS: een logo van een verzonnen partij, diagonale banen
    let logo = image::RgbaImage::from_fn(200, 200, |x, y| {
        if (x + y) / 25 % 2 == 0 {
            image::Rgba([0xf2, 0xa9, 0x00, 255])
        } else {
            image::Rgba([0xd4, 0x20, 0x2c, 255])
        }
    });
    logo.save(map.join("logos/a.png")).unwrap();
    map
}

fn app(kopie: PathBuf) -> App {
    let mut app = maak_app(&Instelling {
        rol: Rol::Ether,
        zonder_venster: true,
        info: false,
        zaal: None,
        kopie,
        bron: None,
    });
    app.insert_resource(TimeUpdateStrategy::ManualDuration(BEELD));
    app.finish();
    app.cleanup();
    app
}

fn rgb(beeld: Image) -> Rgb {
    beeld.try_into_dynamic().unwrap().to_rgb8()
}

/// Neemt het beeld van de volgende stap.
fn opname(app: &mut App) -> Rgb {
    let vak = Arc::new(Mutex::new(None));
    neem(app, &vak);
    stap(app);
    for _ in 0..20 {
        if let Some(b) = vak.lock().unwrap().take() {
            return rgb(b);
        }
        stap(app);
    }
    panic!("geen opname")
}

/// Neemt elk beeld van `aantal` stappen.
fn reeks(app: &mut App, aantal: usize) -> Vec<Rgb> {
    let vakken: Vec<_> = (0..aantal)
        .map(|_| {
            let vak = Arc::new(Mutex::new(None));
            neem(app, &vak);
            stap(app);
            vak
        })
        .collect();
    for _ in 0..20 {
        if vakken.iter().all(|v| v.lock().unwrap().is_some()) {
            break;
        }
        stap(app);
    }
    vakken
        .into_iter()
        .map(|v| rgb(v.lock().unwrap().take().expect("geen opname")))
        .collect()
}

fn bewijs() -> Option<PathBuf> {
    std::env::var_os("UITSLAGAVOND_BEWIJS").map(PathBuf::from)
}

/// Stapt tot het lettertype `naam` binnen is uit de kopie.
fn wacht_op_letter(app: &mut App, naam: &str) {
    for _ in 0..500 {
        if app
            .world()
            .resource::<crate::bestanden::Bestanden>()
            .letter(naam)
            .is_some()
        {
            return;
        }
        stap(app);
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("lettertype {naam} komt niet binnen");
}

fn titelkaart_in_ether(stijl: gedeeld::sjabloon::Stijl) -> GenummerdeStand {
    let graphic = Graphic::Titelkaart {
        kop: "Wie wint de verkiezing?".into(),
        regel: "Voorbeeldgegevens".into(),
    };
    GenummerdeStand {
        volgnummer: 2,
        stand: Stand {
            ether: Some(GraphicInBeeld {
                id: 1,
                versie: 1,
                graphic,
                standpunt: None,
                getoond: 2,
                sinds_ms: 0,
                getoond_door: "Proef".into(),
                stijl: stijl.clone(),
                sjabloon: None,
            }),
            stijl,
            ..Default::default()
        },
    }
}

/// De kop van de Titelkaart ligt in dit vak (x, y, breedte, hoogte).
const KOPVAK: (u32, u32, u32, u32) = (140, 768, 1640, 104);

/// Welke pixels in het kopvak bij de letters horen.
fn letters_in_kopvak(b: &Rgb, past: impl Fn(&[u8]) -> bool) -> Vec<bool> {
    let (x0, y0, w, h) = KOPVAK;
    (y0..y0 + h)
        .flat_map(|y| (x0..x0 + w).map(move |x| (x, y)))
        .map(|(x, y)| past(&b.get_pixel(x, y).0))
        .collect()
}

fn overlap(a: &[bool], b: &[bool]) -> f32 {
    let beide = a.iter().zip(b).filter(|(x, y)| **x && **y).count();
    let een = a.iter().zip(b).filter(|(x, y)| **x || **y).count();
    beide as f32 / een.max(1) as f32
}

/// Geel (Hoofd, #ffd166): de kop in de standaardstijl.
fn geel(p: &[u8]) -> bool {
    p[0] > 220 && (170..=240).contains(&p[1]) && p[2] < 160
}

fn magenta(p: &[u8]) -> bool {
    p[0] > 150 && p[2] > 150 && p[1] < 110
}

/// Tekent de Titelkaart in de Ether met deze Stijl en geeft het beeld.
fn titelkaart(stijl: gedeeld::sjabloon::Stijl, map: &Path) -> Rgb {
    let mut app = app(map.to_path_buf());
    let kop = stijl.lettertypes.kop.clone();
    app.world_mut().resource_mut::<Standen>().huidig = Some(titelkaart_in_ether(stijl));
    stap(&mut app);
    wacht_op_letter(&mut app, &kop);
    for _ in 0..5 {
        stap(&mut app);
    }
    opname(&mut app)
}

#[test]
#[ignore = "heeft Vulkan nodig (GPU of lavapipe)"]
fn de_titelkaart_tekent_met_een_andere_stijl_in_de_nieuwe_kleur_en_het_nieuwe_lettertype() {
    let map = kopie("stijl");
    let standaard = gedeeld::sjabloon::Stijl::default();
    let mut alleen_kleur = standaard.clone();
    alleen_kleur.kleuren.hoofd = "#e000e0".parse().unwrap();
    let mut andere = alleen_kleur.clone();
    andere.lettertypes.kop = "DejaVuSerif-Bold.ttf".into();

    let a = titelkaart(standaard, &map);
    let b = titelkaart(alleen_kleur, &map);
    let c = titelkaart(andere, &map);
    if let Some(m) = bewijs() {
        std::fs::create_dir_all(&m).unwrap();
        a.save(m.join("titelkaart-standaard.png")).unwrap();
        b.save(m.join("titelkaart-andere-kleur.png")).unwrap();
        c.save(m.join("titelkaart-andere-stijl.png")).unwrap();
    }

    // De standaardstijl: gele letters in het kopvak, geen magenta.
    let wit_a = letters_in_kopvak(&a, geel);
    assert!(
        wit_a.iter().filter(|x| **x).count() > 3000,
        "geen gele kop in de standaardstijl"
    );
    assert_eq!(
        letters_in_kopvak(&a, magenta)
            .iter()
            .filter(|x| **x)
            .count(),
        0
    );
    // Een andere kleur Hoofd: dezelfde letters, nu magenta.
    let magenta_b = letters_in_kopvak(&b, magenta);
    assert!(
        letters_in_kopvak(&b, geel).iter().filter(|x| **x).count() < 50,
        "de kop is nog geel"
    );
    assert!(
        overlap(&wit_a, &magenta_b) > 0.8,
        "zelfde letters, andere kleur: {}",
        overlap(&wit_a, &magenta_b)
    );
    // Een ander lettertype voor de kop: magenta, maar andere vormen.
    let magenta_c = letters_in_kopvak(&c, magenta);
    assert!(
        magenta_c.iter().filter(|x| **x).count() > 3000,
        "geen magenta kop"
    );
    let o = overlap(&magenta_b, &magenta_c);
    assert!(
        o < 0.6,
        "de kop heeft nog hetzelfde lettertype (overlap {o})"
    );
}

/// Een Sjabloon met één Tekst-Blok dat invliegt.
fn scherp_sjabloon() -> Sjabloon {
    Sjabloon {
        id: "scherp".into(),
        naam: "Proef scherpte".into(),
        groep: "Proef".into(),
        snelheid: 100,
        achtergrond: Default::default(),
        stoelen: Default::default(),
        standpunt: None,
        blokken: vec![Blok {
            plek: Plek {
                x: 400,
                y: 450,
                breedte: 1200,
                hoogte: 120,
            },
            soort: Bloksoort::Tekst {
                bron: Tekstbron::Vast("Één reünie: ë é è ï ó".into()),
                letter: Letterkeuze::Stijl(Stijlletter::Tekst),
                grootte: 64,
                uitlijning: Uitlijning::Links,
                een_regel: false,
            },
            kleur: Kleurkeuze::Stijl(Stijlkleur::Tekst),
            animatie: Animatie {
                erin: Erin::VanLinks,
                eruit: Eruit::Vervagen,
                start_na_ms: 0,
            },
        }],
    }
}

/// Zet een proef-Sjabloon klaar en neemt de laag over van de Stand.
fn stuur_zelf(app: &mut App, sjablonen: Vec<Sjabloon>, stand: Stand) -> Entity {
    app.world_mut()
        .resource_mut::<Sjablonen>()
        .0
        .extend(sjablonen);
    app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand {
        volgnummer: 1,
        stand,
    });
    stap(app);
    let world = app.world_mut();
    let laag = world
        .query_filtered::<Entity, With<Laag>>()
        .single(world)
        .expect("één laag");
    world.entity_mut(laag).insert(HandGestuurd);
    laag
}

fn wens(app: &mut App, laag: Entity, w: Option<Wens>) {
    app.world_mut().get_mut::<Laag>(laag).unwrap().gewenst = w;
}

/// Het kleinste vak om de lichte pixels.
fn vak_om_letters(b: &Rgb) -> (u32, u32, u32, u32) {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, p) in b.enumerate_pixels() {
        if p.0.iter().any(|&c| c > 60) {
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
    }
    (x0, y0, x1 - x0 + 1, y1 - y0 + 1)
}

/// Het grootste verschil per kleur tussen het vak in `eind` en hetzelfde vak `dx` pixels verder in `b`.
fn verschil(eind: &Rgb, b: &Rgb, vak: (u32, u32, u32, u32), dx: i32) -> u8 {
    let (x0, y0, w, h) = vak;
    let mut grootst = 0;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let xb = x as i32 + dx;
            if xb < 0 || xb >= b.width() as i32 {
                return 255;
            }
            let (p, q) = (eind.get_pixel(x, y).0, b.get_pixel(xb as u32, y).0);
            for i in 0..3 {
                grootst = grootst.max(p[i].abs_diff(q[i]));
            }
        }
    }
    grootst
}

#[test]
#[ignore = "heeft Vulkan nodig (GPU of lavapipe)"]
fn tekst_met_e_trema_en_e_accent_blijft_scherp_tijdens_het_invliegen() {
    let mut app = app(kopie("scherp"));
    let laag = stuur_zelf(&mut app, vec![scherp_sjabloon()], Stand::default());
    let invulling = Invulling {
        sjabloon: "scherp".into(),
        ..Default::default()
    };
    wens(
        &mut app,
        laag,
        Some(Wens {
            toon: 1,
            versie: 1,
            invulling,
            ..Default::default()
        }),
    );
    let tijden = Tijden::new(&gedeeld::sjabloon::Beweging::default(), 100, 100);
    let aantal = (tijden.duur / BEELD.as_secs_f32()).ceil() as usize + 5;
    let beelden = reeks(&mut app, aantal);
    let eind = beelden.last().unwrap();
    let vak = vak_om_letters(eind);
    assert!(
        vak.2 > 400 && vak.3 > 40,
        "geen tekst in het eindbeeld: {vak:?}"
    );

    let mut scherp_onderweg = 0;
    let mut wazig = Vec::new();
    for (i, b) in beelden.iter().enumerate() {
        // Beeld i is getekend op tijd (i+1) x 20 ms; reken met de buren erbij, voor de zekerheid.
        let t = |k: usize| {
            tijden.erin(
                &scherp_sjabloon().blokken[0].animatie,
                crate::animatie::Aard::Tekst,
                (k + 1) as f32 * BEELD.as_secs_f32(),
            )
        };
        let vol = [i.saturating_sub(1), i, i + 1]
            .iter()
            .all(|&k| t(k).dekking >= 1.0);
        if !vol {
            continue;
        }
        let beste = (-200..=0)
            .map(|dx| (verschil(eind, b, vak, dx), dx))
            .min()
            .unwrap();
        if beste.0 <= 2 {
            if beste.1 != 0 {
                scherp_onderweg += 1;
            }
        } else {
            wazig.push((i, beste));
        }
        if let Some(m) = bewijs() {
            let m = m.join("scherp");
            std::fs::create_dir_all(&m).unwrap();
            b.save(m.join(format!("{i:03}.png"))).unwrap();
        }
    }
    assert!(
        wazig.is_empty(),
        "beelden waarin de tekst niet op hele pixels staat (beeld, (verschil, dx)): {wazig:?}"
    );
    assert!(
        scherp_onderweg >= 5,
        "te weinig beelden onderweg: {scherp_onderweg}"
    );
}

/// VOORBEELDGEGEVENS: drie verzonnen partijen en twee sets.
fn stand_met_cijfers(nu: &str) -> Stand {
    let invoer = Invoer::uit_json(&format!(
        r##"{{
          "partijen": [
            {{"sleutel": "a", "naam": "Partij Ééndracht", "kort": "PÉ", "kleur": "#d4202c", "logo": "logos/a.png"}},
            {{"sleutel": "b", "naam": "Beweging Noël", "kort": "BN", "kleur": "#00ae41"}},
            {{"sleutel": "c", "naam": "Calvé-lijst", "kort": "CL", "kleur": "#2f6fd6"}}
          ],
          "sets": [
            {{"id": "toen", "soort": "uitslag", "label": "Uitslag vorige keer", "zetels": {{"a": 40, "b": 30, "c": 80}}}},
            {{"id": "nu", "soort": "prognose", "label": "Prognose 22:00 (voorbeeldgegevens)", "zetels": {nu}}}
          ],
          "vergelijk_met": "toen"
        }}"##
    ));
    let mut stand = Stand::default();
    stand
        .voer_uit(
            &Opdracht::LeesInvoer {
                bestand: "voorbeeld.json".into(),
                ontvangen: "nu".into(),
                invoer,
            },
            &Herkomst {
                door: "Proef",
                volgnummer: 1,
                tijd_ms: 0,
            },
        )
        .unwrap();
    stand
}

/// Alle acht Blokken in één Sjabloon, allemaal met dezelfde animaties.
fn alle_blokken(erin: Erin, eruit: Eruit) -> Sjabloon {
    let blok = |x, y, breedte, hoogte, soort, kleur, start_na_ms| Blok {
        plek: Plek {
            x,
            y,
            breedte,
            hoogte,
        },
        soort,
        kleur: Kleurkeuze::Stijl(kleur),
        animatie: Animatie {
            erin,
            eruit,
            start_na_ms,
        },
    };
    let tekst = |bron, letter, grootte, uitlijning| Bloksoort::Tekst {
        bron,
        letter: Letterkeuze::Stijl(letter),
        grootte,
        uitlijning,
        een_regel: false,
    };
    let getal = |bron, grootte| Bloksoort::Getal {
        bron,
        letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
        grootte,
        uitlijning: Uitlijning::Rechts,
    };
    let label = format!("VOORBEELDGEGEVENS · in: {erin:?} · uit: {eruit:?}");
    use Stijlkleur::*;
    Sjabloon {
        id: "alle".into(),
        naam: "Proef alle Blokken".into(),
        groep: "Proef".into(),
        snelheid: 100,
        achtergrond: Default::default(),
        stoelen: Default::default(),
        standpunt: None,
        blokken: vec![
            blok(
                100,
                600,
                1720,
                400,
                Bloksoort::Vlak {
                    hoek: 16,
                    dekking: 92,
                    verloop: None,
                    draaiing: 0,
                    rand: None,
                    om_tekst: None,
                    bij_tekst: None,
                },
                Vlak,
                0,
            ),
            blok(
                140,
                640,
                150,
                150,
                Bloksoort::Partijlogo {
                    bron: Partijbron::Partij,
                    hoek: 12,
                },
                Tekst,
                80,
            ),
            blok(320, 640, 6, 150, Bloksoort::Lijn { dekking: 100 }, Accent, 120),
            blok(
                350,
                640,
                900,
                80,
                tekst(
                    Tekstbron::Partijnaam,
                    Stijlletter::Kop,
                    56,
                    Uitlijning::Links,
                ),
                Tekst,
                150,
            ),
            blok(
                350,
                725,
                900,
                60,
                tekst(
                    Tekstbron::Setnaam,
                    Stijlletter::Tekst,
                    34,
                    Uitlijning::Links,
                ),
                Accent,
                200,
            ),
            blok(
                1240,
                630,
                300,
                150,
                getal(Getalbron::Zetels, 120),
                Tekst,
                220,
            ),
            blok(
                1560,
                650,
                220,
                110,
                getal(Getalbron::Winst, 72),
                Accent,
                260,
            ),
            blok(
                140,
                860,
                1640,
                90,
                Bloksoort::Zetelbalk {
                    letter: Letterkeuze::Stijl(Stijlletter::Cijfers),
                    lijn_bij_76: true,
                    naamletter: Letterkeuze::Stijl(Stijlletter::Tekst),
                    labels: true,
                    vorige_set: false,
                    pijlen: false,
                    coalitie: false,
                },
                Tekst,
                300,
            ),
            blok(
                1580,
                140,
                200,
                250,
                Bloksoort::FotoKamerlid {
                    hoek: 12,
                    letter: Letterkeuze::Stijl(Stijlletter::Kop),
                },
                Tekst,
                0,
            ),
            blok(
                1400,
                400,
                380,
                120,
                Bloksoort::Partijlogo {
                    bron: Partijbron::PartijVanPersoon,
                    hoek: 12,
                },
                Tekst,
                60,
            ),
            blok(
                800,
                140,
                760,
                60,
                tekst(
                    Tekstbron::Persoonsnaam,
                    Stijlletter::Kop,
                    44,
                    Uitlijning::Rechts,
                ),
                Tekst,
                40,
            ),
            blok(
                1340,
                40,
                540,
                60,
                Bloksoort::Programmalogo {
                    letter: Letterkeuze::Stijl(Stijlletter::Kop),
                    dekking: 100,
                },
                Accent,
                0,
            ),
            Blok {
                plek: Plek {
                    x: 40,
                    y: 40,
                    breedte: 1200,
                    hoogte: 50,
                },
                soort: tekst(
                    Tekstbron::Vast(label),
                    Stijlletter::Tekst,
                    30,
                    Uitlijning::Links,
                ),
                kleur: Kleurkeuze::Stijl(Tekst),
                animatie: Animatie {
                    erin: Erin::Geen,
                    eruit: Eruit::Vervagen,
                    start_na_ms: 0,
                },
            },
        ],
    }
}

fn alle_invulling() -> Invulling {
    Invulling {
        sjabloon: "alle".into(),
        partij: Some("a".into()),
        set: Some("nu".into()),
        persoon: Some(Persoon {
            naam: "Noëlle Bénard".into(),
            foto: None,
            partij: Some("b".into()),
            voorletters: "NB".into(),
        }),
        ..Default::default()
    }
}

fn sla_reeks_op(map: &Path, naam: &str, beelden: &[Rgb]) {
    let m = map.join("animaties").join(naam);
    std::fs::create_dir_all(&m).unwrap();
    for (i, b) in beelden.iter().enumerate() {
        b.save(m.join(format!("{i:04}.png"))).unwrap();
    }
}

#[test]
#[ignore = "maakt bewijsbeelden; heeft Vulkan en UITSLAGAVOND_BEWIJS nodig"]
fn animaties_in_beeld() {
    let map = bewijs().expect("zet UITSLAGAVOND_BEWIJS");
    let tijden = Tijden::new(&gedeeld::sjabloon::Beweging::default(), 100, 100);
    let beelden_voor = |s: f32| (s / BEELD.as_secs_f32()).ceil() as usize;
    // Acht reeksen: elke manier om in beeld te komen, en de zes om weg te gaan.
    for (i, erin) in Erin::ALLE.into_iter().enumerate() {
        let eruit = Eruit::ALLE[i % Eruit::ALLE.len()];
        let sjabloon = alle_blokken(erin, eruit);
        let animaties: Vec<_> = sjabloon.blokken.iter().map(|b| b.animatie).collect();
        let erin_klaar = tijden.erin_klaar(&animaties);
        let n = sjabloon.blokken.len();
        let mut app = app(kopie(&format!("anim-{i}")));
        let laag = stuur_zelf(
            &mut app,
            vec![sjabloon],
            stand_met_cijfers(r#"{"a": 45, "b": 25, "c": 70}"#),
        );
        wacht_op_letter(&mut app, gedeeld::sjabloon::LETTER_VET);
        for _ in 0..30 {
            stap(&mut app);
            std::thread::sleep(Duration::from_millis(2));
        }
        wens(
            &mut app,
            laag,
            Some(Wens {
                toon: 1,
                versie: 1,
                invulling: alle_invulling(),
                ..Default::default()
            }),
        );
        let mut beelden = reeks(&mut app, beelden_voor(erin_klaar) + 25);
        wens(&mut app, laag, None);
        beelden.extend(reeks(&mut app, beelden_voor(tijden.eruit_klaar(n)) + 5));
        sla_reeks_op(
            &map,
            &format!("{}-in-{erin:?}-uit-{eruit:?}", i + 1),
            &beelden,
        );
    }

    // Bijwerken: andere cijfers in dezelfde Graphic; getallen tellen, de Zetelbalk schuift.
    let sjabloon = alle_blokken(Erin::Vervagen, Eruit::Vervagen);
    let mut app = app(kopie("bijwerken"));
    let laag = stuur_zelf(
        &mut app,
        vec![sjabloon],
        stand_met_cijfers(r#"{"a": 45, "b": 25, "c": 70}"#),
    );
    wacht_op_letter(&mut app, gedeeld::sjabloon::LETTER_VET);
    for _ in 0..30 {
        stap(&mut app);
    }
    let mut invulling = alle_invulling();
    wens(
        &mut app,
        laag,
        Some(Wens {
            toon: 1,
            versie: 1,
            invulling: invulling.clone(),
            ..Default::default()
        }),
    );
    for _ in 0..60 {
        stap(&mut app);
    }
    let mut beelden = reeks(&mut app, 10);
    app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand {
        volgnummer: 3,
        stand: stand_met_cijfers(r#"{"a": 61, "b": 31, "c": 50}"#),
    });
    invulling.velden.insert("versie".into(), "2".into());
    wens(
        &mut app,
        laag,
        Some(Wens {
            toon: 1,
            versie: 2,
            invulling,
            ..Default::default()
        }),
    );
    beelden.extend(reeks(&mut app, beelden_voor(tijden.bijwerken()) + 15));
    sla_reeks_op(&map, "9-bijwerken", &beelden);
}

/// Tekent één Graphic met dit Sjabloon in de Ether, zonder animatie, en geeft het beeld.
fn eindbeeld(naam: &str, sjabloon: Sjabloon, invulling: Invulling) -> Rgb {
    let mut app = app(kopie(naam));
    let laag = stuur_zelf(&mut app, vec![sjabloon], Stand::default());
    wens(
        &mut app,
        laag,
        Some(Wens {
            toon: 1,
            versie: 1,
            invulling,
            ..Default::default()
        }),
    );
    // Erin en de naambalk die zich om de tekst legt: ruim een seconde.
    for _ in 0..80 {
        stap(&mut app);
    }
    opname(&mut app)
}

#[test]
#[ignore = "heeft Vulkan nodig (GPU of lavapipe)"]
fn een_kleurverloop_in_beeld_heeft_op_elk_punt_de_berekende_kleur() {
    use gedeeld::sjabloon::{Inhoud, Stijl, Verloop, animatie, plek, vlak_met};
    let verloop = Verloop {
        naar: Kleurkeuze::Stijl(Stijlkleur::Hoofd),
        naar_dekking: 100,
        richting: 120,
        van: 10,
        tot: 80,
    };
    let (x0, y0, b, h) = (100, 200, 1600, 600);
    let sjabloon = Sjabloon {
        id: "verloop".into(),
        naam: "Proef verloop".into(),
        groep: "Proef".into(),
        snelheid: 100,
        achtergrond: Default::default(),
        stoelen: Default::default(),
        standpunt: None,
        blokken: vec![Blok {
            plek: plek(x0, y0, b, h),
            soort: vlak_met(100, Some(verloop), 0, None, None),
            kleur: Kleurkeuze::Stijl(Stijlkleur::Achtergrond),
            animatie: animatie(Erin::Geen, Eruit::Vervagen, 0),
        }],
    };
    let Inhoud::Vlak {
        verloop: Some(v), ..
    } = sjabloon
        .vul_in(&Invulling::default(), &Default::default(), &Stijl::default())
        .remove(0)
        .inhoud
    else {
        panic!("geen verloop")
    };
    let beeld = eindbeeld(
        "verloop",
        sjabloon,
        Invulling {
            sjabloon: "verloop".into(),
            ..Default::default()
        },
    );
    if let Some(m) = bewijs() {
        std::fs::create_dir_all(&m).unwrap();
        beeld.save(m.join("verloop.png")).unwrap();
    }
    let mut afwijkingen = Vec::new();
    for (x, y) in [(20, 20), (400, 100), (800, 300), (1000, 450), (1300, 500), (1580, 580)] {
        let t = v.positie(x as f32 + 0.5, y as f32 + 0.5, b as f32, h as f32);
        let k = v.op(t);
        let p = beeld.get_pixel((x0 + x) as u32, (y0 + y) as u32).0;
        let verschil = (0..3)
            .map(|i| (f32::from(p[i]) - k[i] * 255.0).abs())
            .fold(0.0, f32::max);
        if verschil > 4.0 {
            afwijkingen.push(((x, y), t, p, k.map(|c| (c * 255.0).round())));
        }
    }
    assert!(afwijkingen.is_empty(), "(punt, t, beeld, berekend): {afwijkingen:?}");
}

#[test]
#[ignore = "heeft Vulkan nodig (GPU of lavapipe)"]
fn de_naambalk_van_de_titelkaart_past_om_de_tekst() {
    let titel = |naam: &str, kop: &str| {
        let mut velden = std::collections::BTreeMap::new();
        velden.insert("kop".to_string(), kop.to_string());
        velden.insert("regel".to_string(), "Voorbeeldgegevens".to_string());
        eindbeeld(
            naam,
            gedeeld::sjabloon::vast_sjabloon("titelkaart").unwrap(),
            Invulling {
                sjabloon: "titelkaart".into(),
                velden,
                ..Default::default()
            },
        )
    };
    let kort = titel("balk-kort", "Kort");
    let lang = titel("balk-lang", "Een heel lange kop die niet op één regel past bij deze letter");
    if let Some(m) = bewijs() {
        std::fs::create_dir_all(&m).unwrap();
        kort.save(m.join("naambalk-kort.png")).unwrap();
        lang.save(m.join("naambalk-lang.png")).unwrap();
    }
    // De rechterrand van de balk: op de onderste rij van de balk (y 945) de laatste pixel van
    // links af in de kleur Vlak (op 93 procent).
    let rand = |b: &Rgb| {
        (100..1920u32)
            .take_while(|&x| {
                let p = b.get_pixel(x, 945).0;
                p[0].abs_diff(31) <= 5 && p[1].abs_diff(34) <= 5 && p[2].abs_diff(42) <= 5
            })
            .last()
            .unwrap_or(0)
    };
    // De laatste lichte pixel van de tekst (kop of regel, ook een gele kop), rechts van x 140.
    let tekst = |b: &Rgb| {
        (140..1920u32)
            .rev()
            .find(|&x| (770..935).any(|y| b.get_pixel(x, y).0.iter().map(|&c| u32::from(c)).sum::<u32>() > 420))
            .unwrap_or(0)
    };
    let (rk, tk) = (rand(&kort), tekst(&kort));
    assert!(rk > tk, "de balk is smaller dan de tekst: {rk} {tk}");
    assert!((rk - tk).abs_diff(48) <= 6, "marge achter de tekst: {}", rk - tk);
    // Een te lange kop krimpt op één regel; de balk komt niet verder dan zijn Plek.
    let (rl, tl) = (rand(&lang), tekst(&lang));
    assert!(rl <= 1824, "rand bij een lange kop: {rl}");
    assert!(tl < 1782, "de kop steekt uit: {tl}");
    assert!((rl - tl).abs_diff(48) <= 6, "marge achter een lange kop: {}", rl - tl);
}
