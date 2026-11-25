//! De Graphic-laag: tekent de klaargezette Graphic of de Graphic in de Ether over het beeld.
//!
//! Een Graphic is een ingevuld Sjabloon, en een Sjabloon bestaat uit Blokken (ADR 0003): de
//! laag tekent alleen Blokken ([`crate::blokken`]). Elke camera met een [`Bron`] krijgt een
//! eigen laag. De Ether-laag animeert: een nieuwe Graphic komt Blok voor Blok in beeld, bij een
//! wissel gaat de oude eerst weg, en Bijwerken laat alles vloeiend naar de nieuwe stand gaan.
//! De laag van het Voorbeeld toont meteen het eindbeeld. Een laag die halverwege de uitzending
//! begint, toont ook meteen het eindbeeld (ADR 0001).
//!
//! Een Graphic in de Ether houdt haar vorm (Sjabloon en Stijl) tot de volgende Toon; alleen de
//! gegevens werken bij. De server bewaart de Stijl van het moment van Toon bij de Graphic
//! ([`gedeeld::GraphicInBeeld::stijl`]), zodat ook een Renderer die halverwege start de oude vorm
//! tekent. Zo ook het Sjabloon ([`gedeeld::GraphicInBeeld::sjabloon`]): een Sjabloon dat de
//! omroep bewerkte, geldt in de Ether pas bij de volgende Toon. Het Voorbeeld toont altijd de
//! nieuwste vorm. De achtergrond van het Sjabloon tekent de laag als eerste Blok
//! ([`Sjabloon::met_achtergrond`]).

use bevy::prelude::*;
use gedeeld::Stand;
use gedeeld::cijfers::Cijfers;
use bevy::post_process::dof::{DepthOfField, DepthOfFieldMode};
use gedeeld::sjabloon::{Achtergrond, IngevuldBlok, Inhoud, Invulling, Sjabloon, Stijl, vaste_sjablonen};

use crate::animatie::{Aard, Tijden, Toestand, op_hele_pixels, zacht_uit};
use crate::bestanden::Bestanden;
use crate::blokken::{
    Binnen, BlokKnoop, Deel, Krimp, Letter, Opbouwkaart, Pasbreedte, PastIn, Puls, Randkleur, Rijplek, Segment, Teller,
    Verloopdeel, Vorig, WachtOpBeeld, bouw_blok, bouw_inhoud, krimp,
};
use crate::volging::Standen;
use crate::zaal::Opbouwstand;

pub struct GraphicPlugin;

impl Plugin for GraphicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Sjablonen(vaste_sjablonen()))
            .add_systems(
                Update,
                (
                    maak_laag,
                    vraag_vooraf.run_if(resource_changed::<Standen>),
                    volg_stand.run_if(resource_changed::<Standen>),
                    voer_overgang,
                    // vóór animeer: de naambalk heeft zijn breedte al in het eerste beeld van de lijn
                    pas_om_tekst,
                    animeer,
                    tel_en_schuif,
                    pulseer,
                    toon_opbouwlabel,
                    zet_letters,
                    krimp_teksten,
                    crate::stemmingsblokken::pas_in,
                    verberg_wat_niet_past,
                    wacht_op_beelden,
                )
                    .chain(),
            )
            .add_systems(Update, maak_onscherp.after(voer_overgang));
    }
}

/// De vaste Sjablonen, op `id`: voor een Wens zonder eigen Sjabloon (een proef, of een Stand van
/// vóór #62).
#[derive(Resource)]
pub struct Sjablonen(pub Vec<Sjabloon>);

impl Sjablonen {
    fn zoek(&self, id: &str) -> Option<&Sjabloon> {
        self.0.iter().find(|s| s.id == id)
    }
}

/// Welke Graphic een camera toont.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bron {
    /// De klaargezette Graphic: het Voorbeeld.
    Klaar,
    /// De Graphic in de Ether.
    Ether,
}

/// Wat een laag hoort te tonen.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Wens {
    /// Het volgnummer van de Toon: een nieuwe Toon is altijd een wissel.
    pub toon: u64,
    /// De versie van de Graphic: een nieuwere versie van dezelfde Toon is Bijwerken.
    pub versie: u64,
    pub invulling: Invulling,
    /// De cijfers die de Graphic zelf vasthoudt (zoals een Zetelverdeling); zonder: die van de
    /// Stand.
    pub cijfers: Option<Cijfers>,
    /// De vorm: in de Ether de Stijl van het moment van Toon, in het Voorbeeld de nieuwste.
    pub stijl: Stijl,
    /// Het Sjabloon met zijn achtergrond: in de Ether dat van het moment van Toon, in het
    /// Voorbeeld het nieuwste. Zonder: het vaste Sjabloon uit [`Sjablonen`].
    pub sjabloon: Option<Sjabloon>,
}

impl Wens {
    fn cijfers<'a>(&'a self, stand: &'a Stand) -> &'a Cijfers {
        self.cijfers.as_ref().unwrap_or(&stand.cijfers)
    }

    fn sjabloon<'a>(&'a self, sjablonen: &'a Sjablonen) -> Option<&'a Sjabloon> {
        self.sjabloon.as_ref().or_else(|| sjablonen.zoek(&self.invulling.sjabloon))
    }
}

/// De laag over het beeld van één camera.
#[derive(Component)]
pub struct Laag {
    bron: Bron,
    /// Wat de laag hoort te tonen; [`volg_stand`] zet het, [`voer_overgang`] voert het uit.
    pub gewenst: Option<Wens>,
    /// Wat de laag nu toont; `None` voor de eerste Stand.
    getoond: Option<Option<Wens>>,
}

/// Eén Graphic in beeld, met de fase van zijn animatie.
#[derive(Component)]
pub struct Kaart {
    fase: Fase,
    /// Seconden sinds het begin van de fase.
    tijd: f32,
    tijden: Tijden,
    /// De vorm waarmee de Graphic in beeld kwam.
    sjabloon: Sjabloon,
    stijl: Stijl,
    /// Wat elk Blok nu toont.
    blokken: Vec<IngevuldBlok>,
    /// Hoe zichtbaar de Partijlabels zijn (0 tot 1): in de Ether wachten ze op het eind van een
    /// Opbouw in de Zaal.
    na_opbouw: f32,
}

