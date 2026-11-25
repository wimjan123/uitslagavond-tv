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
use gedeeld::sjabloon::{GRIJS, Kleurovergang, OPBOUW_WACHTKLEUR, Overgangsvorm};
use gedeeld::{AANTAL_STOELEN, Kleur, Stoel};
use serde::Deserialize;

use crate::afwerking::{Afgewerkt, AfwerkingPlugin};
use crate::omgeving::{self, OmgevingPlugin, Reflecties};
use crate::volging::Standen;
#[cfg(test)]
use crate::{BREEDTE, HOOGTE};
use crate::kleurgolf::{self, Opbouwdeel};
use crate::lichtkaart;

/// De kom van een plafondspot: dat is wat je van onderen ziet. In Cycles straalt daar de lamp zelf;
/// de lens van het model zit erachter. Zie [`spots_stralen`].
const SPOTKOM: &str = "daklicht_spotkom";
/// Hoe fel de kom van een plafondspot straalt: zoals de lens in het model (`spot`, sterkte 14).
const SPOT_STRAALT: LinearRgba = LinearRgba::rgb(14.0, 13.0, 11.2);
/// Het materiaal van het leer van een Stoel: dat deel krijgt de kleur uit de Stand.
const STOEL_LEER: &str = "stoel_leer";
/// Het wapen op de rugleuning: een eigen laag over het leer, die nooit de kleur van de Stoel
/// krijgt en niet meedimt. Het straalt zacht wit, zodat het op elke kleur licht en scherp leest.
const STOEL_WAPEN: &str = "stoel_wapen";
const WAPEN_STRAALT: LinearRgba = LinearRgba::rgb(0.22, 0.22, 0.21);
/// Het groene bordje van de nooduitgang: in het model zo fel dat het in een close-up de aandacht
/// trekt (council-ronde 3). Dit deel van zijn licht blijft over.
const NOODUITGANG: &str = "nooduitgang";
const NOODUITGANG_LICHT: f32 = 0.2;

pub struct ZaalPlugin {
    /// Map met de bestanden uit `zaal/maak.sh`; `None` voor een Renderer zonder Zaal.
    pub map: Option<PathBuf>,
}

impl Plugin for ZaalPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ZaalKlaar(self.map.is_none()))
            .init_resource::<StoelMaterialen>()
            .init_resource::<Golf>()
            .init_resource::<Opbouwstand>()
            .init_resource::<Zaaldimmer>()
            .init_resource::<Uitlichting>()
            .add_systems(
                Update,
                (
                    stel_stof_en_hout_bij,
                    ((herken_stoelen, kleur_stoelen, licht_stoel_op).chain(), dim_zaal, geef_stoelen_licht),
                )
                    .chain()
                    .after(Afgewerkt),
            );
        if let Some(map) = &self.map {
            app.add_plugins((OmgevingPlugin, AfwerkingPlugin, crate::occlusie::OcclusiePlugin, crate::schil::SchilPlugin, crate::studio::StudioPlugin, crate::lens::LensPlugin))
                .insert_resource(ZaalMap(map.clone()))
                .init_resource::<ModelGeladen>()
                .insert_resource(GlobalAmbientLight {
                    color: Color::srgb(1.0, 0.95, 0.88),
                    brightness: crate::beeld::BEELD.omgeving,
                    affects_lightmapped_meshes: false,
                })
                .add_systems(Startup, (laad.after(crate::beeld::bouw), zet_lijn_bij_76))
                .add_systems(Update, (stel_materialen_bij, spots_stralen.before(dim_zaal), meld_klaar, toon_lijn_bij_76));
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
                            commands.entity(d).insert((
                                Lightmap {
                                    image: lichtkaart.clone(),
                                    uv_rect: Rect::new(0.0, 0.0, 1.0, 1.0),
                                    bicubic_sampling: true,
                                },
                                crate::schil::Splitsen,
                            ));
                            gebakken += 1;
                        }
                    }
                }
                if namen.get(e).is_ok_and(|n| crate::schil::in_de_schil_volgens_naam(n.as_str())) {
                    for d in kinderen.iter_descendants(e).chain([e]) {
                        if meshes.contains(d) {
                            commands.entity(d).insert(crate::schil::naar_de_schil());
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
    /// Hoe fel elke Stoel toen straalde.
    van_straalt: Vec<LinearRgba>,
    /// De kleur uit de Stand per Stoel; `None`: het eigen leer.
    naar: Vec<Option<Kleur>>,
    /// Plaats van elke Stoel van links naar rechts in beeld.
    rang: Vec<usize>,
    vorm: Overgangsvorm,
    tijd: f32,
    /// De duur van de overgang; bij de Opbouw de tijd per Partij.
    duur: f32,
    /// De stappen van de Opbouw.
    opbouw: kleurgolf::Opbouw,
    bezig: bool,
    /// De Kleurkracht (0 tot 1) waarmee de materialen per kleur gemaakt zijn.
    kracht: f32,
}

/// Waar een Stoel in de Kleurovergang is.
enum Stoelstand {
    /// Nog niet aan de beurt: hij houdt wat hij had.
    Wacht,
    /// Onderweg: dit leer en deze straling.
    Onderweg(LinearRgba, LinearRgba),
    /// Klaar: de nieuwe kleur.
    Klaar,
}

impl Golf {
    /// Waar Stoel `i` nu is. `lineair` en `straalt` geven het leer en de straling van een kleur.
    fn stand(
        &self,
        i: usize,
        lineair: impl Fn(Option<Kleur>) -> LinearRgba,
        straalt: impl Fn(Option<Kleur>) -> LinearRgba,
    ) -> Stoelstand {
        if !self.bezig {
            return Stoelstand::Klaar;
        }
        let (van, van_straalt, naar) = (self.van[i], self.van_straalt[i], self.naar[i]);
        let meng = |a: LinearRgba, a_straalt: LinearRgba, b: Option<Kleur>, f: f32| {
            Stoelstand::Onderweg(a.mix(&lineair(b), f), a_straalt.mix(&straalt(b), f))
        };
        match self.vorm {
            Overgangsvorm::Golf | Overgangsvorm::Vervagen => {
                let f = kleurgolf::voortgang(self.vorm, self.rang[i], AANTAL_STOELEN - 1, self.tijd, self.duur);
                if f >= 1.0 || van == lineair(naar) {
                    Stoelstand::Klaar
                } else if f <= 0.0 {
                    Stoelstand::Wacht
                } else {
                    meng(van, van_straalt, naar, f)
                }
            }
            Overgangsvorm::Opbouw => {
                let wacht = Some(OPBOUW_WACHTKLEUR);
                match self.opbouw.deel(i, self.tijd) {
                    Opbouwdeel::Donker(f) if f <= 0.0 => Stoelstand::Wacht,
                    // zonder kleur gaat hij meteen naar zijn eigen leer
                    Opbouwdeel::Donker(f) if naar.is_none() && f >= 1.0 => Stoelstand::Klaar,
                    Opbouwdeel::Donker(f) if naar.is_none() => meng(van, van_straalt, None, f),
                    // donker zonder straling
                    Opbouwdeel::Donker(f) => {
                        Stoelstand::Onderweg(van.mix(&lineair(wacht), f), van_straalt.mix(&LinearRgba::BLACK, f))
                    }
                    Opbouwdeel::Oplichten(f) => {
                        let (g, d) = (self.opbouw.gloed(i, self.tijd), self.opbouw.gedimd(i, self.tijd));
                        if f >= 1.0 && g <= 0.0 && d <= 0.0 {
                            return Stoelstand::Klaar;
                        }
                        // eerst de kleur, dan gedimd naar grijs, dan de flits van de stap
                        let gedimd = naar.map_or(lineair(None), gedimd_leer);
                        let leer = lineair(wacht)
                            .mix(&lineair(naar), f)
                            .mix(&gedimd, d)
                            .mix(&lineair(Some(Kleur { r: 255, g: 255, b: 255 })), GLOED_LICHT * g);
                        let eigen = naar.filter(|k| *k != GRIJS).map_or(LinearRgba::BLACK, gekalibreerd);
                        let straling = LinearRgba::BLACK.mix(&straalt(naar), f) * (1.0 - d) + eigen * (GLOED_STRAALT * g);
                        Stoelstand::Onderweg(leer, straling)
                    }
                }
            }
        }
    }

    /// Hoe lang de overgang in zijn geheel duurt.
    fn totaal(&self) -> f32 {
        match self.vorm {
            Overgangsvorm::Opbouw => self.opbouw.duur(),
            _ => self.duur,
        }
    }
}

/// Waar de Opbouw in de Zaal is, voor het grote label ([`gedeeld::sjabloon::Bloksoort::Opbouwlabel`])
/// en de Partijlabels die op het eind wachten.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct Opbouwstand {
    /// Er loopt een Opbouw.
    pub bezig: bool,
    /// De kleur die nu oplicht, met de seconden sinds hij begon; `None` als de Stoelen nog donker
    /// worden.
    pub stap: Option<(Kleur, f32)>,
    /// Seconden per Partij.
    pub per_stap: f32,
}

/// Een Stoel die niet meedoet ([`GRIJS`] in de Stand: geen Partij, niet uitgelicht, geen stem)
/// is in de Zaal donkergrijs en straalt niet, zodat de gekleurde Stoelen eruit springen. Niet
/// zwart: dat las als "daar zit niemand" (council-ronde 3). Het grijs zelf blijft voor de Graphics.
const STOEL_UIT: Kleur = Kleur { r: 0x4c, g: 0x4f, b: 0x55 };

/// Hoe een partijkleur leer wordt, zodat de Stoel in beeld de kleur van de Partij heeft en niet
/// een zalmkleurige of bleke variant ervan (council-ronde 3). Het gebakken licht, de belichting en
/// de toonweergave maken een kleur donkerder en grijzer, en een lichte kleur meer dan een donkere.
/// Het leer krijgt hem daarom lichter en voller mee, naar de lichtheid L van de kleur (Oklab, 0
/// tot 1): de lichtheid keer `licht + licht_l * L`, de kleurigheid keer `kleur + kleur_l * L +
/// kleur_c * C`. Gemeten op de render van Standpunt A met 24 kleuren verspreid over alle Stoelen,
/// ook de achterste rij; per kleur een paar keer bijgesteld tot het beeld klopte, en daar deze
/// rechte lijn door gelegd (`scripts/kleurmeting/`, council-ronde 4). Gemeten bij Kleurkracht 50,
/// de standaard; hoger straalt het leer iets meer ([`KLEUR_STRAALT`]).
#[derive(Clone, Copy, Debug)]
struct Kalibratie {
    licht: f32,
    licht_l: f32,
    kleur: f32,
    kleur_l: f32,
    kleur_c: f32,
    /// Het leer: de gekalibreerde kleur keer dit. De leertextuur zelf is licht (0,7 tot 1).
    albedo: f32,
    /// Hoe fel het leer altijd in zijn eigen kleur straalt, bovenop het licht uit de bak: zo houdt
    /// het zijn kleur ook waar de bak donker is.
    straalt: f32,
}

const KALIBRATIE: Kalibratie =
    Kalibratie { licht: 0.87, licht_l: 0.88, kleur: 0.2, kleur_l: 3.1, kleur_c: -1.5, albedo: 0.6, straalt: 0.15 };

/// De kalibratie, of voor het meten (`scripts/kleurmeting/`) een andere uit
/// `UITSLAGAVOND_KAL`: de zeven getallen van [`Kalibratie`] met komma's ertussen.
fn kalibratie() -> Kalibratie {
    static K: std::sync::OnceLock<Kalibratie> = std::sync::OnceLock::new();
    *K.get_or_init(|| {
        let Ok(t) = std::env::var("UITSLAGAVOND_KAL") else { return KALIBRATIE };
        let v: Vec<f32> = t.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        if v.len() != 7 {
            warn!("UITSLAGAVOND_KAL heeft geen zeven getallen: de gewone kalibratie");
            return KALIBRATIE;
        }
        Kalibratie { licht: v[0], licht_l: v[1], kleur: v[2], kleur_l: v[3], kleur_c: v[4], albedo: v[5], straalt: v[6] }
    })
}

/// Voor het meten per kleur (`scripts/kleurmeting/`): uit `UITSLAGAVOND_KAL_PER` de factoren op
/// lichtheid en kleurigheid van losse kleuren, als `rrggbb:licht:kleur` met `;` ertussen.
fn kalibratie_per_kleur() -> &'static HashMap<Kleur, (f32, f32)> {
    static K: std::sync::OnceLock<HashMap<Kleur, (f32, f32)>> = std::sync::OnceLock::new();
    K.get_or_init(|| {
        let t = std::env::var("UITSLAGAVOND_KAL_PER").unwrap_or_default();
        t.split(';')
            .filter_map(|d| {
                let v: Vec<&str> = d.split(':').collect();
                let k: Kleur = format!("#{}", v.first()?).parse().ok()?;
                Some((k, (v.get(1)?.parse().ok()?, v.get(2)?.parse().ok()?)))
            })
            .collect()
    })
}

