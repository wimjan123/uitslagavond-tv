//! De Zaalindeling: welk Kamerlid op welke Stoel zit, uit een bestand per Kamerperiode.

use gedeeld::Stoel;
use gedeeld::zaalindeling::{Controle, Zaalindeling};

const TK2025: &str = include_str!("../../../zaalindeling/tk2025.json");

fn stoel(vak: u8, rij: u8, nummer: u8) -> Stoel {
    Stoel { vak, rij, nummer }
}

#[test]
fn het_overgezette_bestand_heeft_150_stoelen_en_niemand_dubbel() {
    let z = Zaalindeling::uit_json(TK2025).expect("leesbaar");
    assert_eq!(z.kamerperiode, "2025");
    let c = z.controleer();
    assert_eq!(
        c,
        Controle::default(),
        "geen ontbrekende, dubbele of onbekende Stoelen"
    );
    assert!(c.in_orde());
    assert_eq!(
        z.stoelen.iter().filter(|s| s.kamerlid.is_some()).count(),
        150
    );
}

#[test]
fn de_controle_meldt_ontbrekende_dubbele_en_onbekende_stoelen() {
    let mut z = Zaalindeling::uit_json(TK2025).unwrap();
    // Stoel 1-1-2 weg, Stoel 1-1-1 twee keer, een Stoel die niet bestaat, en het Kamerlid van
    // 1-2-1 ook op 1-2-2
    z.stoelen.retain(|s| s.stoel() != stoel(1, 1, 2));
    let eerste = z.stoelen[0].clone();
    z.stoelen.push(eerste);
    let mut buiten = z.stoelen[1].clone();
    (buiten.vak, buiten.rij, buiten.nummer) = (7, 1, 1);
    buiten.kamerlid = None;
    z.stoelen.push(buiten);
    let wie = z
        .stoelen
        .iter()
        .find(|s| s.stoel() == stoel(1, 2, 1))
        .unwrap()
        .kamerlid
        .clone();
    z.stoelen
        .iter_mut()
        .find(|s| s.stoel() == stoel(1, 2, 2))
        .unwrap()
        .kamerlid = wie.clone();

    let c = z.controleer();
    assert_eq!(c.ontbrekend, [stoel(1, 1, 2)]);
    assert_eq!(c.dubbele_stoelen, [stoel(1, 1, 1)]);
    assert_eq!(c.onbekende_stoelen, [stoel(7, 1, 1)]);
    assert_eq!(c.dubbele_kamerleden, [wie.unwrap()]);
    assert!(!c.in_orde());
}

// --- De Zaalindeling in de Stand: voorstel bij een wissel ---

use gedeeld::cijfers::Invoer;
use gedeeld::kamer::{Foto, Fractie, Kamerlid};
use gedeeld::sjabloon::GRIJS;
use gedeeld::zaalindeling::{Bezetting, Stoelvoorstel};
use gedeeld::{Graphic, Herkomst, Indeling, Kleur, Opdracht, Stand, Weigering};

/// VOORBEELDGEGEVENS: verzonnen Kamerleden en Fracties.
fn lid(id: &str, achternaam: &str, fractie: &str, zetel: &str) -> Kamerlid {
    Kamerlid {
        id: id.into(),
        roepnaam: "Proef".into(),
        initialen: "P.".into(),
        tussenvoegsel: None,
        achternaam: achternaam.into(),
        fractie: fractie.into(),
        zetel: zetel.into(),
        fractievoorzitter: false,
        van: "2025-11-12".into(),
        geboortedatum: None,
        geboorteplaats: None,
        woonplaats: None,
        foto: Foto::Geen,
        in_de_kamer_sinds: None,
        kieslijst: None,
    }
}

