//! De Camerarit (#45): een camerabeweging die de Regie tijdens Oefenen met de gamepad opneemt,
//! en daarna met schuifjes bijschaaft. Je kiest een Camerarit overal waar je een Standpunt kiest.
//!
//! De server bewaart 100 standen per seconde ([`Camerarit::opname`]). Wat de camera afspeelt is
//! de [`Ritbaan`]: begin en eind eraf, gladgemaakt, en in de gekozen duur met zacht starten en
//! stoppen. Losse punten verslepen kan niet: een andere route is opnieuw opnemen.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::camera::{
    CameraOvergang, Camerastand, NaAankomst, Opname, Proef, TIKKEN_PER_SECONDE, zachte_voortgang,
};
use crate::{Herkomst, Stand, Weigering};

/// Langste opname, in seconden. Daarna neemt de server niets meer op.
pub const OPNAME_MAX_S: f32 = 60.0;
/// Wat er na afknippen minstens overblijft, in seconden.
pub const RIT_MIN_S: f32 = 0.5;
/// Een Camerarit speelt hooguit zoveel keer sneller of langzamer dan opgenomen.
pub const DUUR_FACTOR: f32 = 4.0;
/// Bij volle gladheid middelt de camera over zoveel seconden naar weerskanten.
pub const GLAD_S: f32 = 1.0;
/// Herhalen kan als begin en eind hooguit zo ver uit elkaar liggen (meter) en zoveel anders
/// kijken (graden).
pub const HERHAAL_AFSTAND: f32 = 0.5;
pub const HERHAAL_HOEK: f32 = 10.0;

/// Hoe een Camerarit begint.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[ts(export)]
pub enum RitBegin {
    /// Eerst een gewone overgang naar het beginpunt, dan de rit zonder pauze.
    #[default]
    Overgang,
    /// Meteen naar het beginpunt knippen.
    Knippen,
}

/// Wat een Camerarit aan het eind doet.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[ts(export)]
pub enum RitEind {
    /// Blijft op het eindpunt staan.
    #[default]
    Stoppen,
    /// Begint opnieuw, zonder knip. Kan alleen als begin en eind bijna gelijk zijn.
    Herhalen,
}

/// De schuifjes van een Camerarit.
#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, PartialEq)]
#[ts(export)]
pub struct Bijschaving {
    /// 0 speelt af zoals opgenomen, 1 middelt over [`GLAD_S`] naar weerskanten.
    pub gladheid: f32,
    /// Hoe lang het overgebleven stuk duurt, in seconden.
    pub duur_s: f32,
    pub begin_eraf_s: f32,
    pub eind_eraf_s: f32,
    /// 0 tot 1, zoals bij de overgang naar een Standpunt.
    pub zacht_starten: f32,
    pub zacht_stoppen: f32,
    pub begin: RitBegin,
    pub eind: RitEind,
}

impl Bijschaving {
    /// Na een opname van `opname_s` seconden: zo afspelen als opgenomen, iets gladder.
    pub fn nieuw(opname_s: f32) -> Self {
        Bijschaving {
            gladheid: 0.2,
            duur_s: opname_s,
            begin_eraf_s: 0.0,
            eind_eraf_s: 0.0,
            zacht_starten: 0.0,
            zacht_stoppen: 0.0,
            begin: RitBegin::Overgang,
            eind: RitEind::Stoppen,
        }
    }
}

/// Een opgenomen camerabeweging door de Zaal.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq)]
#[ts(export)]
pub struct Camerarit {
    /// De sleutel: "R1", "R2" en verder. Verandert nooit; zo kiest een Graphic of Preset hem.
    pub letter: String,
    pub naam: String,
    /// J, K of L, gedeeld met de eigen Standpunten, of geen. Een vaste Camerarit heeft zijn
    /// sleutel als toets (Q, W, E, T, Y of U).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub toets: Option<String>,
    /// Een vaste Camerarit komt mee met het systeem ([`crate::vaste_ritten`]): je kunt hem
    /// bijschaven en hernoemen, niet wissen of opnieuw opnemen.
    #[serde(default)]
    pub vast: bool,
    /// 100 standen per seconde: x, y, z, draai, helling en beeldhoek, zoals [`Camerastand`].
    /// Leeg bij een vaste Camerarit: die opname rekent de software uit ([`Camerarit::punten`]).
    #[ts(type = "Array<[number, number, number, number, number, number]>")]
    pub opname: Vec<[f32; 6]>,
    pub bijschaving: Bijschaving,
}