/// Een kleur met zijn lichtheid en kleurigheid (Oklab) keer deze factoren, lineair. Een kanaal
/// onder nul wordt nul; boven 1 mag, want het leer krijgt maar een deel ervan.
fn in_oklab(k: Kleur, licht: f32, kleur: f32) -> LinearRgba {
    let mut c = Oklcha::from(Srgba::rgb_u8(k.r, k.g, k.b));
    c.lightness *= licht;
    c.chroma *= kleur;
    let l = LinearRgba::from(c);
    LinearRgba::rgb(l.red.max(0.0), l.green.max(0.0), l.blue.max(0.0))
}

/// De factoren op lichtheid en kleurigheid voor deze kleur, zie [`Kalibratie`].
fn factoren(k: Kleur) -> (f32, f32) {
    let kal = kalibratie();
    let c = Oklcha::from(Srgba::rgb_u8(k.r, k.g, k.b));
    (kal.licht + kal.licht_l * c.lightness, (kal.kleur + kal.kleur_l * c.lightness + kal.kleur_c * c.chroma).max(1.0))
}

/// Een partijkleur zoals het leer hem meekrijgt (lineair), zie [`Kalibratie`].
fn gekalibreerd(k: Kleur) -> LinearRgba {
    let (licht, kleur) = kalibratie_per_kleur().get(&k).copied().unwrap_or_else(|| factoren(k));
    in_oklab(k, licht, kleur)
}

/// De kleur van het leer bij een kleur uit de Stand (lineair).
fn leer_in_de_zaal(k: Kleur) -> LinearRgba {
    if k == GRIJS {
        return in_oklab(STOEL_UIT, 1.0, 1.0) * kalibratie().albedo;
    }
    gekalibreerd(k) * kalibratie().albedo
}

/// Hoe het leer straalt bij een kleur uit de Stand en deze Kleurkracht (0 tot 1). Een Stoel die
/// niet meedoet straalt niet.
fn straling_van(k: Kleur, kracht: f32) -> LinearRgba {
    if k == GRIJS {
        return LinearRgba::BLACK;
    }
    gekalibreerd(k) * (kalibratie().straalt + kracht * KLEUR_STRAALT)
}

/// Bij de Opbouw: een Partij na haar stap wordt grijs en donker, met nog een kwart van haar kleur
/// in beeld (council-ronde 3: met de tint erin waren kleine stappen als JA21 en BBB niet te
/// vinden). Een donkere kleur verliest in beeld minder van zijn kleurigheid dan een volle, dus het
/// leer krijgt minder dan een kwart mee.
const GEDIMD_KLEUR: f32 = 0.12;
const GEDIMD_LICHT: f32 = 0.5;

