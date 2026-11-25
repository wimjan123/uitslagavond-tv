//! Een client die de Stand van de server volgt, zoals een Renderer dat doet.
//!
//! De volger draait in een eigen thread. Valt de verbinding weg, dan probeert hij het elke
//! seconde opnieuw; de Renderer houdt intussen het laatste beeld vast (ADR 0001). Weigert de
//! server hem (een andere versie), dan blijft hij het ook elke seconde proberen: zo ziet de Regie
//! in het onderdeel Systeem welke machine het is, en verbindt hij vanzelf na een update van de server.
//! Via [`Volger::beeldzender`] stuurt de Renderer met rol Voorbeeld kleine beelden terug.

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::{Duration, Instant};

use tungstenite::client::IntoClientRequest;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

use crate::camera::Camerabeeld;
use crate::{Afzender, BerichtAanServer, BerichtVanServer, GenummerdeStand, Monitor, Rol, VERSIE};

/// Hoe lang de volger hooguit op de server wacht voordat hij kijkt of er beelden weg moeten.
const WACHTEN: Duration = Duration::from_millis(20);

/// Wat de volger meldt.
#[derive(Debug, Clone, PartialEq)]
pub enum Gebeurtenis {
    /// De server nam de Renderer aan; de hele Stand volgt meteen.
    Verbonden,
    Stand(GenummerdeStand),
    /// Waar de camera's staan, 100 keer per seconde, met het moment van binnenkomen.
    Camera { beeld: Camerabeeld, ontvangen: Instant },
    /// De server weigert de Renderer, of een bericht van hem.
    Geweigerd(String),
    Verbroken(String),
}

/// Waarom een verbinding eindigde.
enum Einde {
    /// Stoppen gevraagd of niemand luistert meer.
    Klaar,
    /// De server weigerde de Renderer al bij Hallo; dat is al gemeld.
    Geweigerd,
    Verbroken(String),
}

impl From<String> for Einde {
    fn from(reden: String) -> Self {
        Einde::Verbroken(reden)
    }
}

/// De naam van deze machine, zoals de Regie hem in het onderdeel Systeem ziet.
pub fn deze_machine() -> String {
    let uit_bestand = std::fs::read_to_string("/proc/sys/kernel/hostname").ok();
    uit_bestand
        .or_else(|| std::env::var("COMPUTERNAME").ok())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "onbekende machine".into())
}

/// Volgt de Stand tot hij wordt opgeruimd.
pub struct Volger {
    pub gebeurtenissen: Receiver<Gebeurtenis>,
    stop: Arc<AtomicBool>,
    beelden: Sender<(Monitor, Vec<u8>)>,
}

/// Stuurt kleine beelden (JPEG) naar de server, voor de multiviewer van het dashboard.
#[derive(Clone)]
pub struct Beeldzender(Sender<(Monitor, Vec<u8>)>);

impl Beeldzender {
    /// Stuurt het beeld zodra de verbinding er is. Een nieuwer beeld voor dezelfde Monitor
    /// vervangt een beeld dat nog niet weg is.
    pub fn stuur(&self, monitor: Monitor, jpeg: Vec<u8>) {
        let _ = self.0.send((monitor, jpeg));
    }
}

impl Volger {
    pub fn beeldzender(&self) -> Beeldzender {
        Beeldzender(self.beelden.clone())
    }
}