fn fracties() -> Vec<Fractie> {
    ["rood", "blauw"]
        .map(|f| Fractie {
            id: f.into(),
            afkorting: f.to_uppercase(),
            naam: f.into(),
            zetels: 2,
        })
        .into()
}

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    /// Vier Kamerleden: A en B van Rood op 1-1-1 en 1-1-2, C en D van Blauw op 1-2-1 en 1-2-2.
    fn new() -> Self {
        let mut p = Proef {
            stand: Stand::default(),
            volgnummer: 0,
        };
        let invoer = Invoer::uit_json(
            r##"{ "partijen": [
                { "sleutel": "rood", "naam": "Rode Partij", "kort": "ROOD", "kleur": "#d4202c" },
                { "sleutel": "blauw", "naam": "Blauwe Partij", "kort": "BLAUW", "kleur": "#12357a" }
            ], "sets": [] }"##,
        );
        p.doe(Opdracht::LeesInvoer {
            bestand: "proef.json".into(),
            ontvangen: "nu".into(),
            invoer,
        })
        .unwrap();
        p.lees_kamer(vec![
            lid("a", "Aal", "rood", "z1"),
            lid("b", "Bes", "rood", "z2"),
            lid("c", "Cor", "blauw", "z3"),
            lid("d", "Dop", "blauw", "z4"),
        ]);
        let stoelen = [("a", 1, 1), ("b", 1, 2), ("c", 2, 1), ("d", 2, 2)]
            .map(|(k, rij, nummer)| Bezetting {
                vak: 1,
                rij,
                nummer,
                kamerlid: Some(k.into()),
                naam: k.into(),
            })
            .into();
        let zaalindeling = Zaalindeling {
            kamerperiode: "2025".into(),
            bron: String::new(),
            stoelen,
        };
        p.doe(Opdracht::LeesZaalindeling {
            bestand: "proef.json".into(),
            zaalindeling,
        })
        .unwrap();
        p
    }

    fn lees_kamer(&mut self, kamerleden: Vec<Kamerlid>) {
        self.doe(Opdracht::LeesKamer {
            opgehaald: "nu".into(),
            peildatum: "2026-11-22".into(),
            kamerleden,
            fracties: fracties(),
        })
        .unwrap();
    }

    fn doe(&mut self, opdracht: Opdracht) -> Result<(), Weigering> {
        let mut nieuw = self.stand.clone();
        let h = Herkomst {
            door: "Regie 1",
            volgnummer: self.volgnummer + 1,
            tijd_ms: 1_000,
        };
        nieuw.voer_uit(&opdracht, &h)?;
        self.stand = nieuw;
        self.volgnummer += 1;
        Ok(())
    }
}

#[test]
fn bij_een_wissel_stelt_het_systeem_de_stoel_van_de_vertrokken_persoon_voor() {
    let mut p = Proef::new();
    assert!(p.stand.zaalmelding.voorstellen.is_empty());
    assert!(p.stand.zaalmelding.zonder_stoel.is_empty());

    // Aal vertrekt; Eik komt op dezelfde FractieZetel
    p.lees_kamer(vec![
        lid("e", "Eik", "rood", "z1"),
        lid("b", "Bes", "rood", "z2"),
        lid("c", "Cor", "blauw", "z3"),
        lid("d", "Dop", "blauw", "z4"),
    ]);
    // tot de Regie bevestigt heeft Eik geen Stoel, en de Stoel van Aal is leeg
    assert_eq!(p.stand.zaalindeling.stoel_van("e"), None);
    assert_eq!(p.stand.zaalmelding.zonder_stoel, ["Proef Eik"]);
    assert_eq!(
        p.stand.zaalmelding.voorstellen,
        [Stoelvoorstel {
            stoel: stoel(1, 1, 1),
            kamerlid: "e".into(),
            naam: "Proef Eik".into(),
            vertrokken: "Proef Aal".into(),
        }]
    );

    // één klik: de Regie bevestigt
    p.doe(Opdracht::GeefStoel {
        kamerlid: "e".into(),
        stoel: stoel(1, 1, 1),
    })
    .unwrap();
    assert_eq!(p.stand.zaalindeling.stoel_van("e"), Some(stoel(1, 1, 1)));
    assert!(p.stand.zaalmelding.voorstellen.is_empty());
    assert!(p.stand.zaalmelding.zonder_stoel.is_empty());
}