/// Het leer van een gedimde Partij in de Opbouw (lineair).
fn gedimd_leer(k: Kleur) -> LinearRgba {
    let (licht, kleur) = factoren(k);
    in_oklab(k, licht * GEDIMD_LICHT, kleur * GEDIMD_KLEUR) * kalibratie().albedo
}

/// Hoe fel het leer bij Kleurkracht 100 extra straalt in zijn eigen kleur. Laag, anders leest
/// de stof als plastic dat zelf licht geeft.
const KLEUR_STRAALT: f32 = 0.12;
/// Bij de Opbouw: hoeveel de Partij die aan de beurt is naar wit opschuift, en hoe fel ze extra
/// straalt in haar eigen kleur.
const GLOED_LICHT: f32 = 0.35;
const GLOED_STRAALT: f32 = 0.4;
/// Hoeveel licht de rest van de Zaal bij Kleurkracht 100 overhoudt, zolang er Stoelen gekleurd
/// zijn. Daartussen daalt het licht in gelijke stappen voor het oog: bij 50 ongeveer een derde.
const MINSTE_LICHT: f32 = 0.1;

/// De Kleurkracht van de Ether, van 0 tot 1.
fn kleurkracht(standen: &Standen) -> f32 {
    let k = standen.huidig.as_ref().map_or(gedeeld::sjabloon::KLEURKRACHT, |s| s.stand.stijl_van_ether().kleurkracht);
    f32::from(k.min(100)) / 100.0
}

/// De Kleurovergang van de Stand en hoe lang die duurt, in seconden.
fn kleurovergang(standen: &Standen) -> (Overgangsvorm, f32) {
    let (overgang, snelheid) = standen
        .huidig
        .as_ref()
        .map_or((Kleurovergang::default(), 100), |s| (s.stand.kleurovergang, s.stand.stijl_van_ether().snelheid));
    (overgang.vorm, kleurgolf::duur(overgang, snelheid))
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
    mut opbouwstand: ResMut<Opbouwstand>,
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
    let lineair = |k: Option<Kleur>| k.map_or(eigen_leer, leer_in_de_zaal);
    // Het leer straalt zacht in zijn eigen kleur, naar de Kleurkracht; het eigen leer en een
    // Stoel die niet meedoet niet.
    let kracht = kleurkracht(&standen);
    let straalt = |k: Option<Kleur>| k.map_or(LinearRgba::BLACK, |k| straling_van(k, kracht));
    if kracht != golf.kracht {
        // een andere Kleurkracht: de materialen per kleur stralen anders
        for (kleur, h) in &pm.per_kleur {
            if let Some(mut m) = materialen.get_mut(h) {
                m.emissive = straalt(Some(*kleur));
            }
        }
        golf.kracht = kracht;
    }
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
            golf.van_straalt = doel.iter().map(|k| straalt(*k)).collect();
            golf.naar = doel;
            golf.bezig = false;
            golf.begonnen = standen.huidig.is_some();
        } else {
            let (vorm, duur) = kleurovergang(&standen);
            // Opbouw overslaan: dezelfde kleuren, de rest met een andere overgang
            let overslaan = golf.bezig && golf.vorm == Overgangsvorm::Opbouw && vorm != Overgangsvorm::Opbouw;
            if doel != golf.naar || overslaan {
                // vanaf wat nu te zien is, ook midden in een vorige golf
                let nu: Vec<(LinearRgba, LinearRgba)> = (0..AANTAL_STOELEN)
                    .map(|i| match golf.stand(i, &lineair, &straalt) {
                        Stoelstand::Wacht => (golf.van[i], golf.van_straalt[i]),
                        Stoelstand::Onderweg(k, st) => (k, st),
                        Stoelstand::Klaar => (lineair(golf.naar[i]), straalt(golf.naar[i])),
                    })
                    .collect();
                (golf.van, golf.van_straalt) = nu.into_iter().unzip();
                golf.naar = doel;
                golf.tijd = 0.0;
                (golf.vorm, golf.duur) = (vorm, duur);
                golf.opbouw = if vorm == Overgangsvorm::Opbouw {
                    kleurgolf::Opbouw::nieuw(&golf.naar, &golf.rang, duur)
                } else {
                    kleurgolf::Opbouw::default()
                };
                golf.bezig = true;
            }
        }
    }

    for (stoel, mut materiaal) in &mut leer {
        let i = stoel.index;
        let (true, Some(naar)) = (i < golf.van.len(), golf.naar.get(i).copied()) else {
            continue;
        };
        let gewenst = match golf.stand(i, &lineair, &straalt) {
            Stoelstand::Klaar => match naar {
                None => origineel.clone(),
                Some(kleur) => match pm.per_kleur.get(&kleur) {
                    Some(h) => h.clone(),
                    None => {
                        let Some(mut m) = materialen.get(&origineel).cloned() else {
                            continue;
                        };
                        m.base_color = lineair(Some(kleur)).into();
                        m.emissive = straalt(Some(kleur));
                        let h = materialen.add(m);
                        pm.per_kleur.insert(kleur, h.clone());
                        h
                    }
                },
            },
            // nog niet aan de beurt: hij houdt wat hij had
            Stoelstand::Wacht => continue,
            // midden in de golf: een eigen materiaal voor deze Stoel, tot hij klaar is
            Stoelstand::Onderweg(kleur, straling) => {
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
                m.base_color = kleur.into();
                m.emissive = straling;
            }
            h
            }
        };
        if materiaal.0 != gewenst {
            materiaal.0 = gewenst;
        }
    }
    if golf.bezig && golf.tijd >= golf.totaal() {
        golf.bezig = false;
        golf.van = golf.naar.iter().map(|k| lineair(*k)).collect();
        golf.van_straalt = golf.naar.iter().map(|k| straalt(*k)).collect();
    }
    let stand = if golf.bezig && golf.vorm == Overgangsvorm::Opbouw {
        Opbouwstand {
            bezig: true,
            stap: golf.opbouw.stap(golf.tijd).map(|(k, t)| (golf.opbouw.stappen[k], t)),
            per_stap: golf.opbouw.per_stap,
        }
    } else {
        Opbouwstand::default()
    };
    if *opbouwstand != stand {
        *opbouwstand = stand;
    }
}

/// Hoe licht de rest van de Zaal is (alles behalve het leer van de Stoelen): 1 zoals gebakken,
/// lager zolang er Stoelen gekleurd zijn, naar de Kleurkracht. Het licht verandert mee met de
/// Kleurovergang.
#[derive(Resource)]
struct Zaaldimmer {
    /// De kleur en de straling van elk materiaal van de Zaal zoals het model ze gaf.
    origineel: HashMap<AssetId<StandardMaterial>, (LinearRgba, LinearRgba)>,
    van: f32,
    naar: f32,
    nu: f32,
    /// Hoe ver het leer buiten de 150 Stoelen (de kabinetsbank) naar donker neutraal is: 1
    /// zolang de Stoelen gekleurd zijn, zodat het blauw daar niet leest als PVV of JA21.
    neutraal_van: f32,
    neutraal_naar: f32,
    neutraal: f32,
    tijd: f32,
    duur: f32,
    begonnen: bool,
    /// Er zijn delen waarvan het materiaal nog niet geladen was: volgend beeld opnieuw.
    onaf: bool,
    /// De sterkte van elke lamp en van de reflecties zoals de Zaal ze gaf.
    lampen: HashMap<Entity, f32>,
    /// Het leer buiten de 150 Stoelen, als eigen materiaal.
    ander_leer: Option<Handle<StandardMaterial>>,
}

