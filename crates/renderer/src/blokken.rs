//! Tekent één ingevuld Blok als UI in Bevy: de acht gewone Blokken.
//!
//! Elk Blok is een knoop op zijn plek in het 1080p-beeld ([`BlokKnoop`]) met daarin de inhoud
//! ([`Binnen`]). De animatie verschuift, schaalt en knipt de buitenste knoop en zet de
//! doorzichtigheid op elk [`Deel`]. Een nieuw Blok: een tak in [`bouw_inhoud`].

use bevy::prelude::*;
use bevy::ui::{ColorStop, Gradient, InterpolationColorSpace, LinearGradient};
use gedeeld::Kleur;
use gedeeld::sjabloon::{Getalwaarde, IngevuldBlok, IngevuldVerloop, Inhoud, Uitlijning, Zeteldeel};
use gedeeld::zetelbalk::Pijl;

use crate::bestanden::Bestanden;

/// De knoop van een Blok op zijn plek. `index` is de plaats in het Sjabloon.
#[derive(Component)]
pub struct BlokKnoop {
    pub index: usize,
}

/// De inhoud van een Blok, even groot als de plek. Bij Bijwerken bouwt de Renderer hem opnieuw.
#[derive(Component)]
pub struct Binnen;

/// Een deel met een kleur. De animatie zet daar de doorzichtigheid op.
#[derive(Component, Clone, Copy)]
pub struct Deel {
    pub kleur: Color,
}

/// Een tekst in dit lettertype (bestandsnaam). Zolang het niet binnen is: het ingebouwde.
#[derive(Component)]
pub struct Letter(pub String);

/// Een getal dat telt: bij optellen vanaf nul, bij Bijwerken vanaf het oude getal.
#[derive(Component)]
pub struct Teller {
    pub waarde: Getalwaarde,
    pub van: i64,
    /// Voortgang van Bijwerken (0 tot 1); `None`: de teller volgt de animatie erin.
    pub bijwerken: Option<(f32, f32)>,
}

/// De teller van de Coalitiebouwer haalt een meerderheid: hij groeit even en komt terug
/// ([`crate::animatie::puls`]). `tijd` begint negatief: zo lang wacht de puls nog.
#[derive(Component)]
pub struct Puls {
    pub tijd: f32,
    pub duur: f32,
}

/// Een deel van de Zetelbalk dat van breedte verandert bij Bijwerken.
#[derive(Component)]
pub struct Segment {
    pub van: f32,
    pub naar: f32,
}

/// Een kleurverloop met de kleuren van het eindbeeld. De animatie zet de doorzichtigheid op elke
/// kleur.
#[derive(Component, Clone)]
pub struct Verloopdeel {
    pub hoek: f32,
    pub kleuren: Vec<(Color, f32)>,
}

impl Verloopdeel {
    fn van(v: &IngevuldVerloop) -> Self {
        let k = |c: Kleur, d: u8| kleur(c).with_alpha(f32::from(d) / 100.0);
        Verloopdeel {
            hoek: f32::from(v.richting).to_radians(),
            kleuren: vec![
                (k(v.begin, v.begin_dekking), f32::from(v.van)),
                (k(v.eind, v.eind_dekking), f32::from(v.tot)),
            ],
        }
    }

    /// Het verloop bij deze doorzichtigheid; tussen de kleuren gelijkmatig in sRGB, zoals CSS.
    pub fn bij(&self, dekking: f32) -> BackgroundGradient {
        let stops = self
            .kleuren
            .iter()
            .map(|(c, punt)| ColorStop::new(c.with_alpha(c.alpha() * dekking), Val::Percent(*punt)))
            .collect();
        BackgroundGradient(vec![Gradient::Linear(LinearGradient {
            color_space: InterpolationColorSpace::Srgba,
            ..LinearGradient::new(self.hoek, stops)
        })])
    }
}

/// Het label van één Partij in het grote label van de Opbouw: te zien zolang haar kleur in de
/// Zaal oplicht.
#[derive(Component, Clone, Copy)]
pub struct Opbouwkaart {
    pub kleur: Kleur,
}

/// De kleur van de rand van een Vlak in het eindbeeld.
#[derive(Component, Clone, Copy)]
pub struct Randkleur(pub Color);

/// Hoeveel kaartjes van Partijlabels op één rij passen voordat ze over twee rijen gaan.
const KAARTJES_PER_RIJ: usize = 8;
/// Ruimte tussen de kaartjes, naast en onder elkaar.
const KAARTJE_TUSSEN: f32 = 8.0;

/// Een tekst die op één regel blijft: de letter krimpt tot hij in `breedte` past.
#[derive(Component, Clone, Copy)]
pub struct Krimp {
    pub grootte: f32,
    pub breedte: f32,
}

/// De lettergrootte waarbij een tekst die bij lettergrootte `nu` zo `breed` is, in `beschikbaar`
/// past; nooit groter dan de eigen `grootte`. Hele pixels, zodat het niet blijft schuiven.
pub fn krimp(grootte: f32, nu: f32, breed: f32, beschikbaar: f32) -> f32 {
    if breed <= 0.0 {
        return nu;
    }
    (nu * beschikbaar / breed).floor().min(grootte)
}

/// De breedte van een Vlak om Tekst, in pixels van het 1080p-beeld; tot de tekst er is: 0.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Pasbreedte(pub f32);

/// Een Blok dat op een beeld wacht dat nog niet binnen is; daarna bouwt de Renderer het opnieuw.
#[derive(Component)]
pub struct WachtOpBeeld(pub String);

pub fn kleur(k: Kleur) -> Color {
    Color::srgb_u8(k.r, k.g, k.b)
}

/// Wit of donker: wat leesbaar is op deze kleur.
pub fn leesbaar_op(k: Kleur) -> Color {
    if is_licht(k) {
        Color::srgb_u8(0x0d, 0x12, 0x20)
    } else {
        Color::WHITE
    }
}

/// Of donkere letters op deze kleur beter te lezen zijn dan witte.
pub fn is_licht(k: Kleur) -> bool {
    let lin = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(k.r) + 0.7152 * lin(k.g) + 0.0722 * lin(k.b) > 0.4
}

