//! De server: de enige bron van de Stand (ADR 0001).
//!
//! - `GET /ws`: WebSocket met JSON voor dashboard, tablet en Renderers
//!   ([`BerichtAanServer`] en [`BerichtVanServer`]).
//! - `POST /api/opdracht` en `GET /api/stand`: dezelfde Opdrachten over HTTP, voor knoppen
//!   zoals een Stream Deck. De naam van de knop gaat mee in de kop `X-Afzender`. Voor Bitfocus
//!   Companion ook de knopstatus en een wisselknop voor de Vrije camera ([`knoppen`]).
//! - `GET /api/bestand/<naam>`: een lettertype, logo of foto uit de map `bestanden` naast de
//!   Stand. De Renderers halen hier wat de Stijl en de Graphics noemen en bewaren een kopie. Een
//!   portretfoto van de Persoonskaart heet `fotos/<soort>/<id>` (zie
//!   [`gedeeld::persoonskaart::foto_van_kamerlid`]) en komt uit de map `fotos`.
//! - `POST /api/bestand?naam=<origineel>`: de vormgevingstab plaatst een programmalogo (PNG of
//!   JPEG) of lettertype (TrueType of OpenType) in de map `bestanden`. De server geeft het een
//!   eigen naam bij de inhoud en antwoordt `{"naam": ...}`; die naam gaat in de Stijl.
//! - Het invoerbestand (partijlijst en sets): de server leest het bij elke wijziging in.
//! - Het bestand met de Zaalindeling: de server leest het bij elke wijziging in en schrijft het
//!   terug als de Regie een Stoel geeft ([`Opdracht::GeefStoel`]).
//! - De Kamer en de Stemmingslijst van de dag uit de open data van de Tweede Kamer: de server
//!   haalt ze steeds opnieuw op. Na de Stemmingen haalt hij de officiële uitslag op, uit Debat
//!   Direct en de open data, voor de Controle na de uitzending ([`controle`]).
//! - `GET`, `PUT` en `DELETE /api/foto/<id>`: de portretfoto van een Kamerlid of een eigen
//!   persoon van de Persoonskaart. `PUT` zet een eigen foto van de omroep (JPEG of PNG), bij een
//!   Kamerlid in plaats van die uit de open data.
//! - `/`: het dashboard; `/tablet`: de tablet van de Presentator (zelfde webapp). De tablet meldt
//!   zich als [`Afzender::Tablet`] en zet alleen klaar ([`gedeeld::Stand::controleer_afzender`]).
//!
//! De server leest de gamepad voor de Vrije camera ([`gamepad`]); zonder gamepad start hij gewoon.
//! De server rekent de camera's 100 keer per seconde uit ([`camera`], ADR 0002) en stuurt ze als
//! [`BerichtVanServer::Camera`] naar elke Renderer, en 10 keer per seconde naar elke Werkplek.
//!
//! De Renderer met rol Voorbeeld stuurt kleine beelden voor de multiviewer als binaire
//! WebSocket-berichten (zie [`Monitor`]). De server geeft het laatste beeld door aan elke
//! Werkplek en meldt Werkplekken welke Renderers verbonden zijn (en welke hij weigerde om een andere
//! versie, met de naam van de machine), en welke set de andere Werkplekken open hebben in het
//! onderdeel Cijfers ([`BerichtVanServer::Bewerkingen`]).
//!
//! Elke verbinding heeft de gedeelde sleutel nodig: als `Authorization: Bearer <sleutel>` of,
//! voor de WebSocket uit de browser, als `?sleutel=<sleutel>`. De server voert Opdrachten één
//! voor één uit, zet de Stand na elke wijziging op schijf en schrijft elke Opdracht in het logboek.

pub mod camera;
pub mod controle;
pub mod gamepad;
mod invoerbestand;
mod kamerleden;
mod knoppen;
pub mod opendata;
mod opslag;
pub mod stemmingslijst;

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::Context;
use axum::Json;
use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use gedeeld::camera::{Camerabeeld, Camerastand, TIKKEN_PER_SECONDE};
use gedeeld::cijfers::Bewerking;
use gedeeld::kamer::Foto;
use gedeeld::{
    Afzender, BerichtAanServer, BerichtVanServer, GenummerdeStand, Herkomst, MAX_BEELD, Monitor,
    Opdracht, RendererOpMachine, Rol, VERSIE,
};
use serde::Deserialize;
use tokio::sync::{Mutex, broadcast, oneshot, watch};
use tokio::task::JoinHandle;
use tower_http::services::{ServeDir, ServeFile};

use crate::camera::{Cameraregie, Ruimte, Stuur};
use crate::opslag::Opslag;

pub struct Instellingen {
    /// Waar de server luistert. Poort 0 kiest een vrije poort.
    pub adres: SocketAddr,
    /// Map met `stand.json` en `logboek.jsonl`.
    pub map: PathBuf,
    /// De gedeelde sleutel. Zonder sleutel start de server niet.
    pub sleutel: String,
    /// Map met het gebouwde dashboard (`dashboard/dist`).
    pub dashboard: Option<PathBuf>,
    /// Het JSON-invoerbestand met partijlijst en sets. De server leest het bij elke wijziging in.
    pub invoer: Option<PathBuf>,
    /// Het bestand met de Zaalindeling, zoals `zaalindeling/tk2025.json`.
    pub zaalindeling: Option<PathBuf>,
    /// Kamerleden en Fracties uit de open data. Zonder deze instelling haalt de server niets op.
    pub open_data: Option<OpenData>,
}

/// Waar en hoe vaak de server de Kamer uit de open data haalt.
#[derive(Clone, Debug)]
pub struct OpenData {
    /// Het adres van de API, normaal [`opendata::BASIS`].
    pub basis: String,
    /// Hoe lang de server wacht tot hij opnieuw kijkt.
    pub elke: Duration,
    /// Een vaste peildatum in plaats van vandaag, om vooruit of terug te kijken: de Kamer op die
    /// dag en de Stemmingslijst van die dag.
    pub peildatum: Option<jiff::civil::Date>,
    /// Het adres van Debat Direct, normaal [`controle::debat_direct::BASIS`], voor de Controle
    /// na de uitzending. Zonder adres komt de officiële uitslag alleen uit de open data.
    pub debat_direct: Option<String>,
}


