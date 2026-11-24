//! De Zaal: het model met gebakken licht, de lampen, de reflecties (zie `omgeving.rs`) en de
//! Stoelen die hun kleur uit de Stand halen. De camera volgt de server (zie `camera.rs`).
//!
//! De bestanden komen uit `zaal/maak.sh` (zie `zaal/README.md`). Zonder `--zaal` tekent de
//! Renderer alleen de graphics op een effen achtergrond, zoals in CI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::gltf::GltfMaterialName;
use bevy::light::{CascadeShadowConfigBuilder, GlobalAmbientLight};
use bevy::pbr::Lightmap;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use bevy::color::Mix;
use gedeeld::uitslagindeling::stoelen_van_links_naar_rechts;
use gedeeld::{AANTAL_STOELEN, Kleur, Stoel};
use serde::Deserialize;

use crate::omgeving::{self, OmgevingPlugin, Reflecties};
use crate::volging::Standen;
#[cfg(test)]
use crate::{BREEDTE, HOOGTE};
use crate::kleurgolf;
use crate::lichtkaart;

/// De kom van een plafondspot: dat is wat je van onderen ziet. In Cycles straalt daar de lamp zelf;
/// de lens van het model zit erachter. Zie [`spots_stralen`].
const SPOTKOM: &str = "daklicht_spotkom";
/// Hoe fel de kom van een plafondspot straalt: zoals de lens in het model (`spot`, sterkte 14).
const SPOT_STRAALT: LinearRgba = LinearRgba::rgb(14.0, 13.0, 11.2);
/// Het materiaal van het leer van een Stoel: dat deel krijgt de kleur uit de Stand.
const STOEL_LEER: &str = "stoel_leer";

pub struct ZaalPlugin {
    /// Map met de bestanden uit `zaal/maak.sh`; `None` voor een Renderer zonder Zaal.
    pub map: Option<PathBuf>,
}

impl Plugin for ZaalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ZaalKlaar(self.map.is_none()))
            .init_resource::<StoelMaterialen>()
            .init_resource::<Golf>()
            .init_resource::<Uitlichting>()
            .add_systems(
                Update,
                ((herken_stoelen, kleur_stoelen, licht_stoel_op).chain(), geef_stoelen_licht),
            );
        if let Some(map) = &self.map {
            app.add_plugins(OmgevingPlugin)
                .insert_resource(ZaalMap(map.clone()))
                .init_resource::<ModelGeladen>()
                .insert_resource(GlobalAmbientLight {
                    color: Color::srgb(1.0, 0.95, 0.88),
                    brightness: crate::beeld::BEELD.omgeving,
                    affects_lightmapped_meshes: false,
                })
                .add_systems(Startup, (laad.after(crate::beeld::bouw), zet_lijn_bij_76))
                .add_systems(Update, (stel_materialen_bij, spots_stralen, meld_klaar, toon_lijn_bij_76));
        }
    }
}

#[derive(Resource)]
struct ZaalMap(PathBuf);

/// Of de Zaal klaar is om te tekenen: het model staat in de wereld en de reflecties zijn
/// gefilterd. Zonder Zaal meteen waar.
#[derive(Resource)]
pub struct ZaalKlaar(pub bool);

/// Het model staat in de wereld.
#[derive(Resource, Default)]
struct ModelGeladen(bool);

fn meld_klaar(model: Res<ModelGeladen>, reflecties: Res<Reflecties>, mut klaar: ResMut<ZaalKlaar>) {
    let nu = model.0 && reflecties.klaar;
    if klaar.0 != nu {
        klaar.0 = nu;
    }
}

/// Een Stoel uit het model.
#[derive(Component, Debug)]
pub struct ZaalStoel;

/// Het leer van een Stoel: dit deel krijgt de kleur uit de Stand.
#[derive(Component, Debug)]
pub struct StoelLeer {
    pub index: usize,
}

/// Eén materiaal per kleur, gedeeld door alle Stoelen met die kleur.
#[derive(Resource, Default)]
struct StoelMaterialen {
    origineel: Option<Handle<StandardMaterial>>,
    per_kleur: HashMap<Kleur, Handle<StandardMaterial>>,
    /// Een eigen materiaal voor een Stoel midden in de kleurgolf.
    per_stoel: HashMap<usize, Handle<StandardMaterial>>,
}

/// Het gebakken licht van de Stoelen (`stoelkaart.bin`): één kaart, per Stoel een eigen vak. De
/// kaart bewaart alleen licht, geen kleur, dus de Stoelen kunnen nog steeds van kleur wisselen.
/// Een oudere Zaal heeft geen stoelkaart; dan krijgen de Stoelen live licht.
#[derive(Resource, Debug)]
pub struct Stoellicht {
    kaart: Handle<Image>,
    /// Per naam uit het model (`Stoel_<vak>_<rij>_<nummer>`) het vak op de kaart, in UV van glTF.
    vakken: HashMap<String, Rect>,
}

#[derive(Deserialize)]
struct Stoelkaart {
    stoelen: HashMap<String, [f32; 4]>,
}

impl Stoellicht {
    /// Leest `stoelkaart.json` uit `zaal/export.py`.
    pub fn uit_json(tekst: &str, kaart: Handle<Image>) -> Result<Self, String> {
        let k: Stoelkaart = serde_json::from_str(tekst).map_err(|e| e.to_string())?;
        let mut vakken = HashMap::new();
        for (naam, [x0, y0, x1, y1]) in k.stoelen {
            if stoel_uit_naam(&naam).is_none() {
                return Err(format!("geen Stoel: {naam}"));
            }
            let binnen = |v: f32| (0.0..=1.0).contains(&v);
            if ![x0, y0, x1, y1].into_iter().all(binnen) || x0 >= x1 || y0 >= y1 {
                return Err(format!("{naam}: vak buiten de kaart"));
            }
            vakken.insert(naam, Rect::new(x0, y0, x1, y1));
        }
        Ok(Self { kaart, vakken })
    }
}