/// Wat een camera afspeelt: een stand per tik.
#[derive(Clone, Debug, PartialEq)]
pub struct Ritbaan {
    pub standen: Vec<Camerastand>,
    pub herhalen: bool,
    /// Per stand: waar in de opname hij vandaan komt, in tikken (met breuk).
    plek: Vec<f32>,
}

impl Ritbaan {
    /// Waar in de opname de stand op tik `i` vandaan komt, in seconden.
    pub fn opname_s(&self, i: usize) -> f32 {
        self.plek[i.min(self.plek.len() - 1)] / TIKKEN_PER_SECONDE as f32
    }
}

pub fn als_stand(p: &[f32; 6]) -> Camerastand {
    Camerastand { positie: [p[0], p[1], p[2]], draai: p[3], helling: p[4], beeldhoek: p[5] }
}

/// Een stand als punt van de opname, afgerond op een millimeter en een milliradiaal: zo blijft
/// de Stand klein.
pub fn als_opnamepunt(s: &Camerastand) -> [f32; 6] {
    let r = |v: f32| (v * 1000.0).round() / 1000.0;
    [r(s.positie[0]), r(s.positie[1]), r(s.positie[2]), r(s.draai), r(s.helling), r(s.beeldhoek)]
}

fn kortste_hoek(d: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    (d + PI).rem_euclid(TAU) - PI
}

impl Camerarit {
    /// De opname: bij een vaste Camerarit die van de software, anders die uit de Stand.
    pub fn punten(&self) -> &[[f32; 6]] {
        match self.vast.then(|| crate::vaste_ritten::opname_van(&self.letter)).flatten() {
            Some(p) => p,
            None => &self.opname,
        }
    }

    /// De toets op het toetsenbord: de sleutel van een vaste Camerarit, J, K of L, of geen.
    pub fn sneltoets(&self) -> Option<&str> {
        if self.vast { Some(&self.letter) } else { self.toets.as_deref() }
    }

    /// Hoe lang de opname is, in seconden.
    pub fn opname_s(&self) -> f32 {
        self.punten().len().saturating_sub(1) as f32 / TIKKEN_PER_SECONDE as f32
    }

    /// Het stuk van de opname dat overblijft na begin en eind eraf, als indexen (tot en met).
    fn stuk(&self) -> (usize, usize) {
        let n = self.punten().len();
        if n < 2 {
            return (0, n.saturating_sub(1));
        }
        let tik = |s: f32| (s.max(0.0) * TIKKEN_PER_SECONDE as f32).round() as usize;
        let min = (RIT_MIN_S * TIKKEN_PER_SECONDE as f32) as usize;
        let b = tik(self.bijschaving.begin_eraf_s).min(n - 1);
        let e = (n - 1).saturating_sub(tik(self.bijschaving.eind_eraf_s)).max(b);
        if e - b < min.min(n - 1) {
            // te veel eraf: de Stand weigert dat, maar speel dan het begin af
            let b = b.min(n - 1 - min.min(n - 1));
            return (b, b + min.min(n - 1));
        }
        (b, e)
    }

    /// Waar de rit begint. Gladmaken laat begin en eind staan, dus dat is de opname na begin eraf.
    pub fn beginstand(&self) -> Option<Camerastand> {
        self.punten().get(self.stuk().0).map(als_stand)
    }

    /// Hoe lang het stuk is dat overblijft, in seconden.
    pub fn stuk_s(&self) -> f32 {
        let (b, e) = self.stuk();
        (e - b) as f32 / TIKKEN_PER_SECONDE as f32
    }

    /// De duur, binnen de grenzen: hooguit [`DUUR_FACTOR`] keer sneller of langzamer.
    pub fn duur_s(&self) -> f32 {
        let stuk = self.stuk_s().max(1.0 / TIKKEN_PER_SECONDE as f32);
        let d = self.bijschaving.duur_s;
        if d.is_finite() { d.clamp(stuk / DUUR_FACTOR, stuk * DUUR_FACTOR) } else { stuk }
    }

    /// Of de rit kan herhalen, en anders waarom niet.
    pub fn kan_herhalen(&self) -> Result<(), String> {
        let (b, e) = self.stuk();
        let (Some(a), Some(z)) = (self.punten().get(b), self.punten().get(e)) else {
            return Err("er is geen opname".into());
        };
        let afstand = ((a[0] - z[0]).powi(2) + (a[1] - z[1]).powi(2) + (a[2] - z[2]).powi(2)).sqrt();
        let hoek = [3, 4]
            .iter()
            .map(|&i| kortste_hoek(z[i] - a[i]).abs())
            .chain([(z[5] - a[5]).abs()])
            .fold(0.0, f32::max)
            .to_degrees();
        let mut redenen = Vec::new();
        if afstand > HERHAAL_AFSTAND {
            redenen.push(format!("liggen {} m uit elkaar", komma(afstand)));
        }
        if hoek > HERHAAL_HOEK {
            redenen.push(format!("kijken {}° anders", hoek.round()));
        }
        if redenen.is_empty() { Ok(()) } else { Err(format!("begin en eind {}", redenen.join(" en "))) }
    }

