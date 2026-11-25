//! Presets: een Graphic bewaren met de gegevens van dat moment, met een vast nummer klaarzetten,
//! en de stempels Klaar, Live en "was live om". Alles via `Stand::voer_uit`, zoals de server het
//! doet. De Kamer, de Partijen en de set hier zijn voorbeeldgegevens.

use gedeeld::cijfers::Invoer;
use gedeeld::kamer::Fractie;
use gedeeld::stemmingen::{GetikteFractie, Lijstpunt, Stem, Stemgedrag};
use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Weigering};

struct Proef {
    stand: Stand,
    volgnummer: u64,
}

impl Proef {
    fn nieuw() -> Proef {
        Proef { stand: Stand::default(), volgnummer: 0 }
    }

    fn doe(&mut self, o: Opdracht) -> Result<(), Weigering> {
        self.volgnummer += 1;
        let h = Herkomst { door: "Regie links", volgnummer: self.volgnummer, tijd_ms: self.volgnummer * 60_000 };
        self.stand.voer_uit(&o, &h)
    }

    /// Voorbeeldgegevens: een partijlijst met kleuren en een set "Exitpoll".
    fn lees_invoer(&mut self, kleur_alfa: &str, zetels_alfa: u16) {
        let json = format!(
            r##"{{ "partijen": [
                {{ "sleutel": "alfa", "naam": "Alfa", "kort": "ALFA", "kleur": "{kleur_alfa}" }},
                {{ "sleutel": "beta", "naam": "Beta", "kort": "BETA", "kleur": "#d33b2c" }} ],
              "sets": [ {{ "id": "exitpoll", "soort": "peiling", "label": "Exitpoll",
                "zetels": {{ "alfa": {zetels_alfa}, "beta": {} }} }} ] }}"##,
            150 - zetels_alfa
        );
        self.doe(Opdracht::LeesInvoer { bestand: "cijfers.json".into(), ontvangen: "21.00".into(), invoer: Invoer::uit_json(&json) })
            .unwrap();
    }

    /// Voorbeeldgegevens: twee Fracties die samen 150 Zetels hebben.
    fn lees_kamer(&mut self, zetels_alfa: u16) {
        let fractie = |id: &str, afk: &str, zetels| Fractie { id: id.into(), afkorting: afk.into(), naam: afk.into(), zetels };
        self.doe(Opdracht::LeesKamer {
            opgehaald: "2026-11-24T08:00:00+01:00".into(),
            peildatum: "2026-11-24".into(),
            kamerleden: vec![],
            fracties: vec![fractie("f-alfa", "Alfa", zetels_alfa), fractie("f-beta", "Beta", 150 - zetels_alfa)],
        })
        .unwrap();
    }

    /// Alfa stemt `alfa`, de rest het andere; daarna Vastleggen en klaarzetten.
    fn stem_en_leg_vast(&mut self, alfa: Stem) {
        self.doe(Opdracht::ZetStemgedrag {
            stemming: "m1".into(),
            stemgedrag: Stemgedrag {
                getikt: vec![GetikteFractie { fractie: "f-alfa".into(), stem: alfa }],
                overige: Some(alfa.andere()),
                ..Default::default()
            },
        })
        .unwrap();
        self.doe(Opdracht::LegStemmingVast { stemming: "m1".into() }).unwrap();
    }

    fn klaar(&self) -> Graphic {
        self.stand.klaar.as_ref().expect("er staat een Graphic klaar").graphic.clone()
    }

    fn bewaar_klaar(&mut self) {
        let klaar = self.stand.klaar.clone().expect("er staat een Graphic klaar");
        self.doe(Opdracht::BewaarPreset { graphic: klaar.graphic, standpunt: klaar.standpunt }).unwrap();
    }
}

fn titelkaart(kop: &str) -> Graphic {
    Graphic::Titelkaart { kop: kop.into(), regel: String::new() }
}

