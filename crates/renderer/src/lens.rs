//! Scherptediepte bij een close-up in de Zaal, zoals een echte camera met een lange lens: de
//! Stoel waar het om gaat scherp, de bureaus ervoor en erachter zacht onscherp. Zonder dat oogden
//! Partij uitgelicht en de Persoonskaart als een spel, met overal even scherpe naden in de
//! bureaubladen en harde randen (council-ronde 3). Bij een wijd shot (Standpunt A, Bovenaf) blijft
//! alles scherp; tijdens een rit komt de onscherpte vloeiend op.
//!
//! Scherp gesteld wordt op de uitgelichte Stoel (Persoonskaart) als die in beeld is, en anders op
//! de Stoelen in het midden van het beeld.

use bevy::post_process::dof::{DepthOfField, DepthOfFieldMode};
use bevy::prelude::*;

use crate::beeld::HoofdCamera;

pub struct LensPlugin;

impl Plugin for LensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Nabijheid>()
            .add_systems(PostUpdate, stel_scherpte_bij.before(TransformSystems::Propagate));
    }
}

/// Op de camera zolang de Zaal achter een Sjabloon met de achtergrond Zaal donker onscherp is
/// (`graphic.rs`). Die onscherpte gaat voor: de lens laat de scherptediepte dan staan. Zonder
/// dit haalde de lens hem bij een wijd shot elk beeld weer weg, zodat de Zaal achter de
/// Stemmingen en de Uitslag per Partij scherp bleef (gevonden in council-ronde 5).
#[derive(Component)]
pub struct ZaalOnscherp;

/// Hoe dichtbij de camera is (0 tot 1): 0 bij een wijd shot, 1 bij een close-up in de Zaal. Ook
/// de kleur van het leer hangt ervan af (zie `zaal.rs`): de kalibratie geldt voor het
/// totaalbeeld, en van dichtbij staat een Stoel recht in het licht.
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub struct Nabijheid(pub f32);

/// Vanaf deze beeldhoek (graden) begint de onscherpte, en bij deze is hij er helemaal: Aanschuiven
/// (Partij uitgelicht) heeft 40, de Persoonskaart 34, Standpunt A 48.
const VANAF_GRADEN: f32 = 46.0;
const HELEMAAL_GRADEN: f32 = 40.0;
/// Het diafragma bij een volle close-up: een groot diafragma geeft weinig scherptediepte. Met de
/// sensor van Bevy (Super 35) en de lens van Partij uitgelicht zijn de rijen voor en achter het
/// blok merkbaar zacht; met 0,7 zag je in ronde 4 bijna geen scherptediepte.
const DIAFRAGMA: f32 = 0.25;
/// Bij een volle close-up is de occlusie sterker en dichter bij: contactschaduw onder bureaus en
/// Stoelen, die in een wijd shot te zwaar zou zijn.
const OCCLUSIE_EXTRA: f32 = 0.45;
const OCCLUSIE_STRAAL_DICHTBIJ: f32 = 0.35;
/// Nooit onscherper dan dit, in pixels.
const MEESTE_ONSCHERPTE: f32 = 20.0;
/// Bij de Persoonskaart is de Zaal achtergrond: minder scherptediepte en meer onscherpte, zodat
/// de Stoel en de bureaus niet eerder het oog trekken dan het gezicht op de kaart (council-ronde 5).
const DIAFRAGMA_PERSOON: f32 = 0.14;
const MEESTE_ONSCHERPTE_PERSOON: f32 = 30.0;
/// Bij de Persoonskaart wordt wat voor de Stoel staat (het bureau vooraan) zoveel donkerder: het
/// trok met de lege Stoel eerder het oog dan het portret (council-ronde 6).
const VOORGROND_PERSOON: f32 = 0.85;
/// Alleen in de Zaal: onder het plafond.
const ONDER_Y: f32 = 5.0;