    /// Wat de camera afspeelt: een stand per tik, van het beginpunt tot het eindpunt.
    pub fn baan(&self) -> Ritbaan {
        let (b, e) = self.stuk();
        let Some(eerste) = self.punten().get(b) else {
            let s = Camerastand { positie: [0.0; 3], draai: 0.0, helling: 0.0, beeldhoek: 1.0 };
            return Ritbaan { standen: vec![s], herhalen: false, plek: vec![0.0] };
        };
        // de draai doorlopend, zodat middelen over -pi en pi heen goed gaat
        let mut stuk: Vec<[f32; 6]> = Vec::with_capacity(e - b + 1);
        let mut draai = eerste[3];
        for p in &self.punten()[b..=e] {
            if let Some(vorige) = stuk.last() {
                draai = vorige[3] + kortste_hoek(p[3] - vorige[3]);
            }
            stuk.push([p[0], p[1], p[2], draai, p[4], p[5]]);
        }
        let glad = glad_gemaakt(&stuk, self.bijschaving.gladheid);

        let b_ = &self.bijschaving;
        let tikken = (self.duur_s() * TIKKEN_PER_SECONDE as f32).round().max(1.0) as usize;
        let laatste = (glad.len() - 1) as f32;
        let mut standen = Vec::with_capacity(tikken + 1);
        let mut plek = Vec::with_capacity(tikken + 1);
        for i in 0..=tikken {
            let u = zachte_voortgang(i as f32 / tikken as f32, b_.zacht_starten, b_.zacht_stoppen) * laatste;
            let (i0, f) = (u.floor() as usize, u.fract());
            let i1 = (i0 + 1).min(glad.len() - 1);
            let p = if f == 0.0 {
                glad[i0]
            } else {
                std::array::from_fn(|k| glad[i0][k] + (glad[i1][k] - glad[i0][k]) * f)
            };
            let mut s = als_stand(&p);
            s.draai = kortste_hoek(s.draai);
            standen.push(s);
            plek.push(b as f32 + u);
        }
        let herhalen = b_.eind == RitEind::Herhalen && self.kan_herhalen().is_ok();
        Ritbaan { standen, herhalen, plek }
    }
}

/// Middelt elk punt met zijn buren, met gewichten die naar buiten toe aflopen. Aan begin en eind
/// krimpt het venster even ver naar beide kanten, zodat begin en eind blijven waar ze waren.
fn glad_gemaakt(stuk: &[[f32; 6]], gladheid: f32) -> Vec<[f32; 6]> {
    let straal = (gladheid.clamp(0.0, 1.0) * GLAD_S * TIKKEN_PER_SECONDE as f32).round() as usize;
    if straal == 0 {
        return stuk.to_vec();
    }
    let n = stuk.len();
    (0..n)
        .map(|i| {
            let r = straal.min(i).min(n - 1 - i);
            if r == 0 {
                return stuk[i];
            }
            let mut som = [0.0f32; 6];
            let mut gewicht = 0.0;
            for j in i - r..=i + r {
                let w = 1.0 - (j as f32 - i as f32).abs() / (r + 1) as f32;
                for k in 0..6 {
                    som[k] += stuk[j][k] * w;
                }
                gewicht += w;
            }
            som.map(|v| v / gewicht)
        })
        .collect()
}

/// Een getal met één cijfer achter de komma, zoals "2,3".
fn komma(v: f32) -> String {
    format!("{v:.1}").replace('.', ",")
}

impl Camerarit {
    /// Hoe de Regie de Camerarit herkent, zoals "Camerarit K (Langs de banken)".
    pub(crate) fn kenmerk(&self) -> String {
        format!("Camerarit {} ({})", self.sneltoets().unwrap_or(&self.letter), self.naam)
    }

    /// Of zoveel eraf knippen kan: er moet [`RIT_MIN_S`] overblijven.
    fn past(&self, b: &Bijschaving) -> bool {
        let over = self.opname_s() - b.begin_eraf_s.max(0.0) - b.eind_eraf_s.max(0.0);
        b.begin_eraf_s.is_finite() && b.eind_eraf_s.is_finite() && over >= RIT_MIN_S - 1e-4
    }
}