/// Hoe lang de Partijlabels na een Opbouw over hun komst doen, in seconden.
const NA_OPBOUW_S: f32 = 0.5;
/// Hoe lang het grote label van een Partij in de Opbouw over zijn komst doet, in seconden.
const OPBOUWLABEL_S: f32 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Fase {
    /// Wacht zoveel seconden tot de vorige Graphic weg is.
    Wacht(f32),
    Erin,
    Staat,
    Eruit,
}

/// Een Blok dat bij Bijwerken van inhoud wisselt: het vervaagt, krijgt de nieuwe inhoud, en
/// komt terug.
#[derive(Component)]
pub struct Wissel {
    tijd: f32,
    duur: f32,
    nieuw: Option<IngevuldBlok>,
}

fn maak_laag(mut commands: Commands, cameras: Query<(Entity, &Bron), Added<Bron>>) {
    for (camera, bron) in &cameras {
        commands.spawn((
            Laag {
                bron: *bron,
                gewenst: None,
                getoond: None,
            },
            UiTargetCamera(camera),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
    }
}

/// Vraagt lettertypes en logo's al op zodra de Stand ze noemt, zodat ze er bij Toon zijn.
fn vraag_vooraf(standen: Res<Standen>, mut bestanden: ResMut<Bestanden>) {
    let Some(s) = &standen.huidig else { return };
    let stand = &s.stand;
    let ether = stand.ether.as_ref().map(|e| &e.stijl);
    for naam in stand.stijl.bestanden().chain(ether.into_iter().flat_map(Stijl::bestanden)) {
        bestanden.vraag(naam);
    }
    // eigen lettertypes en afbeeldingen van bewerkte Sjablonen, ook van de kopie in de Ether
    let sjablonen = stand.sjablonen.iter().map(|b| &b.sjabloon);
    for naam in sjablonen.chain(stand.ether.iter().filter_map(|e| e.sjabloon.as_ref())).flat_map(Sjabloon::eigen_bestanden) {
        bestanden.vraag(&naam);
    }
    for logo in stand
        .cijfers
        .partijen
        .iter()
        .filter_map(|p| p.logo.as_deref())
    {
        bestanden.vraag(logo);
    }
}

/// Een laag die niet de Stand volgt: een proef zet [`Laag::gewenst`] zelf.
#[derive(Component)]
pub struct HandGestuurd;

fn volg_stand(standen: Res<Standen>, mut lagen: Query<&mut Laag, Without<HandGestuurd>>) {
    let Some(huidig) = &standen.huidig else {
        return;
    };
    let stand = &huidig.stand;
    for mut laag in &mut lagen {
        let wens = match laag.bron {
            Bron::Ether => stand.ether.as_ref().map(|e| Wens {
                toon: e.getoond,
                versie: e.versie,
                invulling: e.graphic.invulling(),
                cijfers: e.graphic.cijfers(),
                stijl: e.stijl.clone(),
                sjabloon: e
                    .sjabloon
                    .as_ref()
                    .map(Sjabloon::met_achtergrond)
                    .or_else(|| stand.sjabloon(&e.graphic.sjabloon_id()).map(|s| s.met_achtergrond())),
            }),
            Bron::Klaar => stand.klaar.as_ref().map(|k| Wens {
                toon: 0,
                versie: k.versie,
                invulling: k.graphic.invulling(),
                cijfers: k.graphic.cijfers(),
                stijl: stand.stijl.clone(),
                sjabloon: stand.sjabloon(&k.graphic.sjabloon_id()).map(|s| s.met_achtergrond()),
            }),
        };
        laag.gewenst = wens;
    }
}

/// De Stand waarin de laag de gegevens opzoekt. Zonder Stand: de standaardstijl, geen cijfers.
fn stand_van(standen: &Standen) -> std::borrow::Cow<'_, Stand> {
    match &standen.huidig {
        Some(s) => std::borrow::Cow::Borrowed(&s.stand),
        None => std::borrow::Cow::Owned(Stand::default()),
    }
}

#[allow(clippy::too_many_arguments)]
fn voer_overgang(
    mut commands: Commands,
    standen: Res<Standen>,
    sjablonen: Res<Sjablonen>,
    mut bestanden: ResMut<Bestanden>,
    beelden: Res<Assets<Image>>,
    mut lagen: Query<(Entity, &mut Laag, Option<&Children>)>,
    mut kaarten: Query<(&mut Kaart, &Children)>,
    knopen: Query<&BlokKnoop>,
) {
    // Nog geen Stand van de server: nog niets beslissen. Anders telt "niets in beeld" als het
    // begin en zou de eerste Stand na het opstarten erin animeren in plaats van meteen het
    // eindbeeld te tonen (ADR 0001).
    if standen.huidig.is_none() {
        return;
    }
    let stand = stand_van(&standen);
    for (laag_id, mut laag, kinderen) in &mut lagen {
        let kaart_ids: Vec<Entity> = kinderen
            .into_iter()
            .flatten()
            .copied()
            .filter(|k| kaarten.contains(*k))
            .collect();
        let vorig = laag.getoond.clone();
        let nieuw = laag.gewenst.clone();
        let bouw = |commands: &mut Commands, bestanden: &mut Bestanden, wens: &Wens, fase: Fase| {
            let Some(sjabloon) = wens.sjabloon(&sjablonen) else {
                warn!("onbekend Sjabloon: {}", wens.invulling.sjabloon);
                return;
            };
            spawn_kaart(
                commands,
                bestanden,
                &beelden,
                laag_id,
                sjabloon,
                &wens.invulling,
                wens.cijfers(&stand),
                &wens.stijl,
                fase,
            );
        };

        // Het Voorbeeld en de eerste Stand: meteen het eindbeeld.
        if laag.bron == Bron::Klaar || vorig.is_none() {
            // Het Voorbeeld volgt ook de Stijl en de cijfers, maar rekent alleen na een wijziging.
            if vorig.as_ref() == Some(&nieuw) && !standen.is_changed() {
                continue;
            }
            let zelfde_vorm = match (&nieuw, kaart_ids.first().and_then(|k| kaarten.get(*k).ok())) {
                (Some(w), Some((kaart, _))) => {
                    w.sjabloon(&sjablonen).is_some_and(|s| {
                        *s == kaart.sjabloon
                            && w.stijl == kaart.stijl
                            && s.vul_in(&w.invulling, w.cijfers(&stand), &w.stijl) == kaart.blokken
                    })
                }
                (None, None) => true,
                _ => false,
            };
            if vorig.is_none() || !zelfde_vorm {
                for k in &kaart_ids {
                    commands.entity(*k).despawn();
                }
                if let Some(w) = &nieuw {
                    bouw(&mut commands, &mut bestanden, w, Fase::Staat);
                }
            }
            laag.getoond = Some(nieuw);
            continue;
        }
        let vorig = vorig.flatten();
        if vorig == nieuw {
            continue;
        }
        let aanwezig: Vec<Entity> = kaart_ids
            .iter()
            .copied()
            .filter(|k| kaarten.get(*k).is_ok_and(|(k, _)| k.fase != Fase::Eruit))
            .collect();
        let bijwerken = matches!((&vorig, &nieuw), (Some(v), Some(n)) if v.toon == n.toon);
        match &nieuw {
            Some(n) if bijwerken && aanwezig.len() == 1 => {
                let (mut kaart, blokken) = kaarten.get_mut(aanwezig[0]).expect("aanwezig");
                if kaart.sjabloon.id != n.invulling.sjabloon {
                    // Een ander Sjabloon onder dezelfde Toon: als een wissel.
                    vertrek(&mut kaart);
                    let wacht = kaart.tijden.eruit_klaar(kaart.blokken.len());
                    bouw(&mut commands, &mut bestanden, n, Fase::Wacht(wacht));
                } else {
                    let nieuwe = kaart
                        .sjabloon
                        .vul_in(&n.invulling, n.cijfers(&stand), &kaart.stijl);
                    werk_bij(
                        &mut commands,
                        &mut bestanden,
                        &beelden,
                        &mut kaart,
                        blokken,
                        &knopen,
                        nieuwe,
                    );
                }
            }
            _ => {
                let mut wacht = 0.0f32;
                for k in &aanwezig {
                    let (mut kaart, _) = kaarten.get_mut(*k).expect("aanwezig");
                    vertrek(&mut kaart);
                    wacht = wacht.max(kaart.tijden.eruit_klaar(kaart.blokken.len()));
                }
                if let Some(n) = &nieuw {
                    let fase = if aanwezig.is_empty() {
                        Fase::Erin
                    } else {
                        Fase::Wacht(wacht)
                    };
                    bouw(&mut commands, &mut bestanden, n, fase);
                }
            }
        }
        laag.getoond = Some(nieuw);
    }
}

/// De grootste onscherpte van de Zaal achter een Sjabloon met de achtergrond Zaal donker, in
/// pixels van het 1080p-beeld. Met 22 bleven de Stoelen achter een stemmingstabel te scherp
/// (council-ronde 4).
const ONSCHERP_PX: f32 = 40.0;
/// Hoe lang de Zaal erover doet om onscherp of weer scherp te worden, in seconden.
const ONSCHERP_S: f32 = 0.5;
/// Achter Zaal donker verliest de Zaal ook zijn kleur en wordt hij donkerder: zoveel van de
/// verzadiging blijft over, en zoveel stops gaat de belichting omlaag. Met alleen onscherpte
/// stonden achter de stemmingstabellen grote groene en rode vlekken (gemiddeld RGB 23, 40, 32
/// tegen 5, 4, 4 achter Uitslag per partij, council-ronde 5).
const ONSCHERP_VERZADIGING: f32 = 0.12;
const ONSCHERP_STOPS: f32 = 2.5;

/// De kleurcorrectie van de Zaal achter Zaal donker, bij onscherpte `f` (0 tot 1).
fn kleurcorrectie_achter_donker(f: f32) -> bevy::render::view::ColorGrading {
    let mut k = crate::beeld::BEELD.kleurcorrectie();
    k.global.post_saturation = 1.0 - (1.0 - ONSCHERP_VERZADIGING) * f;
    k.global.exposure = -ONSCHERP_STOPS * f;
    k
}

/// Hoe onscherp de Zaal achter een laag nu is, van 0 (scherp) tot 1.
#[derive(Component, Default)]
struct Onscherp(f32);

/// Achter een Sjabloon met [`Achtergrond::ZaalDonker`] wordt de Zaal ook onscherp: scherptediepte
/// op de camera van de laag, scherp vlak voor de lens, dus de hele Zaal vaag. De Graphic zelf
/// blijft scherp (de laag komt na de nabewerking). In de Ether groeit en slinkt de onscherpte
/// mee met het in en uit beeld gaan; een Graphic die al staat (het Voorbeeld, of het eindbeeld na
/// het opstarten) is meteen onscherp.
fn maak_onscherp(
    mut commands: Commands,
    tijd: Res<Time>,
    mut lagen: Query<(Entity, &Laag, &UiTargetCamera, Option<&Children>, Option<&mut Onscherp>)>,
    kaarten: Query<&Kaart>,
    mut kleur: Query<&mut bevy::render::view::ColorGrading>,
) {
    for (id, laag, camera, kinderen, onscherp) in &mut lagen {
        let donker: Vec<Fase> = kinderen
            .into_iter()
            .flatten()
            .filter_map(|k| kaarten.get(*k).ok())
            .filter(|k| k.sjabloon.achtergrond == Achtergrond::ZaalDonker)
            .map(|k| k.fase)
            .collect();
        // bij een wissel tussen twee van zulke Graphics blijft de Zaal onscherp
        let doel = if donker.iter().any(|f| *f != Fase::Eruit) { 1.0 } else { 0.0 };
        // een Graphic die er al staat (het Voorbeeld, of het eindbeeld na het opstarten): meteen
        let meteen = laag.bron == Bron::Klaar || donker.contains(&Fase::Staat);
        let Some(mut nu) = onscherp else {
            commands.entity(id).insert(Onscherp(0.0));
            continue;
        };
        let vorig = nu.0;
        nu.0 = if meteen {
            doel
        } else {
            let stap = tijd.delta_secs() / ONSCHERP_S;
            if doel > nu.0 { (nu.0 + stap).min(doel) } else { (nu.0 - stap).max(doel) }
        };
        if nu.0 == vorig {
            continue;
        }
        if let Ok(mut k) = kleur.get_mut(camera.entity()) {
            *k = kleurcorrectie_achter_donker(zacht_uit(nu.0));
        }
        let Ok(mut camera) = commands.get_entity(camera.entity()) else { continue };
        if nu.0 > 0.0 {
            camera.insert((
                DepthOfField {
                    mode: DepthOfFieldMode::Gaussian,
                    focal_distance: 0.2,
                    aperture_f_stops: 0.5,
                    max_circle_of_confusion_diameter: ONSCHERP_PX * zacht_uit(nu.0),
                    ..default()
                },
                crate::lens::ZaalOnscherp,
            ));
        } else {
            camera.remove::<(DepthOfField, crate::lens::ZaalOnscherp)>();
        }
    }
}

fn vertrek(kaart: &mut Kaart) {
    kaart.fase = Fase::Eruit;
    kaart.tijd = 0.0;
}

/// Bouwt een Graphic op uit de Blokken van zijn Sjabloon.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_kaart(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    beelden: &Assets<Image>,
    laag: Entity,
    sjabloon: &Sjabloon,
    invulling: &Invulling,
    cijfers: &Cijfers,
    stijl: &Stijl,
    fase: Fase,
) -> Entity {
    let blokken = sjabloon.vul_in(invulling, cijfers, stijl);
    let kaart = commands
        .spawn((
            ChildOf(laag),
            Kaart {
                fase,
                tijd: 0.0,
                tijden: Tijden::new(&stijl.beweging, stijl.snelheid, sjabloon.snelheid),
                sjabloon: sjabloon.clone(),
                stijl: stijl.clone(),
                blokken: blokken.clone(),
                na_opbouw: 1.0,
            },
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .id();
    let begin = if fase == Fase::Staat { 1.0 } else { 0.0 };
    for (i, b) in blokken.iter().enumerate() {
        bouw_blok(commands, bestanden, beelden, kaart, i, b, begin);
    }
    kaart
}

/// Bijwerken: elk Blok gaat vloeiend naar zijn nieuwe inhoud. Getallen tellen, de Zetelbalk
/// schuift, de rest vervaagt en komt terug met de nieuwe inhoud.
fn werk_bij(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    beelden: &Assets<Image>,
    kaart: &mut Kaart,
    kinderen: &Children,
    knopen: &Query<&BlokKnoop>,
    nieuwe: Vec<IngevuldBlok>,
) {
    let duur = kaart.tijden.bijwerken();
    for knoop in kinderen.iter() {
        let Ok(BlokKnoop { index }) = knopen.get(knoop) else {
            continue;
        };
        let (Some(oud), Some(nieuw)) = (kaart.blokken.get(*index), nieuwe.get(*index)) else {
            continue;
        };
        if oud == nieuw {
            continue;
        }
        match (&oud.inhoud, &nieuw.inhoud) {
            (
                Inhoud::Getal {
                    waarde: a,
                    letter: la,
                    grootte: ga,
                    meerderheid: was_meerderheid,
                    ..
                },
                Inhoud::Getal {
                    waarde: b,
                    letter: lb,
                    grootte: gb,
                    meerderheid,
                    ..
                },
            ) if a.getal().is_some()
                && b.getal().is_some()
                && (la, ga) == (lb, gb)
                && oud.kleur == nieuw.kleur =>
            {
                // haalt de coalitie net een meerderheid: na het tellen een puls
                let puls = (*meerderheid && !*was_meerderheid).then_some(duur);
                commands.queue(stel_teller_in(
                    knoop,
                    a.getal().unwrap_or(0),
                    b.clone(),
                    duur,
                    puls,
                ));
            }
            (Inhoud::Zetelbalk { delen: oude_delen, .. }, Inhoud::Zetelbalk { .. })
            | (Inhoud::Uitslaggrafiek { rijen: oude_delen, .. }, Inhoud::Uitslaggrafiek { .. }) => {
                herbouw(commands, knoop);
                bouw_inhoud(
                    commands,
                    bestanden,
                    beelden,
                    knoop,
                    nieuw,
                    1.0,
                    Some(Vorig { delen: oude_delen, duur }),
                );
                commands.entity(knoop).insert(Schuift { tijd: 0.0, duur });
            }
            _ => {
                commands.entity(knoop).insert(Wissel {
                    tijd: 0.0,
                    duur: duur * 0.6,
                    nieuw: Some(nieuw.clone()),
                });
            }
        }
    }
    kaart.blokken = nieuwe;
}

/// De Zetelbalk schuift naar de nieuwe breedtes, en de rijen van een Uitslaggrafiek naar hun
/// nieuwe plek.
#[derive(Component)]
pub struct Schuift {
    tijd: f32,
    duur: f32,
}

/// Zet de teller in het Blok `knoop` op tellen van `van` naar het nieuwe getal.
fn stel_teller_in(
    knoop: Entity,
    van: i64,
    waarde: gedeeld::sjabloon::Getalwaarde,
    duur: f32,
    puls_na: Option<f32>,
) -> impl Command {
    move |world: &mut World| {
        let tellers: Vec<Entity> = world
            .query_filtered::<(Entity, &ChildOf), With<Teller>>()
            .iter(world)
            .filter(|(_, ouder)| {
                world
                    .get::<ChildOf>(ouder.parent())
                    .is_some_and(|o| o.parent() == knoop)
            })
            .map(|(e, _)| e)
            .collect();
        for t in tellers {
            if let Some(mut teller) = world.get_mut::<Teller>(t) {
                *teller = Teller {
                    waarde: waarde.clone(),
                    van,
                    bijwerken: Some((0.0, duur)),
                };
            }
            if let Some(na) = puls_na {
                world.entity_mut(t).insert(Puls { tijd: -na, duur: PULS * duur });
            }
        }
    }
}

/// Duur van de puls bij een meerderheid, als deel van de duur van Bijwerken (zie
/// [`Tijden::bijwerken`]): zo schaalt hij mee met de Beweging en de snelheid.
const PULS: f32 = 0.66;

/// De teller groeit even en komt terug als de coalitie een meerderheid haalt.
fn pulseer(mut commands: Commands, tijd: Res<Time>, mut tellers: Query<(Entity, &mut Puls, &mut UiTransform)>) {
    for (e, mut p, mut transform) in &mut tellers {
        p.tijd += tijd.delta_secs();
        let schaal = Vec2::splat(crate::animatie::puls(p.tijd, p.duur));
        if transform.scale != schaal {
            transform.scale = schaal;
        }
        if p.tijd >= p.duur {
            commands.entity(e).remove::<Puls>();
        }
    }
}

/// Haalt de inhoud van een Blok weg, zodat hij opnieuw gebouwd kan worden.
fn herbouw(commands: &mut Commands, knoop: Entity) {
    commands.queue(move |world: &mut World| {
        let binnen: Vec<Entity> = world
            .get::<Children>(knoop)
            .map(|c| {
                c.iter()
                    .filter(|k| world.get::<Binnen>(*k).is_some())
                    .collect()
            })
            .unwrap_or_default();
        for b in binnen {
            world.entity_mut(b).despawn();
        }
    });
}

/// Zet elk Blok in de stand van zijn animatie.
#[allow(clippy::type_complexity)]
fn animeer(
    mut commands: Commands,
    tijd: Res<Time>,
    schaal: Option<Res<UiScale>>,
    mut bestanden: ResMut<Bestanden>,
    beelden: Res<Assets<Image>>,
    mut kaarten: Query<(Entity, &mut Kaart, &Children, &ChildOf)>,
    lagen: Query<&Laag>,
    opbouw: Option<Res<Opbouwstand>>,
    mut knopen: Query<
        (
            &BlokKnoop,
            &mut Node,
            &mut UiTransform,
            Option<&mut Wissel>,
            Option<&Children>,
            Option<&Pasbreedte>,
        ),
        Without<Binnen>,
    >,
    mut binnen: Query<&mut Node, With<Binnen>>,
    kinderen: Query<&Children>,
    mut delen: Query<(
        &Deel,
        Option<&mut BackgroundColor>,
        Option<&mut TextColor>,
        Option<&mut ImageNode>,
    )>,
    mut tellers: Query<(&mut Teller, &mut Text)>,
    mut verlopen: Query<(&Verloopdeel, &mut BackgroundGradient)>,
    mut randen: Query<(&Randkleur, &mut BorderColor)>,
) {
    let dt = tijd.delta_secs();
    let schaal = schaal.map_or(1.0, |s| s.0);
    let opbouw_bezig = opbouw.is_some_and(|o| o.bezig);
    for (kaart_id, mut kaart, blok_ids, laag) in &mut kaarten {
        kaart.tijd += dt;
        // In de Ether wachten de Partijlabels op het eind van een Opbouw in de Zaal.
        let ether = lagen.get(laag.parent()).is_ok_and(|l| l.bron == Bron::Ether);
        kaart.na_opbouw = if ether && opbouw_bezig && kaart.fase != Fase::Eruit {
            0.0
        } else {
            (kaart.na_opbouw + dt / NA_OPBOUW_S).min(1.0)
        };
        let n = kaart.blokken.len();
        match kaart.fase {
            Fase::Wacht(duur) if kaart.tijd >= duur => {
                kaart.fase = Fase::Erin;
                kaart.tijd = 0.0;
            }
            Fase::Erin => {
                let animaties: Vec<_> = kaart.sjabloon.blokken.iter().map(|b| b.animatie).collect();
                if kaart.tijd >= kaart.tijden.erin_klaar(&animaties) {
                    kaart.fase = Fase::Staat;
                }
            }
            Fase::Eruit if kaart.tijd >= kaart.tijden.eruit_klaar(n) => {
                commands.entity(kaart_id).despawn();
                continue;
            }
            _ => {}
        }
        for knoop in blok_ids.iter() {
            let Ok((BlokKnoop { index }, mut node, mut vorm, wissel, knoop_kinderen, pas)) =
                knopen.get_mut(knoop)
            else {
                continue;
            };
            let Some(blok) = kaart.blokken.get(*index).cloned() else {
                continue;
            };
            let animatie = kaart.sjabloon.blokken[*index].animatie;
            let aard = Aard::van(&blok.inhoud);
            let t = match kaart.fase {
                Fase::Wacht(_) => kaart.tijden.erin(&animatie, aard, 0.0),
                Fase::Erin => kaart.tijden.erin(&animatie, aard, kaart.tijd),
                Fase::Staat => Toestand::VOL,
                Fase::Eruit => kaart.tijden.eruit(&animatie, aard, *index, n, kaart.tijd),
            };
            // Wissel bij Bijwerken: weg, nieuwe inhoud, terug.
            let mut wissel_dekking = 1.0;
            if let Some(mut w) = wissel {
                w.tijd += dt;
                let f = (w.tijd / w.duur).min(1.0);
                if f >= 0.5 {
                    if let Some(nieuw) = w.nieuw.take() {
                        herbouw(&mut commands, knoop);
                        bouw_inhoud(
                            &mut commands,
                            &mut bestanden,
                            &beelden,
                            knoop,
                            &nieuw,
                            0.0,
                            None,
                        );
                    }
                }
                wissel_dekking = (2.0 * f - 1.0).abs();
                if f >= 1.0 {
                    commands.entity(knoop).remove::<Wissel>();
                }
            }

            let p = blok.plek;
            let b = pas.map_or(p.breedte as f32, |p| p.0);
            let h = p.hoogte as f32;
            let schuif = op_hele_pixels(t.schuif, schaal);
            let doel_vorm = UiTransform {
                translation: Val2::px(schuif.x, schuif.y),
                scale: Vec2::splat(t.schaal),
                ..default()
            };
            if *vorm != doel_vorm {
                *vorm = doel_vorm;
            }
            let knip = t.breedte < 1.0 || t.hoogte < 1.0;
            // Openen vanuit lijn: nooit dunner dan de lijn.
            let zichtbaar = if t.hoogte < 1.0 {
                (h * t.hoogte).max(t.lijn.min(h))
            } else {
                h
            };
            let open = ((h - zichtbaar) / 2.0).round();
            let breedte = Val::Px((b * t.breedte).round());
            let hoogte = Val::Px(zichtbaar.round());
            let top = Val::Px(p.y as f32 + open);
            let overflow = if knip {
                Overflow::clip()
            } else {
                Overflow::visible()
            };
            if node.width != breedte
                || node.height != hoogte
                || node.top != top
                || node.overflow != overflow
            {
                node.width = breedte;
                node.height = hoogte;
                node.top = top;
                node.overflow = overflow;
            }
            let na_opbouw = if matches!(blok.inhoud, Inhoud::Partijlabels { .. }) { kaart.na_opbouw } else { 1.0 };
            let dekking = t.dekking * wissel_dekking * na_opbouw;
            for k in knoop_kinderen.into_iter().flatten() {
                if let Ok(mut bn) = binnen.get_mut(*k) {
                    let top = Val::Px(-open);
                    if bn.top != top {
                        bn.top = top;
                    }
                }
                for d in std::iter::once(*k).chain(kinderen.iter_descendants(*k)) {
                    if let Ok((v, mut g)) = verlopen.get_mut(d) {
                        g.set_if_neq(v.bij(dekking));
                    }
                    if let Ok((r, mut rand)) = randen.get_mut(d) {
                        rand.set_if_neq(BorderColor::all(r.0.with_alpha(r.0.alpha() * dekking)));
                    }
                    if let Ok((deel, achtergrond, tekst, beeld)) = delen.get_mut(d) {
                        let c = deel.kleur.with_alpha(deel.kleur.alpha() * dekking);
                        // Een tekst of beeld heeft ook een achtergrond; die blijft leeg.
                        match (tekst, beeld, achtergrond) {
                            (Some(mut tk), _, _) => {
                                tk.set_if_neq(TextColor(c));
                            }
                            (None, Some(mut bl), _) => {
                                if bl.color != c {
                                    bl.color = c;
                                }
                            }
                            (None, None, Some(mut a)) => {
                                a.set_if_neq(BackgroundColor(c));
                            }
                            (None, None, None) => {}
                        }
                    }
                    // Optellen: het getal loopt mee met de animatie erin.
                    if let Ok((teller, mut tekst)) = tellers.get_mut(d) {
                        if teller.bijwerken.is_none() {
                            if let Some(doel) = teller.waarde.getal() {
                                let nu = if animatie.erin == gedeeld::sjabloon::Erin::Optellen
                                    && t.teller < 1.0
                                {
                                    (doel as f32 * t.teller).round() as i64
                                } else {
                                    doel
                                };
                                let s = teller.waarde.tekst_bij(nu);
                                if tekst.0 != s {
                                    tekst.0 = s;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Het grote label van de Opbouw: in de Ether alleen het label van de Partij met de kleur die in
/// de Zaal oplicht. Het schuift kort omhoog als het komt. In het Voorbeeld en na de Opbouw is het
/// weg.
fn toon_opbouwlabel(
    opbouw: Option<Res<Opbouwstand>>,
    mut labels: Query<(Entity, &Opbouwkaart, &mut Visibility, &mut UiTransform)>,
    ouders: Query<&ChildOf>,
    lagen: Query<&Laag>,
) {
    let stap = opbouw.and_then(|o| o.stap.filter(|_| o.bezig));
    for (e, kaart, mut zicht, mut vorm) in &mut labels {
        let ether = || ouders.iter_ancestors(e).any(|a| lagen.get(a).is_ok_and(|l| l.bron == Bron::Ether));
        let t = stap.filter(|(k, _)| *k == kaart.kleur && ether()).map(|(_, t)| t);
        zicht.set_if_neq(if t.is_some() { Visibility::Inherited } else { Visibility::Hidden });
        if let Some(t) = t {
            let f = zacht_uit((t / OPBOUWLABEL_S).min(1.0));
            let doel = UiTransform {
                translation: Val2::px(0.0, (24.0 * (1.0 - f)).round()),
                scale: Vec2::splat(0.94 + 0.06 * f),
                ..default()
            };
            vorm.set_if_neq(doel);
        }
    }
}

/// Bijwerken van getallen en de Zetelbalk.
fn tel_en_schuif(
    mut commands: Commands,
    tijd: Res<Time>,
    mut tellers: Query<(&mut Teller, &mut Text)>,
    mut schuivers: Query<(Entity, &mut Schuift)>,
    kinderen: Query<&Children>,
    mut segmenten: Query<(&Segment, &mut Node), Without<Rijplek>>,
    mut rijen: Query<(&Rijplek, &mut Node), Without<Segment>>,
) {
    let dt = tijd.delta_secs();
    for (mut teller, mut tekst) in &mut tellers {
        let Some((t, duur)) = teller.bijwerken else {
            continue;
        };
        let t = (t + dt).min(duur);
        let doel = teller.waarde.getal().unwrap_or(0);
        let nu = teller.van + ((doel - teller.van) as f32 * zacht_uit(t / duur)).round() as i64;
        let s = teller.waarde.tekst_bij(nu);
        if tekst.0 != s {
            tekst.0 = s;
        }
        teller.bijwerken = if t >= duur { None } else { Some((t, duur)) };
    }
    for (knoop, mut s) in &mut schuivers {
        s.tijd += dt;
        let f = zacht_uit(s.tijd / s.duur);
        for d in kinderen.iter_descendants(knoop) {
            if let Ok((seg, mut node)) = segmenten.get_mut(d) {
                let w = Val::Percent((seg.van + (seg.naar - seg.van) * f) * 100.0);
                if node.width != w {
                    node.width = w;
                }
            }
            // een rij die van plek wisselt, schuift erheen
            if let Ok((rij, mut node)) = rijen.get_mut(d) {
                let top = Val::Px((rij.van + (rij.naar - rij.van) * f).round());
                if node.top != top {
                    node.top = top;
                }
            }
        }
        if s.tijd >= s.duur {
            commands.entity(knoop).remove::<Schuift>();
        }
    }
}

/// Een tekst die op één regel moet blijven, krijgt een kleinere letter tot hij past.
fn krimp_teksten(mut teksten: Query<(&Krimp, &ComputedNode, &mut TextFont)>) {
    for (k, node, mut font) in &mut teksten {
        let bevy::text::FontSize::Px(nu) = font.font_size else {
            continue;
        };
        let breed = node.size.x * node.inverse_scale_factor;
        let nieuw = krimp(k.grootte, nu, breed, k.breedte);
        if (nieuw - nu).abs() >= 1.0 {
            font.font_size = bevy::text::FontSize::Px(nieuw);
        }
    }
}

/// Een label onder een Zetelbalk is alleen te zien als het in zijn deel past, met wat ruimte
/// tot het label van het volgende deel.
fn verberg_wat_niet_past(mut labels: Query<(&PastIn, &ComputedNode, &mut Visibility)>) {
    const RUIMTE: f32 = 10.0;
    for (past, node, mut zicht) in &mut labels {
        let breed = node.size.x * node.inverse_scale_factor;
        // nog niet gemeten (het eerste beeld na Bijwerken): nog niet tonen, anders flitst hij
        let nieuw = if breed > 0.0 && breed + RUIMTE <= past.0 { Visibility::Inherited } else { Visibility::Hidden };
        zicht.set_if_neq(nieuw);
    }
}

/// Een Vlak om Tekst wordt zo breed als de teksten erin, en schuift vloeiend mee als die
/// veranderen (Bijwerken).
fn pas_om_tekst(
    tijd: Res<Time>,
    kaarten: Query<(&Kaart, &Children)>,
    mut knopen: Query<(&BlokKnoop, Option<&mut Pasbreedte>, &Children)>,
    mut binnen: Query<&mut Node, With<Binnen>>,
    kinderen: Query<&Children>,
    teksten: Query<&ComputedNode, With<Text>>,
) {
    let stap = (tijd.delta_secs() * 14.0).min(1.0);
    for (kaart, ids) in &kaarten {
        // De breedte van de tekst van elk Blok, op plaats in het Sjabloon.
        let mut breedtes = vec![None; kaart.blokken.len()];
        for knoop in ids.iter() {
            let Ok((BlokKnoop { index }, _, kids)) = knopen.get(knoop) else {
                continue;
            };
            let b = kids
                .iter()
                .flat_map(|k| std::iter::once(k).chain(kinderen.iter_descendants(k)))
                .find_map(|d| teksten.get(d).ok())
                .map(|n| n.size.x * n.inverse_scale_factor);
            if let Some(slot) = breedtes.get_mut(*index) {
                *slot = b;
            }
        }
        for knoop in ids.iter() {
            let Ok((BlokKnoop { index }, Some(mut pas), kids)) = knopen.get_mut(knoop) else {
                continue;
            };
            let Some(IngevuldBlok {
                plek,
                inhoud: Inhoud::Vlak {
                    om_tekst: Some(om), ..
                },
                ..
            }) = kaart.blokken.get(*index)
            else {
                continue;
            };
            let teksten: Vec<_> = om
                .blokken
                .iter()
                .filter_map(|&n| {
                    let blok = kaart.blokken.get(usize::from(n))?;
                    let Inhoud::Tekst { uitlijning, .. } = &blok.inhoud else {
                        return None;
                    };
                    Some((blok.plek, *uitlijning, breedtes[usize::from(n)]?))
                })
                .collect();
            if teksten.iter().all(|(_, _, b)| *b <= 0.0) {
                continue;
            }
            let doel = om.breedte(*plek, teksten).round();
            // De eerste keer meteen; daarna (Bijwerken) vloeiend.
            let nu = if pas.0 <= 0.0 {
                doel
            } else {
                let v = pas.0 + (doel - pas.0) * stap;
                if (v - doel).abs() < 0.5 { doel } else { v }
            };
            if pas.0 != nu {
                pas.0 = nu;
            }
            for k in kids.iter() {
                if let Ok(mut n) = binnen.get_mut(k) {
                    let w = Val::Px(nu.round());
                    if n.width != w {
                        n.width = w;
                    }
                }
            }
        }
    }
}

/// Een lettertype dat later binnenkomt, vervangt het ingebouwde.
fn zet_letters(bestanden: Res<Bestanden>, mut teksten: Query<(&Letter, &mut TextFont)>) {
    for (letter, mut font) in &mut teksten {
        if let Some(h) = bestanden.letter(&letter.0) {
            let bron = bevy::text::FontSource::Handle(h);
            if font.font != bron {
                font.font = bron;
            }
        }
    }
}

/// Een logo of foto dat later binnenkomt: het Blok wordt opnieuw gebouwd, met het beeld.
fn wacht_op_beelden(
    mut commands: Commands,
    mut bestanden: ResMut<Bestanden>,
    beelden: Res<Assets<Image>>,
    wachtend: Query<(Entity, &WachtOpBeeld, &BlokKnoop, &ChildOf)>,
    kaarten: Query<&Kaart>,
) {
    for (knoop, wacht, BlokKnoop { index }, ouder) in &wachtend {
        if bestanden.beeld(&wacht.0).is_none() {
            continue;
        }
        let Ok(kaart) = kaarten.get(ouder.parent()) else {
            continue;
        };
        let Some(blok) = kaart.blokken.get(*index) else {
            continue;
        };
        commands.entity(knoop).remove::<WachtOpBeeld>();
        herbouw(&mut commands, knoop);
        let dekking = if kaart.fase == Fase::Staat { 1.0 } else { 0.0 };
        bouw_inhoud(
            &mut commands,
            &mut bestanden,
            &beelden,
            knoop,
            blok,
            dekking,
            None,
        );
    }
}

/// De Graphic-laag zonder GPU: alleen de logica die kiest tussen eindbeeld en animatie.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bestanden::BestandenPlugin;
    use gedeeld::{GenummerdeStand, Graphic, GraphicInBeeld};

    fn app(kopie: &std::path::Path) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<Font>()
            .init_resource::<Standen>()
            .add_plugins((BestandenPlugin { kopie: kopie.to_owned(), bron: None }, GraphicPlugin));
        app.world_mut().spawn(Bron::Ether);
        app.update();
        app
    }

    /// Een Titelkaart in de Ether (voorbeeldgegevens), getoond bij volgnummer `toon`.
    fn in_de_ether(volgnummer: u64, toon: u64, kop: &str) -> GenummerdeStand {
        GenummerdeStand {
            volgnummer,
            stand: Stand {
                ether: Some(GraphicInBeeld {
                    id: toon,
                    versie: toon,
                    graphic: Graphic::Titelkaart { kop: kop.into(), regel: "Voorbeeldgegevens".into() },
                    standpunt: None,
                    getoond: toon,
                    sinds_ms: 0,
                    getoond_door: "Proef".into(),
                    stijl: Stijl::default(),
                    sjabloon: None,
                }),
                ..Default::default()
            },
        }
    }

    /// Zoals de volging een Stand binnenkrijgt.
    fn ontvang(app: &mut App, s: GenummerdeStand) {
        let mut standen = app.world_mut().resource_mut::<Standen>();
        let vorige = standen.huidig.replace(s);
        standen.vorige = vorige;
        app.update();
        app.update();
    }

    fn kaarten(app: &mut App) -> Vec<(Entity, Fase)> {
        let world = app.world_mut();
        world.query::<(Entity, &Kaart)>().iter(world).map(|(e, k)| (e, k.fase)).collect()
    }

    #[test]
    fn een_renderer_die_halverwege_start_toont_meteen_het_eindbeeld_en_houdt_het_bij_herverbinden() {
        let kopie = tempfile::tempdir().unwrap();
        let mut app = app(kopie.path());

        // Opgestart terwijl er al een Graphic in de Ether staat: meteen het eindbeeld.
        ontvang(&mut app, in_de_ether(7, 5, "Al in beeld"));
        let eerst = kaarten(&mut app);
        assert_eq!(eerst.len(), 1);
        assert_eq!(eerst[0].1, Fase::Staat);

        // De server viel weg en kwam terug: dezelfde Stand nog eens verandert niets in beeld.
        ontvang(&mut app, in_de_ether(7, 5, "Al in beeld"));
        assert_eq!(kaarten(&mut app), eerst);

        // Een nieuwe Toon daarna animeert wel: de oude gaat eruit, de nieuwe wacht en komt erin.
        ontvang(&mut app, in_de_ether(8, 8, "Nieuw"));
        let mut fasen: Vec<Fase> = kaarten(&mut app).into_iter().map(|(_, f)| f).collect();
        fasen.sort_by_key(|f| format!("{f:?}"));
        assert!(fasen.contains(&Fase::Eruit), "{fasen:?}");
        assert!(!fasen.contains(&Fase::Staat), "{fasen:?}");
    }

    fn stijlen(app: &mut App) -> Vec<Stijl> {
        let world = app.world_mut();
        world.query::<&Kaart>().iter(world).map(|k| k.stijl.clone()).collect()
    }

    /// Een Renderer die halverwege start (zoals de Reserve), na een nieuwe Stijl: de Graphic in
    /// de Ether staat er in haar eigen vorm, niet in de nieuwe Stijl van de Stand.
    #[test]
    fn de_graphic_in_de_ether_houdt_haar_eigen_stijl_ook_na_een_herstart() {
        let kopie = tempfile::tempdir().unwrap();
        let mut app = app(kopie.path());
        let mut nieuw = Stijl::default();
        nieuw.kleuren.hoofd = "#ffd400".parse().unwrap(); // voorbeeldkleur
        let mut s = in_de_ether(9, 5, "Oude vorm");
        s.stand.stijl = nieuw.clone();
        ontvang(&mut app, s.clone());
        assert_eq!(stijlen(&mut app), [Stijl::default()]);

        // Bijwerken van dezelfde Toon houdt de vorm; een nieuwe Toon neemt de nieuwe vorm.
        let e = s.stand.ether.as_mut().unwrap();
        e.versie = 10;
        e.graphic = Graphic::Titelkaart { kop: "Oude vorm, bijgewerkt".into(), regel: "Voorbeeldgegevens".into() };
        s.volgnummer = 10;
        ontvang(&mut app, s.clone());
        assert_eq!(stijlen(&mut app), [Stijl::default()]);

        let e = s.stand.ether.as_mut().unwrap();
        (e.getoond, e.stijl) = (11, nieuw.clone());
        s.volgnummer = 11;
        ontvang(&mut app, s);
        assert!(stijlen(&mut app).contains(&nieuw));
    }

    /// Een eigen Sjabloon (voorbeeldgegevens): een kop uit een invulveld op de Zaal, donker en
    /// onscherp.
    fn eigen_sjabloon(kop_grootte: u16) -> Sjabloon {
        use gedeeld::sjabloon::{self as sj, Blok, Kleurkeuze, Stijlkleur, Stijlletter, Stoelen, Tekstbron};
        Sjabloon {
            id: "eigen-proef".into(),
            naam: "Proef".into(),
            groep: "Eigen".into(),
            snelheid: 100,
            blokken: vec![Blok {
                plek: sj::plek(140, 790, 1000, 100),
                soort: sj::tekst(Tekstbron::Veld("kop".into()), Stijlletter::Kop, kop_grootte),
                kleur: Kleurkeuze::Stijl(Stijlkleur::Hoofd),
                animatie: sj::animatie(sj::Erin::Vervagen, sj::Eruit::Vervagen, 0),
            }],
            achtergrond: Achtergrond::ZaalDonker,
            stoelen: Stoelen::Houden,
            standpunt: None,
            kleurovergang: None,
        }
    }

    /// De Graphic in de Ether tekent de kopie van haar Sjabloon, ook als de omroep het Sjabloon
    /// daarna bewerkte; het Voorbeeld tekent het bewerkte. Beide met de achtergrond als eerste
    /// Blok, en achter Zaal donker is de Zaal onscherp.
    #[test]
    fn de_ether_tekent_haar_kopie_van_het_sjabloon_en_het_voorbeeld_het_nieuwste() {
        let kopie = tempfile::tempdir().unwrap();
        let mut app = app(kopie.path());
        let voorbeeld = app.world_mut().spawn(Bron::Klaar).id();
        app.update();
        let graphic = Graphic::Eigen {
            id: "eigen-proef".into(),
            velden: [("kop".into(), "Voorbeeldgegevens".into())].into(),
            set: None,
            partij: None,
            cijfers: None,
        };
        let mut s = in_de_ether(3, 2, "");
        let e = s.stand.ether.as_mut().unwrap();
        e.graphic = graphic.clone();
        e.sjabloon = Some(eigen_sjabloon(80));
        s.stand.sjablonen.push(gedeeld::sjabloon::BewerktSjabloon { sjabloon: eigen_sjabloon(60), standaard: None });
        s.stand.klaar = Some(gedeeld::KlaargezetteGraphic {
            id: 2,
            versie: 3,
            graphic,
            standpunt: None,
            standpunt_van_sjabloon: None,
            klaargezet_door: "Proef".into(),
            gewijzigd_door: "Vormgeving".into(),
        });
        ontvang(&mut app, s);

        let world = app.world_mut();
        let mut grootten: Vec<(Bron, u16, Inhoud)> = world
            .query::<(&Kaart, &ChildOf)>()
            .iter(world)
            .map(|(k, ouder)| {
                let bron = world.get::<Laag>(ouder.parent()).unwrap().bron;
                let Inhoud::Tekst { grootte, .. } = &k.blokken[1].inhoud else { panic!("{:?}", k.blokken[1]) };
                (bron, *grootte, k.blokken[0].inhoud.clone())
            })
            .collect();
        grootten.sort_by_key(|(b, ..)| *b == Bron::Klaar);
        assert_eq!(grootten.iter().map(|(b, g, _)| (*b, *g)).collect::<Vec<_>>(), [(Bron::Ether, 80), (Bron::Klaar, 60)]);
        assert!(matches!(grootten[0].2, Inhoud::Vlak { dekking: gedeeld::sjabloon::DONKER, .. }), "{:?}", grootten[0].2);
        app.update();
        assert!(app.world().get::<DepthOfField>(voorbeeld).is_some(), "het Voorbeeld is meteen onscherp");
    }

}
