//! Partijen en cijfersets in de Stand: inlezen, winst en verlies, Voorgangers en vergrendelen.
//! Alle cijfers hier zijn voorbeeldgegevens, behalve de uitslag van 2025 van de Kiesraad.

use gedeeld::cijfers::{Aanvulling, Invoer, Regel, Soort};
use gedeeld::{Herkomst, Opdracht, Stand, Weigering};

const H: Herkomst = Herkomst { door: "Proef", volgnummer: 1, tijd_ms: 0 };

/// Een Stand na het inlezen van dit invoerbestand.
fn stand_uit(json: &str) -> Stand {
    let mut stand = Stand::default();
    lees(&mut stand, json);
    stand
}

fn lees(stand: &mut Stand, json: &str) {
    stand
        .voer_uit(&Opdracht::LeesInvoer {
            bestand: "cijfers.json".into(),
            ontvangen: "21.00".into(),
            invoer: Invoer::uit_json(json),
        }, &H)
        .expect("invoer gelezen");
}

/// Voorbeeldgegevens: vorige keer Alfa 60, Beta 50, Gamma 30, Rood 6 en Groen 4.
/// Nu is Groenrood de opvolger van Rood en Groen, is Delta nieuw en doet Gamma niet meer mee.
const VOORBEELD: &str = r##"{
  "partijen": [
    { "sleutel": "alfa", "naam": "Alfa Partij", "kort": "ALFA", "kleur": "#1F6FB2" },
    { "sleutel": "beta", "naam": "Beta Unie", "kort": "BETA", "kleur": "#D33B2C" },
    { "sleutel": "gamma", "naam": "Gamma Lijst", "kort": "GAM", "kleur": "#3A9D5D" },
    { "sleutel": "rood", "naam": "Rood", "kort": "ROOD", "kleur": "#C00000" },
    { "sleutel": "groen", "naam": "Groen", "kort": "GROEN", "kleur": "#00A000" },
    { "sleutel": "groenrood", "naam": "Groenrood", "kort": "GR", "kleur": "#408040", "voorgangers": ["rood", "groen"] },
    { "sleutel": "delta", "naam": "Delta Nieuw", "kort": "DEL" }
  ],
  "sets": [
    { "id": "vorige", "soort": "uitslag", "label": "Vorige verkiezing",
      "zetels": { "alfa": 60, "beta": 50, "gamma": 30, "rood": 6, "groen": 4 } },
    { "id": "exit", "soort": "exitpoll", "label": "Exitpoll 21:00",
      "zetels": { "alfa": 62, "beta": 47, "groenrood": 15, "delta": 26 } }
  ],
  "vergelijk_met": "vorige"
}"##;

fn regels(stand: &Stand, set: &str) -> Vec<(String, String)> {
    stand
        .cijfers
        .winst_en_verlies(set)
        .expect("set en vergelijkingsset bestaan")
        .iter()
        .map(|r: &Regel| (r.partij.clone(), r.tekst()))
        .collect()
}

#[test]
fn winst_en_verlies_telt_voorgangers_op_toont_nieuw_en_min_n_voor_wie_weg_is() {
    let stand = stand_uit(VOORBEELD);
    assert_eq!(
        regels(&stand, "exit"),
        [
            ("alfa".to_string(), "+2".to_string()),
            ("beta".into(), "-3".into()),
            ("gamma".into(), "-30".into()),
            ("groenrood".into(), "+5".into()),
            ("delta".into(), "+26 nieuw".into()),
        ]
    );
}