/// De knoop van een Blok op zijn plek, met de inhoud erin.
pub fn bouw_blok(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    beelden: &Assets<Image>,
    ouder: Entity,
    index: usize,
    blok: &IngevuldBlok,
    dekking: f32,
) -> Entity {
    let p = blok.plek;
    let knoop = commands
        .spawn((
            ChildOf(ouder),
            BlokKnoop { index },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(p.x as f32),
                top: Val::Px(p.y as f32),
                width: Val::Px(p.breedte as f32),
                height: Val::Px(p.hoogte as f32),
                ..default()
            },
            UiTransform::default(),
        ))
        .id();
    if matches!(blok.inhoud, Inhoud::Vlak { om_tekst: Some(_), .. }) {
        commands.entity(knoop).insert(Pasbreedte(0.0));
    }
    bouw_inhoud(commands, bestanden, beelden, knoop, blok, dekking, None);
    knoop
}

/// De oude delen van een Zetelbalk of Uitslaggrafiek bij Bijwerken, en hoe lang dat duurt.
pub struct Vorig<'a> {
    pub delen: &'a [Zeteldeel],
    pub duur: f32,
}

/// Tussen de delen van een Zetelbalk, in pixels.
const TUSSENRUIMTE: f32 = 3.0;
/// Waar de as van de Uitslaggrafiek begint: de korte namen staan ervoor, rechts uitgelijnd.
const AS_X: f32 = 300.0;
/// Waar de baan van de Winst-en-verliesgrafiek begint, met de korte namen ervoor.
const AS_X_MIDDEN: f32 = 200.0;
/// Ruimte aan beide kanten van de baan van de Winst-en-verliesgrafiek voor het getal naast de
/// langste balk, zoals "−20 nu 0" en "+12 nieuw".
const GETALRUIMTE: f32 = 110.0;

/// Een rij van de Uitslaggrafiek die bij Bijwerken van `van` naar `naar` schuift (de afstand
/// van boven in pixels), als hij van plek wisselt.
#[derive(Component)]
pub struct Rijplek {
    pub van: f32,
    pub naar: f32,
}

/// Winst of verlies als tekst: "+17", "0" of "−20" met een echt minteken.
fn winsttekst(verschil: i32) -> String {
    Getalwaarde::Getal { waarde: i64::from(verschil), plus: true, achter: String::new() }.tekst()
}

/// Een label onder een Zetelbalk dat alleen te zien is als het in zijn deel past: zo breed is
/// het deel, in pixels van het 1080p-beeld.
#[derive(Component)]
pub struct PastIn(pub f32);

/// De lettertypes en kleuren van de labels onder een Zetelbalk.
struct Labelletters<'a> {
    cijfers: &'a str,
    naam: &'a str,
    tekst: Color,
    naam_kleur: Color,
}

/// Elk deel met zijn breedte voor en na Bijwerken, en de Zetels ervoor. Zonder Bijwerken staat
/// alles meteen op zijn plek; bij Bijwerken groeit een nieuw deel vanaf nul en krimpt een deel
/// dat weg is naar nul.
fn met_vorige<'a>(
    delen: &'a [Zeteldeel],
    vorig: Option<&Vorig<'a>>,
    breed: impl Fn(&Zeteldeel) -> f32,
) -> Vec<(&'a Zeteldeel, f32, f32, Option<u16>)> {
    let oud = |d: &Zeteldeel| vorig.and_then(|v| v.delen.iter().find(|o| o.partij == d.partij));
    let mut alle: Vec<_> = delen
        .iter()
        .map(|d| match (vorig, oud(d)) {
            (None, _) => (d, breed(d), breed(d), None),
            (Some(_), None) => (d, 0.0, breed(d), Some(0)),
            (Some(_), Some(o)) => (d, breed(o), breed(d), Some(o.zetels)),
        })
        .collect();
    if let Some(v) = vorig {
        for o in v.delen.iter().filter(|o| !delen.iter().any(|d| d.partij == o.partij)) {
            alle.push((o, breed(o), 0.0, Some(o.zetels)));
        }
    }
    alle
}

/// Hoe ver de tweede rij labels onder de eerste staat, in pixels.
const LABELRIJ: f32 = 46.0;

/// Op welke rij het label van elk deel staat en hoeveel ruimte het heeft, in pixels. `delen` is
/// per deel: waar het begint, en de geschatte breedte van zijn label (geen label: `None`). Een
/// label staat op de eerste rij als het vorige label daar al ophoudt; anders, met `twee_rijen`,
/// op de tweede rij; anders nergens. Zijn ruimte loopt tot het volgende label op dezelfde rij,
/// zodat "9 JA21" onder de Coalitiebalk ook past als het deel van JA21 smaller is (mockup
/// `3-zetelbalk.jpg`).
fn labelrijen(delen: &[(f32, Option<f32>)], b: f32, twee_rijen: bool) -> Vec<Option<(usize, f32)>> {
    const RUIMTE: f32 = 10.0;
    let mut eind = [f32::NEG_INFINITY; 2];
    let rijen: Vec<Option<usize>> = delen
        .iter()
        .map(|&(x, breed)| {
            let breed = breed?;
            let rij = (0..if twee_rijen { 2 } else { 1 }).find(|&r| x >= eind[r])?;
            eind[rij] = x + breed + RUIMTE;
            Some(rij)
        })
        .collect();
    rijen
        .iter()
        .enumerate()
        .map(|(i, rij)| {
            let rij = (*rij)?;
            let volgende = (i + 1..delen.len()).find(|&j| rijen[j] == Some(rij)).map_or(b, |j| delen[j].0);
            Some((rij, volgende - delen[i].0))
        })
        .collect()
}

/// De geschatte breedte van een label onder een Zetelbalk bij letter `g`: de Zetels in de
/// cijferletter, 8 pixels ruimte en de korte naam.
fn geschat_label(d: &Zeteldeel, g: f32) -> f32 {
    let cijfers = d.zetels.to_string().len() as f32;
    (cijfers * 0.62 + d.kort.chars().count() as f32 * 0.6) * g + 8.0
}

