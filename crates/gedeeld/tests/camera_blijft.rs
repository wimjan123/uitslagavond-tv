//! Camera blijft staan ([`camera::BLIJF`]): een Graphic tonen of bijwerken zonder dat de camera
//! beweegt.

use gedeeld::camera::{self, Camerakeuze};
use gedeeld::{Graphic, Herkomst, Opdracht, Stand, Weigering};

fn doe(stand: &mut Stand, volgnummer: &mut u64, o: Opdracht) -> Result<(), Weigering> {
    *volgnummer += 1;
    let h = Herkomst { door: "Proef", volgnummer: *volgnummer, tijd_ms: 0 };
    stand.voer_uit(&o, &h)
}

fn titelkaart(kop: &str) -> Graphic {
    Graphic::Titelkaart { kop: kop.into(), regel: String::new() }
}

fn zet_klaar(kop: &str, standpunt: Option<&str>) -> Opdracht {
    Opdracht::ZetKlaar { graphic: titelkaart(kop), standpunt: standpunt.map(Into::into) }
}

/// Wat de camera van de Ether doet, zonder Oefenen of opname.
fn camera(stand: &Stand) -> (String, bool, u64) {
    let Camerakeuze { standpunt, vrij, sinds, .. } = &stand.camera;
    (standpunt.clone(), *vrij, *sinds)
}

#[test]
fn tonen_met_blijf_laat_de_camera_staan_ook_een_camerarit() {
    let mut stand = Stand::default();
    let mut v = 0;
    doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: "Q".into(), knip: false }).unwrap();
    let voor = camera(&stand);
    doe(&mut stand, &mut v, zet_klaar("Voorbeeldgegevens", Some(camera::BLIJF))).unwrap();
    doe(&mut stand, &mut v, Opdracht::Toon { versie: None }).unwrap();
    assert!(stand.ether.is_some());
    assert_eq!(camera(&stand), voor);
}

#[test]
fn tonen_met_blijf_laat_de_vrije_camera_aan() {
    let mut stand = Stand::default();
    let mut v = 0;
    doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true }).unwrap();
    let voor = camera(&stand);
    assert!(voor.1);
    doe(&mut stand, &mut v, zet_klaar("Voorbeeldgegevens", Some(camera::BLIJF))).unwrap();
    doe(&mut stand, &mut v, Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(camera(&stand), voor);
}

#[test]
fn bijwerken_naar_blijf_wisselt_de_camera_niet() {
    let mut stand = Stand::default();
    let mut v = 0;
    doe(&mut stand, &mut v, zet_klaar("Voorbeeldgegevens", Some("D"))).unwrap();
    doe(&mut stand, &mut v, Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(stand.camera.standpunt, "D");
    doe(&mut stand, &mut v, Opdracht::ZetVrijeCamera { aan: true }).unwrap();
    let voor = camera(&stand);
    let id = stand.klaar.as_ref().unwrap().id;
    let wijzig = Opdracht::WijzigKlaar {
        id,
        graphic: titelkaart("Andere voorbeeldgegevens"),
        standpunt: Some(camera::BLIJF.into()),
    };
    doe(&mut stand, &mut v, wijzig).unwrap();
    doe(&mut stand, &mut v, Opdracht::Bijwerken).unwrap();
    assert_eq!(stand.ether.as_ref().unwrap().standpunt(), camera::BLIJF);
    assert_eq!(camera(&stand), voor);
}

#[test]
fn een_onbekende_sleutel_blijft_geweigerd() {
    let mut stand = Stand::default();
    let mut v = 0;
    assert_eq!(
        doe(&mut stand, &mut v, zet_klaar("Voorbeeldgegevens", Some("blijven"))),
        Err(Weigering::OnbekendStandpunt("blijven".into()))
    );
    // blijven staan is een keuze voor een Graphic, geen plek om de camera heen te sturen
    assert_eq!(
        doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: camera::BLIJF.into(), knip: false }),
        Err(Weigering::OnbekendStandpunt(camera::BLIJF.into()))
    );
    assert_eq!(stand.camera.standpunt, "A");
}

#[test]
fn een_preset_en_een_sjabloon_mogen_blijf_kiezen() {
    let mut stand = Stand::default();
    let mut v = 0;
    let bewaar = Opdracht::BewaarPreset { graphic: titelkaart("Voorbeeldgegevens"), standpunt: Some(camera::BLIJF.into()) };
    doe(&mut stand, &mut v, bewaar).unwrap();
    let mut sjabloon = gedeeld::sjabloon::vast_sjabloon("titelkaart").unwrap();
    sjabloon.standpunt = Some(camera::BLIJF.into());
    doe(&mut stand, &mut v, Opdracht::ZetSjabloon { sjabloon }).unwrap();
    doe(&mut stand, &mut v, Opdracht::KiesStandpunt { letter: "G".into(), knip: false }).unwrap();
    let voor = camera(&stand);
    doe(&mut stand, &mut v, zet_klaar("Voorbeeldgegevens", None)).unwrap();
    assert_eq!(stand.klaar.as_ref().unwrap().standpunt(), camera::BLIJF);
    // de stand van een Standpunt-sleutel: die van de Ether, niet Standpunt A
    assert_eq!(stand.camerastand_van(camera::BLIJF), stand.camerastand_van("G"));
    doe(&mut stand, &mut v, Opdracht::Toon { versie: None }).unwrap();
    assert_eq!(camera(&stand), voor);
}
