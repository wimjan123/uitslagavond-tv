//! De schil van de Zaal: het plafond met alles erboven (daklicht, hanglampen, spots) en de
//! voorwand boven de Voorzitter. Een camera die buiten de Zaal staat, zoals Totaal vanaf
//! voorzitter (Standpunt A, hoog achter de voorwand), ziet de Zaal zonder schil: open van boven,
//! als in een virtuele studio. Een camera in de Zaal ziet hem gewoon.
//!
//! Elk deel van het vaste model (één per materiaal) krijgt bij het laden twee meshes: heel, en
//! zonder schil. Staan alle camera's aan dezelfde kant, dan tekent elk deel één van de twee: zo
//! zijn er niet meer delen met gebakken licht dan zonder schil, want elk zo'n deel kost tijd per
//! beeld. Alleen als twee camera's in één Renderer het oneens zijn (het Voorbeeld en zijn kleine
//! Ether), komt de schil als los deel op een eigen [`RenderLayers`]-laag, die alleen een camera in
//! de Zaal ziet.

use bevy::camera::visibility::RenderLayers;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::pbr::Lightmap;
use bevy::prelude::*;

/// De laag van de schil.
const SCHIL_LAAG: usize = 1;
/// De laag voor Stoelen van de Preview-camera (`Rol::Voorbeeld`, ADR 0005).
pub const STOEL_KLAAR_LAAG: usize = 2;
/// De laag voor Stoelen van de Ether-camera (Live).
pub const STOEL_ETHER_LAAG: usize = 3;

/// Boven deze hoogte (meter) hoort alles bij de schil: het plafond hangt op 5,15 tot 5,2 meter,
/// de hanglampen, de spots en het daklicht daarboven.
pub const PLAFOND_Y: f32 = 5.0;
/// Vanaf deze z hoort alles bij de schil: de nis achter de Voorzitter met de zuil, het lage
/// plafond (4,14 m) met de letterband en de voorwand (z 6,5). De vliegruimte van de server houdt
/// de camera in de Zaal voor deze lijn (z 4,5).
pub const VOOR_Z: f32 = 4.55;

/// Of een punt (in de assen van Bevy) in de schil ligt.
pub fn in_de_schil(p: Vec3) -> bool {
    p.y >= PLAFOND_Y || p.z >= VOOR_Z
}

/// Of een camera op deze plek buiten de Zaal staat, in de schil of erachter: dan ziet hij de
/// schil niet.
pub fn buiten(p: Vec3) -> bool {
    in_de_schil(p)
}

pub struct SchilPlugin;

impl Plugin for SchilPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Kant>()
            .add_systems(Update, (splits_af, kies_kant).chain());
    }
}

/// Een deel van het vaste model (`Zaal_Vast`) dat nog in schil en binnenkant gesplitst moet
/// worden. `zaal.rs` zet het erop als het model in de wereld komt.
#[derive(Component)]
pub struct Splitsen;

/// Een los voorwerp dat helemaal in de schil hangt (een hanglamp of spot, zie
/// [`in_de_schil_volgens_naam`]): het staat altijd op de laag van de schil.
#[derive(Component)]
pub struct Schil;

/// De losse voorwerpen uit het model die helemaal in de schil hangen.
pub fn in_de_schil_volgens_naam(naam: &str) -> bool {
    ["Hanglamp_", "Spot", "TribuneSpot"]
        .iter()
        .any(|b| naam.starts_with(b))
}

/// Zet een los voorwerp van de schil op de laag van de schil.
pub fn naar_de_schil() -> impl Bundle {
    (Schil, RenderLayers::layer(SCHIL_LAAG))
}

/// Een camera die de Zaal tekent en per beeld kiest of hij de schil ziet.
#[derive(Component)]
pub struct ZietSchil;

/// Een gesplitst deel van het vaste model: de mesh heel, die zonder schil (geen als het deel
/// helemaal schil is), en het losse deel met alleen de schil voor als de camera's het oneens zijn.
#[derive(Component)]
struct Gesplitst {
    heel: Handle<Mesh>,
    rest: Option<Handle<Mesh>>,
    schil: Entity,
}

/// Waar de camera's van deze Renderer staan.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
enum Kant {
    /// Allemaal in de Zaal (of nog geen camera): elk deel heel.
    #[default]
    Binnen,
    /// Allemaal buiten: elk deel zonder schil.
    Buiten,
    /// Sommige binnen, sommige buiten: elk deel zonder schil, en de schil los op zijn laag.
    Gemengd,
}

/// De kant bij de plekken van de camera's: binnen (`false`) of buiten (`true`).
fn kant(buiten: impl IntoIterator<Item = bool>) -> Kant {
    let (mut b, mut u) = (false, false);
    for x in buiten {
        if x { u = true } else { b = true }
    }
    match (b, u) {
        (_, false) => Kant::Binnen,
        (false, true) => Kant::Buiten,
        (true, true) => Kant::Gemengd,
    }
}