impl Default for Zaaldimmer {
    fn default() -> Self {
        Zaaldimmer {
            origineel: HashMap::new(),
            van: 1.0,
            naar: 1.0,
            nu: 1.0,
            neutraal_van: 0.0,
            neutraal_naar: 0.0,
            neutraal: 0.0,
            tijd: 0.0,
            duur: 1.0,
            begonnen: false,
            onaf: false,
            lampen: HashMap::new(),
            ander_leer: None,
        }
    }
}

/// Het leer van de kabinetsbank zolang de Stoelen gekleurd zijn: grijs, met dit deel van zijn
/// helderheid.
const KABINET_GRIJS: f32 = 0.8;

/// Hoe licht de rest van de Zaal moet zijn bij deze Stand.
fn zaallicht(gekleurd: bool, kracht: f32) -> f32 {
    if gekleurd { MINSTE_LICHT.powf(kracht) } else { 1.0 }
}

/// Dimt de rest van de Zaal zolang er Stoelen gekleurd zijn, zodat hun kleuren opvallen: het
/// gebakken licht en het live licht samen, door de kleur van elk materiaal te verlagen. De
/// eerste Stand na het opstarten komt meteen, zonder overgang.
fn dim_zaal(
    standen: Res<Standen>,
    tijd: Res<Time>,
    mut delen: Query<(Entity, &GltfMaterialName, &mut MeshMaterial3d<StandardMaterial>), Without<StoelLeer>>,
    ouders: Query<&ChildOf>,
    namen: Query<&Name>,
    nieuw: Query<(), Added<GltfMaterialName>>,
    mut licht: ParamSet<(
        Query<(Entity, &mut PointLight)>,
        Query<(Entity, &mut SpotLight)>,
        Query<(Entity, &mut DirectionalLight)>,
        Query<(Entity, &mut EnvironmentMapLight)>,
        Query<(), Or<(Added<PointLight>, Added<SpotLight>, Added<DirectionalLight>, Added<EnvironmentMapLight>)>>,
    )>,
    pm: Res<StoelMaterialen>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut d: ResMut<Zaaldimmer>,
) {
    if standen.is_changed() {
        let gekleurd = standen.huidig.as_ref().is_some_and(|s| !s.stand.stoelkleuren.is_empty());
        let doel = zaallicht(gekleurd, kleurkracht(&standen));
        let neutraal = if gekleurd { 1.0 } else { 0.0 };
        if !d.begonnen {
            (d.van, d.naar, d.nu) = (doel, doel, doel);
            (d.neutraal_van, d.neutraal_naar, d.neutraal) = (neutraal, neutraal, neutraal);
            d.begonnen = standen.huidig.is_some();
        } else if doel != d.naar || neutraal != d.neutraal_naar {
            (d.van, d.neutraal_van) = (d.nu, d.neutraal);
            (d.naar, d.neutraal_naar) = (doel, neutraal);
            d.tijd = 0.0;
            d.duur = kleurovergang(&standen).1;
        }
    }
    let vorig = (d.nu, d.neutraal);
    if d.nu != d.naar || d.neutraal != d.neutraal_naar {
        d.tijd += tijd.delta_secs();
        let f = kleurgolf::zacht((d.tijd / d.duur.max(1e-3)).clamp(0.0, 1.0));
        if f >= 1.0 {
            (d.nu, d.neutraal) = (d.naar, d.neutraal_naar);
        } else {
            d.nu = d.van + (d.naar - d.van) * f;
            d.neutraal = d.neutraal_van + (d.neutraal_naar - d.neutraal_van) * f;
        }
    }
    if (d.nu, d.neutraal) == vorig && nieuw.is_empty() && licht.p4().is_empty() && !d.onaf {
        return;
    }
    let (nu, neutraal) = (d.nu, d.neutraal);
    let ander_leer = d.ander_leer.as_ref().map(|h| h.id());
    d.onaf = false;

    for (e, naam, mut materiaal) in &mut delen {
        if naam.0 == STOEL_WAPEN {
            // het wapen dimt niet mee: anders wordt het op een gekleurde Stoel een donkere vlek
            continue;
        }
        if naam.0 == STOEL_LEER {
            let kamerstoel = ouders
                .iter_ancestors(e)
                .any(|a| namen.get(a).is_ok_and(|n| stoel_uit_naam(n.as_str()).is_some()));
            if kamerstoel {
                // het leer van een Kamerstoel dat herken_stoelen nog niet merkte
                continue;
            }
            // leer buiten de 150 Stoelen, zoals de stoelen van het kabinet: een eigen kopie,
            // zodat het leer van de Stoelen zelf niet dimt
            if pm.origineel.as_ref().is_none_or(|o| o.id() == materiaal.0.id()) {
                let Some(m) = materialen.get(&materiaal.0).cloned() else {
                    d.onaf = true;
                    continue;
                };
                let kopie = d.ander_leer.get_or_insert_with(|| materialen.add(m)).clone();
                materiaal.0 = kopie;
            }
        }
        let id = materiaal.0.id();
        if pm.origineel.as_ref().is_some_and(|o| o.id() == id) {
            continue;
        }
        let origineel = match d.origineel.get(&id) {
            Some(k) => *k,
            None => {
                let Some(m) = materialen.get(id) else {
                    d.onaf = true;
                    continue;
                };
                let k = (LinearRgba::from(m.base_color), m.emissive);
                d.origineel.insert(id, k);
                k
            }
        };
        let (kleur, straling) = origineel;
        if let Some(mut m) = materialen.get_mut(id) {
            // de doorzichtigheid laat stel_materialen_bij staan (glas)
            let alpha = m.base_color.alpha();
            let mut k = LinearRgba::new(kleur.red * nu, kleur.green * nu, kleur.blue * nu, alpha);
            if Some(id) == ander_leer {
                // de kabinetsbank: naar grijs met dezelfde helderheid
                let grijs = helderheid(k) * KABINET_GRIJS;
                k = k.mix(&LinearRgba::new(grijs, grijs, grijs, alpha), neutraal);
            }
            if LinearRgba::from(m.base_color) != k {
                m.base_color = k.into();
            }
            // wat zelf licht geeft (de plafondspots) dimt mee: anders blijven die als felle
            // vlekken in het plafond staan terwijl de rest donker wordt
            let s = (straling * nu).with_alpha(straling.alpha);
            if m.emissive != s {
                m.emissive = s;
            }
        }
    }
    // De glans van de lampen en de reflecties hangen niet van de kleur van het materiaal af:
    // die gaan apart mee.
    for (e, mut l) in &mut licht.p0() {
        l.intensity = d.lampen.entry(e).or_insert(l.intensity).to_owned() * nu;
    }
    for (e, mut l) in &mut licht.p1() {
        l.intensity = d.lampen.entry(e).or_insert(l.intensity).to_owned() * nu;
    }
    for (e, mut l) in &mut licht.p2() {
        l.illuminance = d.lampen.entry(e).or_insert(l.illuminance).to_owned() * nu;
    }
    for (e, mut l) in &mut licht.p3() {
        l.intensity = d.lampen.entry(e).or_insert(l.intensity).to_owned() * nu;
    }
}

/// De lijn bij Zetel 76 in de Coalitiebouwer (besluit #7): een lichtstrook in Accent tussen de
/// 75e en de 76e Stoel van links, plat op de vloer van het middenpad
/// ([`gedeeld::uitslagindeling::lijn_bij_76`]). `zicht` loopt van 0 (weg) tot 1 (helemaal);
/// `stil` telt hoe lang de camera al stilstaat.
#[derive(Component)]
struct LijnBij76 {
    zicht: f32,
    stil: f32,
    camera: Option<Transform>,
}