/// De delen van een Stoel met gebakken licht: het leer en de houten kuip, samen bijna alles wat je
/// ziet. Bevy geeft elk deel met een lichtkaart een eigen plek (vier per groep), en elke groep
/// kost een eigen tekenopdracht; alle zes delen van 150 Stoelen kostte zo 6 ms per beeld. De
/// kleine delen (bies, naad, poten, wapen) krijgen live licht.
const STOEL_GEBAKKEN: [&str; 2] = [STOEL_LEER, "perenhout"];

/// Geeft het leer en de kuip van elke Stoel het gebakken licht uit zijn vak, zodra ze in de
/// wereld komen.
fn geef_stoelen_licht(
    mut commands: Commands,
    licht: Option<Res<Stoellicht>>,
    nieuw: Query<(Entity, &GltfMaterialName), Added<Mesh3d>>,
    ouders: Query<&ChildOf>,
    namen: Query<&Name>,
) {
    let Some(licht) = licht else { return };
    for (e, materiaal) in &nieuw {
        if !STOEL_GEBAKKEN.contains(&materiaal.0.as_str()) {
            continue;
        }
        let vak = ouders
            .iter_ancestors(e)
            .find_map(|a| namen.get(a).ok().and_then(|n| licht.vakken.get(n.as_str())));
        if let Some(vak) = vak {
            commands.entity(e).insert(Lightmap {
                image: licht.kaart.clone(),
                uv_rect: *vak,
                bicubic_sampling: true,
            });
        }
    }
}

/// Een camera uit het model (`standpunten.json`), voor de proeven naast Cycles. De camera van de
/// uitzending komt van de server.
#[cfg(test)]
#[derive(Deserialize, Clone, Debug)]
pub struct Modelcamera {
    pub naam: String,
    pub positie: [f32; 3],
    pub rotatie_xyzw: [f32; 4],
    pub lens_mm: f32,
    pub sensor_mm: f32,
}

#[cfg(test)]
impl Modelcamera {
    pub fn transform(&self) -> Transform {
        let [x, y, z, w] = self.rotatie_xyzw;
        // standpunten.json draait de assen van de wereld al om. De camera houdt haar eigen
        // assen (Blender en Bevy kijken allebei langs de lokale -z), dus nog één keer rechts erbij.
        let q = Quat::from_xyzw(x, y, z, w) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
        Transform::from_translation(Vec3::from(self.positie)).with_rotation(q)
    }

    /// Verticale beeldhoek bij 16:9. De lens geldt voor de breedte van de sensor.
    pub fn fov(&self) -> f32 {
        let horizontaal = 2.0 * (self.sensor_mm / 2.0 / self.lens_mm).atan();
        2.0 * ((horizontaal / 2.0).tan() * HOOGTE as f32 / BREEDTE as f32).atan()
    }
}

#[derive(Deserialize)]
struct Lamp {
    soort: String,
    positie: [f32; 3],
    richting: [f32; 3],
    lumen: f32,
    kleur: [f32; 3],
    kegel: f32,
    zacht: f32,
    live: bool,
}

fn lees_json<T: for<'a> Deserialize<'a>>(map: &Path, naam: &str) -> T {
    let pad = map.join(naam);
    let tekst = std::fs::read_to_string(&pad).unwrap_or_else(|e| panic!("{}: {e}", pad.display()));
    serde_json::from_str(&tekst).unwrap_or_else(|e| panic!("{}: {e}", pad.display()))
}

/// De Stoel bij een naam uit het model, zoals `Stoel_3_2_1`.
pub fn stoel_uit_naam(naam: &str) -> Option<Stoel> {
    let mut delen = naam
        .strip_prefix("Stoel_")?
        .split('_')
        .map(|d| d.parse::<u8>().ok());
    let stoel = Stoel {
        vak: delen.next()??,
        rij: delen.next()??,
        nummer: delen.next()??,
    };
    (delen.next().is_none() && stoel.index().is_some()).then_some(stoel)
}

