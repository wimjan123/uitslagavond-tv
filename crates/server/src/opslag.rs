//! De map van de server op schijf: de Stand als leesbare JSON en het logboek van Opdrachten.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context;
use gedeeld::{Afzender, GenummerdeStand, Opdracht};
use serde::Serialize;

const STAND: &str = "stand.json";
const LOGBOEK: &str = "logboek.jsonl";

pub struct Opslag {
    map: PathBuf,
    logboek: File,
}

/// Eén regel in het logboek.
#[derive(Serialize)]
struct Regel<'a> {
    tijd: String,
    afzender: &'a Afzender,
    opdracht: &'a Opdracht,
    #[serde(skip_serializing_if = "Option::is_none")]
    volgnummer: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    geweigerd: Option<&'a str>,
}

impl Opslag {
    pub fn open(map: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(map).with_context(|| format!("map {} maken", map.display()))?;
        let logboek = OpenOptions::new()
            .create(true)
            .append(true)
            .open(map.join(LOGBOEK))
            .context("logboek openen")?;
        Ok(Self {
            map: map.to_owned(),
            logboek,
        })
    }

    /// De bewaarde Stand, of een lege Stand met volgnummer 0 als er nog niets is.
    pub fn lees_stand(&self) -> anyhow::Result<GenummerdeStand> {
        let pad = self.map.join(STAND);
        match fs::read_to_string(&pad) {
            Ok(tekst) => {
                let mut s: GenummerdeStand = serde_json::from_str(&tekst)
                    .with_context(|| format!("{} is geen geldige Stand", pad.display()))?;
                s.stand.stijl = s.stand.stijl.na_laden();
                s.stand.vul_vaste_ritten_aan();
                Ok(s)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(GenummerdeStand::default()),
            Err(e) => Err(e).with_context(|| format!("{} lezen", pad.display())),
        }
    }

    /// Schrijft de Stand naar schijf. Eerst naar een los bestand, dan hernoemen: zo staat er
    /// na een stroomstoring altijd een hele Stand.
    pub fn schrijf_stand(&self, stand: &GenummerdeStand) -> anyhow::Result<()> {
        let tijdelijk = self.map.join(format!("{STAND}.nieuw"));
        let mut f = File::create(&tijdelijk).context("Stand schrijven")?;
        serde_json::to_writer_pretty(&mut f, stand)?;
        f.write_all(b"\n")?;
        f.sync_all()?;
        fs::rename(&tijdelijk, self.map.join(STAND)).context("Stand vastleggen")?;
        Ok(())
    }

    pub fn log(
        &mut self,
        afzender: &Afzender,
        opdracht: &Opdracht,
        uitkomst: Result<u64, &str>,
    ) -> anyhow::Result<()> {
        let regel = Regel {
            tijd: jiff::Timestamp::now().to_string(),
            afzender,
            opdracht,
            volgnummer: uitkomst.ok(),
            geweigerd: uitkomst.err(),
        };
        let mut tekst = serde_json::to_string(&regel)?;
        tekst.push('\n');
        self.logboek
            .write_all(tekst.as_bytes())
            .context("logboek schrijven")?;
        Ok(())
    }
}
