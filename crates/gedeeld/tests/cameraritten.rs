//! Cameraritten: opnemen tijdens Oefenen en bijschaven met schuifjes (#45).

use gedeeld::camera::Camerastand;
use gedeeld::camerarit::{Bijschaving, Camerarit, RitBegin, RitEind};

/// Een opname van `sec` seconden, 100 standen per seconde, met de plek op elk moment.
fn opname(sec: f32, plek: impl Fn(f32) -> [f32; 3]) -> Vec<[f32; 6]> {
    let n = (sec * 100.0).round() as usize;
    (0..=n)
        .map(|i| {
            let [x, y, z] = plek(i as f32 / 100.0);
            [x, y, z, 0.0, -0.1, 0.9]
        })
        .collect()
}

/// Een Camerarit zonder bijschaving: speelt af zoals opgenomen.
fn rit(opname: Vec<[f32; 6]>) -> Camerarit {
    let duur_s = (opname.len() - 1) as f32 / 100.0;
    Camerarit {
        letter: "R1".into(),
        naam: "Voorbeeldrit".into(),
        toets: None,
        opname,
        bijschaving: Bijschaving {
            gladheid: 0.0,
            duur_s,
            begin_eraf_s: 0.0,
            eind_eraf_s: 0.0,
            zacht_starten: 0.0,
            zacht_stoppen: 0.0,
            begin: RitBegin::Overgang,
            eind: RitEind::Stoppen,
        },
    }
}

fn x(s: &Camerastand) -> f32 {
    s.positie[0]
}