fn laad(
    mut commands: Commands,
    map: Res<ZaalMap>,
    server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut reflecties: ResMut<Reflecties>,
) {
    let lampen: Vec<Lamp> = lees_json(&map.0, "licht.json");
    let lichtkaart = images.add(lichtkaart::lees(&map.0.join("lichtkaart.bin")));
    let (bin, json) = (map.0.join("stoelkaart.bin"), map.0.join("stoelkaart.json"));
    if bin.exists() && json.exists() {
        let kaart = images.add(lichtkaart::lees(&bin));
        let tekst = std::fs::read_to_string(&json)
            .unwrap_or_else(|e| panic!("{}: {e}", json.display()));
        let licht = Stoellicht::uit_json(&tekst, kaart)
            .unwrap_or_else(|e| panic!("{}: {e}", json.display()));
        info!("gebakken licht voor {} Stoelen", licht.vakken.len());
        commands.insert_resource(licht);
    }
    if !omgeving::zet_neer(&mut commands, &mut images, &mut reflecties, &map.0) {
        warn!(
            "geen omgeving.bin in {}: metaal en glas zonder reflecties (maak het met zaal/maak.sh <map> --alleen-omgeving)",
            map.0.display()
        );
    }

    for l in &lampen {
        let positie = Vec3::from(l.positie);
        let kleur = Color::linear_rgb(l.kleur[0], l.kleur[1], l.kleur[2]);
        if l.live {
            // Daglicht door het glas (alleen een oudere Zaal; de nieuwe heeft het in de bak): van
            // boven, iets schuin, met schaduw van de Stoelen.
            commands.spawn((
                DirectionalLight {
                    color: kleur,
                    illuminance: crate::beeld::BEELD.daglicht_lux,
                    shadow_maps_enabled: true,
                    affects_lightmapped_mesh_diffuse: true,
                    ..default()
                },
                Transform::from_translation(positie)
                    .looking_to(Vec3::new(0.12, -1.0, 0.18), Vec3::Z),
                CascadeShadowConfigBuilder {
                    num_cascades: 3,
                    minimum_distance: 0.1,
                    first_cascade_far_bound: 12.0,
                    maximum_distance: 45.0,
                    overlap_proportion: 0.2,
                }
                .build(),
            ));
            continue;
        }
        // De andere lampen zitten in de bak. Hier verlichten ze alleen wat niet gebakken is,
        // zonder schaduw; op de gebakken vlakken geven ze alleen glans.
        let richting = Vec3::from(l.richting).normalize_or(Vec3::NEG_Y);
        let boven = if richting.y.abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let plek = Transform::from_translation(positie).looking_to(richting, boven);
        if l.soort == "POINT" || l.kegel >= 3.0 {
            commands.spawn((
                PointLight {
                    color: kleur,
                    intensity: l.lumen,
                    range: crate::beeld::BEELD.lamp_bereik,
                    shadow_maps_enabled: false,
                    affects_lightmapped_mesh_diffuse: false,
                    ..default()
                },
                plek,
            ));
        } else {
            let half = (l.kegel / 2.0).min(1.5);
            commands.spawn((
                SpotLight {
                    color: kleur,
                    intensity: l.lumen,
                    range: crate::beeld::BEELD.lamp_bereik,
                    outer_angle: half,
                    inner_angle: half * (1.0 - l.zacht).clamp(0.0, 1.0),
                    shadow_maps_enabled: false,
                    affects_lightmapped_mesh_diffuse: false,
                    ..default()
                },
                plek,
            ));
        }
    }

    let wereld = server.load(GltfAssetLabel::Scene(0).from_asset("zaal.glb"));
    commands.spawn(WorldAssetRoot(wereld)).observe(
        move |klaar: On<WorldInstanceReady>,
              mut commands: Commands,
              kinderen: Query<&Children>,
              namen: Query<&Name>,
              meshes: Query<(), With<Mesh3d>>,
              mut geladen: ResMut<ModelGeladen>| {
            let mut gebakken = 0;
            let mut stoelen = 0;
            for e in kinderen.iter_descendants(klaar.entity) {
                stoelen += namen
                    .get(e)
                    .is_ok_and(|n| stoel_uit_naam(n.as_str()).is_some())
                    as usize;
                if namen.get(e).is_ok_and(|n| n.as_str() == "Zaal_Vast") {
                    for d in kinderen.iter_descendants(e).chain([e]) {
                        if meshes.contains(d) {
                            commands.entity(d).insert(Lightmap {
                                image: lichtkaart.clone(),
                                uv_rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                                bicubic_sampling: true,
                            });
                            gebakken += 1;
                        }
                    }
                }
            }
            info!("Zaal geladen: {gebakken} delen met gebakken licht, {stoelen} Stoelen");
            geladen.0 = true;
        },
    );
}

/// Merkt de Stoelen in het model en hun leer, zodra ze in de wereld komen.
fn herken_stoelen(
    mut commands: Commands,
    nieuwe_namen: Query<(Entity, &Name), Added<Name>>,
    nieuwe_materialen: Query<(Entity, &GltfMaterialName), Added<GltfMaterialName>>,
    ouders: Query<&ChildOf>,
    namen: Query<&Name>,
) {
    for (e, naam) in &nieuwe_namen {
        if stoel_uit_naam(naam.as_str()).is_some() {
            commands.entity(e).insert(ZaalStoel);
        }
    }
    for (e, materiaal) in &nieuwe_materialen {
        if materiaal.0 != STOEL_LEER {
            continue;
        }
        let stoel = ouders
            .iter_ancestors(e)
            .find_map(|a| namen.get(a).ok().and_then(|n| stoel_uit_naam(n.as_str())));
        if let Some(index) = stoel.and_then(|s| s.index()) {
            commands.entity(e).insert(StoelLeer { index });
        }
    }
}

/// De kleurgolf: waar elke Stoel vandaan komt, waar hij heen gaat en hoe ver de golf is.
#[derive(Resource, Default)]
struct Golf {
    /// Er is al een Stand op de Stoelen gezet. Daarvoor (de eerste Stand na het opstarten) komt
    /// de kleur meteen, zonder golf (ADR 0001).
    begonnen: bool,
    /// Wat elke Stoel toonde toen de golf begon (lineair), in de volgorde van [`Stoel::alle`].
    van: Vec<LinearRgba>,
    /// De kleur uit de Stand per Stoel; `None`: het eigen leer.
    naar: Vec<Option<Kleur>>,
    /// Plaats van elke Stoel van links naar rechts in beeld.
    rang: Vec<usize>,
    tijd: f32,
    duur: f32,
    bezig: bool,
}

impl Golf {
    fn voortgang(&self, i: usize) -> f32 {
        if self.bezig {
            kleurgolf::voortgang(self.rang[i], AANTAL_STOELEN - 1, self.tijd, self.duur)
        } else {
            1.0
        }
    }
}

