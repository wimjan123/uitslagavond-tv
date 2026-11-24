//! Het venster van de Renderer op een beeldmachine (#64): randloos volledig scherm op een
//! gekozen monitor, met vsync (`Fifo`), zonder muispijl, en een controle of het scherm in
//! het gekozen Beeldformaat staat. De verversing (50 Hz) zet je in de scherminstellingen van
//! het besturingssysteem; de Renderer meldt het in het logboek als die niet klopt.

use std::fmt;
use std::str::FromStr;

use bevy::prelude::*;
use bevy::render::settings::{Backends, WgpuSettings};
use bevy::window::{
    CursorOptions, Monitor, MonitorSelection, PresentMode, PrimaryMonitor, PrimaryWindow,
    WindowMode,
};

use crate::BREEDTE;

/// Het beeld dat de Renderer uitgeeft, zoals `1080p50`: hoogte in pixels (16:9) en beelden
/// per seconde. Standaard 1080p50 (#15); instelbaar voor een andere mixer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beeldformaat {
    pub breedte: u32,
    pub hoogte: u32,
    /// Beelden per seconde maal duizend, zoals de monitor het meldt (59,94 Hz is 59940).
    pub millihertz: u32,
}

/// De hoogtes die een omroep gebruikt; de breedte is altijd 16:9.
const HOOGTES: [u32; 4] = [720, 1080, 1440, 2160];

/// Zoveel millihertz mag het scherm naast het Beeldformaat zitten (59,94 en 60 Hz verschillen 60).
const SPELING_MILLIHERTZ: u32 = 20;

impl FromStr for Beeldformaat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fout = || format!("{s:?} is geen Beeldformaat; schrijf het als 1080p50 of 720p59.94");
        let (hoogte, hz) = s
            .to_ascii_lowercase()
            .split_once('p')
            .map(|(h, f)| (h.to_owned(), f.to_owned()))
            .ok_or_else(fout)?;
        let hoogte: u32 = hoogte.parse().map_err(|_| fout())?;
        if !HOOGTES.contains(&hoogte) {
            return Err(format!("{s:?}: de hoogte moet een van {HOOGTES:?} zijn"));
        }
        let hz: f64 = hz.parse().map_err(|_| fout())?;
        if !(hz > 0.0 && hz <= 240.0) {
            return Err(fout());
        }
        Ok(Beeldformaat {
            breedte: hoogte * 16 / 9,
            hoogte,
            millihertz: (hz * 1000.0).round() as u32,
        })
    }
}

/// Millihertz als Hz, zonder overbodige nullen: 50000 is `50`, 59940 is `59.94`.
fn hz(millihertz: u32) -> String {
    let s = format!("{}.{:03}", millihertz / 1000, millihertz % 1000);
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

impl fmt::Display for Beeldformaat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}p{}", self.hoogte, hz(self.millihertz))
    }
}

/// Op welke monitor de Renderer komt: het hoofdscherm, een nummer uit de lijst in het
/// logboek, of de naam van de aansluiting (zoals `HDMI-A-1`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Monitorkeuze {
    Hoofdscherm,
    Nummer(usize),
    Naam(String),
}

impl FromStr for Monitorkeuze {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            Err("kies een monitor: hoofdscherm, een nummer of een naam zoals HDMI-A-1".into())
        } else if s.eq_ignore_ascii_case("hoofdscherm") {
            Ok(Monitorkeuze::Hoofdscherm)
        } else if let Ok(n) = s.parse() {
            Ok(Monitorkeuze::Nummer(n))
        } else {
            Ok(Monitorkeuze::Naam(s.to_owned()))
        }
    }
}

impl fmt::Display for Monitorkeuze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Monitorkeuze::Hoofdscherm => write!(f, "hoofdscherm"),
            Monitorkeuze::Nummer(n) => write!(f, "{n}"),
            Monitorkeuze::Naam(naam) => write!(f, "{naam}"),
        }
    }
}

/// Wat de Renderer van een monitor weet, in de volgorde van het logboek.
pub struct Monitorgegevens<'a> {
    pub naam: Option<&'a str>,
    pub hoofdscherm: bool,
}

/// Het nummer van de gekozen monitor in `monitors`, of `None` als hij er niet is.
/// Meldt het systeem geen hoofdscherm, dan is de eerste monitor het hoofdscherm.
pub fn kies_monitor(keuze: &Monitorkeuze, monitors: &[Monitorgegevens]) -> Option<usize> {
    match keuze {
        Monitorkeuze::Hoofdscherm => monitors
            .iter()
            .position(|m| m.hoofdscherm)
            .or((!monitors.is_empty()).then_some(0)),
        Monitorkeuze::Nummer(n) => (*n < monitors.len()).then_some(*n),
        Monitorkeuze::Naam(naam) => monitors
            .iter()
            .position(|m| m.naam.is_some_and(|m| m.eq_ignore_ascii_case(naam))),
    }
}