/// Splitst elk deel van `Zaal_Vast`: de driehoeken die helemaal in de schil liggen, en de rest.
/// Materiaal en gebakken licht blijven gelijk, dus het deel ziet er heel precies zo uit als
/// voorheen.
fn splits_af(
    mut commands: Commands,
    delen: Query<
        (
            Entity,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &GlobalTransform,
            Option<&Lightmap>,
        ),
        With<Splitsen>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut kant: ResMut<Kant>,
) {
    for (e, mesh, materiaal, plek, licht) in &delen {
        let Some(m) = meshes.get(&mesh.0) else {
            continue;
        };
        commands.entity(e).remove::<Splitsen>();
        let Some(VertexAttributeValues::Float32x3(punten)) = m.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        let wereld: Vec<Vec3> = punten
            .iter()
            .map(|p| plek.transform_point(Vec3::from(*p)))
            .collect();
        let indices: Vec<u32> = match m.indices() {
            Some(Indices::U32(i)) => i.clone(),
            Some(Indices::U16(i)) => i.iter().map(|&i| u32::from(i)).collect(),
            None => (0..wereld.len() as u32).collect(),
        };
        let (mut binnen, mut schil) = (Vec::new(), Vec::new());
        for d in indices.chunks_exact(3) {
            let doel = if d.iter().all(|&i| in_de_schil(wereld[i as usize])) {
                &mut schil
            } else {
                &mut binnen
            };
            doel.extend_from_slice(d);
        }
        if schil.is_empty() {
            continue;
        }
        let mut deel = m.clone();
        deel.insert_indices(Indices::U32(schil));
        let rest = (!binnen.is_empty()).then(|| {
            let mut rest = m.clone();
            rest.insert_indices(Indices::U32(binnen));
            rest
        });
        let rest = rest.map(|r| meshes.add(r));
        // in de wortel, op dezelfde plek in de wereld als het deel; verborgen tot de camera's het
        // oneens zijn
        let mut los = commands.spawn((
            Mesh3d(meshes.add(deel)),
            materiaal.clone(),
            plek.compute_transform(),
            RenderLayers::layer(SCHIL_LAAG),
            Visibility::Hidden,
        ));
        if let Some(l) = licht {
            los.insert(l.clone());
        }
        let schil = los.id();
        commands.entity(e).insert(Gesplitst { heel: mesh.0.clone(), rest, schil });
        // een nieuw deel krijgt bij de volgende keuze de goede mesh
        kant.set_changed();
    }
}

/// Kiest per beeld de kant van de camera's en zet daar de delen, de losse schil en de lagen van
/// elke camera naar.
fn kies_kant(
    mut commands: Commands,
    camera: Query<(Entity, &GlobalTransform, Option<&RenderLayers>, Option<&crate::graphic::Bron>), With<ZietSchil>>,
    mut delen: Query<(&Gesplitst, &mut Mesh3d, &mut Visibility)>,
    mut zicht: Query<&mut Visibility, Without<Gesplitst>>,
    mut kant_nu: ResMut<Kant>,
) {
    let mut plekken = Vec::new();
    for (e, plek, lagen, bron) in &camera {
        let b = buiten(plek.translation());
        plekken.push(b);
        let stoellaag = match bron {
            Some(crate::graphic::Bron::Klaar) => STOEL_KLAAR_LAAG,
            _ => STOEL_ETHER_LAAG,
        };
        let nodig = if b {
            RenderLayers::from_layers(&[0, stoellaag])
        } else {
            RenderLayers::from_layers(&[0, SCHIL_LAAG, stoellaag])
        };
        if lagen != Some(&nodig) {
            commands.entity(e).insert(nodig);
        }
    }
    let k = kant(plekken);
    kant_nu.set_if_neq(k);
    if !kant_nu.is_changed() {
        return;
    }
    for (g, mut mesh, mut z) in &mut delen {
        let (nodig, te_zien) = match (k, &g.rest) {
            (Kant::Binnen, _) => (&g.heel, true),
            (_, Some(rest)) => (rest, true),
            (_, None) => (&g.heel, false),
        };
        if mesh.0 != *nodig {
            mesh.0 = nodig.clone();
        }
        z.set_if_neq(if te_zien { Visibility::Inherited } else { Visibility::Hidden });
        if let Ok(mut s) = zicht.get_mut(g.schil) {
            s.set_if_neq(if k == Kant::Gemengd { Visibility::Inherited } else { Visibility::Hidden });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn het_plafond_en_wat_erboven_hangt_horen_bij_de_schil_de_zaal_zelf_niet() {
        // plafond, hanglamp, de zuil achter de Voorzitter, het lage plafond en de voorwand
        for p in [
            [0.0, 5.18, -3.0],
            [0.5, 5.4, -3.2],
            [0.0, 2.0, 5.1],
            [2.0, 4.14, 6.0],
            [0.0, 3.0, 6.5],
        ] {
            assert!(in_de_schil(Vec3::from(p)), "{p:?}");
        }
        // Stoel, wand onder het plafond, Voorzitter
        for p in [[5.5, 0.8, -1.4], [0.0, 4.9, -12.0], [0.0, 1.2, 3.0]] {
            assert!(!in_de_schil(Vec3::from(p)), "{p:?}");
        }
    }

    #[test]
    fn buiten_is_boven_het_plafond_of_achter_de_voorzitter() {
        assert!(
            !buiten(Vec3::new(0.0, 4.56, 4.48)),
            "het oude Standpunt A, in de Zaal"
        );
        assert!(buiten(Vec3::new(0.0, 8.0, 8.0)));
    }

    #[test]
    fn de_kant_van_de_cameras() {
        assert_eq!(kant([]), Kant::Binnen);
        assert_eq!(kant([false]), Kant::Binnen);
        assert_eq!(kant([true]), Kant::Buiten);
        assert_eq!(kant([true, true]), Kant::Buiten);
        assert_eq!(kant([false, true]), Kant::Gemengd);
    }
}