/// Een draaiende server.
pub struct Server {
    adres: SocketAddr,
    stop: oneshot::Sender<()>,
    taak: JoinHandle<()>,
}

impl Server {
    pub async fn start(inst: Instellingen) -> anyhow::Result<Server> {
        anyhow::ensure!(
            !inst.sleutel.trim().is_empty(),
            "geen sleutel: start de server met een gedeelde sleutel"
        );
        let opslag = Opslag::open(&inst.map)?;
        let begin = opslag.lees_stand()?;
        let (uitzending, _) = watch::channel(begin.clone());
        // de camera staat er al voor de eerste tik, zodat "Bewaar als Standpunt" meteen kan
        let regie = Cameraregie::nieuw(&begin.stand, 0, Ruimte::zaal());
        let camera_begin = regie.beeld(0).ether.stand;
        let (stoppen, gestopt) = watch::channel(false);
        let gedeeld = Arc::new(Gedeeld {
            kern: Mutex::new(Kern {
                huidig: begin,
                opslag,
            }),
            uitzending,
            gestopt,
            sleutel: inst.sleutel,
            bestanden: inst.map.join("bestanden"),
            zaalindeling: inst.zaalindeling.clone(),
            map: inst.map.clone(),
            beelden: Monitor::ALLE.map(|m| watch::channel(leeg_beeld(m)).0),
            renderers: watch::channel(Renderers::default()).0,
            verbonden: std::sync::Mutex::new(BTreeMap::new()),
            geweigerd: std::sync::Mutex::new(BTreeMap::new()),
            camera: broadcast::channel(CAMERA_BUFFER).0,
            camera_nu: watch::channel(Some(camera_begin)).0,
            regie: std::sync::Mutex::new(regie),
            stuur: watch::channel(Stuur::LOS).0,
            bewerkingen: watch::channel(BTreeMap::new()).0,
            verbindingen: AtomicU64::new(0),
        });
        tokio::spawn(camera_lus(gedeeld.clone()));
        tokio::spawn(vergeet_weigeringen(gedeeld.clone()));
        gamepad::volg(gedeeld.clone());


        let beveiligd = Router::new()
            .route("/ws", get(websocket))
            .route("/api/stand", get(stand))
            .route("/api/opdracht", post(opdracht))
            .route("/api/zaalbeeld", get(zaalbeeld))
            .route("/api/conceptbeeld", post(conceptbeeld))
            .route("/api/knoppen", get(knoppen::alle_velden))
            .route("/api/knoppen/{veld}", get(knoppen::een_veld))
            .route("/api/vrije-camera/wissel", post(knoppen::wissel_vrije_camera))
            .route("/api/bestand/{*naam}", get(bestand))
            .route("/api/bestand", post(plaats_bestand).layer(DefaultBodyLimit::max(MAX_FOTO)))
            .route(
                "/api/foto/{id}",
                get(foto)
                    .put(eigen_foto)
                    .delete(weg_eigen_foto)
                    .layer(DefaultBodyLimit::max(MAX_FOTO)),
            )
            .route_layer(middleware::from_fn_with_state(gedeeld.clone(), toegang));
        let mut app = beveiligd.with_state(gedeeld.clone());
        if let Some(map) = inst.dashboard {
            let index = map.join("index.html");
            app = app.fallback_service(ServeDir::new(map).fallback(ServeFile::new(index)));
        }

        let luisteraar = tokio::net::TcpListener::bind(inst.adres)
            .await
            .with_context(|| format!("luisteren op {}", inst.adres))?;
        let adres = luisteraar.local_addr()?;
        // Pas als de poort open is, zodat een server die niet start ook niets begint.
        if let Some(pad) = inst.invoer {
            invoerbestand::volg(gedeeld.clone(), pad);
        }
        if let Some(pad) = inst.zaalindeling {
            invoerbestand::volg_zaalindeling(gedeeld.clone(), pad);
        }
        if let Some(od) = inst.open_data {
            stemmingslijst::volg(gedeeld.clone(), od.clone());
            controle::volg(gedeeld.clone(), od.clone());
            kamerleden::volg(gedeeld.clone(), od, inst.map.clone());
        }
        let (stop, signaal) = oneshot::channel::<()>();
        let taak = tokio::spawn(async move {
            let _ = axum::serve(luisteraar, app)
                .with_graceful_shutdown(async move {
                    let _ = signaal.await;
                    let _ = stoppen.send(true);
                })
                .await;
        });
        tracing::info!("server luistert op {adres}");
        Ok(Server { adres, stop, taak })
    }

    pub fn adres(&self) -> SocketAddr {
        self.adres
    }

    /// Sluit alle verbindingen en stopt.
    pub async fn stop(self) {
        let _ = self.stop.send(());
        let _ = self.taak.await;
    }
}