fn dicht(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

/// Een meter per seconde langs x, tien seconden lang.
fn rechtdoor() -> Camerarit {
    rit(opname(10.0, |t| [t, 3.0, -4.0]))
}

#[test]
fn zonder_bijschaving_speelt_de_rit_af_zoals_opgenomen() {
    let baan = rechtdoor().baan();
    assert_eq!(baan.standen.len(), 1001, "100 standen per seconde, tien seconden");
    assert!(dicht(x(&baan.standen[0]), 0.0));
    assert!(dicht(x(&baan.standen[250]), 2.5));
    assert!(dicht(x(&baan.standen[1000]), 10.0));
}

#[test]
fn begin_en_eind_eraf_knippen_de_rit_af() {
    let mut r = rechtdoor();
    r.bijschaving.begin_eraf_s = 2.0;
    r.bijschaving.eind_eraf_s = 3.0;
    r.bijschaving.duur_s = 5.0;
    let baan = r.baan();
    assert_eq!(baan.standen.len(), 501, "er blijft vijf seconden over");
    assert!(dicht(x(&baan.standen[0]), 2.0), "begint twee seconden later");
    assert!(dicht(x(&baan.standen[250]), 4.5));
    assert!(dicht(x(&baan.standen[500]), 7.0), "stopt drie seconden eerder");
    // de groene streep op de balk: waar in de opname het Voorbeeld is
    assert!(dicht(baan.opname_s(0), 2.0) && dicht(baan.opname_s(500), 7.0));
}

#[test]
fn een_langere_duur_speelt_hetzelfde_stuk_langzamer_af() {
    let mut r = rechtdoor();
    r.bijschaving.begin_eraf_s = 2.0;
    r.bijschaving.eind_eraf_s = 3.0;
    r.bijschaving.duur_s = 10.0;
    let baan = r.baan();
    assert_eq!(baan.standen.len(), 1001);
    assert!(dicht(x(&baan.standen[0]), 2.0));
    assert!(dicht(x(&baan.standen[500]), 4.5), "halverwege de tijd, halverwege de weg");
    assert!(dicht(x(&baan.standen[1000]), 7.0));
}

#[test]
fn gladheid_haalt_het_trillen_eruit_en_houdt_begin_en_eind() {
    // vooruit langs x, en elke tik 4 cm heen en weer langs z: een trillende hand
    let trillend = opname(6.0, |t| {
        let tik = (t * 100.0).round() as i32;
        [t, 3.0, -4.0 + if tik % 2 == 0 { 0.02 } else { -0.02 }]
    });
    let trilling = |r: &Camerarit| {
        let s = r.baan().standen;
        s.windows(2).skip(100).take(400).map(|w| (w[1].positie[2] - w[0].positie[2]).abs()).fold(0.0, f32::max)
    };
    let ruw = rit(trillend.clone());
    assert!(trilling(&ruw) > 0.039, "zonder gladheid blijft het trillen");

    let mut glad = rit(trillend.clone());
    glad.bijschaving.gladheid = 1.0;
    assert!(trilling(&glad) < 0.002, "{}", trilling(&glad));
    let s = glad.baan().standen;
    // in het midden ligt de route op de lijn tussen de trillingen
    assert!((s[300].positie[2] + 4.0).abs() < 0.002);
    assert!(dicht(x(&s[300]), 3.0));
    // begin en eind blijven waar de opname begint en eindigt
    assert_eq!(s[0].positie, [trillend[0][0], trillend[0][1], trillend[0][2]]);
    let laatste = trillend.last().unwrap();
    assert_eq!(s.last().unwrap().positie, [laatste[0], laatste[1], laatste[2]]);
}

#[test]
fn gladheid_rondt_een_scherpe_hoek_af() {
    // drie seconden langs x, dan drie seconden langs z
    let hoek = opname(6.0, |t| if t <= 3.0 { [t, 3.0, -4.0] } else { [3.0, 3.0, -4.0 - (t - 3.0)] });
    let mut r = rit(hoek);
    r.bijschaving.gladheid = 1.0;
    let s = r.baan().standen;
    // op de hoek zelf snijdt de route af: niet meer op (3, -4)
    let p = s[300].positie;
    assert!(p[0] < 2.9 && p[2] < -4.1, "{p:?}");
    // ver van de hoek blijft de route op de lijn
    assert!((s[100].positie[2] + 4.0).abs() < 1e-3);
}

#[test]
fn zacht_starten_en_stoppen_begint_en_eindigt_langzaam() {
    let mut r = rechtdoor();
    r.bijschaving.zacht_starten = 1.0;
    r.bijschaving.zacht_stoppen = 1.0;
    let s = r.baan().standen;
    let stap = |i: usize| x(&s[i + 1]) - x(&s[i]);
    assert!(stap(0) < 0.001, "zacht starten: {}", stap(0));
    assert!(stap(999) < 0.001, "zacht stoppen: {}", stap(999));
    assert!(stap(500) > 0.01, "in het midden sneller dan opgenomen: {}", stap(500));
    assert!(dicht(x(&s[0]), 0.0) && dicht(x(&s[1000]), 10.0), "zelfde begin en eind");
}

#[test]
fn herhalen_kan_alleen_als_begin_en_eind_bijna_gelijk_zijn() {
    let ver = rechtdoor();
    assert_eq!(ver.kan_herhalen().unwrap_err(), "begin en eind liggen 10,0 m uit elkaar");
    // een rondje: begint en eindigt op dezelfde plek
    let rond = rit(opname(8.0, |t| {
        let a = t / 8.0 * std::f32::consts::TAU;
        [2.0 * a.sin(), 3.0, -4.0 + 2.0 * (1.0 - a.cos())]
    }));
    assert_eq!(rond.kan_herhalen(), Ok(()));
    // een stuk eraf knippen haalt begin en eind uit elkaar
    let mut half = rond.clone();
    half.bijschaving.eind_eraf_s = 4.0;
    assert!(half.kan_herhalen().unwrap_err().starts_with("begin en eind liggen 4,0 m uit elkaar"));
    // en anders kijken telt ook
    let mut draai = rond.opname.clone();
    draai.last_mut().unwrap()[3] = 0.9;
    assert!(rit(draai).kan_herhalen().unwrap_err().contains("kijken 52° anders"));
}

// De Camerarit in de Stand: Oefenen, Opnemen, de letter en bijschaven.

use gedeeld::camera::Proef;
use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Weigering};