#[test]
fn komt_de_opvolger_op_een_andere_fractiezetel_dan_telt_de_fractie() {
    let mut p = Proef::new();
    // Dop (Blauw) vertrekt en Fem (Blauw) komt op een nieuwe FractieZetel; Gijs (Rood) komt erbij
    // zonder dat er bij Rood iemand vertrok
    p.lees_kamer(vec![
        lid("a", "Aal", "rood", "z1"),
        lid("b", "Bes", "rood", "z2"),
        lid("c", "Cor", "blauw", "z3"),
        lid("f", "Fem", "blauw", "z9"),
        lid("g", "Gijs", "rood", "z8"),
    ]);
    let voorstellen = &p.stand.zaalmelding.voorstellen;
    assert_eq!(voorstellen.len(), 1);
    assert_eq!(
        (voorstellen[0].stoel, voorstellen[0].kamerlid.as_str()),
        (stoel(1, 2, 2), "f")
    );
    assert_eq!(
        p.stand.zaalmelding.zonder_stoel,
        ["Proef Fem", "Proef Gijs"]
    );
}

#[test]
fn een_stoel_van_een_ander_kamerlid_geeft_de_server_niet_weg() {
    let mut p = Proef::new();
    let fout = p.doe(Opdracht::GeefStoel {
        kamerlid: "a".into(),
        stoel: stoel(1, 1, 2),
    });
    assert!(matches!(fout, Err(Weigering::Zaal(_))), "{fout:?}");
    // naar een lege Stoel mag wel: de oude Stoel is daarna leeg
    p.doe(Opdracht::GeefStoel {
        kamerlid: "a".into(),
        stoel: stoel(1, 3, 1),
    })
    .unwrap();
    assert_eq!(p.stand.zaalindeling.stoel_van("a"), Some(stoel(1, 3, 1)));
    assert_eq!(p.stand.zaalindeling.op(stoel(1, 1, 1)), None);
}

// --- Zetelverdeling in de Zaal, in de Zaalindeling ---

#[test]
fn zetelverdeling_in_de_zaalindeling_kleurt_elke_stoel_naar_de_fractie_van_wie_er_zit() {
    let mut p = Proef::new();
    // Aal vertrekt: Eik heeft nog geen Stoel, dus de Stoel van Aal is grijs
    p.lees_kamer(vec![
        lid("e", "Eik", "rood", "z1"),
        lid("b", "Bes", "rood", "z2"),
        lid("c", "Cor", "blauw", "z3"),
        lid("d", "Dop", "blauw", "z4"),
    ]);
    let graphic = Graphic::Zetelverdeling {
        set: String::new(),
        namen: true,
        zetels: true,
        indeling: Indeling::Zaalindeling,
        cijfers: None,
    };
    p.doe(Opdracht::ZetKlaar {
        graphic,
        standpunt: None,
    })
    .unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();

    let rood: Kleur = "#d4202c".parse().unwrap();
    let blauw: Kleur = "#12357a".parse().unwrap();
    let kleur = |s: Stoel| p.stand.stoelkleuren[s.index().unwrap()];
    assert_eq!(p.stand.stoelkleuren.len(), 150);
    assert_eq!(kleur(stoel(1, 1, 1)), GRIJS);
    assert_eq!(kleur(stoel(1, 1, 2)), rood);
    assert_eq!(kleur(stoel(1, 2, 1)), blauw);
    assert_eq!(kleur(stoel(1, 2, 2)), blauw);
    assert_eq!(kleur(stoel(6, 6, 6)), GRIJS, "een Stoel zonder Kamerlid");

    // de labels tellen de Stoelen per Partij
    let cijfers = p.stand.ether.as_ref().unwrap().graphic.cijfers().unwrap();
    let set = &cijfers.sets[0];
    assert_eq!((set.zetels_van("rood"), set.zetels_van("blauw")), (1, 2));
}