struct Gedeeld {
    kern: Mutex<Kern>,
    /// Het bestand met de Zaalindeling; een gegeven Stoel komt erin terug.
    zaalindeling: Option<PathBuf>,
    /// De laatste Stand, voor iedereen die meekijkt.
    uitzending: watch::Sender<GenummerdeStand>,
    /// Wordt `true` als de server stopt.
    gestopt: watch::Receiver<bool>,
    sleutel: String,
    /// Map met lettertypes, logo's en foto's voor de Renderers.
    bestanden: PathBuf,
    /// De map van de server, met de foto's.
    map: PathBuf,
    /// Het laatste kleine beeld per [`Monitor`], in de volgorde van [`Monitor::ALLE`].
    beelden: [watch::Sender<Bytes>; 2],
    /// De verbonden en de net geweigerde Renderers, voor de Werkplekken.
    renderers: watch::Sender<Renderers>,
    /// De verbonden Renderers, per verbinding.
    verbonden: std::sync::Mutex<BTreeMap<u64, RendererOpMachine>>,
    /// De Renderers die de server weigerde om hun versie, per rol en machine, met het moment
    /// van de laatste poging.
    geweigerd: std::sync::Mutex<BTreeMap<(Rol, String), (RendererOpMachine, Instant)>>,
    /// De camera's, elke tik.
    camera: broadcast::Sender<Camerabeeld>,
    /// Waar de camera van de Ether nu staat (tijdens Oefenen die van het Voorbeeld), voor
    /// "Bewaar als Standpunt".
    camera_nu: watch::Sender<Option<Camerastand>>,
    /// De camerarekening. De cameralus houdt hem elke tik heel even vast; bij het stoppen van
    /// een opname leest [`Gedeeld::voer_uit`] eruit wat er is opgenomen.
    regie: std::sync::Mutex<Cameraregie>,
    /// Wat de gamepad nu doet.
    stuur: watch::Sender<Stuur>,
    /// Welke set elke Werkplek open heeft in het onderdeel Cijfers, per verbinding.
    bewerkingen: watch::Sender<BTreeMap<u64, Bewerking>>,
    /// Telt de verbindingen, voor een eigen nummer per verbinding.
    verbindingen: AtomicU64,
}

/// Houdt bij welke set een Werkplek open heeft, tot de verbinding dicht gaat.
struct Bewerker {
    g: Arc<Gedeeld>,
    verbinding: u64,
    werkplek: String,
}

impl Bewerker {
    fn bewerk(&self, set: Option<String>) {
        self.g.bewerkingen.send_if_modified(|b| {
            let nieuw = set.map(|set| Bewerking { werkplek: self.werkplek.clone(), set });
            let oud = match nieuw {
                Some(n) => b.insert(self.verbinding, n.clone()).filter(|o| *o == n),
                None => return b.remove(&self.verbinding).is_some(),
            };
            oud.is_none()
        });
    }

    /// Wat de andere Werkplekken open hebben.
    fn van_anderen(&self) -> Vec<Bewerking> {
        let b = self.g.bewerkingen.borrow();
        b.iter().filter(|(v, _)| **v != self.verbinding).map(|(_, b)| b.clone()).collect()
    }
}

impl Drop for Bewerker {
    fn drop(&mut self) {
        self.bewerk(None);
    }
}

/// Hoeveel camerabeelden een trage verbinding achter mag lopen (2,5 seconde).
const CAMERA_BUFFER: usize = 256;
/// Een Werkplek krijgt één op de zoveel camerabeelden: genoeg voor de plattegrond.
const CAMERA_WERKPLEK: u64 = 10;

/// Rekent de camera's uit, 100 keer per seconde, tot de server stopt. Loopt de lus achter, dan
/// haalt hij de tikken in, zodat het tiknummer bij de klok blijft.
async fn camera_lus(g: Arc<Gedeeld>) {
    let mut standen = g.uitzending.subscribe();
    let mut gestopt = g.gestopt.clone();
    standen.mark_unchanged();
    let stuur = g.stuur.subscribe();
    let mut klok = tokio::time::interval(Duration::from_secs(1) / TIKKEN_PER_SECONDE);
    klok.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);
    for tik in 0u64.. {
        tokio::select! {
            _ = gestopt.changed() => return,
            _ = klok.tick() => {}
        }
        let beeld = {
            let mut regie = g.regie.lock().expect("regie");
            if standen.has_changed().unwrap_or(false) {
                regie.volg(&standen.borrow_and_update().stand, tik);
            }
            let st = *stuur.borrow();
            regie.stap(tik, &st);
            let beeld = regie.beeld(tik);
            let nu = if regie.oefent() { beeld.voorbeeld } else { beeld.ether };
            g.camera_nu.send_replace(Some(nu.stand));
            beeld
        };
        // niemand verbonden is geen fout
        let _ = g.camera.send(beeld);
    }
}

/// Een klein beeld zonder JPEG: geen beeld voor deze Monitor.
fn leeg_beeld(m: Monitor) -> Bytes {
    Bytes::from(vec![m.byte()])
}

/// Hoe lang een geweigerde Renderer in de lijst blijft na zijn laatste poging. Een Renderer
/// probeert het elke seconde opnieuw; staat hij uit, dan verdwijnt hij na deze tijd.
pub const WEIGERING_BLIJFT: Duration = Duration::from_secs(3);

/// Wat de Werkplekken over de Renderers horen.
#[derive(Clone, Default, PartialEq)]
struct Renderers {
    verbonden: Vec<RendererOpMachine>,
    geweigerd: Vec<RendererOpMachine>,
}

/// Haalt elke seconde de geweigerde Renderers weg die het niet meer proberen.
async fn vergeet_weigeringen(g: Arc<Gedeeld>) {
    let mut gestopt = g.gestopt.clone();
    let mut klok = tokio::time::interval(Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = gestopt.changed() => return,
            _ = klok.tick() => g.meld_renderers(),
        }
    }
}

/// Houdt bij dat een Renderer verbonden is, tot hij wordt opgeruimd.
struct Aanwezig {
    g: Arc<Gedeeld>,
    verbinding: u64,
    rol: Rol,
}

impl Aanwezig {
    fn nieuw(g: Arc<Gedeeld>, renderer: RendererOpMachine) -> Self {
        let verbinding = g.verbindingen.fetch_add(1, Ordering::Relaxed);
        let rol = renderer.rol;
        g.geweigerd.lock().expect("geweigerd").remove(&(rol, renderer.machine.clone()));
        g.verbonden.lock().expect("verbonden").insert(verbinding, renderer);
        g.meld_renderers();
        Aanwezig { g, verbinding, rol }
    }
}

impl Drop for Aanwezig {
    fn drop(&mut self) {
        self.g.verbonden.lock().expect("verbonden").remove(&self.verbinding);
        if self.rol == Rol::Voorbeeld {
            // Geen Voorbeeld meer: de beelden zijn niet meer bij.
            for (m, b) in Monitor::ALLE.iter().zip(&self.g.beelden) {
                b.send_replace(leeg_beeld(*m));
            }
        }
        self.g.meld_renderers();
    }
}