/// Geeft elke Stoel de kleur uit de Stand. Na de eerste Stand gaat dat als een golf van links
/// naar rechts (zie `kleurgolf.rs`); Stoelen die gelijk blijven, veranderen niet. Zonder kleur
/// in de Stand krijgt de Stoel zijn eigen leer terug.
#[allow(clippy::too_many_arguments)]
fn kleur_stoelen(
    standen: Res<Standen>,
    tijd: Res<Time>,
    mut leer: Query<(&StoelLeer, &mut MeshMaterial3d<StandardMaterial>)>,
    nieuw_leer: Query<(), Added<StoelLeer>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut pm: ResMut<StoelMaterialen>,
    mut golf: ResMut<Golf>,
) {
    let veranderd = standen.is_changed() || !nieuw_leer.is_empty();
    if !veranderd && !golf.bezig {
        return;
    }
    let Some(origineel) = pm
        .origineel
        .clone()
        .or_else(|| leer.iter().next().map(|(_, m)| m.0.clone()))
    else {
        return;
    };
    pm.origineel = Some(origineel.clone());
    let Some(eigen_leer) = materialen.get(&origineel).map(|m| LinearRgba::from(m.base_color))
    else {
        return;
    };
    // De leertextuur heeft ruimte voor een factor 2 (webexport.K bij het model); de helft van de
    // kleur brengt hem terug op de kleur uit de Stand.
    let lineair = |k: Option<Kleur>| {
        k.map_or(eigen_leer, |k| LinearRgba::from(Color::srgb_u8(k.r, k.g, k.b)) * 0.5)
    };
    if golf.rang.is_empty() {
        let mut rang = vec![0; AANTAL_STOELEN];
        for (r, i) in stoelen_van_links_naar_rechts().into_iter().enumerate() {
            rang[i] = r;
        }
        golf.rang = rang;
    }

    // De golf loopt verder; een nieuwe golf begint hieronder op nul.
    if golf.bezig {
        golf.tijd += tijd.delta_secs();
    }
    if veranderd {
        let kleuren = standen
            .huidig
            .as_ref()
            .map(|s| s.stand.stoelkleuren.as_slice())
            .unwrap_or_default();
        let doel: Vec<Option<Kleur>> = (0..AANTAL_STOELEN).map(|i| kleuren.get(i).copied()).collect();
        if !golf.begonnen || !nieuw_leer.is_empty() {
            golf.van = doel.iter().map(|k| lineair(*k)).collect();
            golf.naar = doel;
            golf.bezig = false;
            golf.begonnen = standen.huidig.is_some();
        } else if doel != golf.naar {
            // vanaf wat nu te zien is, ook midden in een vorige golf
            let nu: Vec<LinearRgba> = (0..AANTAL_STOELEN)
                .map(|i| golf.van[i].mix(&lineair(golf.naar[i]), golf.voortgang(i)))
                .collect();
            golf.van = nu;
            golf.naar = doel;
            golf.tijd = 0.0;
            golf.duur = kleurgolf::duur(
                standen.huidig.as_ref().map_or(100, |s| s.stand.stijl_van_ether().snelheid),
            );
            golf.bezig = true;
        }
    }

    for (stoel, mut materiaal) in &mut leer {
        let i = stoel.index;
        let (Some(van), Some(naar)) = (golf.van.get(i).copied(), golf.naar.get(i).copied()) else {
            continue;
        };
        let f = golf.voortgang(i);
        let gewenst = if f >= 1.0 || van == lineair(naar) {
            match naar {
                None => origineel.clone(),
                Some(kleur) => match pm.per_kleur.get(&kleur) {
                    Some(h) => h.clone(),
                    None => {
                        let Some(mut m) = materialen.get(&origineel).cloned() else {
                            continue;
                        };
                        m.base_color = lineair(Some(kleur)).into();
                        let h = materialen.add(m);
                        pm.per_kleur.insert(kleur, h.clone());
                        h
                    }
                },
            }
        } else if f <= 0.0 {
            // nog niet aan de beurt: hij houdt wat hij had
            continue;
        } else {
            // midden in de golf: een eigen materiaal voor deze Stoel, tot hij klaar is
            let h = match pm.per_stoel.get(&i) {
                Some(h) => h.clone(),
                None => {
                    let Some(m) = materialen.get(&origineel).cloned() else {
                        continue;
                    };
                    let h = materialen.add(m);
                    pm.per_stoel.insert(i, h.clone());
                    h
                }
            };
            if let Some(mut m) = materialen.get_mut(&h) {
                m.base_color = van.mix(&lineair(naar), f).into();
            }
            h
        };
        if materiaal.0 != gewenst {
            materiaal.0 = gewenst;
        }
    }
    if golf.bezig && golf.tijd >= golf.duur {
        golf.bezig = false;
        golf.van = golf.naar.iter().map(|k| lineair(*k)).collect();
    }
}

/// De lijn bij Zetel 76 in de Coalitiebouwer (besluit #7): een streep in Accent tussen de 75e en
/// de 76e Stoel van links, van voor naar achter. `zicht` loopt van 0 (weg) tot 1 (helemaal).
#[derive(Component)]
struct LijnBij76 {
    zicht: f32,
}

/// Breedte en hoogte van de lijn bij 76, in meter.
const LIJN_76_DIKTE: Vec2 = Vec2::new(0.18, 0.05);
/// Zo lang groeit de lijn van voor naar achter, of krimpt hij weer, bij snelheid 100.
const LIJN_76_GROEIT: f32 = 0.5;

fn zet_lijn_bij_76(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
) {
    let (voor, achter) = gedeeld::uitslagindeling::lijn_bij_76();
    let lengte = Vec3::from(achter).distance(Vec3::from(voor));
    commands.spawn((
        LijnBij76 { zicht: 0.0 },
        Mesh3d(meshes.add(Cuboid::new(LIJN_76_DIKTE.x, LIJN_76_DIKTE.y, lengte))),
        MeshMaterial3d(materialen.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::from(voor)),
        Visibility::Hidden,
    ));
}