/// Hoe sterk de onscherpte is (0 tot 1) bij deze beeldhoek (radialen) en hoogte van de camera.
pub fn sterkte(beeldhoek: f32, hoogte: f32) -> f32 {
    if hoogte >= ONDER_Y {
        return 0.0;
    }
    let t = ((VANAF_GRADEN - beeldhoek.to_degrees()) / (VANAF_GRADEN - HELEMAAL_GRADEN)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Waar de camera op scherp stelt, in meter: de uitgelichte Stoel als die in beeld is, anders de
/// mediaan van de Stoelen in het midden van het beeld. `None` zonder Stoel in beeld.
pub fn scherp_op(camera: &Transform, beeldhoek: f32, uitgelicht: Option<usize>) -> Option<f32> {
    let naar_camera = camera.to_matrix().inverse();
    let th = (beeldhoek / 2.0).tan();
    let tw = th * crate::BREEDTE as f32 / crate::HOOGTE as f32;
    // het leer van de rugleuning, iets onder de bovenrand
    let diepte = |i: usize| {
        let p = gedeeld::zaalkaart::STOELPLEKKEN[i];
        let v = naar_camera.transform_point3(Vec3::new(p[0], p[1] - 0.2, p[2]));
        let d = -v.z;
        (d > 0.3).then(|| (d, v.x / d / tw, v.y / d / th))
    };
    if let Some((d, x, y)) = uitgelicht.and_then(diepte)
        && x.abs() < 1.0
        && y.abs() < 1.0
    {
        return Some(d);
    }
    let mut midden: Vec<f32> = (0..gedeeld::AANTAL_STOELEN)
        .filter_map(diepte)
        .filter(|&(_, x, y)| x.abs() < 0.5 && y.abs() < 0.6)
        .map(|(d, _, _)| d)
        .collect();
    midden.sort_by(f32::total_cmp);
    midden.get(midden.len() / 2).copied()
}

fn stel_scherpte_bij(
    mut commands: Commands,
    mut camera: Query<
        (Entity, &Transform, &Projection, Option<&DepthOfField>, Option<&mut crate::occlusie::Occlusie>, Has<ZaalOnscherp>),
        With<HoofdCamera>,
    >,
    uitlichting: Res<crate::zaal::Uitlichting>,
    mut nabijheid: ResMut<Nabijheid>,
) {
    let b = &crate::beeld::BEELD;
    for (e, plek, projectie, nu, mut occlusie, zaal_onscherp) in &mut camera {
        let Projection::Perspective(p) = projectie else { continue };
        let s = sterkte(p.fov, plek.translation.y);
        nabijheid.set_if_neq(Nabijheid(s));
        if let Some(o) = occlusie.as_mut() {
            let (sterk, straal) = (b.occlusie + OCCLUSIE_EXTRA * s, b.occlusie_straal + (OCCLUSIE_STRAAL_DICHTBIJ - b.occlusie_straal) * s);
            if o.sterkte != sterk || o.straal != straal {
                (o.sterkte, o.straal) = (sterk, straal);
            }
        }
        if zaal_onscherp {
            continue;
        }
        let afstand = (s > 0.0).then(|| scherp_op(plek, p.fov, uitlichting.stoel)).flatten();
        let persoon = uitlichting.sterkte();
        if let Some(o) = occlusie.as_mut() {
            let (voor, donker) = (afstand.unwrap_or(0.0), VOORGROND_PERSOON * persoon * s);
            if o.voorgrond != voor || o.voorgrond_donker != donker {
                (o.voorgrond, o.voorgrond_donker) = (voor, donker);
            }
        }
        match afstand {
            Some(afstand) => {
                let dof = DepthOfField {
                    mode: DepthOfFieldMode::Gaussian,
                    focal_distance: afstand,
                    // zwakker is een kleiner diafragma (een groter getal)
                    aperture_f_stops: (DIAFRAGMA + (DIAFRAGMA_PERSOON - DIAFRAGMA) * persoon) / s,
                    max_circle_of_confusion_diameter: MEESTE_ONSCHERPTE + (MEESTE_ONSCHERPTE_PERSOON - MEESTE_ONSCHERPTE) * persoon,
                    max_depth: 60.0,
                    ..default()
                };
                // niet elk beeld opnieuw voor een paar millimeter
                let anders = |n: &DepthOfField| {
                    (n.focal_distance - dof.focal_distance).abs() > 0.01 * dof.focal_distance
                        || (n.aperture_f_stops - dof.aperture_f_stops).abs() > 0.01 * dof.aperture_f_stops
                };
                if nu.is_none_or(anders) {
                    commands.entity(e).insert(dof);
                }
            }
            None if nu.is_some() => {
                commands.entity(e).remove::<DepthOfField>();
            }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alleen_een_close_up_in_de_zaal_is_onscherp() {
        let a = gedeeld::camera::Standpunt::vaste()[0].stand;
        assert_eq!(sterkte(a.beeldhoek, a.positie[1]), 0.0, "Standpunt A blijft scherp");
        let stoel = gedeeld::Stoel { vak: 3, rij: 2, nummer: 1 };
        let shot = gedeeld::camera::shot_op_stoel(stoel).unwrap();
        assert_eq!(sterkte(shot.beeldhoek, shot.positie[1]), 1.0, "de Persoonskaart helemaal");
        let half = sterkte(43.0_f32.to_radians(), 2.0);
        assert!(half > 0.3 && half < 0.7, "tijdens de rit komt hij vloeiend op: {half}");
    }

    #[test]
    fn de_onscherpte_achter_zaal_donker_blijft_staan_bij_een_wijd_shot() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(LensPlugin).init_resource::<crate::zaal::Uitlichting>();
        let a = gedeeld::camera::Standpunt::vaste()[0].stand;
        let projectie = Projection::Perspective(PerspectiveProjection { fov: a.beeldhoek, ..default() });
        let dof = DepthOfField { focal_distance: 0.2, ..default() };
        let donker = app
            .world_mut()
            .spawn((HoofdCamera, crate::camera::transform(&a), projectie.clone(), dof.clone(), ZaalOnscherp))
            .id();
        let gewoon = app.world_mut().spawn((HoofdCamera, crate::camera::transform(&a), projectie, dof)).id();
        app.update();
        app.update();
        assert!(app.world().get::<DepthOfField>(donker).is_some(), "achter Zaal donker blijft de Zaal onscherp");
        assert!(app.world().get::<DepthOfField>(gewoon).is_none(), "zonder: een wijd shot is scherp");
    }

    #[test]
    fn de_persoonskaart_stelt_scherp_op_de_uitgelichte_stoel() {
        let stoel = gedeeld::Stoel { vak: 3, rij: 2, nummer: 1 };
        let shot = gedeeld::camera::shot_op_stoel(stoel).unwrap();
        let camera = crate::camera::transform(&shot);
        let afstand = scherp_op(&camera, shot.beeldhoek, stoel.index()).unwrap();
        // de camera staat 3,2 m langs de vloer van de Stoel en 0,95 m hoger
        assert!((afstand - 3.3).abs() < 0.3, "{afstand}");
        // zonder uitgelichte Stoel: op de Stoelen in het midden, verder weg
        let midden = scherp_op(&camera, shot.beeldhoek, None).unwrap();
        assert!(midden > 2.0 && midden < 12.0, "{midden}");
    }
}