struct Kern {
    huidig: GenummerdeStand,
    opslag: Opslag,
}

impl Gedeeld {
    /// Werkt de lijst van Renderers voor de Werkplekken bij, zonder de weigeringen die te oud zijn.
    fn meld_renderers(&self) {
        let mut verbonden: Vec<RendererOpMachine> =
            self.verbonden.lock().expect("verbonden").values().cloned().collect();
        verbonden.sort();
        let geweigerd: Vec<RendererOpMachine> = {
            let mut w = self.geweigerd.lock().expect("geweigerd");
            w.retain(|_, (_, laatst)| laatst.elapsed() < WEIGERING_BLIJFT);
            w.values().map(|(r, _)| r.clone()).collect()
        };
        let nieuw = Renderers { verbonden, geweigerd };
        self.renderers.send_if_modified(|r| {
            let anders = *r != nieuw;
            *r = nieuw;
            anders
        });
    }

    /// Onthoudt een Renderer die de server weigerde om zijn versie.
    fn weiger(&self, renderer: RendererOpMachine) {
        self.geweigerd
            .lock()
            .expect("geweigerd")
            .insert((renderer.rol, renderer.machine.clone()), (renderer, Instant::now()));
        self.meld_renderers();
    }

    /// Voert één Opdracht uit. Het slot zorgt dat Opdrachten één voor één gaan.
    async fn voer_uit(&self, afzender: &Afzender, opdracht: &Opdracht) -> Result<u64, String> {
        // Bewaren zonder plek: waar de camera van de Ether nu staat. Zo komt de plek in het logboek.
        let opname = matches!(opdracht, Opdracht::ZetOpnemen { aan: false, opname: None, .. })
            .then(|| self.regie.lock().expect("regie").opname().map(<[_]>::to_vec))
            .flatten();
        let opdracht = &camera_ingevuld(opdracht, *self.camera_nu.borrow(), opname);
        let mut kern = self.kern.lock().await;
        let mut stand = kern.huidig.stand.clone();
        let volgnummer = kern.huidig.volgnummer + 1;
        let naam = afzender.naam();
        let herkomst = Herkomst {
            door: &naam,
            volgnummer,
            tijd_ms: u64::try_from(jiff::Timestamp::now().as_millisecond()).unwrap_or(0),
        };
        // De tablet van de Presentator zet alleen klaar; de server weigert de rest.
        let uitkomst = match stand.controleer_afzender(afzender, opdracht).and_then(|()| stand.voer_uit(opdracht, &herkomst)) {
            Err(w) => Err(w.to_string()),
            Ok(()) => {
                let nieuw = GenummerdeStand { volgnummer, stand };
                match kern.opslag.schrijf_stand(&nieuw) {
                    Ok(()) => {
                        if let (Opdracht::GeefStoel { .. }, Some(pad)) = (opdracht, &self.zaalindeling) {
                            schrijf_zaalindeling(pad, &nieuw.stand.zaalindeling);
                        }
                        let volgnummer = nieuw.volgnummer;
                        kern.huidig = nieuw.clone();
                        self.uitzending.send_replace(nieuw);
                        Ok(volgnummer)
                    }
                    Err(e) => {
                        tracing::error!("Stand niet op schijf: {e:#}");
                        Err("de server kon de Stand niet opslaan".into())
                    }
                }
            }
        };
        if let Err(e) = kern
            .opslag
            .log(afzender, opdracht, uitkomst.as_ref().copied().map_err(|s| s.as_str()))
        {
            tracing::error!("{e:#}");
        }
        uitkomst
    }
}

/// Schrijft de Zaalindeling terug naar haar bestand, via een tijdelijk bestand ernaast.
fn schrijf_zaalindeling(pad: &std::path::Path, z: &gedeeld::zaalindeling::Zaalindeling) {
    let tijdelijk = pad.with_extension("json.nieuw");
    let gelukt = std::fs::write(&tijdelijk, z.als_json()).and_then(|()| std::fs::rename(&tijdelijk, pad));
    if let Err(e) = gelukt {
        tracing::error!("Zaalindeling niet teruggeschreven naar {}: {e}", pad.display());
    }
}

/// Vult bij Bewaren en Overschrijven van een Standpunt zonder plek de camera van nu in, en bij
/// het stoppen van een opname wat de server opnam.
fn camera_ingevuld(
    opdracht: &Opdracht,
    nu: Option<Camerastand>,
    opgenomen: Option<Vec<[f32; 6]>>,
) -> Opdracht {
    let mut o = opdracht.clone();
    match &mut o {
        Opdracht::BewaarStandpunt { stand, .. } | Opdracht::OverschrijfStandpunt { stand, .. } => {
            *stand = stand.or(nu);
        }
        Opdracht::ZetOpnemen { aan: false, opname, .. } if opname.is_none() => *opname = opgenomen,
        _ => {}
    }
    o
}

#[derive(Deserialize)]
struct SleutelInUrl {
    sleutel: Option<String>,
}

/// Laat alleen verbindingen met de gedeelde sleutel door.
async fn toegang(
    State(g): State<Arc<Gedeeld>>,
    Query(url): Query<SleutelInUrl>,
    verzoek: Request,
    verder: Next,
) -> Response {
    let uit_kop = verzoek
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|w| w.to_str().ok())
        .and_then(|w| w.strip_prefix("Bearer "));
    let gegeven = uit_kop.or(url.sleutel.as_deref()).unwrap_or("");
    if gelijk(gegeven.as_bytes(), g.sleutel.as_bytes()) {
        verder.run(verzoek).await
    } else {
        (StatusCode::UNAUTHORIZED, "geen of verkeerde sleutel").into_response()
    }
}