/// Oefenen, Opnemen, afspelen in het Voorbeeld en bijschaven (#45).
impl Stand {
    pub(crate) fn zet_vrije_camera(&mut self, aan: bool) -> Result<(), Weigering> {
        if aan && self.camera.opname.is_some() {
            return Err(Weigering::OpnameLoopt);
        }
        self.camera.vrij = aan;
        if aan {
            self.camera.oefenen = false;
            self.camera.proef = None;
        }
        Ok(())
    }

    pub(crate) fn zet_oefenen(&mut self, aan: bool) -> Result<(), Weigering> {
        if self.camera.opname.is_some() {
            return Err(Weigering::OpnameLoopt);
        }
        self.camera.oefenen = aan;
        self.camera.proef = None;
        if aan {
            self.camera.vrij = false;
        }
        Ok(())
    }

    pub(crate) fn zet_opnemen(
        &mut self,
        aan: bool,
        voor: Option<&str>,
        opname: Option<&[[f32; 6]]>,
        h: &Herkomst,
    ) -> Result<(), Weigering> {
        if aan {
            if !self.camera.oefenen {
                return Err(Weigering::OpnemenZonderOefenen);
            }
            if self.camera.opname.is_some() {
                return Err(Weigering::OpnameLoopt);
            }
            if let Some(l) = voor.filter(|l| self.camerarit(l).is_none()) {
                return Err(Weigering::OnbekendStandpunt(l.into()));
            }
            if let Some(l) = voor.filter(|l| self.camerarit(l).is_some_and(|r| r.vast)) {
                return Err(Weigering::VasteCamerarit(l.into()));
            }
            self.camera.opname = Some(Opname { voor: voor.map(Into::into), sinds: h.volgnummer });
            self.camera.proef = None;
            return Ok(());
        }
        let lopend = self.camera.opname.clone().ok_or(Weigering::GeenOpname)?;
        let opname = opname.unwrap_or_default();
        if opname.len() < 2 || opname.iter().flatten().any(|v| !v.is_finite()) {
            return Err(Weigering::OpnameTeKort);
        }
        self.camera.opname = None;
        let opname = opname[..opname.len().min(1 + (OPNAME_MAX_S * TIKKEN_PER_SECONDE as f32) as usize)].to_vec();
        let lengte = (opname.len() - 1) as f32 / TIKKEN_PER_SECONDE as f32;
        let letter = match lopend.voor.as_deref().and_then(|l| self.cameraritten.iter_mut().find(|r| r.letter == l)) {
            Some(r) => {
                let (begin, eind) = (r.bijschaving.begin, r.bijschaving.eind);
                r.opname = opname;
                r.bijschaving = Bijschaving { begin, eind, ..Bijschaving::nieuw(lengte) };
                if r.kan_herhalen().is_err() {
                    r.bijschaving.eind = RitEind::Stoppen;
                }
                r.letter.clone()
            }
            None => {
                let bezet = |l: &str| self.cameraritten.iter().any(|r| r.letter == l);
                let letter = (1..).map(|n| format!("R{n}")).find(|l| !bezet(l)).expect("oneindig");
                let naam_bezet = |n: &str| self.cameraritten.iter().any(|r| r.naam == n);
                let eigen = self.cameraritten.iter().filter(|r| !r.vast).count();
                let naam = (eigen + 1..)
                    .map(|n| format!("Camerarit {n}"))
                    .find(|n| !naam_bezet(n))
                    .expect("oneindig");
                let toets = self.eerste_vrije_toets();
                self.cameraritten.push(Camerarit {
                    letter: letter.clone(),
                    naam,
                    toets,
                    vast: false,
                    opname,
                    bijschaving: Bijschaving::nieuw(lengte),
                });
                letter
            }
        };
        self.camera.proef = Some(Proef { letter, sinds: h.volgnummer });
        Ok(())
    }

    pub(crate) fn probeer(&mut self, letter: &str, h: &Herkomst) -> Result<(), Weigering> {
        if self.standpunt(letter).is_none() && self.camerarit(letter).is_none() {
            return Err(Weigering::OnbekendStandpunt(letter.into()));
        }
        if self.camera.opname.is_some() {
            return Err(Weigering::OpnameLoopt);
        }
        self.camera.proef = Some(Proef { letter: letter.into(), sinds: h.volgnummer });
        Ok(())
    }