#[test]
fn een_tweede_koppeling_van_dezelfde_voorganger_weigert_met_de_partij_waar_hij_al_bij_hoort() {
    let mut stand = stand_uit(VOORBEELD);
    let fout = stand
        .voer_uit(&Opdracht::KoppelVoorganger {
            partij: "delta".into(),
            voorganger: "rood".into(),
        }, &H)
        .unwrap_err();
    assert!(matches!(fout, Weigering::Cijfers(_)));
    assert_eq!(
        fout.to_string(),
        "\"rood\" is al Voorganger van Groenrood: een Voorganger hoort bij hooguit één Partij"
    );
    assert!(
        stand
            .cijfers
            .partij("delta")
            .unwrap()
            .voorgangers
            .is_empty()
    );

    // Na ontkoppelen bij Groenrood kan het wel, en telt Delta de Zetels van Rood mee.
    for o in [
        Opdracht::OntkoppelVoorganger {
            partij: "groenrood".into(),
            voorganger: "rood".into(),
        },
        Opdracht::KoppelVoorganger {
            partij: "delta".into(),
            voorganger: "rood".into(),
        },
    ] {
        stand.voer_uit(&o, &H).unwrap();
    }
    let delta = stand.cijfers.winst_en_verlies("exit").unwrap();
    let delta = delta.iter().find(|r| r.partij == "delta").unwrap();
    assert_eq!(delta.tekst(), "+20");
}

#[test]
fn een_dubbele_voorganger_in_het_bestand_wordt_niet_gekoppeld_en_gemeld() {
    let json = VOORBEELD.replace(
        r#""kort": "DEL" }"#,
        r#""kort": "DEL", "voorgangers": ["groen"] }"#,
    );
    let stand = stand_uit(&json);
    assert!(
        stand
            .cijfers
            .partij("delta")
            .unwrap()
            .voorgangers
            .is_empty()
    );
    let melding = stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert!(
        melding
            .waarschuwingen
            .iter()
            .any(|w| w.contains("delta") && w.contains("al Voorganger van Groenrood")),
        "{:?}",
        melding.waarschuwingen
    );
}

