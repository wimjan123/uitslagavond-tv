//! De schil van de Zaal: het plafond met alles erboven (daklicht, hanglampen, spots) en de
//! voorwand boven de Voorzitter. Een camera die buiten de Zaal staat, zoals Totaal vanaf
//! voorzitter (Standpunt A, hoog achter de voorwand), ziet de Zaal zonder schil: open van boven,
//! als in een virtuele studio. Een camera in de Zaal ziet hem gewoon.
//!
//! De schil ligt op een eigen [`RenderLayers`]-laag: elke camera kiest per beeld of hij die laag
//! ziet, naar zijn eigen plek. Zo werkt het ook met twee camera's in één Renderer (het Voorbeeld
//! en zijn kleine Ether).

use bevy::camera::visibility::RenderLayers;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::pbr::Lightmap;
use bevy::prelude::*;

/// De laag van de schil.
const SCHIL_LAAG: usize = 1;

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
        app.add_systems(Update, (splits_af, kies_lagen).chain());
    }
}

/// Een deel van het vaste model (`Zaal_Vast`) dat nog in schil en binnenkant gesplitst moet
/// worden. `zaal.rs` zet het erop als het model in de wereld komt.
#[derive(Component)]
pub struct Splitsen;

/// Een deel van de schil: afgesplitst uit `Zaal_Vast`, of een los voorwerp dat er helemaal in
/// hangt (een hanglamp of spot, zie [`in_de_schil_volgens_naam`]).
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

/// Splitst elk deel van `Zaal_Vast` in twee meshes: de driehoeken die helemaal in de schil
/// liggen naar een eigen entity op de laag van de schil, de rest blijft. Materiaal en gebakken
/// licht blijven gelijk, dus de schil ziet er van binnen precies zo uit als voorheen.
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
) {
    for (e, mesh, materiaal, plek, licht) in &delen {
        let Some(m) = meshes.get(&mesh.0) else {
            continue;
        };
        let Some(VertexAttributeValues::Float32x3(punten)) = m.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            commands.entity(e).remove::<Splitsen>();
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
        commands.entity(e).remove::<Splitsen>();
        if schil.is_empty() {
            continue;
        }
        let mut deel = m.clone();
        deel.insert_indices(Indices::U32(schil));
        let mut rest = m.clone();
        rest.insert_indices(Indices::U32(binnen));
        let _ = meshes.insert(&mesh.0, rest);
        let mut nieuw = commands.spawn((
            Schil,
            Mesh3d(meshes.add(deel)),
            materiaal.clone(),
            plek.compute_transform(),
            RenderLayers::layer(SCHIL_LAAG),
        ));
        if let Some(l) = licht {
            nieuw.insert(l.clone());
        }
    }
}

/// Elke camera die de Zaal tekent, ziet de schil alleen als hij in de Zaal staat.
fn kies_lagen(
    mut commands: Commands,
    camera: Query<(Entity, &GlobalTransform, Option<&RenderLayers>), With<ZietSchil>>,
) {
    for (e, plek, lagen) in &camera {
        let nodig = if buiten(plek.translation()) {
            RenderLayers::layer(0)
        } else {
            RenderLayers::from_layers(&[0, SCHIL_LAAG])
        };
        if lagen != Some(&nodig) {
            commands.entity(e).insert(nodig);
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
}
