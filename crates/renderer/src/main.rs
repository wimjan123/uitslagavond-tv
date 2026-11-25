//! De Renderer: tekent één beeld en volgt de Stand van de server (ADR 0001).
//!
//!     uitslagavond-renderer --rol ether --server 127.0.0.1:7400 --sleutel <sleutel>
//!     uitslagavond-renderer --rol ether --zaal <map> --zonder-venster --beeld ether.png
//!     uitslagavond-renderer --rol ether --zaal <map> --zonder-venster --meet 600
//!
//!     uitslagavond-renderer --rol voorbeeld --zaal <map> --server 127.0.0.1:7400
//!     uitslagavond-renderer --rol ether --volledig-scherm --monitor HDMI-A-1 --beeldformaat 1080p50
//!
//! Zonder venster tekent hij in een beeld in het geheugen; met `--beeld` slaat hij dat op
//! zodra de Stand binnen is en stopt dan. Met `--reeks <map>` slaat hij een reeks beelden op,
//! zoals een camera-overgang (zie `scripts/camera-reeks.sh`). Met `--meet` meet hij de tijd per beeld.
//! De map van `--zaal` maakt `zaal/maak.sh`; zonder `--zaal` is er geen Zaal in beeld.
//! De rol Voorbeeld draait altijd zonder venster en stuurt kleine beelden naar het dashboard.

mod afwerking;
mod animatie;
mod beeld;
mod bestanden;
mod blokken;
mod camera;
mod graphic;
mod kleurgolf;
mod lichtkaart;
mod meting;
mod occlusie;
mod omgeving;
mod opname;
mod scherm;
mod schil;
mod stemmingsblokken;
mod volging;
mod voorbeeld;
mod zaal;

#[cfg(test)]
mod proef_beeld;
#[cfg(test)]
mod proef_gpu;

use std::path::PathBuf;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use clap::{Parser, ValueEnum};
use gedeeld::Rol;

/// Breedte en hoogte van het beeld (1080p).
pub const BREEDTE: u32 = 1920;
pub const HOOGTE: u32 = 1080;

#[derive(Clone, Copy, ValueEnum)]
enum RolKeuze {
    Ether,
    Reserve,
    Voorbeeld,
    Videowand,
}

#[derive(Parser)]
#[command(version = gedeeld::VERSIE, about = "De Renderer van uitslagavond-tv")]
struct Opties {
    /// Welk beeld deze Renderer tekent.
    #[arg(long, value_enum)]
    rol: RolKeuze,
    /// Adres van de server.
    #[arg(long, env = "UITSLAGAVOND_SERVER", default_value = "127.0.0.1:7400")]
    server: String,
    /// De gedeelde sleutel van de server.
    #[arg(long, env = "UITSLAGAVOND_SLEUTEL", hide_env_values = true)]
    sleutel: String,
    /// Teken zonder venster, in een beeld in het geheugen. De rol Voorbeeld doet dat altijd.
    #[arg(long)]
    zonder_venster: bool,
    /// Sla het beeld op als PNG zodra de Stand binnen is, en stop dan (alleen zonder venster).
    #[arg(long)]
    beeld: Option<PathBuf>,
    /// Wacht met opslaan tot de Stand minstens dit volgnummer heeft.
    #[arg(long, default_value_t = 0)]
    volgnummer: u64,
    /// Toon rol, volgnummer en verbinding in de hoek (voor proeven, niet voor de Ether).
    #[arg(long)]
    info: bool,
    /// Map met de Zaal uit `zaal/maak.sh`.
    #[arg(long, env = "UITSLAGAVOND_ZAAL")]
    zaal: Option<PathBuf>,
    /// Meet zoveel seconden de tijd per beeld (zonder venster) en stop dan met een rapport.
    #[arg(long, requires = "zonder_venster", conflicts_with = "beeld")]
    meet: Option<u64>,
    /// Sla een reeks beelden op in deze map, zoals een camera-overgang (alleen zonder venster):
    /// vanaf de Stand met `--volgnummer` elke `--reeks-elke-ms`, `--reeks-seconden` lang.
    #[arg(long, requires = "zonder_venster", conflicts_with_all = ["beeld", "meet"])]
    reeks: Option<PathBuf>,
    #[arg(long, default_value_t = 100, requires = "reeks")]
    reeks_elke_ms: u64,
    #[arg(long, default_value_t = 3, requires = "reeks")]
    reeks_seconden: u64,
    /// Schrijf het rapport van `--meet` ook naar dit JSON-bestand.
    #[arg(long, requires = "meet")]
    uitslag: Option<PathBuf>,
    /// Map voor de eigen kopie van lettertypes, logo's en foto's van de server.
    /// Standaard `~/.cache/uitslagavond-tv/bestanden`.
    #[arg(long, env = "UITSLAGAVOND_KOPIE")]
    kopie: Option<PathBuf>,
    /// Randloos volledig scherm, zonder muispijl, op de monitor van `--monitor` (beeldmachine).
    #[arg(long, conflicts_with = "zonder_venster")]
    volledig_scherm: bool,
    /// Monitor voor `--volledig-scherm`: `hoofdscherm`, een nummer of een naam zoals `HDMI-A-1`.
    /// De Renderer noemt de monitors bij de start in het logboek.
    #[arg(long, env = "UITSLAGAVOND_MONITOR", default_value = "hoofdscherm")]
    monitor: scherm::Monitorkeuze,
    /// Het beeld van het venster, zoals `1080p50` of `720p59.94`. Zet dezelfde verversing in de
    /// scherminstellingen; wijkt het scherm af, dan meldt de Renderer dat in het logboek.
    #[arg(long, env = "UITSLAGAVOND_BEELDFORMAAT", default_value = "1080p50")]
    beeldformaat: scherm::Beeldformaat,
}