/// Vergelijkt zonder vroeg te stoppen, zodat de tijd niets over de sleutel verraadt.
fn gelijk(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

async fn stand(State(g): State<Arc<Gedeeld>>) -> Json<GenummerdeStand> {
    Json(g.uitzending.borrow().clone())
}

#[derive(Deserialize)]
struct ZaalbeeldVraag {
    monitor: Option<String>,
}

/// Het laatste 720p JPEG-beeld van de Zaal uit de Voorbeeld-renderer, voor de Studio-editor en
/// het Conceptbeeld. Geeft 204 terug als er nog geen Voorbeeld-renderer verbonden is.
async fn zaalbeeld(
    State(g): State<Arc<Gedeeld>>,
    Query(q): Query<ZaalbeeldVraag>,
) -> Response {
    let voorkeur = match q.monitor.as_deref() {
        Some("voorbeeld") => [0, 1],
        _ => [1, 0], // standaard Ether (1), fallback Voorbeeld (0)
    };
    for i in voorkeur {
        let beeld = g.beelden[i].borrow().clone();
        if beeld.len() > 1 {
            return (
                [
                    (header::CONTENT_TYPE, "image/jpeg"),
                    (header::CACHE_CONTROL, "no-cache"),
                ],
                beeld.slice(1..),
            )
                .into_response();
        }
    }
    StatusCode::NO_CONTENT.into_response()
}

#[derive(Deserialize)]
struct ConceptVraag {
    graphic: Option<gedeeld::Graphic>,
    stoelbeeld: Option<gedeeld::Stoelbeeld>,
    sjabloon: Option<gedeeld::sjabloon::Sjabloon>,
    stijl: Option<gedeeld::sjabloon::Stijl>,
}

/// Rekent een privé-Conceptbeeld uit voor één Werkplek zonder de gedeelde Stand te wijzigen
/// (ADR 0001 / ADR 0005): geeft de ingevulde Blokken (1920×1080) en eventuele stoelkleuren terug.
async fn conceptbeeld(
    State(g): State<Arc<Gedeeld>>,
    Json(vraag): Json<ConceptVraag>,
) -> Response {
    let stand = g.uitzending.borrow().stand.clone();
    let stijl = vraag.stijl.unwrap_or_else(|| stand.stijl.clone());
    let mut stoelkleuren: Vec<gedeeld::Kleur> = Vec::new();
    if let Some(sb) = &vraag.stoelbeeld {
        match sb.bereken_kleuren(&stand) {
            Ok(k) => stoelkleuren = k,
            Err(e) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(serde_json::json!({ "fout": e.to_string() })),
                )
                    .into_response();
            }
        }
    }
    let blokken = if let Some(graphic) = &vraag.graphic {
        if let Err(e) = graphic.controleer() {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "fout": e.to_string() })),
            )
                .into_response();
        }
        let invulling = graphic.invulling();
        let sjabloon = vraag
            .sjabloon
            .or_else(|| stand.sjabloon(&graphic.sjabloon_id()).map(|s| s.into_owned()));
        match sjabloon {
            Some(s) => {
                let cijfers = graphic.cijfers().unwrap_or_else(|| stand.cijfers.clone());
                s.met_achtergrond().vul_in(&invulling, &cijfers, &stijl)
            }
            None => Vec::new(),
        }
    } else if let Some(s) = &vraag.sjabloon {
        let g = gedeeld::Graphic::Titelkaart { kop: s.naam.clone(), regel: "Voorbeeldtekst".into() };
        let invulling = g.invulling();
        s.met_achtergrond().vul_in(&invulling, &stand.cijfers, &stijl)
    } else {
        Vec::new()
    };
    let blokken_json: Vec<serde_json::Value> = blokken
        .into_iter()
        .map(|b| {
            let (soort, tekst) = match &b.inhoud {
                gedeeld::sjabloon::Inhoud::Vlak { .. } => ("Vlak", String::new()),
                gedeeld::sjabloon::Inhoud::Tekst { tekst, .. } => ("Tekst", tekst.clone()),
                gedeeld::sjabloon::Inhoud::Getal { waarde, .. } => ("Getal", waarde.tekst()),
                gedeeld::sjabloon::Inhoud::Lijn { .. } => ("Lijn", String::new()),
                gedeeld::sjabloon::Inhoud::Zetelbalk { .. } => ("Zetelbalk", String::new()),
                gedeeld::sjabloon::Inhoud::Uitslaggrafiek { .. } => ("Uitslaggrafiek", String::new()),
                gedeeld::sjabloon::Inhoud::Partijlogo { kort, .. } => ("Partijlogo", kort.clone()),
                gedeeld::sjabloon::Inhoud::Foto { naam, .. } => ("Foto", naam.clone()),
                gedeeld::sjabloon::Inhoud::Partijregel { kort, tekst, .. } => ("Partijregel", format!("{kort} {tekst}")),
                gedeeld::sjabloon::Inhoud::Programmalogo { tekst, .. } => ("Programmalogo", tekst.clone()),
                gedeeld::sjabloon::Inhoud::Afbeelding { bestand, .. } => ("Afbeelding", bestand.clone()),
                gedeeld::sjabloon::Inhoud::Partijlabels { .. } => ("Partijlabels", String::new()),
                gedeeld::sjabloon::Inhoud::Opbouwlabel { .. } => ("Opbouwlabel", String::new()),
                gedeeld::sjabloon::Inhoud::Stemmingsbalk { voor, tegen, .. } => ("Stemmingsbalk", format!("{voor} / {tegen}")),
                gedeeld::sjabloon::Inhoud::Uitkomst { tekst, .. } => ("Uitkomst", tekst.clone()),
                gedeeld::sjabloon::Inhoud::Fractielijst { .. } => ("Fractielijst", String::new()),
                gedeeld::sjabloon::Inhoud::Namenlijst { .. } => ("Namenlijst", String::new()),
            };
            serde_json::json!({
                "plek": b.plek,
                "kleur": b.kleur,
                "soort": soort,
                "tekst": tekst,
            })
        })
        .collect();
    Json(serde_json::json!({
        "blokken": blokken_json,
        "stoelkleuren": stoelkleuren,
    }))
    .into_response()
}

