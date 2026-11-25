//! Het donkere verloop van de studio: boven en onder in beeld, zolang de camera buiten de Zaal
//! staat, zoals bij Totaal vanaf voorzitter (Standpunt A, zie `schil.rs`). Bovenin valt daar
//! het plafond weg: zonder verloop zag je een vlakke donkere strook met de zaagtand van de
//! afgesneden wanden, en een lichte tribune zonder informatie. Onderin staan de regeringsbanken
//! schuin afgesneden in beeld. Het verloop maakt beide rustig donker, achter de Graphics, en laat
//! de band met de Stoelen ertussen vrij (council-ronde 3).
//!
//! Het is een vlak vlak voor de lens, met per rij een doorzichtigheid. Het komt op naarmate de
//! camera hoger dan het plafond staat, dus tijdens een rit de Zaal in verdwijnt het vloeiend.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::beeld::HoofdCamera;

pub struct StudioPlugin;

impl Plugin for StudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (zet_verloop, stel_verloop_bij).chain());
    }
}

/// De kleur van het verloop: die van de lege ruimte achter de Zaal, zodat het weggevallen
/// plafond erin opgaat.
const VERLOOP_KLEUR: Srgba = Srgba::rgb(0x0b as f32 / 255.0, 0x0f as f32 / 255.0, 0x1a as f32 / 255.0);

/// Het verloop bovenin, van de bovenrand van het beeld (0) tot net boven de achterste rij
/// Stoelen (1): hoe dicht het daar is. Dicht tot boven de tribune, en weg waar de Stoelen beginnen.
const BOVEN: [(f32, f32); 7] = [(0.0, 0.9), (0.24, 0.85), (0.46, 0.72), (0.65, 0.5), (0.81, 0.24), (0.93, 0.07), (1.0, 0.0)];
/// Het verloop onderin, van net onder de voorste Stoelen (0) tot de onderrand van het beeld (1).
const ONDER: [(f32, f32); 5] = [(0.0, 0.0), (0.27, 0.16), (0.57, 0.36), (0.85, 0.5), (1.0, 0.55)];
/// Zoveel ruimte (van -1 tot 1 over de hoogte) blijft tussen het verloop en de Stoelen.
const MARGE: f32 = 0.01;
/// Een rugleuning met de Stoel eronder: van de bovenrand tot zoveel meter lager.
const STOEL_HOOGTE: f32 = 0.8;

/// Zo ver voor de lens staat het vlak, in meter: net achter het near-vlak van de camera (0,05).
const AFSTAND: f32 = 0.08;
/// Het vlak is ruimer dan het beeld, zodat een lens die in hetzelfde beeld verandert geen rand
/// laat zien.
const RUIMER: f32 = 1.25;

/// Vanaf deze hoogte van de camera (meter) begint het verloop, en op deze hoogte is het er
/// helemaal: het plafond hangt op 5,15 m, Standpunt A staat op 10 m.
const VANAF_Y: f32 = 5.5;
const HELEMAAL_Y: f32 = 8.0;

/// Het vlak met het verloop, als kind van de hoofdcamera, met waar de Stoelen stonden toen het
/// gemaakt werd.
#[derive(Component)]
struct Verloop {
    stoelen: (f32, f32),
}

/// Hoe sterk het verloop is (0 tot 1) bij een camera op deze hoogte.
pub fn sterkte(hoogte: f32) -> f32 {
    let t = ((hoogte - VANAF_Y) / (HELEMAAL_Y - VANAF_Y)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Waar de Stoelen in beeld staan, van -1 (onderrand) tot 1 (bovenrand): de bovenkant van de
/// hoogste rugleuning en de onderkant van de laagste Stoel. `None` als er een Stoel achter de
/// camera staat.
pub fn stoelen_in_beeld(camera: &Transform, beeldhoek: f32) -> Option<(f32, f32)> {
    let naar_camera = camera.to_matrix().inverse();
    let th = (beeldhoek / 2.0).tan();
    let (mut boven, mut onder) = (f32::MIN, f32::MAX);
    for p in gedeeld::zaalkaart::STOELPLEKKEN {
        for y in [p[1], p[1] - STOEL_HOOGTE] {
            let v = naar_camera.transform_point3(Vec3::new(p[0], y, p[2]));
            if v.z >= -0.01 {
                return None;
            }
            let h = v.y / -v.z / th;
            (boven, onder) = (boven.max(h), onder.min(h));
        }
    }
    Some((boven, onder))
}

/// De rijen van het vlak van boven naar onder: hoogte (-1 tot 1) en dichtheid, met de Stoelen
/// van `boven` tot `onder`. Boven en onder loopt het door tot [`RUIMER`].
pub fn rijen(boven: f32, onder: f32) -> Vec<(f32, f32)> {
    let (boven, onder) = ((boven + MARGE).min(1.0), (onder - MARGE).max(-1.0));
    let mut r = vec![(RUIMER, BOVEN[0].1)];
    r.extend(BOVEN.iter().map(|&(t, a)| (1.0 + (boven - 1.0) * t, a)));
    r.extend(ONDER.iter().map(|&(t, a)| (onder + (-1.0 - onder) * t, a)));
    r.push((-RUIMER, ONDER[ONDER.len() - 1].1));
    r
}

/// Het vlak: x en y van -1 tot 1 over het beeld (y omhoog), met de dichtheid per rij in de alfa
/// van de vertexkleur.
fn vlak(rijen: &[(f32, f32)]) -> Mesh {
    let (mut punten, mut kleuren, mut driehoeken) = (Vec::new(), Vec::new(), Vec::new());
    for (i, &(y, a)) in rijen.iter().enumerate() {
        for x in [-RUIMER, RUIMER] {
            punten.push([x, y, 0.0]);
            kleuren.push([1.0, 1.0, 1.0, a]);
        }
        if i > 0 {
            let r = 2 * i as u32;
            driehoeken.extend([r - 2, r, r + 1, r - 2, r + 1, r - 1]);
        }
    }
    let normalen = vec![[0.0, 0.0, 1.0]; punten.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, punten)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normalen)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, kleuren)
        .with_inserted_indices(Indices::U32(driehoeken))
}

