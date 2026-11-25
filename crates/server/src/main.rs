//! Start de server: `uitslagavond-server --sleutel <sleutel>` (of `UITSLAGAVOND_SLEUTEL`).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use server::{Instellingen, OpenData, Server};

#[derive(Parser)]
#[command(version = gedeeld::VERSIE, about = "De server van uitslagavond-tv: de enige bron van de Stand")]
struct Opties {
    /// Adres en poort waarop de server luistert.
    #[arg(long, default_value = "0.0.0.0:7400")]
    adres: SocketAddr,
    /// Map voor de Stand (`stand.json`), het logboek (`logboek.jsonl`) en de foto's (`fotos/`).
    #[arg(long, default_value = "data")]
    map: PathBuf,
    /// De gedeelde sleutel. Zonder sleutel start de server niet.
    #[arg(long, env = "UITSLAGAVOND_SLEUTEL", hide_env_values = true)]
    sleutel: String,
    /// Map met het gebouwde dashboard.
    #[arg(long, default_value = "dashboard/dist")]
    dashboard: PathBuf,
    /// Het JSON-invoerbestand met partijlijst en sets, bijvoorbeeld `cijfers/uitslag-tk2025.json`.
    /// De server leest het bij elke wijziging opnieuw in.
    #[arg(long)]
    invoer: Option<PathBuf>,
    /// Het bestand met de Zaalindeling, bijvoorbeeld `zaalindeling/tk2025.json`. De server leest
    /// het bij elke wijziging opnieuw in, en schrijft er een Stoel in die de Regie geeft.
    #[arg(long)]
    zaalindeling: Option<PathBuf>,
    /// Adres van de OData API van de Tweede Kamer, voor Kamerleden, Fracties en foto's.
    #[arg(long, default_value = server::opendata::BASIS)]
    open_data: String,
    /// Na hoeveel minuten de server opnieuw in de open data kijkt.
    #[arg(long, default_value_t = 10)]
    open_data_elke: u64,
    /// Adres van Debat Direct, voor de officiële uitslag van de Stemmingen (Controle na de uitzending).
    #[arg(long, default_value = server::controle::debat_direct::BASIS)]
    debat_direct: String,
    /// Haal niets uit de open data (de Kamer blijft zoals hij in de Stand staat).
    #[arg(long)]
    zonder_open_data: bool,
    /// Kijk naar een andere dag dan vandaag (`JJJJ-MM-DD`): de Kamer en de Stemmingslijst van die
    /// dag, bijvoorbeeld om de Stemmingen van morgen voor te bereiden.
    #[arg(long)]
    peildatum: Option<jiff::civil::Date>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();
    let o = Opties::parse();
    if !o.dashboard.join("index.html").exists() {
        tracing::warn!("geen dashboard in {}: bouw het met `npm run build` in dashboard/", o.dashboard.display());
    }
    let server = Server::start(Instellingen {
        adres: o.adres,
        map: o.map,
        sleutel: o.sleutel,
        dashboard: Some(o.dashboard),
        invoer: o.invoer,
        zaalindeling: o.zaalindeling,
        open_data: (!o.zonder_open_data).then(|| OpenData {
            basis: o.open_data,
            elke: Duration::from_secs(o.open_data_elke.max(1) * 60),
            peildatum: o.peildatum,
            debat_direct: Some(o.debat_direct),
        }),
    })
    .await?;
    wacht_op_stop().await?;
    tracing::info!("stoppen");
    server.stop().await;
    Ok(())
}

/// Ctrl+C, of SIGTERM van `systemctl stop` op Linux.
async fn wacht_op_stop() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate())?;
        tokio::select! {
            r = tokio::signal::ctrl_c() => r,
            _ = term.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await
}