/// Een lettertype, logo of foto voor een Renderer. Alleen namen die in de map blijven.
async fn bestand(
    State(g): State<Arc<Gedeeld>>,
    axum::extract::Path(naam): axum::extract::Path<String>,
) -> Response {
    if !gedeeld::sjabloon::geldige_bestandsnaam(&naam) {
        return (StatusCode::BAD_REQUEST, "ongeldige bestandsnaam").into_response();
    }
    if let Some(rest) = naam.strip_prefix("fotos/") {
        let pad = rest.split_once('/').and_then(|(soort, id)| kamerleden::portret(&g.map, soort, id));
        return match pad {
            Some(pad) => match tokio::fs::read(pad).await {
                Ok(inhoud) => ([(header::CONTENT_TYPE, "application/octet-stream")], inhoud).into_response(),
                Err(_) => niet_gevonden(),
            },
            None => niet_gevonden(),
        };
    }
    match tokio::fs::read(g.bestanden.join(&naam)).await {
        Ok(inhoud) => ([(header::CONTENT_TYPE, "application/octet-stream")], inhoud).into_response(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            (StatusCode::NOT_FOUND, "geen bestand met die naam").into_response()
        }
        Err(e) => {
            tracing::error!("bestand {naam}: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "bestand niet te lezen").into_response()
        }
    }
}

#[derive(Deserialize)]
struct Bestandsnaam {
    naam: Option<String>,
}

/// Plaatst een programmalogo of lettertype in de map `bestanden`, onder een naam bij de inhoud:
/// `<naam>-<16 tekens>.<soort>`. Een ander bestand krijgt zo nooit de naam van een oud bestand,
/// dus een Renderer met een kopie toont nooit het verkeerde en de Graphic in de Ether houdt
/// haar oude logo tot de volgende Toon.
async fn plaats_bestand(
    State(g): State<Arc<Gedeeld>>,
    Query(q): Query<Bestandsnaam>,
    bytes: Bytes,
) -> Response {
    let Some(soort) = kamerleden::soort_beeld(&bytes).or_else(|| soort_lettertype(&bytes)) else {
        return antwoord(Err("dit is geen PNG, JPEG of lettertype (TTF of OTF)".into()));
    };
    let naam = bestandsnaam_bij_inhoud(q.naam.as_deref().unwrap_or(""), &bytes, soort);
    let pad = g.bestanden.join(&naam);
    let geplaatst = async {
        tokio::fs::create_dir_all(&g.bestanden).await?;
        if !tokio::fs::try_exists(&pad).await? {
            kamerleden::schrijf_veilig(&pad, &bytes).await?;
        }
        anyhow::Ok(())
    };
    if let Err(e) = geplaatst.await {
        tracing::error!("bestand {naam} niet opgeslagen: {e:#}");
        return antwoord(Err("de server kon het bestand niet opslaan".into()));
    }
    tracing::info!("bestand geplaatst: {naam}");
    Json(serde_json::json!({ "naam": naam })).into_response()
}

/// "ttf" of "otf" aan de eerste bytes.
fn soort_lettertype(b: &[u8]) -> Option<&'static str> {
    match b.get(..4)? {
        [0, 1, 0, 0] | b"true" => Some("ttf"),
        b"OTTO" => Some("otf"),
        _ => None,
    }
}

/// `<naam>-<16 tekens>.<soort>`: de naam zonder extensie en zonder vreemde tekens, en een
/// FNV-1a-hash van de inhoud (vast, ook na een nieuwe versie van Rust).
fn bestandsnaam_bij_inhoud(origineel: &str, inhoud: &[u8], soort: &str) -> String {
    let stam = origineel.rsplit('/').next().unwrap_or("");
    let stam = stam.rsplit_once('.').map_or(stam, |(s, _)| s);
    let mut schoon: String = stam
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .take(40)
        .collect();
    schoon = schoon.trim_matches('-').to_string();
    if schoon.is_empty() {
        schoon = "bestand".into();
    }
    let hash = inhoud
        .iter()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3));
    format!("{schoon}-{hash:016x}.{soort}")
}

/// De naam van de knop of Werkplek uit de kop `X-Afzender`.
fn knop(koppen: &HeaderMap) -> Afzender {
    let naam = koppen
        .get("x-afzender")
        .and_then(|w| w.to_str().ok())
        .filter(|n| !n.trim().is_empty())
        .unwrap_or("HTTP");
    Afzender::Knop { naam: naam.into() }
}

async fn opdracht(
    State(g): State<Arc<Gedeeld>>,
    koppen: HeaderMap,
    Json(opdracht): Json<Opdracht>,
) -> Response {
    antwoord(g.voer_uit(&knop(&koppen), &opdracht).await)
}

fn antwoord(uitkomst: Result<u64, String>) -> Response {
    match uitkomst {
        Ok(volgnummer) => Json(BerichtVanServer::Uitgevoerd { volgnummer }).into_response(),
        Err(reden) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(BerichtVanServer::Geweigerd { reden }),
        )
            .into_response(),
    }
}

/// Grootste eigen foto, in bytes. De foto's uit de open data zijn ongeveer 2 tot 3 MB.
const MAX_FOTO: usize = 25 * 1024 * 1024;

fn niet_gevonden() -> Response {
    (StatusCode::NOT_FOUND, "geen foto").into_response()
}

/// De foto van een Kamerlid (de eigen foto van de omroep, anders die uit de open data) of van een
/// eigen persoon.
async fn foto(State(g): State<Arc<Gedeeld>>, Path(id): Path<String>) -> Response {
    if !kamerleden::geldig_id(&id) {
        return niet_gevonden();
    }
    let Some(pad) = kamerleden::eigen_foto(&g.map, &id)
        .or_else(|| kamerleden::foto_uit_open_data(&g.map, &id))
        .or_else(|| kamerleden::portret(&g.map, "personen", &id))
    else {
        return niet_gevonden();
    };
    match tokio::fs::read(&pad).await {
        Ok(bytes) => {
            let soort = match kamerleden::soort_beeld(&bytes) {
                Some("png") => "image/png",
                _ => "image/jpeg",
            };
            (
                [
                    (header::CONTENT_TYPE, soort),
                    (header::CACHE_CONTROL, "no-cache"),
                ],
                bytes,
            )
                .into_response()
        }
        Err(_) => niet_gevonden(),
    }
}