/// Wat elke Renderer-app nodig heeft, met of zonder server: zo bouwen ook de tests hem.
pub struct Instelling {
    pub rol: Rol,
    pub zonder_venster: bool,
    pub info: bool,
    pub zaal: Option<PathBuf>,
    /// Map voor de eigen kopie van lettertypes, logo's en foto's.
    pub kopie: PathBuf,
    /// De server waar de bestanden vandaan komen; zonder server alleen de eigen kopie.
    pub bron: Option<bestanden::Bron>,
}

/// De Renderer zonder verbinding met de server en zonder lus.
pub fn maak_app(i: &Instelling) -> App {
    let zaal = i.zaal.as_ref().map(|m| {
        std::fs::canonicalize(m).unwrap_or_else(|e| panic!("--zaal {}: {e}", m.display()))
    });
    let assets = AssetPlugin {
        file_path: zaal.as_ref().map_or_else(
            || "assets".to_string(),
            |m| m.to_string_lossy().into_owned(),
        ),
        ..default()
    };
    let mut app = App::new();
    if i.zonder_venster || i.rol == Rol::Voorbeeld {
        app.add_plugins(
            DefaultPlugins
                .set(assets)
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: scherm::wgpu_instelling().into(),
                    synchronous_pipeline_compilation: true,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        );
        let grootte = if i.rol == Rol::Voorbeeld {
            voorbeeld::KLEIN
        } else {
            (BREEDTE, HOOGTE)
        };
        let doel = opname::maak_doel(&mut app, grootte);
        app.insert_resource(doel);
    } else {
        let render = RenderPlugin {
            render_creation: scherm::wgpu_instelling().into(),
            ..default()
        };
        app.add_plugins(DefaultPlugins.set(assets).set(render).set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Renderer: {:?}", i.rol),
                resolution: (BREEDTE, HOOGTE).into(),
                ..default()
            }),
            ..default()
        }));
    }
    app.insert_resource(Rolinfo {
        rol: i.rol,
        info: i.info,
    })
    .init_resource::<volging::Standen>()
    .init_resource::<volging::Verbinding>()
    .add_plugins((
        bestanden::BestandenPlugin {
            kopie: i.kopie.clone(),
            bron: i.bron.clone(),
        },
        beeld::BeeldPlugin,
        graphic::GraphicPlugin,
        zaal::ZaalPlugin { map: zaal },
    ));
    app
}

/// Instellingen die de systemen nodig hebben.
#[derive(Resource, Clone)]
pub struct Rolinfo {
    pub rol: Rol,
    pub info: bool,
}

fn main() -> AppExit {
    let o = Opties::parse();
    let rol = match o.rol {
        RolKeuze::Ether => Rol::Ether,
        RolKeuze::Reserve => Rol::Reserve,
        RolKeuze::Voorbeeld => Rol::Voorbeeld,
        RolKeuze::Videowand => {
            eprintln!("De rol Videowand bestaat, maar wordt nog niet gebouwd.");
            return AppExit::from_code(2);
        }
    };

    let zonder_venster = o.zonder_venster || rol == Rol::Voorbeeld;
    if o.beeld.is_some() && !zonder_venster {
        eprintln!("--beeld werkt alleen met --zonder-venster.");
        return AppExit::from_code(2);
    }
    let mut app = maak_app(&Instelling {
        rol,
        zonder_venster,
        info: o.info,
        zaal: o.zaal,
        kopie: o.kopie.unwrap_or_else(bestanden::standaard_kopie),
        bron: Some(bestanden::Bron {
            adres: o.server.clone(),
            sleutel: o.sleutel.clone(),
        }),
    });
    if !zonder_venster {
        app.add_plugins(scherm::SchermPlugin {
            formaat: o.beeldformaat,
            monitor: o.monitor,
            volledig_scherm: o.volledig_scherm,
        });
    }
    app.add_plugins(camera::CameraPlugin);
    app.add_plugins(volging::VolgingPlugin {
        server: o.server,
        sleutel: o.sleutel,
        rol,
    });
    if let Some(seconden) = o.meet {
        app.set_runner(meting::runner(Duration::from_secs(seconden), o.uitslag));
    } else if zonder_venster {
        app.add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / 50.0,
        )));
    }
    if rol == Rol::Voorbeeld {
        app.add_plugins(voorbeeld::VoorbeeldPlugin);
    }
    if let Some(map) = o.reeks {
        app.add_plugins(opname::ReeksPlugin {
            map,
            volgnummer: o.volgnummer,
            elke: Duration::from_millis(o.reeks_elke_ms),
            seconden: Duration::from_secs(o.reeks_seconden),
        });
    }
    if let Some(pad) = o.beeld {
        app.add_plugins(opname::OpnamePlugin {
            pad,
            volgnummer: o.volgnummer,
        });
    }
    app.run()
}