#[test]
fn een_preset_houdt_zijn_oude_gegevens_na_een_wijziging_in_de_set() {
    let mut p = Proef::nieuw();
    p.lees_invoer("#1f6fb2", 80);
    p.lees_kamer(80);
    p.doe(Opdracht::LeesStemmingslijst {
        dag: "2026-11-24".into(),
        ingelezen: "2026-11-24T08:00:00+01:00".into(),
        punten: vec![Lijstpunt {
            id: "m1".into(),
            soort: Some("Motie".into()),
            nummer: None,
            titel: "Motie van het lid Een over de eerste zaak".into(),
            indieners: None,
            agendapunt: None,
        }],
    })
    .unwrap();
    p.stem_en_leg_vast(Stem::Voor);
    let bewaard = p.klaar();
    let Graphic::UitslagStemming { uitslag } = &bewaard else { panic!("verwacht een Uitslag, kreeg {bewaard:?}") };
    assert_eq!((uitslag.voor, uitslag.tegen), (80, 70));
    assert_eq!(uitslag.fracties[0].kleur.as_deref(), Some("#1F6FB2"), "kleuren komen in hoofdletters");
    p.bewaar_klaar();

    // Daarna verandert alles waar de Graphic uit kwam: de partijlijst en de set, de Zetels in
    // de Kamer en het stemgedrag. De nieuwe Graphic laat dat zien, de Preset niet.
    p.lees_invoer("#00a000", 60);
    p.doe(Opdracht::ZetZetels { set: "exitpoll".into(), partij: "alfa".into(), zetels: 61 }).unwrap();
    p.lees_kamer(60);
    p.stem_en_leg_vast(Stem::Tegen);
    let nieuw = p.klaar();
    let Graphic::UitslagStemming { uitslag } = &nieuw else { panic!() };
    assert_eq!((uitslag.voor, uitslag.tegen), (90, 60));
    let alfa = uitslag.fracties.iter().find(|f| f.afkorting == "Alfa").unwrap();
    assert_eq!(alfa.kleur.as_deref(), Some("#00A000"));

    let preset = &p.stand.presets[0];
    assert_eq!(preset.nummer, 1);
    assert_eq!(preset.graphic, bewaard, "de Preset werkt zichzelf niet bij");

    p.doe(Opdracht::ZetPresetKlaar { nummer: 1 }).unwrap();
    assert_eq!(p.klaar(), bewaard, "klaarzetten geeft de gegevens van het bewaren");
}

#[test]
fn een_nieuwe_preset_krijgt_het_eerste_vrije_nummer_en_houdt_het() {
    let mut p = Proef::nieuw();
    for kop in ["Een", "Twee", "Drie"] {
        p.doe(Opdracht::BewaarPreset { graphic: titelkaart(kop), standpunt: None }).unwrap();
    }
    p.doe(Opdracht::WisPreset { nummer: 2 }).unwrap();
    let nummers: Vec<u16> = p.stand.presets.iter().map(|x| x.nummer).collect();
    assert_eq!(nummers, [1, 3], "Drie houdt nummer 3");

    p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Vier"), standpunt: None }).unwrap();
    let nummers: Vec<(u16, Graphic)> = p.stand.presets.iter().map(|x| (x.nummer, x.graphic.clone())).collect();
    assert_eq!(nummers, [(1, titelkaart("Een")), (2, titelkaart("Vier")), (3, titelkaart("Drie"))]);

    assert_eq!(p.doe(Opdracht::WisPreset { nummer: 7 }), Err(Weigering::OnbekendePreset(7)));
    assert_eq!(p.doe(Opdracht::ZetPresetKlaar { nummer: 7 }), Err(Weigering::OnbekendePreset(7)));
}

#[test]
fn een_preset_bewaart_zijn_standpunt_en_weigert_er_een_dat_niet_bestaat() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Flank"), standpunt: Some("D".into()) }).unwrap();
    p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Standaard"), standpunt: None }).unwrap();
    assert_eq!(
        p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Nergens"), standpunt: Some("P".into()) }),
        Err(Weigering::OnbekendStandpunt("P".into()))
    );
    assert_eq!(p.doe(Opdracht::BewaarPreset { graphic: titelkaart(""), standpunt: None }), Err(Weigering::KopLeeg));

    p.doe(Opdracht::ZetPresetKlaar { nummer: 1 }).unwrap();
    assert_eq!(p.stand.klaar.as_ref().unwrap().standpunt(), "D");

    // Zo ziet het wissen van een Standpunt welke Presets het gebruiken. Zonder eigen keuze is
    // dat het Standpunt van het Sjabloon (A).
    let met = |letter: &str| p.stand.presets_met_standpunt(letter).map(|x| x.nummer).collect::<Vec<_>>();
    assert_eq!(met("D"), [1]);
    assert_eq!(met("A"), [2]);
    assert!(met("S").is_empty());
}