impl Drop for Volger {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

/// Begint de Stand te volgen. `server` is het adres van de server, zoals `127.0.0.1:7400`.
pub fn volg(server: &str, sleutel: &str, rol: Rol) -> Volger {
    volg_als(server, sleutel, rol, VERSIE, &deze_machine())
}

/// Als [`volg`], met een eigen versie en machinenaam (voor proeven).
pub fn volg_als(server: &str, sleutel: &str, rol: Rol, versie: &str, machine: &str) -> Volger {
    let afzender = Afzender::Renderer { rol, versie: versie.into(), machine: machine.into() };
    let (zender, gebeurtenissen) = channel();
    let (beelden, te_sturen) = channel();
    let stop = Arc::new(AtomicBool::new(false));
    let url = format!("ws://{server}/ws");
    let sleutel = sleutel.to_owned();
    let stop_thread = stop.clone();
    thread::Builder::new()
        .name("stand-volger".into())
        .spawn(move || {
            let mut uit = Uitgaand { te_sturen, laatste: BTreeMap::new() };
            while !stop_thread.load(Ordering::Relaxed) {
                match verbind_en_volg(&url, &sleutel, &afzender, &zender, &mut uit, &stop_thread) {
                    Einde::Klaar => return,
                    Einde::Geweigerd => {}
                    Einde::Verbroken(reden) => {
                        if zender.send(Gebeurtenis::Verbroken(reden)).is_err() {
                            return;
                        }
                    }
                }
                thread::sleep(Duration::from_secs(1));
            }
        })
        .expect("thread voor de volger");
    Volger {
        gebeurtenissen,
        stop,
        beelden,
    }
}

/// De beelden die nog naar de server moeten, en het laatste beeld per Monitor: dat gaat na
/// een nieuwe verbinding opnieuw mee, want de server vergeet de beelden als het Voorbeeld wegvalt.
struct Uitgaand {
    te_sturen: Receiver<(Monitor, Vec<u8>)>,
    laatste: BTreeMap<u8, Vec<u8>>,
}

impl Uitgaand {
    /// Neemt de nieuwe beelden aan en geeft de berichten die weg moeten (alleen het nieuwste
    /// per Monitor).
    fn nieuw(&mut self) -> Vec<Vec<u8>> {
        let mut nieuw = BTreeMap::new();
        while let Ok((m, jpeg)) = self.te_sturen.try_recv() {
            let mut bericht = Vec::with_capacity(jpeg.len() + 1);
            bericht.push(m.byte());
            bericht.extend_from_slice(&jpeg);
            nieuw.insert(m.byte(), bericht);
        }
        for (m, b) in &nieuw {
            self.laatste.insert(*m, b.clone());
        }
        nieuw.into_values().collect()
    }
}

/// Eén verbinding, tot die eindigt.
fn verbind_en_volg(
    url: &str,
    sleutel: &str,
    afzender: &Afzender,
    zender: &Sender<Gebeurtenis>,
    uit: &mut Uitgaand,
    stop: &AtomicBool,
) -> Einde {
    match verbinding(url, sleutel, afzender, zender, uit, stop) {
        Ok(einde) | Err(einde) => einde,
    }
}

fn verbinding(
    url: &str,
    sleutel: &str,
    afzender: &Afzender,
    zender: &Sender<Gebeurtenis>,
    uit: &mut Uitgaand,
    stop: &AtomicBool,
) -> Result<Einde, Einde> {
    let mut verzoek = url.into_client_request().map_err(|e| e.to_string())?;
    verzoek.headers_mut().insert(
        "authorization",
        format!("Bearer {sleutel}")
            .parse()
            .map_err(|_| "sleutel bevat ongeldige tekens".to_string())?,
    );
    let (mut ws, _) = tungstenite::connect(verzoek).map_err(|e| e.to_string())?;
    if let MaybeTlsStream::Plain(s) = ws.get_mut() {
        s.set_read_timeout(Some(WACHTEN))
            .map_err(|e| e.to_string())?;
    }
    stuur(&mut ws, &BerichtAanServer::Hallo { afzender: afzender.clone() })?;
    // Aangenomen is de Renderer pas bij de eerste Stand; tot dan kan de server nog weigeren.
    let mut aangenomen = false;
    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = ws.close(None);
            return Ok(Einde::Klaar);
        }
        if aangenomen {
            for bericht in uit.nieuw() {
                ws.send(Message::binary(bericht)).map_err(|e| e.to_string())?;
            }
        }
        let bericht = match ws.read() {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) => return Err(Einde::Verbroken("de server sloot de verbinding".into())),
            Ok(_) => continue,
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
            {
                continue;
            }
            Err(e) => return Err(Einde::Verbroken(e.to_string())),
        };
        let gebeurtenis = match serde_json::from_str::<BerichtVanServer>(&bericht) {
            Ok(BerichtVanServer::Stand(s)) => {
                if !aangenomen {
                    aangenomen = true;
                    if zender.send(Gebeurtenis::Verbonden).is_err() {
                        return Ok(Einde::Klaar);
                    }
                    // het laatste kleine beeld per Monitor opnieuw, want de server vergat ze
                    uit.nieuw();
                    for bericht in uit.laatste.values() {
                        ws.send(Message::binary(bericht.clone())).map_err(|e| e.to_string())?;
                    }
                }
                Gebeurtenis::Stand(s)
            }
            Ok(BerichtVanServer::Geweigerd { reden }) if !aangenomen => {
                let _ = zender.send(Gebeurtenis::Geweigerd(reden));
                return Ok(Einde::Geweigerd);
            }
            Ok(BerichtVanServer::Camera(beeld)) => Gebeurtenis::Camera { beeld, ontvangen: Instant::now() },
            Ok(BerichtVanServer::Geweigerd { reden }) => Gebeurtenis::Geweigerd(reden),
            Ok(
                BerichtVanServer::Uitgevoerd { .. }
                | BerichtVanServer::Renderers { .. }
                | BerichtVanServer::Bewerkingen { .. },
            ) => continue,
            Err(e) => return Err(Einde::Verbroken(format!("onbekend bericht van de server: {e}"))),
        };
        if zender.send(gebeurtenis).is_err() {
            return Ok(Einde::Klaar);
        }
    }
}

fn stuur(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>, b: &BerichtAanServer) -> Result<(), String> {
    let tekst = serde_json::to_string(b).map_err(|e| e.to_string())?;
    ws.send(Message::text(tekst)).map_err(|e| e.to_string())
}