fn bekend_kamerlid(g: &Gedeeld, id: &str) -> bool {
    kamerleden::geldig_id(id) && g.uitzending.borrow().stand.kamer.kamerlid(id).is_some()
}

fn bekende_persoon(g: &Gedeeld, id: &str) -> bool {
    kamerleden::geldig_id(id) && g.uitzending.borrow().stand.persoon(id).is_some()
}

/// Zet een eigen foto (JPEG of PNG): van een Kamerlid in plaats van die uit de open data, of van
/// een eigen persoon.
async fn eigen_foto(
    State(g): State<Arc<Gedeeld>>,
    Path(id): Path<String>,
    koppen: HeaderMap,
    bytes: Bytes,
) -> Response {
    let persoon = bekende_persoon(&g, &id);
    if !persoon && !bekend_kamerlid(&g, &id) {
        return (StatusCode::NOT_FOUND, "geen Kamerlid of eigen persoon met dit id").into_response();
    }
    let Some(ext) = kamerleden::soort_beeld(&bytes) else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(BerichtVanServer::Geweigerd {
                reden: "de foto is geen JPEG of PNG".into(),
            }),
        )
            .into_response();
    };
    let map = g.map.join("fotos").join(if persoon { "personen" } else { "eigen" });
    let opgeslagen = async {
        tokio::fs::create_dir_all(&map).await?;
        kamerleden::schrijf_veilig(&map.join(format!("{id}.{ext}")), &bytes).await?;
        let ander = if ext == "jpg" { "png" } else { "jpg" };
        let _ = tokio::fs::remove_file(map.join(format!("{id}.{ander}"))).await;
        anyhow::Ok(())
    };
    if let Err(e) = opgeslagen.await {
        tracing::error!("eigen foto van {id} niet opgeslagen: {e:#}");
        return antwoord(Err("de server kon de foto niet opslaan".into()));
    }
    let opdracht = if persoon {
        Opdracht::ZetFotoPersoon { persoon: id, foto: true }
    } else {
        Opdracht::ZetFoto {
            kamerlid: id,
            foto: Foto::Eigen,
        }
    };
    antwoord(g.voer_uit(&knop(&koppen), &opdracht).await)
}

/// Haalt de eigen foto weg: daarna geldt weer de foto uit de open data, of de voorletters.
async fn weg_eigen_foto(
    State(g): State<Arc<Gedeeld>>,
    Path(id): Path<String>,
    koppen: HeaderMap,
) -> Response {
    if bekende_persoon(&g, &id) {
        while let Some(pad) = kamerleden::portret(&g.map, "personen", &id) {
            if let Err(e) = tokio::fs::remove_file(&pad).await {
                tracing::error!("foto {} niet weg: {e}", pad.display());
                return antwoord(Err("de server kon de foto niet weghalen".into()));
            }
        }
        let opdracht = Opdracht::ZetFotoPersoon { persoon: id, foto: false };
        return antwoord(g.voer_uit(&knop(&koppen), &opdracht).await);
    }
    if !bekend_kamerlid(&g, &id) {
        return (StatusCode::NOT_FOUND, "geen Kamerlid met dit id").into_response();
    }
    while let Some(pad) = kamerleden::eigen_foto(&g.map, &id) {
        if let Err(e) = tokio::fs::remove_file(&pad).await {
            tracing::error!("eigen foto {} niet weg: {e}", pad.display());
            return antwoord(Err("de server kon de foto niet weghalen".into()));
        }
    }
    let opdracht = Opdracht::ZetFoto {
        kamerlid: id.clone(),
        foto: kamerleden::foto_op_schijf(&g.map, &id),
    };
    antwoord(g.voer_uit(&knop(&koppen), &opdracht).await)
}

async fn websocket(State(g): State<Arc<Gedeeld>>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(move |ws| async move {
        if let Err(e) = scherm(g, ws).await {
            tracing::debug!("verbinding dicht: {e:#}");
        }
    })
}