/// Waarin het scherm afwijkt van het Beeldformaat, als zinnen voor het logboek.
pub fn afwijkingen(
    formaat: Beeldformaat,
    breedte: u32,
    hoogte: u32,
    millihertz: Option<u32>,
) -> Vec<String> {
    let mut a = Vec::new();
    if (breedte, hoogte) != (formaat.breedte, formaat.hoogte) {
        a.push(format!(
            "het scherm is {breedte}x{hoogte}, het Beeldformaat {formaat} vraagt {}x{}",
            formaat.breedte, formaat.hoogte
        ));
    }
    match millihertz {
        None => a.push(format!(
            "het scherm meldt geen verversing; controleer {} Hz in de scherminstellingen",
            hz(formaat.millihertz)
        )),
        Some(m) if m.abs_diff(formaat.millihertz) > SPELING_MILLIHERTZ => a.push(format!(
            "het scherm ververst op {} Hz, het Beeldformaat {formaat} vraagt {} Hz",
            hz(m),
            hz(formaat.millihertz)
        )),
        Some(_) => {}
    }
    a
}

/// De instelling van wgpu. Op Windows DirectX 12, want Vulkan op Windows heeft bevy#18898 (#19);
/// `WGPU_BACKEND` gaat voor.
pub fn wgpu_instelling() -> WgpuSettings {
    let mut w = WgpuSettings::default();
    if cfg!(windows) && Backends::from_env().is_none() {
        w.backends = Some(Backends::DX12);
    }
    w
}

/// Zet het venster randloos op volledig scherm op de gekozen monitor, zonder muispijl.
pub struct SchermPlugin {
    pub formaat: Beeldformaat,
    pub monitor: Monitorkeuze,
    pub volledig_scherm: bool,
}

#[derive(Resource)]
struct Schermkeuze {
    formaat: Beeldformaat,
    monitor: Monitorkeuze,
    gekozen: bool,
    /// De gekozen monitor was er niet; het venster staat op het hoofdscherm.
    noodscherm: bool,
}

impl Plugin for SchermPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();
        let mut vensters =
            world.query_filtered::<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>();
        let Ok((mut venster, mut muis)) = vensters.single_mut(world) else {
            return;
        };
        // Altijd vsync die nooit scheurt; zonder schaalfactor, zodat 1080p ook 1080 pixels is.
        venster.present_mode = PresentMode::Fifo;
        venster.resolution = (self.formaat.breedte, self.formaat.hoogte).into();
        venster.resolution.set_scale_factor_override(Some(1.0));
        if self.volledig_scherm {
            muis.visible = false;
        }
        app.add_systems(Update, schaal_graphics);
        if self.volledig_scherm {
            app.insert_resource(Schermkeuze {
                formaat: self.formaat,
                monitor: self.monitor.clone(),
                gekozen: false,
                noodscherm: false,
            })
            .add_systems(Update, kies_scherm);
        }
    }
}

/// Zodra de monitors bekend zijn: noem ze in het logboek, kies er een en zet het venster daar
/// op volledig scherm. Wijkt het scherm af van het Beeldformaat, dan staat dat als fout erbij.
/// Is de gekozen monitor er nog niet (de omzetter staat nog uit), dan staat het venster op het
/// hoofdscherm tot hij er is.
fn kies_scherm(
    mut keuze: ResMut<Schermkeuze>,
    monitors: Query<(Entity, &Monitor, Has<PrimaryMonitor>)>,
    mut venster: Single<&mut Window, With<PrimaryWindow>>,
) {
    if keuze.gekozen || monitors.is_empty() {
        return;
    }
    // In de volgorde waarin het systeem ze meldt, net als MonitorSelection::Index.
    let mut lijst: Vec<_> = monitors.iter().collect();
    lijst.sort_by_key(|(e, ..)| *e);
    let gegevens: Vec<_> = lijst
        .iter()
        .map(|(_, m, hoofd)| Monitorgegevens {
            naam: m.name.as_deref(),
            hoofdscherm: *hoofd,
        })
        .collect();
    let gevonden = kies_monitor(&keuze.monitor, &gegevens);
    if gevonden.is_none() && keuze.noodscherm {
        return;
    }
    for (n, (_, m, hoofd)) in lijst.iter().enumerate() {
        info!(
            "monitor {n}: {}, {}x{}, {}{}",
            m.name.as_deref().unwrap_or("zonder naam"),
            m.physical_width,
            m.physical_height,
            m.refresh_rate_millihertz
                .map_or("? Hz".into(), |v| format!("{} Hz", hz(v))),
            if *hoofd { " (hoofdscherm)" } else { "" }
        );
    }
    let n = gevonden.unwrap_or_else(|| {
        error!(
            "monitor {} is er niet; de Renderer staat op het hoofdscherm tot hij er is",
            keuze.monitor
        );
        keuze.noodscherm = true;
        kies_monitor(&Monitorkeuze::Hoofdscherm, &gegevens).unwrap_or(0)
    });
    let (entity, m, _) = lijst[n];
    venster.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Entity(entity));
    info!(
        "volledig scherm op monitor {n} ({}), Beeldformaat {}, vsync Fifo",
        m.name.as_deref().unwrap_or("zonder naam"),
        keuze.formaat
    );
    for a in afwijkingen(
        keuze.formaat,
        m.physical_width,
        m.physical_height,
        m.refresh_rate_millihertz,
    ) {
        error!("{a}");
    }
    keuze.gekozen = gevonden.is_some();
}