/// Breedte van de lijn bij 76, in meter: smal, zodat hij leest als een lijn en niet als een strook
/// (council-ronde 3).
const LIJN_76_BREEDTE: f32 = 0.1;
/// Het getal 76 klein op de vloer aan het voeteind van de lijn: zo hoog (meter), en zo ver voor
/// het begin van de lijn.
const GETAL_76_HOOGTE: f32 = 0.6;
const GETAL_76_VOOR_LIJN: f32 = 0.12;
/// Zo lang groeit de lijn van voor naar achter, of krimpt hij weer, bij snelheid 100.
const LIJN_76_GROEIT: f32 = 0.6;
/// Zo lang staat de camera stil voor de lijn verschijnt, in seconden: tijdens een camerarit
/// leest een lijn in de Zaal als een losse prop.
const LIJN_76_NA_STIL: f32 = 0.25;
/// Langzamer dan dit (meter en radialen per seconde) telt de camera als stil, ook bij langzaam
/// schuiven na aankomst.
const STIL_M_PER_S: f32 = 0.15;
const STIL_RAD_PER_S: f32 = 0.05;
/// Hoe fel de lijn straalt in Accent: iets boven de drempel van de gloed, zodat hij leest als licht.
const LIJN_76_LICHT: f32 = 1.6;

/// De strook over de vloer van de lijn bij 76, tot `deel` (0 tot 1) van zijn lengte van voor
/// naar achter.
fn lijnstrook(lijn: &[Vec3], deel: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let lengte: f32 = lijn.windows(2).map(|p| p[0].distance(p[1])).sum();
    let mut over = lengte * deel.clamp(0.0, 1.0);
    // opzij: horizontaal, loodrecht op de lijn
    let langs = (lijn[lijn.len() - 1] - lijn[0]).with_y(0.0).normalize_or(Vec3::NEG_Z);
    let opzij = Vec3::Y.cross(langs) * (LIJN_76_BREEDTE / 2.0);
    let (mut punten, mut normalen, mut driehoeken) = (Vec::new(), Vec::new(), Vec::new());
    for paar in lijn.windows(2) {
        let (a, mut b) = (paar[0], paar[1]);
        let stuk = a.distance(b);
        if over <= 0.0 || stuk <= 0.0 {
            break;
        }
        if stuk > over {
            b = a + (b - a) * (over / stuk);
        }
        over -= stuk;
        // een trede kijkt omhoog, een stootbord naar voren
        let normaal = if (b.y - a.y).abs() > 1e-4 { -langs } else { Vec3::Y };
        let i = punten.len() as u32;
        punten.extend([a - opzij, a + opzij, b + opzij, b - opzij].map(|p| p.to_array()));
        normalen.extend([normaal.to_array(); 4]);
        driehoeken.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, punten)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normalen)
        .with_inserted_indices(Indices::U32(driehoeken))
}

fn lijn_76_punten() -> Vec<Vec3> {
    gedeeld::uitslagindeling::lijn_bij_76().into_iter().map(Vec3::from).collect()
}

/// Het getal 76 als beeld: wit, met de dekking van de letters in de alfa.
fn getal_76_beeld() -> Image {
    use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let letter = FontRef::try_from_slice(include_bytes!("../assets/fonts/NunitoSans-ExtraBold.ttf"))
        .expect("NunitoSans ExtraBold");
    let schaal = PxScale::from(192.0);
    let geschaald = letter.as_scaled(schaal);
    let rand = 8.0;
    let mut x = rand;
    let mut tekens = Vec::new();
    for c in "76".chars() {
        let id = geschaald.glyph_id(c);
        tekens.push(id.with_scale_and_position(schaal, point(x, rand + geschaald.ascent())));
        x += geschaald.h_advance(id);
    }
    let (b, h) = ((x + rand).ceil() as usize, (geschaald.ascent() - geschaald.descent() + 2.0 * rand).ceil() as usize);
    let mut alfa = vec![0.0_f32; b * h];
    for t in tekens {
        if let Some(o) = letter.outline_glyph(t) {
            let grens = o.px_bounds();
            o.draw(|gx, gy, dekking| {
                let (px, py) = (grens.min.x as i64 + gx as i64, grens.min.y as i64 + gy as i64);
                if (0..b as i64).contains(&px) && (0..h as i64).contains(&py) {
                    let a = &mut alfa[py as usize * b + px as usize];
                    *a = a.max(dekking);
                }
            });
        }
    }
    let data = alfa.iter().flat_map(|a| [255, 255, 255, (a.clamp(0.0, 1.0) * 255.0).round() as u8]).collect();
    Image::new(
        Extent3d { width: b as u32, height: h as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Het vlak voor het getal 76: plat op de vloer voor het begin van de lijn, te lezen vanaf de
/// kant van de Voorzitter (de bovenkant van het getal wijst langs de lijn naar achteren).
fn getal_76_vlak(lijn: &[Vec3], verhouding: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let langs = (lijn[1] - lijn[0]).with_y(0.0).normalize_or(Vec3::NEG_Z);
    let rechts = langs.cross(Vec3::Y);
    let (h, b) = (GETAL_76_HOOGTE, GETAL_76_HOOGTE * verhouding);
    let midden = lijn[0] - langs * (GETAL_76_VOOR_LIJN + h / 2.0);
    let hoek = |x: f32, y: f32| (midden + rechts * (x * b / 2.0) + langs * (y * h / 2.0)).to_array();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![hoek(-1.0, 1.0), hoek(1.0, 1.0), hoek(1.0, -1.0), hoek(-1.0, -1.0)])
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; 4])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
        .with_inserted_indices(Indices::U32(vec![0, 2, 1, 0, 3, 2]))
}

/// Het getal 76 op de vloer bij de lijn bij 76.
#[derive(Component)]
struct Getal76;

