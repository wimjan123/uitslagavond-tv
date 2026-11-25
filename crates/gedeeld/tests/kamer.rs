//! Kamerleden in de Stand: voorletters voor wie geen foto heeft, en een eigen foto van de omroep.
//! De namen zijn echte Kamerleden uit de open data; ophalen, wissels en onbekende Fracties
//! testen we in de server, met vastgelegde antwoorden van de API.

use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::{Herkomst, Opdracht, Stand, Weigering};

const H: Herkomst = Herkomst {
    door: "Proef",
    volgnummer: 1,
    tijd_ms: 0,
};

fn kamerlid(id: &str, roepnaam: &str, tussenvoegsel: Option<&str>, achternaam: &str) -> Kamerlid {
    Kamerlid {
        id: id.into(),
        roepnaam: roepnaam.into(),
        initialen: String::new(),
        tussenvoegsel: tussenvoegsel.map(Into::into),
        achternaam: achternaam.into(),
        fractie: "f-vvd".into(),
        zetel: format!("zetel-{id}"),
        fractievoorzitter: false,
        van: "2025-11-12".into(),
        geboortedatum: None,
        geboorteplaats: None,
        woonplaats: None,
        foto: Foto::OpenData,
        in_de_kamer_sinds: None,
        kieslijst: None,
    }
}

#[test]
fn voorletters_zijn_de_eerste_letter_van_roepnaam_en_achternaam() {
    let al_biyati = kamerlid("1", "Iem", Some("Al"), "Biyati");
    assert_eq!(al_biyati.naam(), "Iem Al Biyati");
    assert_eq!(al_biyati.voorletters(), "IB");

    let van_groningen = kamerlid("2", "Dieke", Some("van"), "Groningen");
    assert_eq!(van_groningen.naam(), "Dieke van Groningen");
    assert_eq!(van_groningen.voorletters(), "DG");

    // Zonder roepnaam de eerste voorletter; kleine letters en een Ö worden hoofdletters.
    let mut zonder_roepnaam = kamerlid("3", "", None, "öztürk");
    zonder_roepnaam.initialen = "d.a.".into();
    assert_eq!(zonder_roepnaam.voorletters(), "DÖ");
}

fn stand_met(kamerleden: Vec<Kamerlid>) -> Stand {
    let mut stand = Stand::default();
    stand
        .voer_uit(
            &Opdracht::LeesKamer {
                opgehaald: "2026-11-22T12:00:00+01:00".into(),
                peildatum: "2026-11-22".into(),
                kamerleden,
                fracties: vec![Fractie {
                    id: "f-vvd".into(),
                    afkorting: "VVD".into(),
                    naam: "Volkspartij voor Vrijheid en Democratie".into(),
                    zetels: 22,
                }],
            },
            &H,
        )
        .expect("Kamer gelezen");
    stand
}

#[test]
fn een_kamerlid_zonder_foto_staat_op_de_lijst_tot_de_omroep_een_eigen_foto_geeft() {
    let mut zonder = kamerlid("1", "Iem", Some("Al"), "Biyati");
    zonder.foto = Foto::Geen;
    let mut stand = stand_met(vec![
        zonder,
        kamerlid("2", "Dieke", Some("van"), "Groningen"),
    ]);

    let namen = |s: &Stand| s.kamer.zonder_foto().map(|k| k.naam()).collect::<Vec<_>>();
    assert_eq!(namen(&stand), ["Iem Al Biyati"]);

    stand
        .voer_uit(
            &Opdracht::ZetFoto {
                kamerlid: "1".into(),
                foto: Foto::Eigen,
            },
            &H,
        )
        .unwrap();
    assert!(namen(&stand).is_empty());
    assert_eq!(stand.kamer.kamerlid("1").unwrap().foto, Foto::Eigen);

    let fout = stand.voer_uit(
        &Opdracht::ZetFoto {
            kamerlid: "onbekend".into(),
            foto: Foto::Eigen,
        },
        &H,
    );
    assert!(matches!(fout, Err(Weigering::Kamer(_))), "{fout:?}");
}

#[test]
fn komt_de_partijlijst_na_de_kamer_dan_hoort_de_fractie_daarna_bij_de_echte_partij() {
    let mut stand = Stand::default();
    let cu = Fractie {
        id: "f-cu".into(),
        afkorting: "ChristenUnie".into(),
        naam: "ChristenUnie".into(),
        zetels: 3,
    };
    let lees = |stand: &mut Stand| {
        let mut k = kamerlid("1", "Mirjam", None, "Bikker");
        k.fractie = "f-cu".into();
        stand
            .voer_uit(
                &Opdracht::LeesKamer {
                    opgehaald: "2026-11-22T12:00:00+01:00".into(),
                    peildatum: "2026-11-22".into(),
                    kamerleden: vec![k],
                    fracties: vec![cu.clone()],
                },
                &H,
            )
            .unwrap();
    };
    lees(&mut stand);
    assert_eq!(stand.kamer.partij_van["f-cu"], "christenunie");

    stand
        .voer_uit(&Opdracht::LeesInvoer {
            bestand: "cijfers.json".into(),
            ontvangen: "12.01".into(),
            invoer: gedeeld::cijfers::Invoer::uit_json(
                r##"{ "partijen": [ { "sleutel": "cu", "naam": "ChristenUnie", "kort": "CU", "kleur": "#00A7EB" } ] }"##,
            ),
        }, &H)
        .unwrap();
    lees(&mut stand);
    assert_eq!(stand.kamer.partij_van["f-cu"], "cu");
}