/// De graphics zijn ontworpen op 1920 pixels breed; bij een ander formaat schalen ze mee.
fn schaal_graphics(venster: Single<&Window, With<PrimaryWindow>>, mut schaal: ResMut<UiScale>) {
    let s = venster.width() / BREEDTE as f32;
    if schaal.0 != s {
        schaal.0 = s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(s: &str) -> Beeldformaat {
        s.parse().unwrap()
    }

    #[test]
    fn beeldformaat_uit_de_korte_naam() {
        assert_eq!(
            f("1080p50"),
            Beeldformaat {
                breedte: 1920,
                hoogte: 1080,
                millihertz: 50_000
            }
        );
        assert_eq!(
            f("720p59.94"),
            Beeldformaat {
                breedte: 1280,
                hoogte: 720,
                millihertz: 59_940
            }
        );
        assert_eq!(
            f("2160p25"),
            Beeldformaat {
                breedte: 3840,
                hoogte: 2160,
                millihertz: 25_000
            }
        );
        assert_eq!(f("1080P50"), f("1080p50"));
    }

    #[test]
    fn beeldformaat_weigert_wat_geen_16_bij_9_of_geen_verversing_is() {
        for fout in [
            "1080",
            "1080i50",
            "1000p50",
            "1080p",
            "1080p0",
            "1080pvijftig",
            "p50",
        ] {
            assert!(
                fout.parse::<Beeldformaat>().is_err(),
                "{fout} hoort fout te zijn"
            );
        }
    }

    #[test]
    fn beeldformaat_schrijft_zich_terug_als_korte_naam() {
        assert_eq!(f("1080p50").to_string(), "1080p50");
        assert_eq!(f("720p59.94").to_string(), "720p59.94");
    }

    #[test]
    fn monitorkeuze_uit_tekst() {
        assert_eq!("hoofdscherm".parse(), Ok(Monitorkeuze::Hoofdscherm));
        assert_eq!("1".parse(), Ok(Monitorkeuze::Nummer(1)));
        assert_eq!(
            "HDMI-A-1".parse(),
            Ok(Monitorkeuze::Naam("HDMI-A-1".into()))
        );
        assert!("".parse::<Monitorkeuze>().is_err());
    }

    fn monitors() -> Vec<Monitorgegevens<'static>> {
        vec![
            Monitorgegevens {
                naam: Some("DP-1"),
                hoofdscherm: true,
            },
            Monitorgegevens {
                naam: Some("HDMI-A-1"),
                hoofdscherm: false,
            },
            Monitorgegevens {
                naam: None,
                hoofdscherm: false,
            },
        ]
    }

    #[test]
    fn kies_monitor_op_hoofdscherm_nummer_of_naam() {
        let m = monitors();
        assert_eq!(kies_monitor(&Monitorkeuze::Hoofdscherm, &m), Some(0));
        assert_eq!(kies_monitor(&Monitorkeuze::Nummer(2), &m), Some(2));
        assert_eq!(
            kies_monitor(&Monitorkeuze::Naam("hdmi-a-1".into()), &m),
            Some(1)
        );
    }

    #[test]
    fn kies_monitor_die_er_niet_is() {
        let m = monitors();
        assert_eq!(kies_monitor(&Monitorkeuze::Nummer(3), &m), None);
        assert_eq!(
            kies_monitor(&Monitorkeuze::Naam("HDMI-A-2".into()), &m),
            None
        );
        let zonder_hoofdscherm = [Monitorgegevens {
            naam: None,
            hoofdscherm: false,
        }];
        assert_eq!(
            kies_monitor(&Monitorkeuze::Hoofdscherm, &zonder_hoofdscherm),
            Some(0)
        );
    }

    #[test]
    fn scherm_in_het_beeldformaat_heeft_geen_afwijking() {
        assert!(afwijkingen(f("1080p50"), 1920, 1080, Some(50_000)).is_empty());
        // Een monitor meldt soms een paar millihertz naast de echte waarde.
        assert!(afwijkingen(f("1080p50"), 1920, 1080, Some(49_999)).is_empty());
    }

    #[test]
    fn scherm_op_60_hz_of_een_andere_maat_valt_op() {
        let a = afwijkingen(f("1080p50"), 1920, 1080, Some(60_000));
        assert_eq!(a.len(), 1);
        assert!(a[0].contains("60 Hz") && a[0].contains("50 Hz"), "{a:?}");

        let a = afwijkingen(f("1080p50"), 3840, 2160, Some(50_000));
        assert_eq!(a.len(), 1);
        assert!(
            a[0].contains("3840x2160") && a[0].contains("1920x1080"),
            "{a:?}"
        );

        assert_eq!(
            afwijkingen(f("1080p59.94"), 1920, 1080, Some(60_000)).len(),
            1
        );
        assert_eq!(afwijkingen(f("1080p50"), 1920, 1080, None).len(), 1);
    }
}