/// Eén scherm aan de WebSocket: eerst Hallo, dan de Stand bij elke wijziging en Opdrachten terug.
async fn scherm(g: Arc<Gedeeld>, mut ws: WebSocket) -> anyhow::Result<()> {
    let afzender = match tokio::time::timeout(Duration::from_secs(10), lees(&mut ws)).await {
        Ok(Some(Inkomend::Bericht(BerichtAanServer::Hallo { afzender }))) => afzender,
        _ => {
            stuur(&mut ws, &BerichtVanServer::Geweigerd {
                reden: "begin met Hallo".into(),
            })
            .await?;
            return Ok(());
        }
    };
    let renderer = match &afzender {
        Afzender::Renderer { rol, versie, machine } => Some(RendererOpMachine {
            rol: *rol,
            machine: if machine.trim().is_empty() { "onbekende machine".into() } else { machine.clone() },
            versie: versie.clone(),
        }),
        _ => None,
    };
    if let Some(r) = renderer.as_ref().filter(|r| r.versie != VERSIE) {
        tracing::warn!("Renderer {:?} op {} met versie {} geweigerd (server {VERSIE})", r.rol, r.machine, r.versie);
        g.weiger(r.clone());
        stuur(&mut ws, &BerichtVanServer::Geweigerd {
            reden: format!("andere versie: Renderer {}, server {VERSIE}", r.versie),
        })
        .await?;
        return Ok(());
    }
    tracing::info!("verbonden: {afzender:?}");
    let rol = renderer.as_ref().map(|r| r.rol);
    let _aanwezig = renderer.map(|r| Aanwezig::nieuw(g.clone(), r));
    // Alleen een Werkplek van de Regie kijkt naar de multiviewer en de status van de Renderers;
    // de tablet krijgt alleen de Stand.
    let werkplek = matches!(afzender, Afzender::Werkplek { .. });
    let bewerker = werkplek.then(|| Bewerker {
        g: g.clone(),
        verbinding: g.verbindingen.fetch_add(1, Ordering::Relaxed),
        werkplek: afzender.naam(),
    });
    let mut bewerkingen = g.bewerkingen.subscribe();
    let mut gemeld: Vec<Bewerking> = Vec::new();

    let mut standen = g.uitzending.subscribe();
    let mut renderers = g.renderers.subscribe();
    let [mut voorbeeld, mut ether] = g.beelden.each_ref().map(|b| b.subscribe());
    let mut camera = g.camera.subscribe();
    let mut gestopt = g.gestopt.clone();
    let eerste = standen.borrow_and_update().clone();
    stuur(&mut ws, &BerichtVanServer::Stand(eerste)).await?;
    if werkplek {
        let r = renderers.borrow_and_update().clone();
        stuur(&mut ws, &r.bericht()).await?;
        for b in [&mut voorbeeld, &mut ether] {
            let beeld = b.borrow_and_update().clone();
            if beeld.len() > 1 {
                ws.send(Message::Binary(beeld)).await?;
            }
        }
        bewerkingen.mark_unchanged();
        if let Some(b) = &bewerker {
            gemeld = b.van_anderen();
            stuur(&mut ws, &BerichtVanServer::Bewerkingen { open: gemeld.clone() }).await?;
        }
    }
    loop {
        tokio::select! {
            _ = gestopt.changed() => {
                let _ = ws.send(Message::Close(None)).await;
                return Ok(());
            }
            r = standen.changed() => {
                if r.is_err() {
                    return Ok(());
                }
                let s = standen.borrow_and_update().clone();
                stuur(&mut ws, &BerichtVanServer::Stand(s)).await?;
            }
            c = camera.recv(), if rol.is_some() || werkplek => {
                match c {
                    Ok(beeld) if rol.is_some() || beeld.tik % CAMERA_WERKPLEK == 0 => {
                        stuur(&mut ws, &BerichtVanServer::Camera(beeld)).await?;
                    }
                    Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => return Ok(()),
                }
            }
            Ok(()) = renderers.changed(), if werkplek => {
                let r = renderers.borrow_and_update().clone();
                stuur(&mut ws, &r.bericht()).await?;
            }
            Ok(()) = bewerkingen.changed(), if werkplek => {
                bewerkingen.mark_unchanged();
                let open = bewerker.as_ref().map(Bewerker::van_anderen).unwrap_or_default();
                if open != gemeld {
                    gemeld = open.clone();
                    stuur(&mut ws, &BerichtVanServer::Bewerkingen { open }).await?;
                }
            }
            Ok(()) = voorbeeld.changed(), if werkplek => {
                let beeld = voorbeeld.borrow_and_update().clone();
                ws.send(Message::Binary(beeld)).await?;
            }
            Ok(()) = ether.changed(), if werkplek => {
                let beeld = ether.borrow_and_update().clone();
                ws.send(Message::Binary(beeld)).await?;
            }
            bericht = lees(&mut ws) => {
                match bericht {
                    None => return Ok(()),
                    Some(Inkomend::Onleesbaar(reden)) => {
                        stuur(&mut ws, &BerichtVanServer::Geweigerd { reden }).await?;
                    }
                    Some(Inkomend::Bericht(BerichtAanServer::Hallo { .. })) => {}
                    Some(Inkomend::Bericht(BerichtAanServer::Bewerk { set })) => {
                        if let Some(b) = &bewerker {
                            b.bewerk(set);
                        }
                    }
                    Some(Inkomend::Bericht(BerichtAanServer::Opdracht { opdracht })) => {
                        let antwoord = match g.voer_uit(&afzender, &opdracht).await {
                            Ok(volgnummer) => BerichtVanServer::Uitgevoerd { volgnummer },
                            Err(reden) => BerichtVanServer::Geweigerd { reden },
                        };
                        stuur(&mut ws, &antwoord).await?;
                    }
                    Some(Inkomend::Beeld(beeld)) => {
                        if let Err(reden) = g.neem_beeld(rol, beeld) {
                            stuur(&mut ws, &BerichtVanServer::Geweigerd { reden }).await?;
                        }
                    }
                }
            }
        }
    }
}

impl Renderers {
    fn bericht(self) -> BerichtVanServer {
        BerichtVanServer::Renderers {
            versie: VERSIE.into(),
            verbonden: self.verbonden,
            geweigerd: self.geweigerd,
        }
    }
}

impl Gedeeld {
    /// Bewaart een klein beeld van de Renderer met rol Voorbeeld voor de Werkplekken.
    fn neem_beeld(&self, rol: Option<Rol>, beeld: Bytes) -> Result<(), String> {
        if rol != Some(Rol::Voorbeeld) {
            return Err("alleen de Renderer met rol Voorbeeld stuurt beelden".into());
        }
        if beeld.len() > MAX_BEELD {
            return Err(format!("beeld groter dan {MAX_BEELD} bytes"));
        }
        let m = beeld
            .first()
            .and_then(|b| Monitor::uit_byte(*b))
            .ok_or_else(|| "beeld voor een onbekende monitor".to_string())?;
        let i = Monitor::ALLE.iter().position(|x| *x == m).expect("in ALLE");
        self.beelden[i].send_replace(beeld);
        Ok(())
    }
}

/// Wat er binnenkomt over de WebSocket.
enum Inkomend {
    Bericht(BerichtAanServer),
    Beeld(Bytes),
    Onleesbaar(String),
}

/// Het volgende bericht, of `None` als de verbinding dicht is.
async fn lees(ws: &mut WebSocket) -> Option<Inkomend> {
    while let Some(Ok(m)) = ws.recv().await {
        match m {
            Message::Text(t) => {
                return Some(match serde_json::from_str(&t) {
                    Ok(b) => Inkomend::Bericht(b),
                    Err(e) => Inkomend::Onleesbaar(format!("onbekend bericht: {e}")),
                });
            }
            Message::Binary(b) => return Some(Inkomend::Beeld(b)),
            Message::Close(_) => return None,
            _ => {}
        }
    }
    None
}

async fn stuur(ws: &mut WebSocket, b: &BerichtVanServer) -> anyhow::Result<()> {
    ws.send(Message::text(serde_json::to_string(b)?)).await?;
    Ok(())
}