fn doe(stand: &mut Stand, volgnummer: &mut u64, o: Opdracht) -> Result<(), Weigering> {
    *volgnummer += 1;
    let h = Herkomst { door: "Proef", volgnummer: *volgnummer, tijd_ms: 0 };
    stand.voer_uit(&o, &h)
}

fn oefenen(aan: bool) -> Opdracht {
    Opdracht::ZetOefenen { aan }
}

fn start_opname() -> Opdracht {
    Opdracht::ZetOpnemen { aan: true, voor: None, opname: None }
}

/// Stopt de opname; de server vult in wat hij opnam.
fn stop_opname(opname: Vec<[f32; 6]>) -> Opdracht {
    Opdracht::ZetOpnemen { aan: false, voor: None, opname: Some(opname) }
}

/// Oefenen aan, een rit opnemen, en de sleutel van de nieuwe Camerarit.
fn neem_op(stand: &mut Stand, v: &mut u64, o: Vec<[f32; 6]>) -> String {
    doe(stand, v, oefenen(true)).unwrap();
    doe(stand, v, start_opname()).unwrap();
    doe(stand, v, stop_opname(o)).unwrap();
    stand.cameraritten.last().unwrap().letter.clone()
}

fn vijf_meter() -> Vec<[f32; 6]> {
    opname(5.0, |t| [t, 3.0, -4.0])
}

#[test]
fn opnemen_kan_alleen_tijdens_oefenen_en_oefenen_zet_de_vrije_camera_uit() {
    let mut stand = Stand::default();
    let mut v = 0;
    assert_eq!(doe(&mut stand, &mut v, start_opname()), Err(Weigering::OpnemenZonderOefenen));
    doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true }).unwrap();
    doe(&mut stand, &mut v, oefenen(true)).unwrap();
    assert!(stand.camera.oefenen && !stand.camera.vrij, "Oefenen en Vrij gaan niet samen");
    doe(&mut stand, &mut v, start_opname()).unwrap();
    assert!(stand.camera.opname.is_some());
    // tijdens de opname blijft Oefenen aan
    assert_eq!(doe(&mut stand, &mut v, oefenen(false)), Err(Weigering::OpnameLoopt));
    assert_eq!(doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true }), Err(Weigering::OpnameLoopt));
    // een Standpunt kiezen voor de Ether mag wel: de Ether staat los van Oefenen
    doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: "G".into(), knip: false }).unwrap();
    assert!(stand.camera.oefenen && stand.camera.opname.is_some());
    doe(&mut stand, &mut v, stop_opname(vijf_meter())).unwrap();
    doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true }).unwrap();
    assert!(stand.camera.vrij && !stand.camera.oefenen, "Vrij zet Oefenen uit");
}

#[test]
fn na_een_opname_staat_de_camerarit_op_de_eerste_vrije_letter_en_speelt_hij_af() {
    let mut stand = Stand::default();
    let mut v = 0;
    let plek = gedeeld::camera::Camerastand::uit_graden([1.0, 2.5, -4.0], 0.0, -8.0, 50.0);
    doe(&mut stand, &mut v, Opdracht::BewaarStandpunt { naam: "Close-up".into(), stand: Some(plek) }).unwrap();
    let r = neem_op(&mut stand, &mut v, vijf_meter());
    let rit = stand.camerarit(&r).unwrap();
    assert_eq!(rit.toets.as_deref(), Some("K"), "J is van het Standpunt");
    assert_eq!(rit.naam, "Camerarit 1");
    assert_eq!(rit.opname, vijf_meter());
    assert!((rit.bijschaving.duur_s - 5.0).abs() < 1e-4);
    assert_eq!(stand.camera.opname, None);
    assert_eq!(stand.camera.proef, Some(Proef { letter: r.clone(), sinds: v }), "speelt af in het Voorbeeld");
    // de volgende krijgt L, en daarna geen toets meer
    let r2 = neem_op(&mut stand, &mut v, vijf_meter());
    let r3 = neem_op(&mut stand, &mut v, vijf_meter());
    assert_eq!(stand.camerarit(&r2).unwrap().toets.as_deref(), Some("L"));
    assert_eq!(stand.camerarit(&r3).unwrap().toets, None);
    assert_eq!(stand.camerarit(&r3).unwrap().naam, "Camerarit 3");
    assert_ne!(r2, r3);
}