fn zet_verloop(
    mut commands: Commands,
    camera: Query<Entity, Added<HoofdCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
) {
    for c in &camera {
        commands.spawn((
            Verloop { stoelen: (1.0, -1.0) },
            ChildOf(c),
            Mesh3d(meshes.add(vlak(&rijen(1.0, -1.0)))),
            MeshMaterial3d(materialen.add(StandardMaterial {
                base_color: VERLOOP_KLEUR.with_alpha(0.0).into(),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                fog_enabled: false,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, -AFSTAND),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
    }
}

/// Past het vlak aan de lens en de plek van de Stoelen aan, en zet de sterkte bij de hoogte van
/// de camera.
fn stel_verloop_bij(
    camera: Query<(&Transform, &Projection), With<HoofdCamera>>,
    mut verloop: Query<
        (&ChildOf, &mut Verloop, &Mesh3d, &mut Transform, &mut Visibility, &MeshMaterial3d<StandardMaterial>),
        Without<HoofdCamera>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
) {
    for (ouder, mut v, mesh, mut t, mut zicht, materiaal) in &mut verloop {
        let Ok((plek, Projection::Perspective(p))) = camera.get(ouder.parent()) else { continue };
        let half = (p.fov / 2.0).tan() * AFSTAND;
        let schaal = Vec3::new(half * p.aspect_ratio, half, 1.0);
        if t.scale != schaal {
            t.scale = schaal;
        }
        let s = sterkte(plek.translation.y);
        if s > 0.0
            && let Some(stoelen) = stoelen_in_beeld(plek, p.fov)
            && ((stoelen.0 - v.stoelen.0).abs() > 1e-3 || (stoelen.1 - v.stoelen.1).abs() > 1e-3)
            && let Some(mut m) = meshes.get_mut(&mesh.0)
        {
            *m = vlak(&rijen(stoelen.0, stoelen.1));
            v.stoelen = stoelen;
        }
        zicht.set_if_neq(if s > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
        let kleur: Color = VERLOOP_KLEUR.with_alpha(s).into();
        if materialen.get(&materiaal.0).is_some_and(|m| m.base_color != kleur)
            && let Some(mut m) = materialen.get_mut(&materiaal.0)
        {
            m.base_color = kleur;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn het_verloop_is_er_alleen_boven_het_plafond_en_komt_vloeiend_op() {
        assert_eq!(sterkte(1.7), 0.0, "in de Zaal geen verloop");
        assert_eq!(sterkte(5.0), 0.0);
        assert_eq!(sterkte(10.0), 1.0, "bij Standpunt A helemaal");
        let half = sterkte((VANAF_Y + HELEMAAL_Y) / 2.0);
        assert!(half > 0.4 && half < 0.6);
    }

    #[test]
    fn het_verloop_stopt_boven_de_achterste_rij_en_begint_onder_de_voorste_stoelen() {
        for stand in [gedeeld::camera::Standpunt::vaste()[0].stand, gedeeld::camera::boven_het_paneel()] {
            let camera = crate::camera::transform(&stand);
            let (boven, onder) = stoelen_in_beeld(&camera, stand.beeldhoek).expect("alle Stoelen voor de camera");
            assert!(boven < 1.0 && onder > -1.0, "alle Stoelen in beeld: {boven} {onder}");
            let r = rijen(boven, onder);
            // tussen boven en onder is het verloop overal weg
            for &(y, a) in &r {
                if y < boven && y > onder {
                    assert_eq!(a, 0.0, "rij op {y} tussen de Stoelen ({boven} tot {onder})");
                }
            }
            assert!(r.first().unwrap().1 > 0.8 && r.last().unwrap().1 > 0.5, "dicht aan de randen");
            assert!(r.windows(2).all(|w| w[0].0 >= w[1].0), "van boven naar onder");
        }
    }
}