/// De lijn bij 76 staat er zolang de Graphic in de Ether een Coalitie of Coalitiebalk is, in de
/// kleur Accent van de Stijl. Hij groeit van voor naar achter en krimpt terug als hij weggaat.
fn toon_lijn_bij_76(
    standen: Res<Standen>,
    tijd: Res<Time>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut lijn: Query<(&mut LijnBij76, &mut Transform, &mut Visibility, &MeshMaterial3d<StandardMaterial>)>,
) {
    let Some(stand) = standen.huidig.as_ref().map(|s| &s.stand) else {
        return;
    };
    let coalitie = matches!(
        stand.ether.as_ref().map(|e| &e.graphic),
        Some(gedeeld::Graphic::Coalitie { .. } | gedeeld::Graphic::Coalitiebalk { .. })
    );
    let snelheid = f32::from(stand.stijl.snelheid.max(10)) / 100.0;
    for (mut l, mut transform, mut zicht, materiaal) in &mut lijn {
        let stap = tijd.delta_secs() * snelheid / LIJN_76_GROEIT;
        l.zicht = if coalitie { (l.zicht + stap).min(1.0) } else { (l.zicht - stap).max(0.0) };
        let (voor, achter) = gedeeld::uitslagindeling::lijn_bij_76();
        let (voor, achter) = (Vec3::from(voor), Vec3::from(achter));
        let f = crate::animatie::zacht_uit(l.zicht);
        let mut t = Transform::from_translation(voor + (achter - voor) * f / 2.0).looking_at(achter, Vec3::Y);
        t.scale = Vec3::new(1.0, 1.0, f.max(0.001));
        if *transform != t {
            *transform = t;
        }
        zicht.set_if_neq(if l.zicht > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
        let accent = stand.stijl.kleur(gedeeld::sjabloon::Stijlkleur::Accent);
        let kleur = Color::srgb_u8(accent.r, accent.g, accent.b);
        if let Some(m) = materialen.get(&materiaal.0)
            && m.base_color != kleur
            && let Some(mut m) = materialen.get_mut(&materiaal.0)
        {
            m.base_color = kleur;
        }
    }
}

/// Hoe fel de Stoel van de Persoonskaart straalt: warm wit, zodat hij oplicht in elke kleur maar
/// zijn eigen kleur houdt.
const UITGELICHT: LinearRgba = LinearRgba::rgb(0.34, 0.3, 0.22);
/// Zo lang duurt het oplichten en uitgaan, in seconden bij snelheid 100.
const UITLICHTEN_S: f32 = 0.5;

/// De Stoel die oplicht (Persoonskaart): welke, hoe ver, en per materiaal van het leer de
/// stralende kopie.
#[derive(Resource, Default)]
struct Uitlichting {
    stoel: Option<usize>,
    /// 0 uit, 1 helemaal aan.
    sterkte: f32,
    /// Van het gewone materiaal naar de stralende kopie, en terug.
    straalt: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
    gewoon: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
}

/// Laat de Stoel van de Graphic oplichten: in de Ether en de Reserve die in de Ether, in het
/// Voorbeeld de klaargezette. Het leer krijgt een stralende kopie van zijn eigen materiaal, zodat
/// de kleur van de Stoel blijft (ook midden in een kleurgolf); de andere Stoelen houden hun
/// materiaal.
fn licht_stoel_op(
    standen: Res<Standen>,
    rolinfo: Option<Res<crate::Rolinfo>>,
    tijd: Res<Time>,
    mut leer: Query<(&StoelLeer, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut u: ResMut<Uitlichting>,
) {
    let stand = standen.huidig.as_ref().map(|s| &s.stand);
    let voorbeeld = rolinfo.is_some_and(|r| r.rol == gedeeld::Rol::Voorbeeld);
    let gewenst = stand
        .and_then(|s| {
            if voorbeeld {
                s.klaar.as_ref().map(|k| &k.graphic)
            } else {
                s.ether.as_ref().map(|e| &e.graphic)
            }
        })
        .and_then(|g| g.uitgelichte_stoel())
        .and_then(|s| s.index());
    let stijl = |s: &'_ gedeeld::Stand| if voorbeeld { s.stijl.snelheid } else { s.stijl_van_ether().snelheid };
    let snelheid = f32::from(stand.map_or(100, stijl).max(10)) / 100.0;
    let stap = tijd.delta_secs() * snelheid / UITLICHTEN_S;
    if gewenst.is_some() && gewenst != u.stoel {
        // een andere Stoel: die licht op, de vorige is meteen uit
        u.stoel = gewenst;
        u.sterkte = 0.0;
    }
    u.sterkte = if gewenst.is_some() {
        (u.sterkte + stap).min(1.0)
    } else {
        (u.sterkte - stap).max(0.0)
    };
    if u.sterkte == 0.0 && gewenst.is_none() {
        u.stoel = None;
    }
    if u.stoel.is_none() && u.gewoon.is_empty() {
        return;
    }
    let gloed = UITGELICHT * u.sterkte;
    for (stoel, mut materiaal) in &mut leer {
        let gewoon = u.gewoon.get(&materiaal.0.id()).cloned().unwrap_or_else(|| materiaal.0.clone());
        let doel = if Some(stoel.index) == u.stoel {
            let Some(basis) = materialen.get(&gewoon).cloned() else {
                continue;
            };
            let h = match u.straalt.get(&gewoon.id()) {
                Some(h) => h.clone(),
                None => {
                    let h = materialen.add(basis.clone());
                    u.straalt.insert(gewoon.id(), h.clone());
                    u.gewoon.insert(h.id(), gewoon.clone());
                    h
                }
            };
            if let Some(mut m) = materialen.get_mut(&h) {
                // de kleur van het leer, ook als die midden in een golf verandert
                m.base_color = basis.base_color;
                m.emissive = gloed;
            }
            h
        } else {
            gewoon
        };
        if materiaal.0 != doel {
            materiaal.0 = doel;
        }
    }
    if u.stoel.is_none() {
        // alles is weer gewoon: de kopieën zijn niet meer nodig
        u.straalt.clear();
        u.gewoon.clear();
    }
}

/// Materialen uit Blender die Bevy anders tekent dan Cycles.
///
/// Glas komt uit Blender als "transmission". Bevy tekent dat hier grijs, waardoor het daklicht
/// dicht lijkt. Daarom gewone doorzichtigheid met een beetje glans (besluit #8).
fn stel_materialen_bij(
    mut events: MessageReader<AssetEvent<StandardMaterial>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
) {
    let nieuw: Vec<_> = events
        .read()
        .filter_map(|e| match e {
            AssetEvent::Added { id } => Some(*id),
            _ => None,
        })
        .collect();
    for id in nieuw {
        let Some(mut m) = materialen.get_mut(id) else {
            continue;
        };
        if m.specular_transmission > 0.5 {
            m.specular_transmission = 0.0;
            m.alpha_mode = AlphaMode::Blend;
            m.base_color = m.base_color.with_alpha(0.12);
            m.perceptual_roughness = 0.1;
        }
    }
}

/// De plafondspots stralen, zoals in de zaal en in Cycles: hun kom krijgt het licht van de lamp.
fn spots_stralen(
    mut kommen: Query<(&GltfMaterialName, &mut MeshMaterial3d<StandardMaterial>), Added<GltfMaterialName>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut stralend: Local<Option<Handle<StandardMaterial>>>,
) {
    for (naam, mut materiaal) in &mut kommen {
        if naam.0 != SPOTKOM {
            continue;
        }
        let h = match &*stralend {
            Some(h) => h.clone(),
            None => {
                let Some(mut m) = materialen.get(&materiaal.0).cloned() else {
                    continue;
                };
                m.emissive = SPOT_STRAALT;
                let h = materialen.add(m);
                *stralend = Some(h.clone());
                h
            }
        };
        materiaal.0 = h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gedeeld::{GenummerdeStand, Stand};

    #[test]
    fn namen_uit_het_model_worden_stoelen() {
        assert_eq!(
            stoel_uit_naam("Stoel_3_2_1"),
            Some(Stoel {
                vak: 3,
                rij: 2,
                nummer: 1
            })
        );
        assert_eq!(
            stoel_uit_naam("Stoel_6_6_6"),
            Some(Stoel {
                vak: 6,
                rij: 6,
                nummer: 6
            })
        );
        // rij 1 heeft maar twee Stoelen, en de voorzittersstoel is geen Stoel van een Kamerlid
        assert_eq!(stoel_uit_naam("Stoel_1_1_3"), None);
        assert_eq!(stoel_uit_naam("Stoel_1_1_1_kuip"), None);
        assert_eq!(stoel_uit_naam("Voorzittersstoel"), None);
    }

    /// Een Zaal zonder GPU: 150 Stoelen zoals de glTF-lader ze neerzet (een Stoel met een kind
    /// met het leer), en een kuip die nooit van kleur wisselt.
    fn zaal_zonder_gpu() -> (App, Handle<StandardMaterial>) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_millis(BEELD_MS),
            ))
            .init_asset::<StandardMaterial>()
            .init_resource::<Standen>()
            .add_plugins(ZaalPlugin { map: None });
        let blauw = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::from_color(Color::srgb(0.1, 0.2, 0.6)));
        for s in Stoel::alle() {
            let naam = format!("Stoel_{}_{}_{}", s.vak, s.rij, s.nummer);
            let stoel = app.world_mut().spawn(Name::new(naam)).id();
            app.world_mut().spawn((
                ChildOf(stoel),
                GltfMaterialName(STOEL_LEER.into()),
                MeshMaterial3d(blauw.clone()),
                Mesh3d::default(),
            ));
            app.world_mut().spawn((
                ChildOf(stoel),
                GltfMaterialName("stoel_kuip".into()),
                MeshMaterial3d(blauw.clone()),
                Mesh3d::default(),
            ));
        }
        (app, blauw)
    }

    /// Tijd per beeld in de proeven: 50 beelden per seconde, zoals de uitzending.
    const BEELD_MS: u64 = 20;

    /// Laat `seconden` voorbijgaan, beeld voor beeld.
    fn wacht(app: &mut App, seconden: f32) {
        for _ in 0..(seconden * 1000.0 / BEELD_MS as f32).round() as usize {
            app.update();
        }
    }

    fn zet_stand(app: &mut App, volgnummer: u64, kleuren: Vec<Kleur>) {
        let mut standen = app.world_mut().resource_mut::<Standen>();
        let vorige = standen.huidig.replace(GenummerdeStand {
            volgnummer,
            stand: Stand {
                stoelkleuren: kleuren,
                ..default()
            },
        });
        standen.vorige = vorige;
    }

    /// Kleur van het leer per Stoel, in de volgorde van de Stand.
    fn leerkleuren(app: &mut App) -> Vec<Srgba> {
        let mut q = app
            .world_mut()
            .query::<(&StoelLeer, &MeshMaterial3d<StandardMaterial>)>();
        let mut per_stoel: Vec<(usize, Handle<StandardMaterial>)> = q
            .iter(app.world())
            .map(|(s, m)| (s.index, m.0.clone()))
            .collect();
        per_stoel.sort_by_key(|(i, _)| *i);
        let materialen = app.world().resource::<Assets<StandardMaterial>>();
        per_stoel
            .iter()
            .map(|(_, h)| Srgba::from(materialen.get(h).unwrap().base_color))
            .collect()
    }

    fn ongeveer(a: Srgba, b: Srgba) -> bool {
        (a.red - b.red).abs() < 1e-3
            && (a.green - b.green).abs() < 1e-3
            && (a.blue - b.blue).abs() < 1e-3
    }

    #[test]
    fn alle_150_stoelen_krijgen_in_het_volgende_beeld_de_kleur_uit_de_stand() {
        let (mut app, blauw) = zaal_zonder_gpu();
        app.update();
        let origineel = Srgba::from(
            app.world()
                .resource::<Assets<StandardMaterial>>()
                .get(&blauw)
                .unwrap()
                .base_color,
        );
        let leer = leerkleuren(&mut app);
        assert_eq!(leer.len(), 150, "elke Stoel heeft één deel leer");
        assert!(
            leer.iter().all(|k| ongeveer(*k, origineel)),
            "zonder kleuren in de Stand: eigen leer"
        );

        // VOORBEELDGEGEVENS: alle Stoelen rood (#d4202c); het leer krijgt de helft (lineair) door de textuur
        zet_stand(&mut app, 1, vec!["#d4202c".parse().unwrap(); 150]);
        app.update();
        let rood_half = Srgba::from(LinearRgba::from(Color::srgb_u8(0xd4, 0x20, 0x2c)) * 0.5);
        let leer = leerkleuren(&mut app);
        let fout: Vec<usize> = (0..150)
            .filter(|i| !ongeveer(leer[*i], rood_half))
            .collect();
        assert!(fout.is_empty(), "Stoelen zonder de nieuwe kleur: {fout:?}");

        // één Stoel anders: alleen die wisselt, en een lege lijst geeft het eigen leer terug
        let mut kleuren = vec!["#d4202c".parse().unwrap(); 150];
        kleuren[Stoel {
            vak: 4,
            rij: 6,
            nummer: 6,
        }
        .index()
        .unwrap()] = "#00ae41".parse().unwrap();
        zet_stand(&mut app, 2, kleuren);
        wacht(&mut app, 1.05);
        let leer = leerkleuren(&mut app);
        assert_eq!(
            leer.iter().filter(|k| ongeveer(**k, rood_half)).count(),
            149
        );
        zet_stand(&mut app, 3, vec![]);
        wacht(&mut app, 1.05);
        assert!(
            leerkleuren(&mut app)
                .iter()
                .all(|k| ongeveer(*k, origineel))
        );
    }

    #[test]
    fn na_herverbinden_met_dezelfde_stand_loopt_er_geen_kleurgolf() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        // VOORBEELDGEGEVENS: alle Stoelen rood
        let rood: Vec<Kleur> = vec!["#d4202c".parse().unwrap(); 150];
        zet_stand(&mut app, 4, rood.clone());
        app.update();
        let voor = leerkleuren(&mut app);
        // de server viel weg en kwam terug: dezelfde Stand nog eens
        zet_stand(&mut app, 4, rood);
        app.update();
        assert!(!app.world().resource::<Golf>().bezig, "geen golf bij dezelfde Stand");
        assert_eq!(leerkleuren(&mut app), voor);
    }

    /// Het leermateriaal per Stoel, in de volgorde van de Stand.
    fn leermaterialen(app: &mut App) -> Vec<Handle<StandardMaterial>> {
        let mut q = app
            .world_mut()
            .query::<(&StoelLeer, &MeshMaterial3d<StandardMaterial>)>();
        let mut v: Vec<(usize, Handle<StandardMaterial>)> =
            q.iter(app.world()).map(|(s, m)| (s.index, m.0.clone())).collect();
        v.sort_by_key(|(i, _)| *i);
        v.into_iter().map(|(_, h)| h).collect()
    }

    #[test]
    fn de_kleur_gaat_als_een_golf_van_links_naar_rechts_in_een_seconde() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        // VOORBEELDGEGEVENS: alle Stoelen rood, daarna alle Stoelen groen
        let rood: Kleur = "#d4202c".parse().unwrap();
        let groen: Kleur = "#00ae41".parse().unwrap();
        let half = |k: Kleur| Srgba::from(LinearRgba::from(Color::srgb_u8(k.r, k.g, k.b)) * 0.5);
        zet_stand(&mut app, 1, vec![rood; 150]);
        app.update();
        assert!(leerkleuren(&mut app).iter().all(|k| ongeveer(*k, half(rood))), "de eerste Stand meteen");

        zet_stand(&mut app, 2, vec![groen; 150]);
        app.update();
        let links_naar_rechts = stoelen_van_links_naar_rechts();
        wacht(&mut app, 0.5);
        let leer = leerkleuren(&mut app);
        let (links, rechts) = (links_naar_rechts[0], links_naar_rechts[149]);
        assert!(ongeveer(leer[links], half(groen)), "halverwege: links al groen");
        assert!(ongeveer(leer[rechts], half(rood)), "halverwege: rechts nog rood");
        let groen_nu = links_naar_rechts.iter().take_while(|&&i| ongeveer(leer[i], half(groen))).count();
        assert!((40..110).contains(&groen_nu), "halverwege een aaneengesloten stuk links: {groen_nu}");

        wacht(&mut app, 0.52);
        assert!(leerkleuren(&mut app).iter().all(|k| ongeveer(*k, half(groen))), "na een seconde alles groen");
    }

    #[test]
    fn stoelen_die_gelijk_blijven_veranderen_niet() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        let rood: Kleur = "#d4202c".parse().unwrap();
        let blauw: Kleur = "#12357a".parse().unwrap();
        // VOORBEELDGEGEVENS: links 75 rood en rechts 75 blauw; daarna rechts ook rood
        let links_naar_rechts = stoelen_van_links_naar_rechts();
        let mut eerst = vec![blauw; 150];
        for &i in &links_naar_rechts[..75] {
            eerst[i] = rood;
        }
        zet_stand(&mut app, 1, eerst);
        app.update();
        let voor = leermaterialen(&mut app);
        zet_stand(&mut app, 2, vec![rood; 150]);
        for _ in 0..60 {
            app.update();
            let nu = leermaterialen(&mut app);
            for &i in &links_naar_rechts[..75] {
                assert_eq!(nu[i], voor[i], "een rode Stoel links houdt zijn materiaal");
            }
        }
    }

    #[test]
    fn de_snelheid_van_de_stijl_maakt_de_golf_sneller() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        let rood: Kleur = "#d4202c".parse().unwrap();
        let mut stand = Stand { stoelkleuren: vec![rood; 150], ..default() };
        stand.stijl.snelheid = 200;
        app.world_mut().resource_mut::<Standen>().huidig =
            Some(GenummerdeStand { volgnummer: 1, stand: stand.clone() });
        app.update();
        stand.stoelkleuren = vec!["#00ae41".parse().unwrap(); 150];
        app.world_mut().resource_mut::<Standen>().huidig =
            Some(GenummerdeStand { volgnummer: 2, stand });
        app.update();
        wacht(&mut app, 0.52);
        let groen = Srgba::from(LinearRgba::from(Color::srgb_u8(0x00, 0xae, 0x41)) * 0.5);
        assert!(leerkleuren(&mut app).iter().all(|k| ongeveer(*k, groen)), "snelheid 200: na een halve seconde klaar");
    }

    #[test]
    fn elke_stoel_krijgt_zijn_eigen_vak_van_de_stoelkaart() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<StandardMaterial>()
            .init_asset::<Image>()
            .init_resource::<Standen>()
            .add_plugins(ZaalPlugin { map: None });
        // zoals zaal/export.py hem schrijft: per Stoel een vak in UV van glTF
        let json = r#"{"px": 3072, "stoelen": {
            "Stoel_1_1_1": [0.0, 0.0, 0.0769, 0.0833],
            "Stoel_3_2_1": [0.5, 0.25, 0.5769, 0.3333]}}"#;
        let kaart = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        app.insert_resource(Stoellicht::uit_json(json, kaart.clone()).unwrap());
        // een Stoel zoals de glTF-lader hem neerzet: per materiaal een deel
        let mut stoel = |naam: &str| {
            let s = app.world_mut().spawn(Name::new(naam.to_string())).id();
            ["stoel_leer", "perenhout", "stoel_poot"].map(|m| {
                app.world_mut()
                    .spawn((ChildOf(s), Mesh3d::default(), GltfMaterialName(m.into())))
                    .id()
            })
        };
        let [a_leer, a_kuip, a_poot] = stoel("Stoel_1_1_1");
        let [b_leer, b_kuip, _] = stoel("Stoel_3_2_1");
        let zonder = stoel("Stoel_6_6_6");
        app.update();
        let vak = |e: Entity| app.world().get::<Lightmap>(e).map(|l| (l.uv_rect, l.image.clone()));
        let a = Some((Rect::new(0.0, 0.0, 0.0769, 0.0833), kaart.clone()));
        let b = Some((Rect::new(0.5, 0.25, 0.5769, 0.3333), kaart.clone()));
        assert_eq!((vak(a_leer), vak(a_kuip)), (a.clone(), a));
        assert_eq!((vak(b_leer), vak(b_kuip)), (b.clone(), b));
        assert_eq!(vak(a_poot), None, "kleine delen krijgen live licht (zie STOEL_GEBAKKEN)");
        for e in zonder {
            assert_eq!(vak(e), None, "een Stoel zonder vak houdt het live licht");
        }
    }

    #[test]
    fn een_vak_buiten_de_stoelkaart_is_een_fout() {
        let json = r#"{"px": 3072, "stoelen": {"Stoel_1_1_1": [0.9, 0.0, 1.2, 0.1]}}"#;
        assert!(Stoellicht::uit_json(json, Handle::default()).is_err());
        let json = r#"{"px": 3072, "stoelen": {"Voorzittersstoel": [0.0, 0.0, 0.1, 0.1]}}"#;
        assert!(Stoellicht::uit_json(json, Handle::default()).is_err());
    }

    #[test]
    fn de_kuip_van_een_stoel_houdt_zijn_eigen_kleur() {
        let (mut app, blauw) = zaal_zonder_gpu();
        app.update();
        zet_stand(&mut app, 1, vec!["#d4202c".parse().unwrap(); 150]);
        app.update();
        let mut q = app
            .world_mut()
            .query_filtered::<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>), Without<StoelLeer>>();
        let kuipen: Vec<_> = q.iter(app.world()).collect();
        assert_eq!(kuipen.len(), 150);
        assert!(kuipen.iter().all(|(_, m)| m.0 == blauw));
    }

    /// Een Stand met een Persoonskaart in de Ether voor het Kamerlid op `stoel`, of zonder.
    fn zet_persoonskaart(app: &mut App, volgnummer: u64, stoel: Option<Stoel>, kleuren: Vec<Kleur>) {
        use gedeeld::persoonskaart::{Extra, Persoonsgegevens, Wie};
        use gedeeld::{Graphic, GraphicInBeeld};
        // VOORBEELDGEGEVENS
        let graphic = Graphic::Persoonskaart {
            wie: Wie::Eigen { id: "p1".into() },
            regel: String::new(),
            extra: Extra::default(),
            gegevens: Some(Persoonsgegevens {
                naam: "Proef Persoon".into(),
                voorletters: "PP".into(),
                foto: None,
                partij: None,
                functie: None,
                stoel,
                leeftijd: None,
                woonplaats: None,
                geboorteplaats: None,
                in_de_kamer_sinds: None,
            }),
        };
        let ether = GraphicInBeeld {
            id: 1,
            versie: 1,
            graphic,
            standpunt: None,
            getoond: 1,
            sinds_ms: 0,
            getoond_door: "Proef".into(),
            stijl: Default::default(),
            sjabloon: None,
        };
        let mut standen = app.world_mut().resource_mut::<Standen>();
        let vorige = standen.huidig.replace(GenummerdeStand {
            volgnummer,
            stand: Stand { ether: Some(ether), stoelkleuren: kleuren, ..default() },
        });
        standen.vorige = vorige;
    }

    /// Hoe fel het leer van elke Stoel straalt, in de volgorde van de Stand.
    fn straling(app: &mut App) -> Vec<f32> {
        let mut q = app.world_mut().query::<(&StoelLeer, &MeshMaterial3d<StandardMaterial>)>();
        let mut per_stoel: Vec<(usize, Handle<StandardMaterial>)> =
            q.iter(app.world()).map(|(s, m)| (s.index, m.0.clone())).collect();
        per_stoel.sort_by_key(|(i, _)| *i);
        let materialen = app.world().resource::<Assets<StandardMaterial>>();
        per_stoel
            .iter()
            .map(|(_, h)| {
                let e = materialen.get(h).unwrap().emissive;
                e.red + e.green + e.blue
            })
            .collect()
    }

    #[test]
    fn de_stoel_van_de_persoonskaart_licht_op_ook_als_de_stoelen_van_kleur_wisselen() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        let stoel = Stoel { vak: 2, rij: 3, nummer: 1 };
        let i = stoel.index().unwrap();
        zet_persoonskaart(&mut app, 1, Some(stoel), vec![]);
        wacht(&mut app, 1.0);
        let s = straling(&mut app);
        assert!(s[i] > 0.0, "de Stoel straalt");
        assert!(s.iter().enumerate().all(|(n, e)| n == i || *e == 0.0), "alleen die Stoel");
        let leer = leerkleuren(&mut app);
        assert!(ongeveer(leer[i], leer[0]), "het leer houdt zijn kleur");

        // nieuwe Stoelkleuren (zoals een Zetelverdeling): de Stoel straalt in de nieuwe kleur
        zet_persoonskaart(&mut app, 2, Some(stoel), vec!["#d4202c".parse().unwrap(); 150]);
        wacht(&mut app, 2.0);
        let rood_half = Srgba::from(LinearRgba::from(Color::srgb_u8(0xd4, 0x20, 0x2c)) * 0.5);
        assert!(ongeveer(leerkleuren(&mut app)[i], rood_half));
        assert!(straling(&mut app)[i] > 0.0);

        // zonder Persoonskaart straalt niets meer
        zet_persoonskaart(&mut app, 3, None, vec!["#d4202c".parse().unwrap(); 150]);
        wacht(&mut app, 1.0);
        assert!(straling(&mut app).iter().all(|e| *e == 0.0));
        assert!(ongeveer(leerkleuren(&mut app)[i], rood_half));
    }
}