#[test]
fn j_k_en_l_horen_bij_een_standpunt_of_een_camerarit_niet_bij_allebei() {
    let mut stand = Stand::default();
    let mut v = 0;
    let plek = gedeeld::camera::Camerastand::uit_graden([1.0, 2.5, -4.0], 0.0, -8.0, 50.0);
    doe(&mut stand, &mut v, Opdracht::BewaarStandpunt { naam: "Close-up".into(), stand: Some(plek) }).unwrap();
    let r = neem_op(&mut stand, &mut v, vijf_meter());
    doe(&mut stand, &mut v, Opdracht::ZetStandpuntToets { letter: r.clone(), toets: Some("J".into()) }).unwrap();
    assert_eq!(stand.camerarit(&r).unwrap().toets.as_deref(), Some("J"));
    assert_eq!(stand.standpunten.last().unwrap().toets, None, "het Standpunt raakt J kwijt");
    // een nieuw Standpunt krijgt de eerste toets die geen van beide heeft
    doe(&mut stand, &mut v, Opdracht::BewaarStandpunt { naam: "Twee".into(), stand: Some(plek) }).unwrap();
    assert_eq!(stand.standpunten.last().unwrap().toets.as_deref(), Some("K"));
    doe(&mut stand, &mut v, Opdracht::HernoemStandpunt { letter: r.clone(), naam: "Langs de banken".into() }).unwrap();
    assert_eq!(stand.camerarit(&r).unwrap().naam, "Langs de banken");
}

#[test]
fn een_camerarit_kies_je_overal_waar_je_een_standpunt_kiest() {
    let mut stand = Stand::default();
    let mut v = 0;
    let r = neem_op(&mut stand, &mut v, vijf_meter());
    doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: r.clone(), knip: false }).unwrap();
    assert_eq!(stand.camera.standpunt, r);
    assert_eq!(stand.camera.proef, None, "een keuze voor de Ether stopt het afspelen in het Voorbeeld");
    let graphic = Graphic::Titelkaart { kop: "Voorbeeldgegevens".into(), regel: String::new() };
    doe(&mut stand, &mut v, Opdracht::ZetKlaar { graphic: graphic.clone(), standpunt: Some(r.clone()) }).unwrap();
    doe(&mut stand, &mut v, Opdracht::BewaarPreset { graphic, standpunt: Some(r.clone()) }).unwrap();
    doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: "A".into(), knip: true }).unwrap();
    let fout = doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: r.clone() }).unwrap_err();
    assert_eq!(
        fout.to_string(),
        "Camerarit J (Camerarit 1) wordt gebruikt door de klaargezette Graphic Titelkaart \"Voorbeeldgegevens\" \
         en Preset 1 Titelkaart \"Voorbeeldgegevens\""
    );
    doe(&mut stand, &mut v, Opdracht::WisPreset { nummer: 1 }).unwrap();
    doe(&mut stand, &mut v, Opdracht::ZetKlaar {
        graphic: Graphic::Titelkaart { kop: "Ander".into(), regel: String::new() },
        standpunt: None,
    })
    .unwrap();
    doe(&mut stand, &mut v, Opdracht::WisStandpunt { letter: r.clone() }).unwrap();
    assert!(stand.camerarit(&r).is_none());
}