#[test]
fn een_gewijzigd_invoerbestand_komt_binnen_en_een_graphic_met_de_oude_cijfers_krijgt_nieuwere_cijfers()
 {
    let mut stand = stand_uit(VOORBEELD);
    let versie_in_beeld = stand.cijfers.set("exit").unwrap().versie;
    assert!(!stand.cijfers.nieuwere_cijfers("exit", versie_in_beeld));

    lees(
        &mut stand,
        &VOORBEELD.replace(r#""alfa": 62, "beta": 47"#, r#""alfa": 61, "beta": 48"#),
    );

    let exit = stand.cijfers.set("exit").unwrap();
    assert_eq!((exit.zetels_van("alfa"), exit.zetels_van("beta")), (61, 48));
    assert!(stand.cijfers.nieuwere_cijfers("exit", versie_in_beeld));
    assert!(!stand.cijfers.nieuwere_cijfers("vorige", 1));
    let melding = stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert_eq!(melding.sets_binnen, ["exit"]);
    assert_eq!(melding.sets_ongewijzigd, ["vorige"]);
    assert!(melding.overgeslagen.is_empty());
}

#[test]
fn een_vergrendelde_set_blijft_gelijk_bij_een_nieuw_bestand_en_bij_de_hand_tot_iemand_ontgrendelt()
{
    let mut stand = stand_uit(VOORBEELD);
    stand
        .voer_uit(&Opdracht::VergrendelSet { set: "exit".into() }, &H)
        .unwrap();
    let voor = stand.cijfers.set("exit").unwrap().clone();

    let gewijzigd = VOORBEELD.replace(r#""alfa": 62, "beta": 47"#, r#""alfa": 61, "beta": 48"#);
    lees(&mut stand, &gewijzigd);
    assert_eq!(stand.cijfers.set("exit").unwrap(), &voor);
    let melding = stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert_eq!(melding.overgeslagen.len(), 1);
    assert_eq!(melding.overgeslagen[0].wat, "exit");
    assert_eq!(melding.overgeslagen[0].reden, "de set is vergrendeld");

    let hand = Opdracht::ZetZetels {
        set: "exit".into(),
        partij: "alfa".into(),
        zetels: 70,
    };
    assert_eq!(
        stand.voer_uit(&hand, &H).unwrap_err().to_string(),
        "de set \"exit\" is vergrendeld: ontgrendel hem eerst"
    );
    assert_eq!(stand.cijfers.set("exit").unwrap(), &voor);

    stand
        .voer_uit(&Opdracht::OntgrendelSet { set: "exit".into() }, &H)
        .unwrap();
    lees(&mut stand, &gewijzigd);
    assert_eq!(stand.cijfers.set("exit").unwrap().zetels_van("alfa"), 61);
}

#[test]
fn sets_met_een_onbekende_partij_of_fout_veld_worden_overgeslagen_en_de_rest_komt_binnen() {
    let json = r##"{
      "partijen": [ { "sleutel": "alfa", "naam": "Alfa" }, { "naam": "zonder sleutel" } ],
      "sets": [
        { "id": "goed", "soort": "peiling", "label": "Peiling", "zetels": { "alfa": 140, "overig": 10 } },
        { "id": "typfout", "soort": "peiling", "label": "Peiling", "zetels": { "alfaa": 150 } },
        { "id": "soort", "soort": "gok", "label": "Gok", "zetels": { "alfa": 150 } },
        { "id": "krap", "soort": "prognose", "label": "Prognose", "zetels": { "alfa": 149 } }
      ]
    }"##;
    let stand = stand_uit(json);
    let m = stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert_eq!(m.sets_binnen, ["goed", "krap"]);
    let reden = |wat: &str| {
        m.overgeslagen
            .iter()
            .find(|o| o.wat == wat)
            .map(|o| o.reden.as_str())
    };
    assert_eq!(m.overgeslagen.len(), 3);
    assert!(reden("partij").unwrap().starts_with("Partij onleesbaar"));
    assert_eq!(
        reden("typfout"),
        Some("Partij \"alfaa\" staat niet in de partijlijst")
    );
    assert!(reden("soort").unwrap().starts_with("set onleesbaar"));
    // Niet op 150: wel binnen, met een waarschuwing, en niets stilzwijgend gecorrigeerd.
    assert_eq!(m.waarschuwingen, ["krap: telt op tot 149 Zetels, niet 150"]);
    assert_eq!(stand.cijfers.set("krap").unwrap().totaal(), 149);
}

#[test]
fn een_bestand_dat_geen_json_is_verandert_niets_en_wordt_gemeld() {
    let mut stand = stand_uit(VOORBEELD);
    let cijfers_voor = stand.cijfers.sets.clone();
    lees(&mut stand, "{ \"sets\": [ half");
    assert_eq!(stand.cijfers.sets, cijfers_voor);
    let m = stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert_eq!(m.overgeslagen[0].wat, "bestand");
    assert!(m.overgeslagen[0].reden.starts_with("geen geldige JSON"));
}

#[test]
fn een_partij_zonder_kleur_krijgt_een_tijdelijke_kleur_die_afwijkt_van_de_rest() {
    let json = r##"{ "partijen": [
        { "sleutel": "a", "naam": "A", "kleur": "#E6194B" },
        { "sleutel": "b", "naam": "B", "kleur": "#3CB44B", "logo": "logos/b.png" },
        { "sleutel": "c", "naam": "C Nieuw", "kort": "C" },
        { "sleutel": "d", "naam": "D Nieuw", "kort": "D", "kleur": "rood" },
        { "sleutel": "e", "naam": "Groep E", "kort": "E", "fractie_zonder_lijst": true }
    ] }"##;
    let stand = stand_uit(json);
    let p = |s: &str| stand.cijfers.partij(s).unwrap();
    assert!(!p("a").kleur_tijdelijk);
    assert_eq!(p("b").logo.as_deref(), Some("logos/b.png"));
    assert_eq!(p("c").logo, None);
    let tijdelijk: Vec<&str> = ["c", "d", "e"]
        .iter()
        .map(|s| p(s).kleur.as_str())
        .collect();
    for (s, k) in ["c", "d", "e"].iter().zip(&tijdelijk) {
        assert!(p(s).kleur_tijdelijk);
        assert!(k.starts_with('#') && k.len() == 7, "{k}");
        // Geen twee Partijen met dezelfde kleur, en geen tijdelijke kleur gelijk aan die van A of B.
        assert_ne!(*k, "#E6194B");
        assert_ne!(*k, "#3CB44B");
    }
    assert_ne!(tijdelijk[0], tijdelijk[1]);
    assert_ne!(tijdelijk[1], tijdelijk[2]);
    assert_ne!(tijdelijk[0], tijdelijk[2]);
    assert!(p("e").fractie_zonder_lijst);
    let m = stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert!(
        m.waarschuwingen
            .iter()
            .any(|w| w.starts_with("d: kleur \"rood\""))
    );
}