fn zet_lijn_bij_76(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut beelden: ResMut<Assets<Image>>,
) {
    let beeld = getal_76_beeld();
    let verhouding = beeld.width() as f32 / beeld.height() as f32;
    commands.spawn((
        Getal76,
        Mesh3d(meshes.add(getal_76_vlak(&lijn_76_punten(), verhouding))),
        MeshMaterial3d(materialen.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.0),
            base_color_texture: Some(beelden.add(beeld)),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform::IDENTITY,
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
    ));
    commands.spawn((
        LijnBij76 { zicht: 0.0, stil: 0.0, camera: None },
        Mesh3d(meshes.add(lijnstrook(&lijn_76_punten(), 0.0))),
        MeshMaterial3d(materialen.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform::IDENTITY,
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
    ));
}

/// Of de camera tussen twee beelden stilstond (of heel langzaam schoof).
fn camera_stil(vorig: &Transform, nu: &Transform, dt: f32) -> bool {
    let dt = dt.max(1e-4);
    let schuift = vorig.translation.distance(nu.translation) / dt;
    let draait = vorig.rotation.angle_between(nu.rotation) / dt;
    schuift < STIL_M_PER_S && draait < STIL_RAD_PER_S
}

/// Eén beeld verder voor de lijn bij 76: hoe ver hij zichtbaar is (0 tot 1) en hoe lang de camera
/// al stilstaat, met `stap` het deel dat hij in dit beeld groeit of krimpt. Stilstaan telt pas
/// zodra de Coalitie in de Ether staat: stond de camera daarvoor al stil, dan rijdt hij er nu
/// misschien net mee weg naar zijn Standpunt.
fn stap_lijn_76(zicht: f32, stil: f32, coalitie: bool, camera_stil: bool, dt: f32, stap: f32) -> (f32, f32) {
    let stil = if coalitie && camera_stil { stil + dt } else { 0.0 };
    let zicht = if coalitie && (zicht > 0.0 || stil >= LIJN_76_NA_STIL) {
        (zicht + stap).min(1.0)
    } else if coalitie {
        zicht
    } else {
        (zicht - stap).max(0.0)
    };
    (zicht, stil)
}

/// De lijn bij 76 staat er zolang de Graphic in de Ether een Coalitie of Coalitiebalk is, in de
/// kleur Accent van de Stijl. Hij verschijnt pas als de camera stilstaat, groeit dan van voor naar
/// achter, en krimpt terug als hij weggaat. Wat er al staat, blijft staan als de camera weer gaat
/// rijden.
fn toon_lijn_bij_76(
    standen: Res<Standen>,
    tijd: Res<Time>,
    camera: Query<&Transform, (With<crate::beeld::HoofdCamera>, Without<LijnBij76>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut lijn: Query<(&mut LijnBij76, &Mesh3d, &mut Visibility, &MeshMaterial3d<StandardMaterial>)>,
    mut getal: Query<(&mut Visibility, &MeshMaterial3d<StandardMaterial>), (With<Getal76>, Without<LijnBij76>)>,
    mut punten: Local<Vec<Vec3>>,
) {
    let Some(stand) = standen.huidig.as_ref().map(|s| &s.stand) else {
        return;
    };
    let coalitie = matches!(
        stand.ether.as_ref().map(|e| &e.graphic),
        Some(gedeeld::Graphic::Coalitie { .. } | gedeeld::Graphic::Coalitiebalk { .. })
    );
    if punten.is_empty() {
        *punten = lijn_76_punten();
    }
    let dt = tijd.delta_secs();
    let snelheid = f32::from(stand.stijl.snelheid.max(10)) / 100.0;
    let nu = camera.iter().next().copied();
    for (mut l, mesh, mut zicht, materiaal) in &mut lijn {
        let stil = match (l.camera, nu) {
            (Some(vorig), Some(nu)) => camera_stil(&vorig, &nu, dt),
            _ => true,
        };
        l.camera = nu;
        let vorig = l.zicht;
        (l.zicht, l.stil) = stap_lijn_76(l.zicht, l.stil, coalitie, stil, dt, dt * snelheid / LIJN_76_GROEIT);
        if l.zicht != vorig
            && let Some(mut m) = meshes.get_mut(&mesh.0)
        {
            *m = lijnstrook(&punten, crate::animatie::zacht_uit(l.zicht));
        }
        zicht.set_if_neq(if l.zicht > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
        let accent = stand.stijl.kleur(gedeeld::sjabloon::Stijlkleur::Accent);
        let kleur = LinearRgba::from(Color::srgb_u8(accent.r, accent.g, accent.b)) * LIJN_76_LICHT;
        let kleur = Color::from(kleur.with_alpha(1.0));
        if let Some(m) = materialen.get(&materiaal.0)
            && m.base_color != kleur
            && let Some(mut m) = materialen.get_mut(&materiaal.0)
        {
            m.base_color = kleur;
        }
        // het getal komt op zodra de lijn het voeteind uit is, en gaat met hem weg
        let dekking = crate::animatie::zacht_uit((l.zicht * 3.0).min(1.0));
        for (mut z, m) in &mut getal {
            z.set_if_neq(if dekking > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
            let kleur = kleur.with_alpha(dekking);
            if materialen.get(&m.0).is_some_and(|m| m.base_color != kleur)
                && let Some(mut m) = materialen.get_mut(&m.0)
            {
                m.base_color = kleur;
            }
        }
    }
}

/// Hoe fel de Stoel van de Persoonskaart straalt in de kleur van zijn Partij (of, zonder Partij,
/// in zijn eigen kleur), en welk deel van het gekalibreerde leer hij krijgt. De kalibratie geldt
/// voor het totaalbeeld; in de close-up staat de Stoel recht van voren in het licht, dus minder,
/// zodat hij daar de kleur van de Partij heeft en geen lichtere tint ervan (gemeten bij
/// Bontenbal, CDA). De markering is dat alleen hij kleur heeft.
const UITGELICHT_STRAALT: f32 = 0.08;
const UITGELICHT_LEER: f32 = 0.6;
/// Zo lang duurt het oplichten en uitgaan, in seconden bij snelheid 100.
const UITLICHTEN_S: f32 = 0.5;

/// De Stoel die oplicht (Persoonskaart): welke, hoe ver, en per materiaal van het leer de
/// stralende kopie.
#[derive(Resource, Default)]
pub struct Uitlichting {
    pub stoel: Option<usize>,
    /// 0 uit, 1 helemaal aan.
    sterkte: f32,
    /// De kleur van de Partij van wie er zit.
    kleur: Option<Kleur>,
    /// Van het gewone materiaal naar de stralende kopie, en terug.
    straalt: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
    gewoon: HashMap<AssetId<StandardMaterial>, Handle<StandardMaterial>>,
}

/// De kleur van de Partij van het Kamerlid op een Persoonskaart.
fn partijkleur(stand: &gedeeld::Stand, graphic: &gedeeld::Graphic) -> Option<Kleur> {
    let gedeeld::Graphic::Persoonskaart { gegevens: Some(g), .. } = graphic else {
        return None;
    };
    stand.cijfers.partij(g.partij.as_deref()?)?.kleur.parse().ok()
}

/// Het leer en de straling van de uitgelichte Stoel bij `sterkte` (0 tot 1): het leer gaat naar
/// de kleur van de Partij en straalt zacht in die kleur. Zonder Partij houdt het zijn eigen kleur
/// en straalt daarin.
fn uitgelicht(leer: LinearRgba, partij: Option<Kleur>, sterkte: f32) -> (LinearRgba, LinearRgba) {
    let naar = partij.map_or(leer, |k| leer_in_de_zaal(k) * UITGELICHT_LEER);
    let kleur = leer.mix(&naar, sterkte);
    let eigen = partij.map_or(leer * (1.0 / kalibratie().albedo), gekalibreerd);
    (kleur, eigen.with_alpha(1.0) * (UITGELICHT_STRAALT * sterkte))
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
    let graphic = stand.and_then(|s| {
        if voorbeeld {
            s.klaar.as_ref().map(|k| &k.graphic)
        } else {
            s.ether.as_ref().map(|e| &e.graphic)
        }
    });
    let gewenst = graphic.and_then(|g| g.uitgelichte_stoel()).and_then(|s| s.index());
    if let Some((s, g)) = stand.zip(graphic).filter(|_| gewenst.is_some()) {
        u.kleur = partijkleur(s, g);
    }
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
    let (sterkte, partij) = (u.sterkte, u.kleur);
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
                let (leer, straling) = uitgelicht(LinearRgba::from(basis.base_color), partij, sterkte);
                m.base_color = leer.into();
                m.emissive = straling;
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

/// Het neutrale beukenhout van bladen en randen (#C9A87C): het model gaf ze een oranje of
/// geelgroene kleur, die in close-ups zalmroze en limoengroen las.
const BEUKEN: Srgba = Srgba::rgb(0xC9 as f32 / 255.0, 0xA8 as f32 / 255.0, 0x7C as f32 / 255.0);
/// Het hout dat neutraal beuken wordt: de bureaubladen, de kuip van de Stoelen, de balies en de
/// tafels voor de Voorzitter. Elk houdt zijn eigen helderheid.
const HOUT: [&str; 9] = [
    "perenhout",
    "perenhout_blad",
    "middenvak_blad",
    "balies_fineer_lichta",
    "balies_fineer_lichtb",
    "balies_fineer_middena",
    "balies_fineer_middenb",
    "balies_fineer_donkera",
    "balies_fineer_donkerb",
];
/// De rand van de bureaubladen en balies: hetzelfde hout, donkerder, zodat het blad een rand
/// houdt zonder te gloeien.
const HOUTRAND: [&str; 2] = ["desk_rand", "balies_bladrand"];
/// De kleur van de rand: beuken op deze helderheid.
const RAND_DONKER: f32 = 0.55;
/// Het leer van de Stoelen is matte stof: geen glans van de reflecties, geen snoepkleur.
const STOF_RUWHEID: f32 = 0.85;
const STOF_REFLECTIE: f32 = 0.3;

/// De helderheid van een lineaire kleur, zoals het oog hem ziet (Rec. 709).
fn helderheid(k: LinearRgba) -> f32 {
    0.2126 * k.red + 0.7152 * k.green + 0.0722 * k.blue
}

/// Het hout in neutraal beuken met de helderheid van `oud`; de rand vast iets donkerder.
fn neutraal_hout(oud: LinearRgba, rand: bool) -> LinearRgba {
    let beuken = LinearRgba::from(BEUKEN);
    let f = if rand { RAND_DONKER } else { helderheid(oud) / helderheid(beuken) };
    LinearRgba::new(beuken.red * f, beuken.green * f, beuken.blue * f, oud.alpha)
}

/// Stelt het hout en het leer uit het model bij, één keer per materiaal en vóór de Stoelen hun
/// kleur krijgen en de Zaal dimt (die nemen de bijgestelde kleur als hun eigen):
/// - hout ([`HOUT`], [`HOUTRAND`]) naar neutraal beuken;
/// - het leer van de Stoelen ([`STOEL_LEER`]) naar matte stof, zonder de ruwheidstextuur die het
///   liet glanzen als plastic;
/// - het wapen ([`STOEL_WAPEN`]) zacht stralend, en de nooduitgang ([`NOODUITGANG`]) gedempt.
fn stel_stof_en_hout_bij(
    nieuw: Query<(&GltfMaterialName, &MeshMaterial3d<StandardMaterial>), Added<GltfMaterialName>>,
    mut materialen: ResMut<Assets<StandardMaterial>>,
    mut gedaan: Local<std::collections::HashSet<AssetId<StandardMaterial>>>,
) {
    for (naam, materiaal) in &nieuw {
        let naam = naam.0.as_str();
        let (hout, rand, leer) = (HOUT.contains(&naam), HOUTRAND.contains(&naam), naam == STOEL_LEER);
        let (wapen, nood) = (naam == STOEL_WAPEN, naam == NOODUITGANG);
        if !(hout || rand || leer || wapen || nood) || !gedaan.insert(materiaal.0.id()) {
            continue;
        }
        let Some(mut m) = materialen.get_mut(&materiaal.0) else {
            // een glTF-scène komt pas in de wereld als zijn materialen geladen zijn
            warn!("materiaal {naam} nog niet geladen: niet bijgesteld");
            continue;
        };
        if wapen {
            m.emissive = WAPEN_STRAALT;
        } else if nood {
            m.emissive = m.emissive * NOODUITGANG_LICHT;
        } else if leer {
            m.metallic_roughness_texture = None;
            m.perceptual_roughness = STOF_RUWHEID;
            m.metallic = 0.0;
            m.reflectance = STOF_REFLECTIE;
        } else {
            m.base_color = neutraal_hout(LinearRgba::from(m.base_color), rand).into();
        }
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
    fn de_lijn_bij_76_wacht_tot_de_camera_na_de_rit_stilstaat() {
        let dt = 0.02;
        let stap = dt / LIJN_76_GROEIT;
        // de camera stond al lang stil voor de Coalitie kwam
        let (mut zicht, mut stil) = (0.0, 0.0);
        for _ in 0..100 {
            (zicht, stil) = stap_lijn_76(zicht, stil, false, true, dt, stap);
        }
        // de Coalitie komt; na 0,1 s rijdt de camera 2 s naar A: geen lijn tijdens de rit
        for i in 0..110 {
            (zicht, stil) = stap_lijn_76(zicht, stil, true, i < 5, dt, stap);
            assert_eq!(zicht, 0.0, "beeld {i}");
        }
        // stil bij A: na een kwart seconde groeit hij, en is na zijn groeitijd helemaal
        let wacht = (LIJN_76_NA_STIL / dt).ceil() as usize;
        for _ in 0..wacht {
            (zicht, stil) = stap_lijn_76(zicht, stil, true, true, dt, stap);
        }
        assert!(zicht > 0.0);
        for _ in 0..((LIJN_76_GROEIT / dt).ceil() as usize) {
            (zicht, stil) = stap_lijn_76(zicht, stil, true, true, dt, stap);
        }
        assert_eq!(zicht, 1.0);
        // rijdt de camera weer (naar een ander Standpunt), dan blijft hij staan
        (zicht, stil) = stap_lijn_76(zicht, stil, true, false, dt, stap);
        assert_eq!(zicht, 1.0);
        // weg met de Coalitie: hij krimpt
        (zicht, _) = stap_lijn_76(zicht, stil, false, false, dt, stap);
        assert!(zicht < 1.0);
    }

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
        let rood_half = Srgba::from(leer_in_de_zaal(Kleur { r: 0xd4, g: 0x20, b: 0x2c }));
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
        let half = |k: Kleur| Srgba::from(leer_in_de_zaal(k));
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
    fn vervagen_laat_alle_stoelen_tegelijk_van_kleur_wisselen() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        // VOORBEELDGEGEVENS: alle Stoelen rood, daarna groen met Vervagen van anderhalve seconde
        let vervagen = Kleurovergang::standaard(Overgangsvorm::Vervagen);
        let mut stand = Stand { stoelkleuren: vec!["#d4202c".parse().unwrap(); 150], kleurovergang: vervagen, ..default() };
        app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand { volgnummer: 1, stand: stand.clone() });
        app.update();
        stand.stoelkleuren = vec!["#00ae41".parse().unwrap(); 150];
        app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand { volgnummer: 2, stand });
        app.update();
        wacht(&mut app, 0.75);
        let leer = leerkleuren(&mut app);
        let groen = Srgba::from(leer_in_de_zaal(Kleur { r: 0x00, g: 0xae, b: 0x41 }));
        assert!(!ongeveer(leer[0], groen), "halverwege nog niet groen");
        assert!(leer.iter().all(|k| ongeveer(*k, leer[0])), "halverwege links en rechts gelijk");
        wacht(&mut app, 0.8);
        assert!(leerkleuren(&mut app).iter().all(|k| ongeveer(*k, groen)), "na anderhalve seconde alles groen");
    }

    #[test]
    fn de_opbouw_kleurt_een_partij_tegelijk_de_grootste_eerst_en_kan_worden_overgeslagen() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        // VOORBEELDGEGEVENS: 40 Stoelen rood links, 110 groen rechts; anderhalve seconde per Partij
        let rood: Kleur = "#d4202c".parse().unwrap();
        let groen: Kleur = "#00ae41".parse().unwrap();
        let mut kleuren = vec![groen; 150];
        for i in stoelen_van_links_naar_rechts().into_iter().take(40) {
            kleuren[i] = rood;
        }
        let opbouw = Kleurovergang::standaard(Overgangsvorm::Opbouw);
        let stand = Stand { stoelkleuren: kleuren.clone(), kleurovergang: opbouw, ..default() };
        app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand { volgnummer: 1, stand: stand.clone() });
        app.update();
        let lin = |k: Kleur| Srgba::from(leer_in_de_zaal(k));
        let stap = |app: &App| app.world().resource::<Opbouwstand>().stap.map(|(k, _)| k);
        let tel = |app: &mut App, k: Kleur| leerkleuren(app).iter().filter(|l| ongeveer(**l, lin(k))).count();

        // de eerste Stand na het opstarten komt meteen; de volgende bouwt op
        let mut nieuw = stand.clone();
        nieuw.stoelkleuren.iter_mut().for_each(|k| *k = if *k == rood { groen } else { rood });
        app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand { volgnummer: 2, stand: nieuw.clone() });
        app.update();
        wacht(&mut app, 0.74);
        assert!(app.world().resource::<Opbouwstand>().bezig);
        assert_eq!(stap(&app), None, "eerst worden de Stoelen donker");
        assert_eq!(tel(&mut app, OPBOUW_WACHTKLEUR), 150);

        // de grootste Partij (nu rood, 110) eerst en gloeit; de rest wacht donker
        wacht(&mut app, 0.7);
        assert_eq!(stap(&app), Some(rood));
        assert_eq!(tel(&mut app, OPBOUW_WACHTKLEUR), 40);
        assert_eq!(tel(&mut app, rood), 0, "rood gloeit nog, lichter dan zijn eigen kleur");
        // in de volgende stap is rood gedimd, zodat alleen groen vol oplicht
        wacht(&mut app, 1.5);
        assert_eq!(stap(&app), Some(groen));
        assert_eq!(tel(&mut app, rood), 0, "rood gedimd");
        // na de laatste stap komt alles tegelijk terug in zijn eigen kleur
        wacht(&mut app, 1.5);
        assert_eq!(tel(&mut app, rood), 110);
        assert_eq!(tel(&mut app, groen), 40);
        assert!(!app.world().resource::<Opbouwstand>().bezig, "na de laatste Partij en haar nagloed klaar");

        // overslaan: dezelfde kleuren met Vervagen; de Partijen die nog wachtten, kleuren meteen mee
        app.world_mut().resource_mut::<Standen>().huidig = Some(GenummerdeStand { volgnummer: 3, stand });
        app.update();
        wacht(&mut app, 1.0);
        assert_eq!(stap(&app), Some(groen), "groen (110) is nu het grootst");
        let mut over = app.world().resource::<Standen>().huidig.clone().unwrap();
        over.volgnummer = 4;
        over.stand.kleurovergang = Kleurovergang { vorm: Overgangsvorm::Vervagen, duur_ms: gedeeld::OPBOUW_OVERSLAAN_MS };
        app.world_mut().resource_mut::<Standen>().huidig = Some(over);
        app.update();
        assert!(!app.world().resource::<Opbouwstand>().bezig);
        wacht(&mut app, 0.55);
        assert_eq!((tel(&mut app, groen), tel(&mut app, rood)), (110, 40));
    }

    #[test]
    fn de_zaal_dimt_naar_de_kleurkracht_zolang_er_stoelen_gekleurd_zijn() {
        let (mut app, _) = zaal_zonder_gpu();
        let grijs = Color::srgb(0.6, 0.6, 0.6);
        let vloer = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial::from(grijs));
        app.world_mut().spawn((GltfMaterialName("vloer".into()), MeshMaterial3d(vloer.clone()), Mesh3d::default()));
        let licht = |app: &mut App| LinearRgba::from(app.world().resource::<Assets<StandardMaterial>>().get(&vloer).unwrap().base_color).red;
        let origineel = LinearRgba::from(grijs).red;
        zet_stand(&mut app, 1, vec![]);
        app.update();
        assert_eq!(licht(&mut app), origineel, "zonder kleur zoals gebakken");

        // VOORBEELDGEGEVENS: alle Stoelen rood, standaard-Kleurkracht 50: een derde van het licht
        zet_stand(&mut app, 2, vec!["#d4202c".parse().unwrap(); 150]);
        wacht(&mut app, 0.5);
        let halverwege = licht(&mut app);
        let doel = origineel * MINSTE_LICHT.sqrt();
        assert!(halverwege < origineel && halverwege > doel, "het licht gaat mee met de overgang");
        wacht(&mut app, 0.6);
        assert!((licht(&mut app) - doel).abs() < 1e-4);

        // Kleur uit: weer zoals gebakken
        zet_stand(&mut app, 3, vec![]);
        wacht(&mut app, 1.1);
        assert!((licht(&mut app) - origineel).abs() < 1e-4);
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
        let groen = Srgba::from(leer_in_de_zaal(Kleur { r: 0x00, g: 0xae, b: 0x41 }));
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
        zet_persoonskaart_van(app, volgnummer, stoel, kleuren, None);
    }

    /// Idem, voor een Kamerlid van een Partij met deze kleur.
    fn zet_persoonskaart_van(
        app: &mut App,
        volgnummer: u64,
        stoel: Option<Stoel>,
        kleuren: Vec<Kleur>,
        partijkleur: Option<&str>,
    ) {
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
                partij: partijkleur.map(|_| "cda".into()),
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
        let mut cijfers = gedeeld::cijfers::Cijfers::default();
        if let Some(k) = partijkleur {
            cijfers.partijen.push(gedeeld::cijfers::Partij {
                sleutel: "cda".into(),
                naam: "CDA".into(),
                kort: "CDA".into(),
                kleur: k.into(),
                kleur_tijdelijk: false,
                logo: None,
                fractie_zonder_lijst: false,
                voorgangers: vec![],
            });
        }
        let mut standen = app.world_mut().resource_mut::<Standen>();
        let vorige = standen.huidig.replace(GenummerdeStand {
            volgnummer,
            stand: Stand { ether: Some(ether), stoelkleuren: kleuren, cijfers, ..default() },
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
        let rood_half = Srgba::from(leer_in_de_zaal(Kleur { r: 0xd4, g: 0x20, b: 0x2c }));
        assert!(ongeveer(leerkleuren(&mut app)[i], rood_half));
        // gekleurd leer straalt zacht in zijn kleur (de Kleurkracht); de uitgelichte Stoel feller
        let s = straling(&mut app);
        assert!(s[i] > s[(i + 1) % 150] + 0.1, "de Stoel straalt boven de rest uit");

        // zonder Persoonskaart straalt hij weer als de rest
        zet_persoonskaart(&mut app, 3, None, vec!["#d4202c".parse().unwrap(); 150]);
        wacht(&mut app, 1.0);
        let s = straling(&mut app);
        assert!(s.iter().all(|e| (*e - s[0]).abs() < 1e-4));
        assert!(ongeveer(leerkleuren(&mut app)[i], rood_half));
    }

    #[test]
    fn de_stoel_van_de_persoonskaart_krijgt_de_kleur_van_de_partij_en_straalt_daarin() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        let stoel = Stoel { vak: 2, rij: 3, nummer: 1 };
        let i = stoel.index().unwrap();
        // VOORBEELDGEGEVENS: alle Stoelen blauw, het Kamerlid van een groene Partij
        zet_persoonskaart_van(&mut app, 1, Some(stoel), vec!["#1f4fd6".parse().unwrap(); 150], Some("#007c5e"));
        wacht(&mut app, 1.0);
        let groen: Kleur = "#007c5e".parse().unwrap();
        assert!(ongeveer(leerkleuren(&mut app)[i], Srgba::from(leer_in_de_zaal(groen))), "het leer is groen");
        let mut q = app.world_mut().query::<(&StoelLeer, &MeshMaterial3d<StandardMaterial>)>();
        let h = q.iter(app.world()).find(|(s, _)| s.index == i).unwrap().1.0.clone();
        let e = app.world().resource::<Assets<StandardMaterial>>().get(&h).unwrap().emissive;
        assert!(e.green > e.red && e.green > e.blue, "straalt groen, geen wit: {e:?}");
    }

    #[test]
    fn een_stoel_die_niet_meedoet_is_donker_en_straalt_niet() {
        let (mut app, _) = zaal_zonder_gpu();
        app.update();
        // VOORBEELDGEGEVENS: Partij uitgelicht, 26 groene Stoelen en de rest grijs
        let groen: Kleur = "#00ae41".parse().unwrap();
        let kleuren: Vec<Kleur> = (0..150).map(|i| if i < 26 { groen } else { GRIJS }).collect();
        zet_stand(&mut app, 1, kleuren);
        app.update();
        let leer = leerkleuren(&mut app);
        let licht = |k: Srgba| 0.2126 * k.red + 0.7152 * k.green + 0.0722 * k.blue;
        assert!(licht(leer[100]) < licht(leer[0]) * 0.5, "grijs is veel donkerder dan de Partij");
        let s = straling(&mut app);
        assert_eq!(s[100], 0.0);
        assert!(s[0] > 0.0);
    }
}