fn bijschaaf(stand: &Stand, r: &str, f: impl FnOnce(&mut Bijschaving)) -> Opdracht {
    let mut b = stand.camerarit(r).unwrap().bijschaving;
    f(&mut b);
    Opdracht::BijschaafCamerarit { letter: r.into(), bijschaving: b }
}

#[test]
fn bijschaven_speelt_meteen_af_en_herhalen_vraagt_een_gelijk_begin_en_eind() {
    let mut stand = Stand::default();
    let mut v = 0;
    let rond = opname(8.0, |t| {
        let a = t / 8.0 * std::f32::consts::TAU;
        [2.0 * a.sin(), 3.0, -4.0 + 2.0 * (1.0 - a.cos())]
    });
    let r = neem_op(&mut stand, &mut v, rond);
    let o = bijschaaf(&stand, &r, |b| b.eind = RitEind::Herhalen);
    doe(&mut stand, &mut v, o).unwrap();
    assert_eq!(stand.camerarit(&r).unwrap().bijschaving.eind, RitEind::Herhalen);
    assert_eq!(stand.camera.proef, Some(Proef { letter: r.clone(), sinds: v }), "elke wijziging speelt af");

    // de helft eraf knippen: begin en eind liggen uit elkaar, dus stopt hij weer
    let o = bijschaaf(&stand, &r, |b| b.eind_eraf_s = 4.0);
    doe(&mut stand, &mut v, o).unwrap();
    let rit = stand.camerarit(&r).unwrap();
    assert_eq!(rit.bijschaving.eind, RitEind::Stoppen);
    // de duur gaat mee met wat overblijft: even snel als opgenomen
    assert!((rit.bijschaving.duur_s - 4.0).abs() < 1e-3, "{}", rit.bijschaving.duur_s);
    let o = bijschaaf(&stand, &r, |b| b.eind = RitEind::Herhalen);
    assert_eq!(
        doe(&mut stand, &mut v, o),
        Err(Weigering::HerhalenKanNiet("begin en eind liggen 4,0 m uit elkaar".into()))
    );
    // er moet een halve seconde overblijven
    let o = bijschaaf(&stand, &r, |b| b.begin_eraf_s = 3.8);
    assert_eq!(doe(&mut stand, &mut v, o), Err(Weigering::RitTeKort));
}

#[test]
fn opnieuw_opnemen_vervangt_de_opname_van_dezelfde_camerarit() {
    let mut stand = Stand::default();
    let mut v = 0;
    let r = neem_op(&mut stand, &mut v, vijf_meter());
    let o = bijschaaf(&stand, &r, |b| b.begin_eraf_s = 1.0);
    doe(&mut stand, &mut v, o).unwrap();
    doe(&mut stand, &mut v, Opdracht::ZetOpnemen { aan: true, voor: Some(r.clone()), opname: None }).unwrap();
    let nieuw = opname(3.0, |t| [0.0, 3.0, -4.0 - t]);
    doe(&mut stand, &mut v, stop_opname(nieuw.clone())).unwrap();
    assert_eq!(stand.cameraritten.len(), 1);
    let rit = stand.camerarit(&r).unwrap();
    assert_eq!((rit.opname.clone(), rit.toets.as_deref(), rit.bijschaving.begin_eraf_s), (nieuw, Some("J"), 0.0));
    assert!((rit.bijschaving.duur_s - 3.0).abs() < 1e-4);
}

#[test]
fn de_beginstand_is_de_eerste_stand_van_de_baan() {
    let mut r = rechtdoor();
    r.bijschaving.begin_eraf_s = 2.5;
    r.bijschaving.gladheid = 1.0;
    assert_eq!(r.beginstand(), Some(r.baan().standen[0]));
}