    pub(crate) fn zet_standpunt_overgang(
        &mut self,
        letter: &str,
        overgang: CameraOvergang,
        na_aankomst: NaAankomst,
        h: &Herkomst,
    ) -> Result<(), Weigering> {
        let schuiven = match na_aankomst {
            NaAankomst::Stil => [0.0; 4],
            NaAankomst::Schuiven { richting: [x, y, z], cm_per_s } => [x, y, z, cm_per_s],
        };
        let getallen = [overgang.duur_s, overgang.zacht_starten, overgang.zacht_stoppen];
        if !getallen.iter().chain(&schuiven).all(|v| v.is_finite()) {
            return Err(Weigering::GeenCamerastand);
        }
        let s = self.standpunt_mut(letter)?;
        s.overgang = CameraOvergang {
            duur_s: overgang.duur(),
            zacht_starten: overgang.zacht_starten.clamp(0.0, 1.0),
            zacht_stoppen: overgang.zacht_stoppen.clamp(0.0, 1.0),
            ..overgang
        };
        s.na_aankomst = na_aankomst;
        self.speel_af(letter, h);
        Ok(())
    }

    /// Speelt af in het Voorbeeld, behalve tijdens een opname: dan stuurt de gamepad het.
    fn speel_af(&mut self, letter: &str, h: &Herkomst) {
        if self.camera.opname.is_none() {
            self.camera.proef = Some(Proef { letter: letter.into(), sinds: h.volgnummer });
        }
    }

    pub(crate) fn herstel_camerarit(&mut self, letter: &str, h: &Herkomst) -> Result<(), Weigering> {
        let standaard = crate::vaste_ritten::standaard_van(letter)
            .ok_or_else(|| Weigering::OnbekendStandpunt(letter.into()))?;
        match self.cameraritten.iter_mut().find(|r| r.letter == letter) {
            Some(r) => *r = standaard,
            None => {
                let plek = self.cameraritten.iter().take_while(|r| r.vast).count();
                self.cameraritten.insert(plek, standaard);
            }
        }
        self.speel_af(letter, h);
        Ok(())
    }

    /// Zet de vaste Cameraritten die er niet zijn (een Stand van een oudere versie) vooraan, op
    /// volgorde. Een eigen Camerarit blijft zoals hij is.
    pub fn vul_vaste_ritten_aan(&mut self) {
        let mut vaste = crate::vaste_ritten::vaste();
        vaste.retain(|v| self.camerarit(&v.letter).is_none());
        let plek = self.cameraritten.iter().take_while(|r| r.vast).count();
        self.cameraritten.splice(plek..plek, vaste);
        self.cameraritten.sort_by_key(|r| !r.vast);
    }

    pub(crate) fn bijschaaf_camerarit(&mut self, letter: &str, b: Bijschaving, h: &Herkomst) -> Result<(), Weigering> {
        let r = self
            .cameraritten
            .iter_mut()
            .find(|r| r.letter == letter)
            .ok_or_else(|| Weigering::OnbekendStandpunt(letter.into()))?;
        if !r.past(&b) {
            return Err(Weigering::RitTeKort);
        }
        let oud = r.clone();
        let mut nieuw = Bijschaving {
            gladheid: b.gladheid.clamp(0.0, 1.0),
            zacht_starten: b.zacht_starten.clamp(0.0, 1.0),
            zacht_stoppen: b.zacht_stoppen.clamp(0.0, 1.0),
            begin_eraf_s: b.begin_eraf_s.max(0.0),
            eind_eraf_s: b.eind_eraf_s.max(0.0),
            ..b
        };
        r.bijschaving = nieuw;
        // Afknippen met dezelfde duur: de rit blijft even snel en wordt korter.
        let geknipt = (nieuw.begin_eraf_s, nieuw.eind_eraf_s) != (oud.bijschaving.begin_eraf_s, oud.bijschaving.eind_eraf_s);
        if geknipt && nieuw.duur_s == oud.bijschaving.duur_s {
            nieuw.duur_s = oud.duur_s() * r.stuk_s() / oud.stuk_s().max(1e-3);
        }
        r.bijschaving = nieuw;
        r.bijschaving.duur_s = r.duur_s();
        match r.kan_herhalen() {
            Err(reden) if nieuw.eind == RitEind::Herhalen => {
                if oud.bijschaving.eind == RitEind::Herhalen && geknipt {
                    // knippen gaat voor: dan stopt hij weer
                    r.bijschaving.eind = RitEind::Stoppen;
                } else {
                    *r = oud;
                    return Err(Weigering::HerhalenKanNiet(reden));
                }
            }
            _ => {}
        }
        self.speel_af(letter, h);
        Ok(())
    }
}