/// Een getal dat bij Bijwerken telt vanaf `was`.
fn teller(zetels: u16, was: Option<u16>, duur: Option<f32>) -> Teller {
    Teller {
        waarde: Getalwaarde::Getal { waarde: i64::from(zetels), plus: false, achter: String::new() },
        van: was.map_or(0, i64::from),
        bijwerken: was.zip(duur).map(|(_, d)| (0.0, d)),
    }
}

/// De Zetels in de cijferletter en Tekst, de korte naam in Hoofd, en bij Twee sets het pijltje
/// op een regel eronder.
#[allow(clippy::too_many_arguments)]
fn zetellabel(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    label: Entity,
    d: &Zeteldeel,
    was: Option<u16>,
    duur: Option<f32>,
    l: &Labelletters,
    grootte: f32,
    dekking: f32,
) {
    let regel = commands
        .spawn((
            ChildOf(label),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Baseline,
                column_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .id();
    let t = tekst_deel(commands, bestanden, regel, &d.zetels.to_string(), l.cijfers, grootte, l.tekst, Uitlijning::Links, dekking);
    commands.entity(t).insert((TextLayout::no_wrap(), teller(d.zetels, was, duur)));
    let t = tekst_deel(commands, bestanden, regel, &d.kort, l.naam, grootte, l.naam_kleur, Uitlijning::Links, dekking);
    commands.entity(t).insert(TextLayout::no_wrap());
    if let Some(p) = d.pijl {
        let (teken, getal) = match p {
            Pijl::Omhoog(n) => ("\u{25B2}", n.to_string()),
            Pijl::Omlaag(n) => ("\u{25BC}", n.to_string()),
            Pijl::Gelijk => ("=", String::new()),
        };
        let pijl = commands
            .spawn((
                ChildOf(label),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Baseline,
                    column_gap: Val::Px(4.0),
                    ..default()
                },
            ))
            .id();
        let t = tekst_deel(commands, bestanden, pijl, teken, gedeeld::sjabloon::LETTER_VET, (grootte * 0.7).round(), l.naam_kleur, Uitlijning::Links, dekking);
        commands.entity(t).insert(TextLayout::no_wrap());
        if !getal.is_empty() {
            let t = tekst_deel(commands, bestanden, pijl, &getal, l.cijfers, grootte, l.naam_kleur, Uitlijning::Links, dekking);
            commands.entity(t).insert(TextLayout::no_wrap());
        }
    }
}

/// De inhoud van een Blok in `knoop`. `vorig`: de oude delen van een Zetelbalk of
/// Uitslaggrafiek bij Bijwerken.
#[allow(clippy::too_many_arguments)]
pub fn bouw_inhoud(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    beelden: &Assets<Image>,
    knoop: Entity,
    blok: &IngevuldBlok,
    dekking: f32,
    vorig: Option<Vorig>,
) {
    let p = blok.plek;
    let (b, h) = (p.breedte as f32, p.hoogte as f32);
    let vol = Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: Val::Px(b),
        height: Val::Px(h),
        align_items: AlignItems::Center,
        ..default()
    };
    let binnen = commands.spawn((ChildOf(knoop), Binnen, vol.clone())).id();
    let k = kleur(blok.kleur);
    // Een vlak in een kleur, met ronde hoeken.
    let vlak = |commands: &mut Commands, kl: Color, hoek: u16, node: Node| {
        commands.entity(binnen).insert((
            BackgroundColor(kl.with_alpha(kl.alpha() * dekking)),
            Deel { kleur: kl },
            Node {
                border_radius: BorderRadius::all(Val::Px(f32::from(hoek))),
                ..node
            },
        ));
    };
    let midden = Node {
        justify_content: JustifyContent::Center,
        ..vol.clone()
    };
    match &blok.inhoud {
        Inhoud::Vlak {
            hoek,
            dekking: d,
            verloop,
            draaiing,
            rand,
            om_tekst: _,
        } => {
            let mut node = vol;
            if let Some((rk, rd, dikte)) = rand {
                let rk = kleur(*rk).with_alpha(f32::from(*rd) / 100.0);
                node.border = UiRect::all(Val::Px(f32::from(*dikte)));
                commands
                    .entity(binnen)
                    .insert((Randkleur(rk), BorderColor::all(rk.with_alpha(rk.alpha() * dekking))));
            }
            if *draaiing != 0 {
                commands.entity(binnen).insert(UiTransform::from_rotation(Rot2::degrees(
                    f32::from(*draaiing),
                )));
            }
            match verloop {
                Some(v) => {
                    let deel = Verloopdeel::van(v);
                    commands.entity(binnen).insert((
                        deel.bij(dekking),
                        deel,
                        Node {
                            border_radius: BorderRadius::all(Val::Px(f32::from(*hoek))),
                            ..node
                        },
                    ));
                }
                None => vlak(commands, k.with_alpha(f32::from(*d) / 100.0), *hoek, node),
            }
        }
        Inhoud::Lijn { dekking: d } => vlak(commands, k.with_alpha(f32::from(*d) / 100.0), 0, vol),
        Inhoud::Tekst {
            tekst,
            letter,
            grootte,
            uitlijning,
            een_regel,
        } => {
            commands.entity(binnen).insert(Node {
                justify_content: rechtvaardig(*uitlijning),
                ..vol
            });
            let t = tekst_deel(
                commands,
                bestanden,
                binnen,
                tekst,
                letter,
                f32::from(*grootte),
                k,
                *uitlijning,
                dekking,
            );
            if *een_regel {
                commands.entity(t).insert((
                    TextLayout::no_wrap().with_justify(rechtvaardig_tekst(*uitlijning)),
                    Node {
                        flex_shrink: 0.0,
                        ..default()
                    },
                    Krimp {
                        grootte: f32::from(*grootte),
                        breedte: b,
                    },
                ));
            }
        }
        Inhoud::Getal {
            waarde,
            letter,
            grootte,
            uitlijning,
            meerderheid: _,
        } => {
            commands.entity(binnen).insert(Node {
                justify_content: rechtvaardig(*uitlijning),
                ..vol
            });
            let t = tekst_deel(
                commands,
                bestanden,
                binnen,
                &waarde.tekst(),
                letter,
                f32::from(*grootte),
                k,
                *uitlijning,
                dekking,
            );
            // een getal blijft op één regel; past het niet (de teller op 100 of meer), dan wordt
            // de letter kleiner
            commands.entity(t).insert((
                Teller {
                    waarde: waarde.clone(),
                    van: 0,
                    bijwerken: None,
                },
                TextLayout::no_wrap().with_justify(rechtvaardig_tekst(*uitlijning)),
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                Krimp {
                    grootte: f32::from(*grootte),
                    breedte: b,
                },
            ));
        }
        Inhoud::Zetelbalk {
            delen,
            letter,
            lijn_bij_76,
            naamletter,
            labels,
            twee_labelrijen,
            tekstkleur,
            naamkleur,
        } => {
            commands.entity(binnen).insert(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..vol
            });
            let breed = |d: &Zeteldeel| f32::from(d.zetels) / gedeeld::cijfers::ZETELS as f32;
            let letters = Labelletters {
                cijfers: letter,
                naam: naamletter,
                tekst: kleur(*tekstkleur),
                naam_kleur: kleur(*naamkleur),
            };
            let delen = met_vorige(delen, vorig.as_ref(), breed);
            let mut x = 0.0;
            let plaatsen: Vec<(f32, Option<f32>)> = delen
                .iter()
                .map(|(d, _, naar, _)| {
                    let begin = x;
                    x += naar * b;
                    let label = *labels && *naar > 0.0 && d.partij.is_some() && !d.kort.is_empty();
                    (begin, label.then(|| geschat_label(d, 30.0)))
                })
                .collect();
            let rijen = labelrijen(&plaatsen, b, *twee_labelrijen);
            for ((d, van, naar, was), plaats) in delen.into_iter().zip(rijen) {
                let kl = kleur(d.kleur).with_alpha(f32::from(d.dekking) / 100.0);
                let seg = commands
                    .spawn((
                        ChildOf(binnen),
                        Segment { van, naar },
                        Node {
                            width: Val::Percent(van * 100.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ))
                    .id();
                // het vlak, met 3 pixels tussenruimte naar het volgende deel
                commands.spawn((
                    ChildOf(seg),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        right: Val::Px(TUSSENRUIMTE),
                        ..default()
                    },
                    BackgroundColor(kl.with_alpha(kl.alpha() * dekking)),
                    Deel { kleur: kl },
                ));
                // zonder korte naam geen label: Zetels zonder Partij, en in de Coalitiebalk de
                // Partijen buiten de coalitie
                if let Some((rij, ruimte)) = plaats {
                    let label = commands
                        .spawn((
                            ChildOf(seg),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(h + 20.0 + rij as f32 * LABELRIJ),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                ..default()
                            },
                            PastIn(ruimte),
                        ))
                        .id();
                    zetellabel(commands, bestanden, label, d, was, vorig.as_ref().map(|v| v.duur), &letters, 30.0, dekking);
                }
            }
            if *lijn_bij_76 {
                let x = b * 76.0 / gedeeld::cijfers::ZETELS as f32;
                commands.spawn((
                    ChildOf(binnen),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px((x - 4.0).round()),
                        top: Val::Px(-16.0),
                        width: Val::Px(8.0),
                        height: Val::Px(h + 32.0),
                        ..default()
                    },
                    BackgroundColor(k.with_alpha(dekking)),
                    Deel { kleur: k },
                ));
                let label = commands
                    .spawn((
                        ChildOf(binnen),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px((x + 16.0).round()),
                            top: Val::Px(-44.0),
                            ..default()
                        },
                    ))
                    .id();
                let t = tekst_deel(commands, bestanden, label, "76", letter, 30.0, k, Uitlijning::Links, dekking);
                commands.entity(t).insert(TextLayout::no_wrap());
            }
        }
        Inhoud::Uitslaggrafiek {
            rijen,
            as_in_het_midden,
            schaal,
            stap,
            letter,
            naamletter,
            tekstkleur,
            rasterkleur,
        } => {
            if rijen.is_empty() {
                return;
            }
            let midden = *as_in_het_midden;
            let schaal = (*schaal).max(1) as f32;
            // De korte namen staan voor `as_x`. Rechts daarvan de baan; met de as in het midden
            // houdt de baan aan beide kanten ruimte voor het getal naast de langste balk.
            let as_x = if midden { AS_X_MIDDEN } else { AS_X };
            let baan = b - as_x;
            let (nul, lengte) = if midden {
                let half = ((baan - 2.0 * GETALRUIMTE) / 2.0).floor();
                (GETALRUIMTE + half, half)
            } else {
                (0.0, baan)
            };
            let rij_van = |n: usize| (h / n as f32).min(45.0).floor();
            let rij = rij_van(rijen.len());
            let rk = kleur(*rasterkleur).with_alpha(0.22);
            let tk = kleur(*tekstkleur);
            let raster_hoogte = (rij * rijen.len() as f32 + 16.0).min(h);
            let raster = |commands: &mut Commands, x: f32, kl: Color| {
                commands.spawn((
                    ChildOf(binnen),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px((as_x + x).round()),
                        top: Val::Px(0.0),
                        width: Val::Px(2.0),
                        height: Val::Px(raster_hoogte),
                        ..default()
                    },
                    BackgroundColor(kl.with_alpha(kl.alpha() * dekking)),
                    Deel { kleur: kl },
                ));
            };
            for z in (0..schaal as u32).step_by((*stap).max(1) as usize) {
                let x = lengte * z as f32 / schaal;
                if !midden {
                    raster(commands, x, rk);
                } else if z == 0 {
                    // de as zelf, in Tekst op 70 procent
                    raster(commands, nul, tk.with_alpha(0.7));
                } else {
                    raster(commands, nul - x, rk);
                    raster(commands, nul + x, rk);
                }
            }
            // Bij Bijwerken: waar elke rij stond, zodat hij naar zijn nieuwe plek schuift.
            let oude_rijen: Vec<&Zeteldeel> = vorig
                .as_ref()
                .map(|v| v.delen.iter().filter(|d| d.zetels > 0 || d.winst.is_some()).collect())
                .unwrap_or_default();
            let oude_plek = |d: &Zeteldeel| {
                let i = oude_rijen.iter().position(|o| o.partij == d.partij)?;
                Some(8.0 + rij_van(oude_rijen.len()) * i as f32)
            };
            let duur = vorig.as_ref().map(|v| v.duur);
            let breed = |d: &Zeteldeel| f32::from(d.zetels) / schaal;
            let alle = if midden {
                // elke rij, ook met nul verschil; een rij die weggaat verdwijnt meteen
                rijen.iter().map(|d| (d, 0.0, 0.0, None)).collect()
            } else {
                met_vorige(rijen, vorig.as_ref(), breed)
                    .into_iter()
                    .filter(|(_, _, naar, _)| *naar > 0.0)
                    .collect::<Vec<_>>()
            };
            for (i, (d, van, naar, was)) in alle.into_iter().enumerate() {
                let top = 8.0 + rij * i as f32;
                let regel = commands
                    .spawn((
                        ChildOf(binnen),
                        Rijplek { van: oude_plek(d).unwrap_or(top), naar: top },
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(oude_plek(d).unwrap_or(top)),
                            width: Val::Px(b),
                            height: Val::Px(rij),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .id();
                let naam = commands
                    .spawn((
                        ChildOf(regel),
                        Node {
                            width: Val::Px(as_x - 22.0),
                            margin: UiRect::right(Val::Px(22.0)),
                            justify_content: JustifyContent::FlexEnd,
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ))
                    .id();
                let t = tekst_deel(commands, bestanden, naam, &d.kort, naamletter, 32.0, tk, Uitlijning::Rechts, dekking);
                commands.entity(t).insert(TextLayout::no_wrap().with_justify(Justify::Right));
                let baan_id = commands
                    .spawn((
                        ChildOf(regel),
                        Node {
                            width: Val::Px(baan),
                            height: Val::Px(rij),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ))
                    .id();
                let balk = |commands: &mut Commands, ouder: Entity, van: f32, naar: f32| {
                    let kl = kleur(d.kleur).with_alpha(f32::from(d.dekking) / 100.0);
                    commands.spawn((
                        ChildOf(ouder),
                        Segment { van, naar },
                        Node {
                            width: Val::Percent(van * 100.0),
                            height: Val::Px(32.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(kl.with_alpha(kl.alpha() * dekking)),
                        Deel { kleur: kl },
                    ));
                };
                let getal_vak = |commands: &mut Commands, ouder: Entity, marge: UiRect| {
                    commands
                        .spawn((
                            ChildOf(ouder),
                            Node {
                                margin: marge,
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Baseline,
                                column_gap: Val::Px(8.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ))
                        .id()
                };
                let Some(w) = d.winst else {
                    balk(commands, baan_id, van, naar);
                    let getal = getal_vak(commands, baan_id, UiRect::left(Val::Px(16.0)));
                    let t = tekst_deel(commands, bestanden, getal, &d.zetels.to_string(), letter, 34.0, tk, Uitlijning::Links, dekking);
                    commands.entity(t).insert((TextLayout::no_wrap(), teller(d.zetels, was, duur)));
                    continue;
                };
                // Winst naar rechts, verlies naar links: twee balken vanaf de as, waarvan er
                // hooguit één te zien is. Bij Bijwerken krimpt de ene en groeit de andere.
                let oud = vorig.as_ref().map(|v| {
                    v.delen
                        .iter()
                        .find(|o| o.partij == d.partij)
                        .and_then(|o| o.winst)
                        .map_or(0, |o| o.verschil)
                });
                let deel = |v: i32| v.unsigned_abs() as f32 / schaal;
                let (links_naar, rechts_naar) = (deel(w.verschil.min(0)), deel(w.verschil.max(0)));
                let (links_van, rechts_van) =
                    oud.map_or((links_naar, rechts_naar), |o| (deel(o.min(0)), deel(o.max(0))));
                let kant = |commands: &mut Commands, x: f32, eind: bool| {
                    commands
                        .spawn((
                            ChildOf(baan_id),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(x),
                                width: Val::Px(lengte),
                                height: Val::Px(rij),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                // een getal dat niet past, steekt naar buiten uit
                                justify_content: if eind { JustifyContent::FlexEnd } else { JustifyContent::FlexStart },
                                ..default()
                            },
                        ))
                        .id()
                };
                let links = kant(commands, nul - lengte, true);
                let rechts = kant(commands, nul, false);
                let getal = if w.verschil < 0 {
                    let g = getal_vak(commands, links, UiRect::right(Val::Px(16.0)));
                    balk(commands, links, links_van, links_naar);
                    balk(commands, rechts, rechts_van, rechts_naar);
                    g
                } else {
                    balk(commands, links, links_van, links_naar);
                    balk(commands, rechts, rechts_van, rechts_naar);
                    getal_vak(commands, rechts, UiRect::left(Val::Px(14.0)))
                };
                let nul_kleur = kleur(*rasterkleur);
                let t = tekst_deel(
                    commands,
                    bestanden,
                    getal,
                    &winsttekst(w.verschil),
                    letter,
                    34.0,
                    if w.verschil == 0 { nul_kleur } else { tk },
                    Uitlijning::Links,
                    dekking,
                );
                commands.entity(t).insert((
                    TextLayout::no_wrap(),
                    Teller {
                        waarde: Getalwaarde::Getal { waarde: i64::from(w.verschil), plus: true, achter: String::new() },
                        van: i64::from(oud.unwrap_or(w.verschil)),
                        bijwerken: oud.zip(duur).map(|(_, d)| (0.0, d)),
                    },
                ));
                // een nieuwe Partij: "nieuw"; een Partij die verdwijnt: "nu 0"
                let erbij = if w.nieuw {
                    Some("nieuw")
                } else if d.zetels == 0 && w.verschil < 0 {
                    Some("nu 0")
                } else {
                    None
                };
                if let Some(e) = erbij {
                    let t = tekst_deel(commands, bestanden, getal, e, naamletter, 30.0, nul_kleur, Uitlijning::Links, dekking);
                    commands.entity(t).insert(TextLayout::no_wrap());
                }
            }
        }
        Inhoud::Partijlogo {
            logo,
            kort,
            kleur: pk,
            hoek,
        } => {
            commands.entity(binnen).insert(midden.clone());
            if let Some(beeld) = logo
                .as_deref()
                .and_then(|l| beeld_of_wacht(commands, bestanden, knoop, l))
            {
                beeld_deel(commands, binnen, beeld, beelden, b, h, false, dekking, 1.0);
            } else {
                vlak(commands, kleur(*pk), *hoek, midden);
                let grootte = if kort.chars().count() > 4 {
                    h * 0.22
                } else {
                    h * 0.32
                };
                let t = tekst_deel(
                    commands,
                    bestanden,
                    binnen,
                    kort,
                    gedeeld::sjabloon::LETTER_VET,
                    grootte.round(),
                    leesbaar_op(*pk),
                    Uitlijning::Midden,
                    dekking,
                );
                commands
                    .entity(t)
                    .insert(TextLayout::no_wrap().with_justify(Justify::Center));
            }
        }
        Inhoud::Foto {
            foto,
            naam: _,
            hoek,
            voorletters,
            vlakkleur,
            letter,
        } => {
            let foto_node = Node {
                overflow: Overflow::clip(),
                border_radius: BorderRadius::all(Val::Px(f32::from(*hoek))),
                ..midden
            };
            commands.entity(binnen).insert(foto_node.clone());
            if let Some(beeld) = foto
                .as_deref()
                .and_then(|f| beeld_of_wacht(commands, bestanden, knoop, f))
            {
                beeld_deel(commands, binnen, beeld, beelden, b, h, true, dekking, 1.0);
            } else {
                // geen foto: de voorletters in de kleur van het Blok op de partijkleur; op een
                // donkere partijkleur in wit
                vlak(commands, kleur(*vlakkleur), *hoek, foto_node);
                let k = if is_licht(*vlakkleur) { k } else { Color::WHITE };
                let t = tekst_deel(
                    commands,
                    bestanden,
                    binnen,
                    voorletters,
                    letter,
                    (h * 0.5).round(),
                    k,
                    Uitlijning::Midden,
                    dekking,
                );
                commands
                    .entity(t)
                    .insert(TextLayout::no_wrap().with_justify(Justify::Center));
            }
        }
        Inhoud::Partijregel {
            logo,
            kort,
            partijkleur,
            letter,
            grootte,
            tekst,
            tekstletter,
            tekstgrootte,
        } => {
            // op één regel: het logo of de korte naam op een vlak dat om de naam past, dan de tekst
            commands.entity(binnen).insert(Node {
                column_gap: Val::Px(16.0),
                ..vol
            });
            if let Some(beeld) = logo
                .as_deref()
                .and_then(|l| beeld_of_wacht(commands, bestanden, knoop, l))
            {
                let vak = commands
                    .spawn((
                        ChildOf(binnen),
                        Node {
                            height: Val::Px(h),
                            flex_shrink: 0.0,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .id();
                beeld_deel(commands, vak, beeld, beelden, b, h, false, dekking, 1.0);
            } else if !kort.is_empty() {
                let pk = kleur(*partijkleur);
                let chip = commands
                    .spawn((
                        ChildOf(binnen),
                        Node {
                            height: Val::Px(h),
                            padding: UiRect::horizontal(Val::Px(14.0)),
                            align_items: AlignItems::Center,
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(pk.with_alpha(dekking)),
                        Deel { kleur: pk },
                    ))
                    .id();
                let t = tekst_deel(
                    commands,
                    bestanden,
                    chip,
                    kort,
                    letter,
                    f32::from(*grootte),
                    leesbaar_op(*partijkleur),
                    Uitlijning::Links,
                    dekking,
                );
                commands.entity(t).insert(TextLayout::no_wrap());
            }
            if !tekst.is_empty() {
                let t = tekst_deel(
                    commands,
                    bestanden,
                    binnen,
                    tekst,
                    tekstletter,
                    f32::from(*tekstgrootte),
                    k,
                    Uitlijning::Links,
                    dekking,
                );
                commands.entity(t).insert(TextLayout::no_wrap());
            }
        }
        Inhoud::Partijlabels {
            labels,
            letter,
            naamletter,
            grootte,
            tekstkleur,
            naamkleur,
            kaartjes: true,
        } => {
            // Elke Partij een eigen kaartje, even breed, net als het Opbouwlabel: een streep in de
            // partijkleur links, de Zetels, dan de naam. Meer dan KAARTJES_PER_RIJ Partijen gaan
            // over twee rijen, de linker helft van de Zaal boven, zodat de naam groot genoeg blijft.
            let n = labels.len().max(1);
            let per_rij = if n > KAARTJES_PER_RIJ { n.div_ceil(2) } else { n };
            let rijen = n.div_ceil(per_rij) as f32;
            let breed = ((b - KAARTJE_TUSSEN * (per_rij as f32 - 1.0)) / per_rij as f32).floor();
            let hoog = ((h - KAARTJE_TUSSEN * (rijen - 1.0)) / rijen).floor();
            commands.entity(binnen).insert(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(KAARTJE_TUSSEN),
                row_gap: Val::Px(KAARTJE_TUSSEN),
                align_content: AlignContent::FlexStart,
                ..vol
            });
            let (tk, nk) = (kleur(*tekstkleur), kleur(*naamkleur));
            let g = f32::from(*grootte);
            let vlakkleur = k.with_alpha(0.97);
            let streep = 10.0;
            for l in labels {
                let kaartje = commands
                    .spawn((
                        ChildOf(binnen),
                        Node {
                            width: Val::Px(breed),
                            height: Val::Px(hoog),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(14.0),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(vlakkleur.with_alpha(vlakkleur.alpha() * dekking)),
                        Deel { kleur: vlakkleur },
                    ))
                    .id();
                let pk = kleur(l.kleur);
                commands.spawn((
                    ChildOf(kaartje),
                    Node { width: Val::Px(streep), height: Val::Percent(100.0), flex_shrink: 0.0, ..default() },
                    BackgroundColor(pk.with_alpha(dekking)),
                    Deel { kleur: pk },
                ));
                let cijfer = (g * 1.5).round();
                // de Zetels in een vaste kolom, zodat de namen onder elkaar beginnen
                let vak = commands
                    .spawn((
                        ChildOf(kaartje),
                        Node { width: Val::Px(cijfer * 1.25), flex_shrink: 0.0, justify_content: JustifyContent::FlexEnd, ..default() },
                    ))
                    .id();
                if let Some(z) = l.zetels {
                    let t = tekst_deel(commands, bestanden, vak, &z.to_string(), letter, cijfer, tk, Uitlijning::Rechts, dekking);
                    commands.entity(t).insert(TextLayout::no_wrap());
                }
                if let Some(kort) = l.kort.as_deref() {
                    let rest = breed - streep - cijfer * 1.25 - 2.0 * 14.0 - 10.0;
                    let t = tekst_deel(commands, bestanden, kaartje, kort, naamletter, g, nk, Uitlijning::Links, dekking);
                    commands.entity(t).insert((TextLayout::no_wrap(), Krimp { grootte: g, breedte: rest }));
                }
            }
        }
        Inhoud::Opbouwlabel {
            labels,
            letter,
            naamletter,
            grootte,
            tekstkleur,
            naamkleur,
        } => {
            // Per Partij een label midden in de Plek, verborgen tot haar kleur in de Zaal oplicht
            // (zie `crate::graphic::toon_opbouwlabel`).
            let (tk, nk) = (kleur(*tekstkleur), kleur(*naamkleur));
            let g = f32::from(*grootte);
            let vlakkleur = k.with_alpha(0.94);
            for l in labels {
                let plaats = commands
                    .spawn((
                        ChildOf(binnen),
                        Opbouwkaart { kleur: l.kleur },
                        Visibility::Hidden,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            top: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Stretch,
                            ..default()
                        },
                        UiTransform::default(),
                    ))
                    .id();
                let kaart = commands
                    .spawn((
                        ChildOf(plaats),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            padding: UiRect::right(Val::Px(50.0)),
                            ..default()
                        },
                        BackgroundColor(vlakkleur.with_alpha(vlakkleur.alpha() * dekking)),
                        Deel { kleur: vlakkleur },
                    ))
                    .id();
                let pk = kleur(l.kleur);
                commands.spawn((
                    ChildOf(kaart),
                    Node {
                        width: Val::Px(18.0),
                        height: Val::Percent(100.0),
                        margin: UiRect::right(Val::Px(34.0)),
                        ..default()
                    },
                    BackgroundColor(pk.with_alpha(dekking)),
                    Deel { kleur: pk },
                ));
                if let Some(z) = l.zetels {
                    let t = tekst_deel(commands, bestanden, kaart, &z.to_string(), letter, (g * 2.1).round(), tk, Uitlijning::Links, dekking);
                    commands.entity(t).insert(TextLayout::no_wrap());
                }
                let kolom = commands
                    .spawn((
                        ChildOf(kaart),
                        Node {
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::left(Val::Px(36.0)),
                            ..default()
                        },
                    ))
                    .id();
                if let Some(kort) = l.kort.as_deref() {
                    let t = tekst_deel(commands, bestanden, kolom, kort, naamletter, g, nk, Uitlijning::Links, dekking);
                    commands.entity(t).insert(TextLayout::no_wrap());
                }
                if l.zetels.is_some() {
                    let woord = if l.zetels == Some(1) { "zetel" } else { "zetels" };
                    let zacht = tk.with_alpha(0.75);
                    let t = tekst_deel(commands, bestanden, kolom, woord, naamletter, (g * 0.58).round(), zacht, Uitlijning::Links, dekking);
                    commands.entity(t).insert(TextLayout::no_wrap());
                }
            }
        }
        Inhoud::Partijlabels {
            labels,
            letter,
            naamletter,
            grootte,
            tekstkleur,
            naamkleur,
            kaartjes: false,
        } => {
            if labels.is_empty() {
                return;
            }
            // Onder elk deel van de Zetelbalk: een streep in de partijkleur, daaronder de Zetels in de cijferletter en de korte naam in Hoofd.
            vlak(
                commands,
                k.with_alpha(0.93),
                0,
                Node {
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::axes(Val::Px(40.0), Val::Px(18.0)),
                    column_gap: Val::Px(18.0),
                    align_items: AlignItems::Stretch,
                    ..vol
                },
            );
            let (tk, nk) = (kleur(*tekstkleur), kleur(*naamkleur));
            let g = f32::from(*grootte);
            for l in labels {
                let pk = kleur(l.kleur);
                let label = commands
                    .spawn((
                        ChildOf(binnen),
                        Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                            justify_content: JustifyContent::Center,
                            row_gap: Val::Px(2.0),
                            ..default()
                        },
                    ))
                    .id();
                commands.spawn((
                    ChildOf(label),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(8.0),
                        margin: UiRect::bottom(Val::Px(4.0)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    BackgroundColor(pk.with_alpha(dekking)),
                    Deel { kleur: pk },
                ));
                let regels = [
                    l.zetels.map(|z| (z.to_string(), letter, (g * 1.4).round(), tk)),
                    l.kort.as_deref().map(|t| (t.to_string(), naamletter, g, nk)),
                ];
                for (tekst, letter, grootte, kl) in regels.into_iter().flatten() {
                    let t = tekst_deel(
                        commands,
                        bestanden,
                        label,
                        &tekst,
                        letter,
                        grootte,
                        kl,
                        Uitlijning::Links,
                        dekking,
                    );
                    commands.entity(t).insert(TextLayout::no_wrap());
                }
            }
        }
        Inhoud::Stemmingsbalk { .. }
        | Inhoud::Uitkomst { .. }
        | Inhoud::Fractielijst { .. }
        | Inhoud::Namenlijst { .. } => crate::stemmingsblokken::bouw(
            commands,
            bestanden,
            beelden,
            knoop,
            binnen,
            k,
            (b, h),
            &blok.inhoud,
            dekking,
        ),
        Inhoud::Programmalogo {
            logo,
            tekst,
            letter,
            dekking: d,
        } => {
            commands.entity(binnen).insert(Node {
                justify_content: JustifyContent::FlexEnd,
                ..vol
            });
            let d = f32::from(*d) / 100.0;
            if let Some(beeld) = logo
                .as_deref()
                .and_then(|l| beeld_of_wacht(commands, bestanden, knoop, l))
            {
                beeld_deel(commands, binnen, beeld, beelden, b, h, false, dekking, d);
            } else {
                let t = tekst_deel(
                    commands,
                    bestanden,
                    binnen,
                    tekst,
                    letter,
                    (h * 0.3).round(),
                    k.with_alpha(d),
                    Uitlijning::Rechts,
                    dekking,
                );
                commands
                    .entity(t)
                    .insert(TextLayout::no_wrap().with_justify(Justify::Right));
            }
        }
        // De achtergrond van een Sjabloon: vult de hele Plek, het midden blijft in beeld.
        Inhoud::Afbeelding { bestand, dekking: d } => {
            commands.entity(binnen).insert(Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..vol
            });
            if let Some(beeld) = beeld_of_wacht(commands, bestanden, knoop, bestand) {
                beeld_deel(commands, binnen, beeld, beelden, b, h, true, dekking, f32::from(*d) / 100.0);
            }
        }
    }
}

fn rechtvaardig_tekst(u: Uitlijning) -> Justify {
    match u {
        Uitlijning::Links => Justify::Left,
        Uitlijning::Midden => Justify::Center,
        Uitlijning::Rechts => Justify::Right,
    }
}

fn rechtvaardig(u: Uitlijning) -> JustifyContent {
    match u {
        Uitlijning::Links => JustifyContent::FlexStart,
        Uitlijning::Midden => JustifyContent::Center,
        Uitlijning::Rechts => JustifyContent::FlexEnd,
    }
}

/// Een tekst in `ouder`. Het lettertype vraagt de Renderer bij de server op als het er nog niet is.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tekst_deel(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    ouder: Entity,
    tekst: &str,
    letter: &str,
    grootte: f32,
    kl: Color,
    uitlijning: Uitlijning,
    dekking: f32,
) -> Entity {
    bestanden.vraag(letter);
    let font = bestanden
        .letter(letter)
        .unwrap_or_else(|| bestanden.reserve());
    let justify = rechtvaardig_tekst(uitlijning);
    commands
        .spawn((
            ChildOf(ouder),
            Text::new(tekst),
            TextFont::from_font_size(grootte).with_font(font),
            TextColor(kl.with_alpha(kl.alpha() * dekking)),
            TextLayout::justify(justify),
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
            Letter(letter.to_string()),
            Deel { kleur: kl },
        ))
        .id()
}

/// Het beeld als het binnen is; anders vraagt de Renderer het op en wacht het Blok erop.
pub(crate) fn beeld_of_wacht(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    knoop: Entity,
    naam: &str,
) -> Option<Handle<Image>> {
    bestanden.vraag(naam);
    let b = bestanden.beeld(naam);
    if b.is_none() {
        commands
            .entity(knoop)
            .insert(WachtOpBeeld(naam.to_string()));
    }
    b
}

/// Een beeld in `ouder`: passend (logo's) of vullend en bijgesneden (foto's).
#[allow(clippy::too_many_arguments)]
pub(crate) fn beeld_deel(
    commands: &mut Commands,
    ouder: Entity,
    beeld: Handle<Image>,
    beelden: &Assets<Image>,
    b: f32,
    h: f32,
    vullend: bool,
    dekking: f32,
    eigen_dekking: f32,
) {
    let (bw, bh) = beelden
        .get(&beeld)
        .map_or((b, h), |i| (i.width() as f32, i.height() as f32));
    let schaal = if vullend {
        (b / bw).max(h / bh)
    } else {
        (b / bw).min(h / bh)
    };
    commands.spawn((
        ChildOf(ouder),
        ImageNode {
            image: beeld,
            color: Color::WHITE.with_alpha(eigen_dekking * dekking),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        Node {
            width: Val::Px((bw * schaal).round()),
            height: Val::Px((bh * schaal).round()),
            flex_shrink: 0.0,
            ..default()
        },
        Deel {
            kleur: Color::WHITE.with_alpha(eigen_dekking),
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn een_label_mag_doorlopen_over_de_delen_erna_zonder_label() {
        // (begin, label): D66, CDA, JA21, dan de rest van de Coalitiebalk zonder label; 1000 breed
        let delen = [(0.0, Some(150.0)), (200.0, Some(90.0)), (300.0, Some(120.0)), (350.0, None), (650.0, None)];
        let r = labelrijen(&delen, 1000.0, false);
        assert_eq!(r[0], Some((0, 200.0)), "tot het volgende label");
        assert_eq!(r[2], Some((0, 700.0)), "de laatste in de coalitie loopt door tot het eind");
        assert_eq!(r[3], None);
        assert_eq!(labelrijen(&[], 1000.0, false), Vec::new());
    }

    #[test]
    fn een_label_dat_niet_naast_het_vorige_past_gaat_naar_de_tweede_rij_of_valt_weg() {
        // drie smalle delen na elkaar: het tweede botst met het eerste label
        let delen = [(0.0, Some(100.0)), (40.0, Some(100.0)), (80.0, Some(100.0)), (300.0, Some(100.0))];
        let twee = labelrijen(&delen, 1000.0, true);
        assert_eq!(twee[0], Some((0, 300.0)), "het eerste loopt door tot het volgende op zijn rij");
        assert_eq!(twee[1], Some((1, 960.0)), "het tweede op de tweede rij");
        assert_eq!(twee[2], None, "beide rijen bezet");
        assert_eq!(twee[3], Some((0, 700.0)));
        let een = labelrijen(&delen, 1000.0, false);
        assert_eq!(een[1], None, "zonder tweede rij valt het weg");
    }

    #[test]
    fn een_tekst_die_niet_past_krimpt_tot_hij_past_en_groeit_terug_tot_zijn_eigen_grootte() {
        // 92 pixels, 2000 breed, 1640 beschikbaar: 75 pixels
        assert_eq!(krimp(92.0, 92.0, 2000.0, 1640.0), 75.0);
        // daarna blijft het staan: bij 75 pixels is hij 1630 breed
        assert_eq!(krimp(92.0, 75.0, 1630.0, 1640.0), 75.0);
        // een kortere tekst (Bijwerken): terug naar de eigen grootte, niet groter
        assert_eq!(krimp(92.0, 75.0, 300.0, 1640.0), 92.0);
        // nog niet gemeten: niets veranderen
        assert_eq!(krimp(92.0, 92.0, 0.0, 1640.0), 92.0);
    }
}