#[test]
fn de_uitslag_van_2025_van_de_kiesraad_laadt_en_telt_op_tot_150_zetels() {
    let json = include_str!("../../../cijfers/uitslag-tk2025.json");
    let stand = stand_uit(json);
    let m = stand.cijfers.laatste_invoer.as_ref().unwrap();
    assert!(m.overgeslagen.is_empty(), "{:?}", m.overgeslagen);
    assert!(m.waarschuwingen.is_empty(), "{:?}", m.waarschuwingen);
    assert_eq!(stand.cijfers.vergelijk_met.as_deref(), Some("tk2025"));
    let set = stand.cijfers.set("tk2025").unwrap();
    assert_eq!(set.totaal(), 150);
    // Officiële zetelverdeling, vastgesteld door de Kiesraad op 7 november 2025.
    let verwacht = [
        ("d66", 26),
        ("pvv", 26),
        ("vvd", 22),
        ("gl-pvda", 20),
        ("cda", 18),
        ("ja21", 9),
        ("fvd", 7),
        ("bbb", 4),
        ("denk", 3),
        ("cu", 3),
        ("sp", 3),
        ("sgp", 3),
        ("pvdd", 3),
        ("50plus", 2),
        ("volt", 1),
    ];
    for (p, z) in verwacht {
        assert_eq!(set.zetels_van(p), z, "{p}");
    }
    assert_eq!(set.zetels.len(), 15);
    assert_eq!(stand.cijfers.partijen.len(), 27);
}

// Live tijdens de uitzending: een Partij toevoegen en aanvullen, een set maken en de
// vergelijkingsset kiezen (#47).

fn voer_uit(stand: &mut Stand, o: Opdracht) -> Result<(), Weigering> {
    stand.voer_uit(&o, &H)
}

#[test]
fn een_live_toegevoegde_partij_zonder_kleur_krijgt_een_tijdelijke_kleur_en_kan_meteen_zetels_krijgen() {
    let mut stand = stand_uit(VOORBEELD);
    voer_uit(&mut stand, Opdracht::VoegPartijToe {
        naam: "Epsilon Beweging".into(),
        kort: "EPS".into(),
        kleur: None,
        logo: None,
        fractie_zonder_lijst: false,
    })
    .unwrap();
    let p = stand.cijfers.partijen.last().unwrap();
    assert_eq!((p.sleutel.as_str(), p.naam.as_str(), p.kort.as_str()), ("eps", "Epsilon Beweging", "EPS"));
    assert!(p.kleur_tijdelijk, "zonder kleur een tijdelijke kleur");
    assert!(p.logo.is_none());
    voer_uit(&mut stand, Opdracht::ZetZetels { set: "exit".into(), partij: "eps".into(), zetels: 3 }).unwrap();
    assert_eq!(stand.cijfers.set("exit").unwrap().zetels_van("eps"), 3);
    assert!(regels(&stand, "exit").contains(&("eps".into(), "+3 nieuw".into())));
}