#[test]
fn klaar_live_en_was_live_volgen_de_graphic_uit_de_preset() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Een"), standpunt: None }).unwrap();
    p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Twee"), standpunt: None }).unwrap();
    let preset = |p: &Proef, n: u16| p.stand.presets.iter().find(|x| x.nummer == n).unwrap().clone();

    p.doe(Opdracht::ZetPresetKlaar { nummer: 1 }).unwrap();
    assert!(p.stand.preset_staat_klaar(&preset(&p, 1)));
    assert!(!p.stand.preset_staat_klaar(&preset(&p, 2)));
    assert!(!p.stand.preset_in_beeld(&preset(&p, 1)));

    p.doe(Opdracht::Toon { versie: None }).unwrap();
    let getoond_om = p.volgnummer * 60_000;
    assert!(p.stand.preset_in_beeld(&preset(&p, 1)));
    assert_eq!(preset(&p, 1).laatst_live_ms, Some(getoond_om));

    // Een andere Preset klaarzetten en tonen: de eerste was live, de tweede is het nu.
    p.doe(Opdracht::ZetPresetKlaar { nummer: 2 }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert!(!p.stand.preset_in_beeld(&preset(&p, 1)));
    assert_eq!(preset(&p, 1).laatst_live_ms, Some(getoond_om));
    assert!(p.stand.preset_in_beeld(&preset(&p, 2)));

    // Dezelfde Preset opnieuw klaarzetten is een nieuwe Graphic: wat in beeld staat blijft van
    // de oude keer, de nieuwe staat klaar.
    p.doe(Opdracht::ZetPresetKlaar { nummer: 2 }).unwrap();
    assert!(p.stand.preset_in_beeld(&preset(&p, 2)));
    assert!(p.stand.preset_staat_klaar(&preset(&p, 2)));
}

#[test]
fn een_gewijzigde_graphic_is_de_preset_niet_meer() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Een"), standpunt: None }).unwrap();
    let preset = |p: &Proef| p.stand.presets[0].clone();
    p.doe(Opdracht::ZetPresetKlaar { nummer: 1 }).unwrap();
    p.doe(Opdracht::Toon { versie: None }).unwrap();
    assert!(p.stand.preset_in_beeld(&preset(&p)));

    let id = p.stand.klaar.as_ref().unwrap().id;
    p.doe(Opdracht::WijzigKlaar { id, graphic: titelkaart("Een, gewijzigd"), standpunt: None }).unwrap();
    assert!(!p.stand.preset_staat_klaar(&preset(&p)), "wat klaarstaat is niet meer de Preset");
    assert!(p.stand.preset_in_beeld(&preset(&p)), "de Ether is nog niet bijgewerkt");
    p.doe(Opdracht::Bijwerken).unwrap();
    assert!(!p.stand.preset_in_beeld(&preset(&p)), "na Bijwerken staat de Preset niet meer in beeld");
    assert_eq!(preset(&p).graphic, titelkaart("Een"), "de Preset zelf blijft gelijk");
}

#[test]
fn op_de_tablet_zetten_en_eraf_halen() {
    let mut p = Proef::nieuw();
    p.doe(Opdracht::BewaarPreset { graphic: titelkaart("Een"), standpunt: None }).unwrap();
    assert!(!p.stand.presets[0].op_tablet, "een nieuwe Preset staat niet op de tablet");
    p.doe(Opdracht::ZetPresetOpTablet { nummer: 1, aan: true }).unwrap();
    assert!(p.stand.presets[0].op_tablet);
    p.doe(Opdracht::ZetPresetOpTablet { nummer: 1, aan: false }).unwrap();
    assert!(!p.stand.presets[0].op_tablet);
    assert_eq!(p.doe(Opdracht::ZetPresetOpTablet { nummer: 4, aan: true }), Err(Weigering::OnbekendePreset(4)));
}

#[test]
fn een_oude_stand_zonder_presets_laadt_nog() {
    let s: Stand = serde_json::from_str("{}").unwrap();
    assert!(s.presets.is_empty());
}