#[test]
fn een_partij_die_er_al_is_kan_niet_nog_een_keer_erbij() {
    let mut stand = stand_uit(VOORBEELD);
    let fout = voer_uit(&mut stand, Opdracht::VoegPartijToe {
        naam: "Nog een Alfa".into(),
        kort: "Alfa".into(),
        kleur: None,
        logo: None,
        fractie_zonder_lijst: false,
    })
    .unwrap_err();
    assert_eq!(fout.to_string(), "ALFA staat al in de partijlijst");
    assert_eq!(stand.cijfers.partijen.len(), 7);
}

#[test]
fn aanvullen_verandert_alleen_de_velden_die_meegaan_en_een_eigen_kleur_vervangt_de_tijdelijke() {
    let mut stand = stand_uit(VOORBEELD);
    let vul_aan = |aanvulling| Opdracht::VulPartijAan { partij: "delta".into(), aanvulling };
    assert!(stand.cijfers.partij("delta").unwrap().kleur_tijdelijk);
    // Twee Werkplekken vullen elk een ander veld aan: allebei blijven staan.
    voer_uit(&mut stand, vul_aan(Aanvulling { kleur: Some("#7b2d8e".into()), ..Default::default() })).unwrap();
    voer_uit(&mut stand, vul_aan(Aanvulling { logo: Some("logos/delta.png".into()), ..Default::default() })).unwrap();
    let p = stand.cijfers.partij("delta").unwrap();
    assert_eq!((p.kleur.as_str(), p.kleur_tijdelijk), ("#7B2D8E", false));
    assert_eq!(p.logo.as_deref(), Some("logos/delta.png"));
    assert_eq!((p.naam.as_str(), p.kort.as_str()), ("Delta Nieuw", "DEL"));

    let fout = voer_uit(&mut stand, vul_aan(Aanvulling { kleur: Some("paars".into()), ..Default::default() })).unwrap_err();
    assert_eq!(fout.to_string(), "kleur \"paars\" is geen #RRGGBB");
    assert_eq!(stand.cijfers.partij("delta").unwrap().kleur, "#7B2D8E");
}

#[test]
fn een_nieuwe_set_begint_leeg_en_de_vergelijkingsset_bepaalt_winst_en_verlies() {
    let mut stand = stand_uit(VOORBEELD);
    voer_uit(&mut stand, Opdracht::MaakSet { setsoort: Soort::Prognose, label: "Prognose 22:00".into() }).unwrap();
    voer_uit(&mut stand, Opdracht::MaakSet { setsoort: Soort::Prognose, label: "Prognose 22.00".into() }).unwrap();
    let ids: Vec<&str> = stand.cijfers.sets.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["vorige", "exit", "prognose2200", "prognose2200-2"]);
    let nieuw = stand.cijfers.set("prognose2200").unwrap();
    assert_eq!((nieuw.totaal(), nieuw.vergrendeld, nieuw.soort), (0, false, Soort::Prognose));

    voer_uit(&mut stand, Opdracht::ZetZetels { set: "prognose2200".into(), partij: "alfa".into(), zetels: 64 }).unwrap();
    voer_uit(&mut stand, Opdracht::KiesVergelijkingsset { set: Some("exit".into()) }).unwrap();
    assert_eq!(regels(&stand, "prognose2200")[0], ("alfa".into(), "+2".into()));

    let fout = voer_uit(&mut stand, Opdracht::KiesVergelijkingsset { set: Some("weg".into()) }).unwrap_err();
    assert_eq!(fout.to_string(), "er is geen set \"weg\"");
    assert_eq!(stand.cijfers.vergelijk_met.as_deref(), Some("exit"));
    voer_uit(&mut stand, Opdracht::KiesVergelijkingsset { set: None }).unwrap();
    assert!(stand.cijfers.winst_en_verlies("exit").is_none(), "zonder vergelijkingsset geen winst en verlies");
}
