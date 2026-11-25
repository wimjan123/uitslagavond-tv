//! De Blokken van de Sjablonen voor Stemmingen (`docs/vormgeving.md`, Stemming): de
//! Stemmingsbalk, de Uitkomst, de Fractielijst en de Namenlijst. [`crate::blokken::bouw_inhoud`]
//! roept ze aan met de lege inhoudsknoop `binnen`.

use bevy::prelude::*;
use gedeeld::sjabloon::{Fractielabel, Inhoud, Naamgroep, STEM_TEGEN, STEM_VOOR, Uitlijning};

use crate::bestanden::Bestanden;
use crate::blokken::{Deel, Krimp, beeld_deel, beeld_of_wacht, kleur, tekst_deel};

/// Ruimte tussen voor en tegen in de balk, in pixels.
const TUSSEN: f32 = 3.0;
/// Ruimte boven en onder de korte naam in een label onder de balk.
const LABEL_RAND: f32 = 4.0;
/// Ruimte tussen de labels onder de balk: naast elkaar en tussen de rijen.
const LABEL_TUSSEN: (f32, f32) = (10.0, 12.0);

/// De geschatte breedte van elk label bij letter `g`: de tekens op 0,6 keer de letter (de
/// cijferletter van Nunito Sans, gemeten op kapitalen) plus de marge.
fn geschatte_breedtes(labels: &[Fractielabel], g: f32) -> Vec<f32> {
    labels.iter().map(|f| naam_met_aantal(f).chars().count() as f32 * 0.6 * g + 28.0).collect()
}

/// Hoeveel rijen labels van deze breedtes nodig hebben in `ruimte`, van links naar rechts.
fn rijen(breedtes: &[f32], ruimte: f32) -> usize {
    let mut n = 0;
    let mut x = f32::INFINITY;
    for &b in breedtes {
        if x + b > ruimte {
            n += 1;
            x = b + LABEL_TUSSEN.0;
        } else {
            x += b + LABEL_TUSSEN.0;
        }
    }
    n
}

/// Waar de labels voor tegen beginnen, in pixels: bij de wissel van de balk (`wissel`, deel van
/// de breedte) zoals in de mockup, maar past een kant daar niet in de rijen van hoogte `h`, dan
/// naar verhouding van de breedte die de labels nodig hebben.
fn verdeel_onder_balk(b: f32, h: f32, wissel: f32, voor: &[f32], tegen: &[f32], g: f32) -> f32 {
    let rij = g * 1.36 + 2.0 * LABEL_RAND;
    let max_rijen = (((h + LABEL_TUSSEN.1) / (rij + LABEL_TUSSEN.1)).floor() as usize).max(1);
    let w = b * wissel;
    if voor.is_empty() || tegen.is_empty() {
        return w;
    }
    if rijen(voor, w - 30.0) <= max_rijen && rijen(tegen, b - w - 20.0) <= max_rijen {
        return w;
    }
    let (v, t): (f32, f32) = (voor.iter().sum(), tegen.iter().sum());
    (30.0 + (b - 50.0) * v / (v + t)).clamp(300.0, b - 300.0)
}

/// Een vlak in een kleur dat meedoet met de animatie.
fn gekleurd(kl: Color, dekking: f32) -> (BackgroundColor, Deel) {
    (BackgroundColor(kl.with_alpha(kl.alpha() * dekking)), Deel { kleur: kl })
}

/// Een knoop op een vaste plek in `ouder`.
fn op(left: f32, top: f32, width: Val, height: Val) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left.round()),
        top: Val::Px(top.round()),
        width,
        height,
        ..default()
    }
}

/// Waar voor overgaat in tegen, als deel van de breedte.
fn wissel(voor: u16, tegen: u16) -> f32 {
    let totaal = f32::from(voor) + f32::from(tegen);
    if totaal == 0.0 { 0.5 } else { f32::from(voor) / totaal }
}

/// "101 voor": het getal in de cijferletter, het woord in de tekstletter, op één basislijn.
#[allow(clippy::too_many_arguments)]
fn getal_en_woord(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    ouder: Entity,
    getal: &str,
    woord: &str,
    (letter, tekstletter): (&str, &str),
    (groot, klein): (f32, f32),
    kl: Color,
    dekking: f32,
) {
    for (t, l, g) in [(getal, letter, groot), (woord, tekstletter, klein)] {
        let e = tekst_deel(commands, bestanden, ouder, t, l, g, kl, Uitlijning::Links, dekking);
        commands.entity(e).insert(TextLayout::no_wrap());
    }
}

/// Een rij met getal en woord, voor in de balk en boven een kolom.
fn rij(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Baseline,
        column_gap: Val::Px(gap),
        ..default()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn bouw(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    beelden: &Assets<Image>,
    knoop: Entity,
    binnen: Entity,
    blokkleur: Color,
    (b, h): (f32, f32),
    inhoud: &Inhoud,
    dekking: f32,
) {
    match inhoud {
        Inhoud::Stemmingsbalk {
            voor,
            tegen,
            algemene_stemmen,
            letter,
            tekstletter,
            tekstkleur,
        } => {
            let tk = kleur(*tekstkleur);
            let w = wissel(*voor, *tegen);
            let breed_voor = if *tegen == 0 { b } else { (b * w - TUSSEN / 2.0).max(0.0) };
            if *voor > 0 {
                commands.spawn((ChildOf(binnen), op(0.0, 0.0, Val::Px(breed_voor.round()), Val::Px(h)), gekleurd(kleur(STEM_VOOR), dekking)));
            }
            if *tegen > 0 {
                let links = if *voor == 0 { 0.0 } else { breed_voor + TUSSEN };
                commands.spawn((ChildOf(binnen), op(links, 0.0, Val::Px((b - links).round()), Val::Px(h)), gekleurd(kleur(STEM_TEGEN), dekking)));
            }
            let letters = (letter.as_str(), tekstletter.as_str());
            // de getallen 50 en de woorden 34 pixels bij een balk van 72 hoog
            let maten = ((h * 50.0 / 72.0).round(), (h * 34.0 / 72.0).round());
            let label = |commands: &mut Commands, rechts: bool| {
                let mut n = op(22.0, 0.0, Val::Auto, Val::Px(h));
                if rechts {
                    n.left = Val::Auto;
                    n.right = Val::Px(22.0);
                }
                n.align_items = AlignItems::Center;
                commands.spawn((ChildOf(binnen), n)).id()
            };
            if *voor > 0 {
                let l = label(commands, false);
                let r = commands.spawn((ChildOf(l), rij(12.0))).id();
                let woord = if *algemene_stemmen { "voor, met algemene stemmen" } else { "voor" };
                getal_en_woord(commands, bestanden, r, &voor.to_string(), woord, letters, maten, tk, dekking);
            }
            if *tegen > 0 {
                let l = label(commands, true);
                let r = commands.spawn((ChildOf(l), rij(12.0))).id();
                getal_en_woord(commands, bestanden, r, &tegen.to_string(), "tegen", letters, maten, tk, dekking);
            }
            // de lijn bij 76, 8 breed, 14 pixels boven en onder de balk uit
            let x = b * 76.0 / gedeeld::cijfers::ZETELS as f32;
            commands.spawn((ChildOf(binnen), op(x - 4.0, -14.0, Val::Px(8.0), Val::Px(h + 28.0)), gekleurd(blokkleur, dekking)));
        }
        Inhoud::Uitkomst {
            tekst,
            achter,
            letter,
            grootte,
        } => {
            if tekst.is_empty() {
                return;
            }
            commands.entity(binnen).insert(Node {
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexStart,
                ..op(0.0, 0.0, Val::Px(b), Val::Px(h))
            });
            let vlak = commands
                .spawn((
                    ChildOf(binnen),
                    Node {
                        padding: UiRect::new(Val::Px(28.0), Val::Px(28.0), Val::Px(14.0), Val::Px(12.0)),
                        ..default()
                    },
                    gekleurd(kleur(*achter), dekking),
                ))
                .id();
            let t = tekst_deel(commands, bestanden, vlak, tekst, letter, f32::from(*grootte), blokkleur, Uitlijning::Links, dekking);
            commands.entity(t).insert(TextLayout::no_wrap());
        }
        Inhoud::Fractielijst {
            voor,
            tegen,
            aantal_voor,
            aantal_tegen,
            letter,
            tekstletter,
            grootte,
            onder_balk,
            tekstkleur,
        } => {
            let g = f32::from(*grootte);
            if *onder_balk {
                // Onder de balk: voor links tot vlak voor de wissel, tegen erna. Passen de labels
                // daar niet in de rijen, dan krijgt elke kant ruimte naar zijn labels.
                let w = verdeel_onder_balk(
                    b,
                    h,
                    wissel(*aantal_voor, *aantal_tegen),
                    &geschatte_breedtes(voor, g),
                    &geschatte_breedtes(tegen, g),
                    g,
                );
                let kanten = [(voor, 0.0, w - 30.0), (tegen, w + 20.0, b - w - 20.0)];
                let kanten = if tegen.is_empty() { &kanten[..1] } else if voor.is_empty() { &[(tegen, 0.0, b)][..] } else { &kanten[..] };
                for (labels, links, breed) in kanten.iter().copied() {
                    let kolom = commands
                        .spawn((
                            ChildOf(binnen),
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(LABEL_TUSSEN.0),
                                row_gap: Val::Px(LABEL_TUSSEN.1),
                                align_content: AlignContent::FlexStart,
                                ..op(links, 0.0, Val::Px(breed.max(0.0).round()), Val::Auto)
                            },
                            Passend { basis: 0.0, ruimte: Vec2::new(breed, h), schaal: 1.0 },
                        ))
                        .id();
                    for f in labels.iter() {
                        label(commands, bestanden, kolom, f, letter, g, (14.0, LABEL_RAND), dekking);
                    }
                }
                return;
            }
            // Twee kolommen: een kop in de stemkleur met "101 voor", daaronder de partijlogo's.
            let kolombreed = (b - 48.0) / 2.0;
            let tk = kleur(*tekstkleur);
            for (n, (labels, aantal, woord, achter)) in
                [(voor, aantal_voor, "voor", STEM_VOOR), (tegen, aantal_tegen, "tegen", STEM_TEGEN)].into_iter().enumerate()
            {
                let kolom = commands
                    .spawn((
                        ChildOf(binnen),
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(24.0),
                            ..op(n as f32 * (kolombreed + 48.0), 0.0, Val::Px(kolombreed.round()), Val::Px(h))
                        },
                    ))
                    .id();
                let kop = commands
                    .spawn((
                        ChildOf(kolom),
                        Node {
                            height: Val::Px(72.0),
                            padding: UiRect::horizontal(Val::Px(22.0)),
                            align_items: AlignItems::Center,
                            flex_shrink: 0.0,
                            ..default()
                        },
                        gekleurd(kleur(achter), dekking),
                    ))
                    .id();
                let r = commands.spawn((ChildOf(kop), rij(12.0))).id();
                getal_en_woord(commands, bestanden, r, &aantal.to_string(), woord, (letter, tekstletter), (50.0, 34.0), tk, dekking);
                let raster = commands
                    .spawn((
                        ChildOf(kolom),
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(24.0),
                            row_gap: Val::Px(20.0),
                            ..default()
                        },
                    ))
                    .id();
                let tegel = ((kolombreed - 48.0) / 3.0).floor();
                for f in labels.iter() {
                    let vak = commands
                        .spawn((
                            ChildOf(raster),
                            Node {
                                width: Val::Px(tegel),
                                height: Val::Px(96.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                overflow: Overflow::clip(),
                                ..default()
                            },
                        ))
                        .id();
                    match f.logo.as_deref().and_then(|l| beeld_of_wacht(commands, bestanden, knoop, l)) {
                        Some(beeld) => beeld_deel(commands, vak, beeld, beelden, tegel - 16.0, 80.0, false, dekking, 1.0),
                        None => {
                            commands.entity(vak).insert(gekleurd(kleur(f.kleur), dekking));
                            let t = tekst_deel(commands, bestanden, vak, &naam_met_aantal(f), letter, g, kleur(f.tekstkleur), Uitlijning::Midden, dekking);
                            commands.entity(t).insert((
                                TextLayout::no_wrap(),
                                Node { flex_shrink: 0.0, ..default() },
                                Krimp { grootte: g, breedte: tegel - 24.0 },
                            ));
                        }
                    }
                }
            }
        }
        Inhoud::Namenlijst {
            voor,
            tegen,
            aantal_voor,
            aantal_tegen,
            letter,
            naamletter,
            grootte,
        } => {
            let g = f32::from(*grootte);
            commands.entity(binnen).insert(Node { overflow: Overflow::clip(), ..op(0.0, 0.0, Val::Px(b), Val::Px(h)) });
            // Twee kolommen, voor links en tegen rechts, als een tabel: elke Fractie op een eigen
            // regel met haar label (en aantal) en daarachter de namen van wie anders stemde. Past
            // het niet, dan krimpt alles.
            let lijst = commands
                .spawn((
                    ChildOf(binnen),
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(48.0),
                        align_items: AlignItems::FlexStart,
                        ..op(0.0, 0.0, Val::Px(b), Val::Auto)
                    },
                    Passend { basis: 0.0, ruimte: Vec2::new(b, h), schaal: 1.0 },
                ))
                .id();
            let kolombreed = ((b - 48.0) / 2.0).floor();
            let tk = blokkleur;
            for (groepen, aantal, woord, achter) in [(voor, aantal_voor, "voor", STEM_VOOR), (tegen, aantal_tegen, "tegen", STEM_TEGEN)] {
                let kolom = commands
                    .spawn((
                        ChildOf(lijst),
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            width: Val::Px(kolombreed),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ))
                    .id();
                if groepen.is_empty() {
                    continue;
                }
                let kop = commands
                    .spawn((
                        ChildOf(kolom),
                        Node {
                            padding: UiRect::new(Val::Px(18.0), Val::Px(22.0), Val::Px(4.0), Val::Px(4.0)),
                            margin: UiRect::bottom(Val::Px(8.0)),
                            align_self: AlignSelf::Stretch,
                            ..default()
                        },
                        gekleurd(kleur(achter), dekking),
                    ))
                    .id();
                let r = commands.spawn((ChildOf(kop), rij(10.0))).id();
                for (t, l, s) in [(aantal.to_string(), letter.as_str(), g * 1.4), (woord.to_string(), naamletter.as_str(), g)] {
                    let e = tekst_deel(commands, bestanden, r, &t, l, s.round(), Color::WHITE, Uitlijning::Links, dekking);
                    commands.entity(e).insert((TextLayout::no_wrap(), Passend::basis(s.round())));
                }
                for Naamgroep { fractie, namen } in groepen.iter() {
                    let regel = commands
                        .spawn((
                            ChildOf(kolom),
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(12.0),
                                row_gap: Val::Px(4.0),
                                ..default()
                            },
                        ))
                        .id();
                    // het label even breed als de andere, met de naam links en het aantal rechts
                    let vak = commands
                        .spawn((
                            ChildOf(regel),
                            Node {
                                min_width: Val::Px((kolombreed * 0.4).round()),
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(2.0)),
                                justify_content: JustifyContent::SpaceBetween,
                                column_gap: Val::Px(12.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            gekleurd(kleur(fractie.kleur), dekking),
                        ))
                        .id();
                    let aantal = fractie.aantal.map(|a| a.to_string()).unwrap_or_default();
                    for t in [fractie.kort.as_str(), aantal.as_str()] {
                        let e = tekst_deel(commands, bestanden, vak, t, letter, g, kleur(fractie.tekstkleur), Uitlijning::Links, dekking);
                        commands.entity(e).insert((TextLayout::no_wrap(), Passend::basis(g)));
                    }
                    let namen = namen.join(", ");
                    if !namen.is_empty() {
                        let e = tekst_deel(commands, bestanden, regel, &namen, naamletter, g, tk, Uitlijning::Links, dekking);
                        commands.entity(e).insert(Passend::basis(g));
                    }
                }
            }
        }
        _ => {}
    }
}

/// Een label: de korte naam in `letter` op de partijkleur, met deze binnenmarge (links en
/// rechts, boven en onder).
#[allow(clippy::too_many_arguments)]
fn label(
    commands: &mut Commands,
    bestanden: &mut Bestanden,
    ouder: Entity,
    f: &Fractielabel,
    letter: &str,
    grootte: f32,
    (x, y): (f32, f32),
    dekking: f32,
) -> Entity {
    let vak = commands
        .spawn((
            ChildOf(ouder),
            Node {
                padding: UiRect::axes(Val::Px(x), Val::Px(y)),
                flex_shrink: 0.0,
                ..default()
            },
            gekleurd(kleur(f.kleur), dekking),
        ))
        .id();
    let t = tekst_deel(commands, bestanden, vak, &naam_met_aantal(f), letter, grootte.round(), kleur(f.tekstkleur), Uitlijning::Links, dekking);
    commands.entity(t).insert((TextLayout::no_wrap(), Passend::basis(grootte.round())));
    vak
}

/// De korte naam, en bij een verdeelde Fractie het aantal Kamerleden aan deze kant erachter.
fn naam_met_aantal(f: &Fractielabel) -> String {
    match f.aantal {
        Some(a) => format!("{}\u{2002}{a}", f.kort),
        None => f.kort.clone(),
    }
}

/// Iets dat in zijn `ruimte` moet passen: past het niet, dan krimpen de letters van alle teksten
/// erin met [`Passend::basis`] (de Namenlijst, een korte naam in een tegel). Op een tekst: de
/// eigen lettergrootte.
#[derive(Component, Clone, Copy)]
pub struct Passend {
    pub basis: f32,
    pub ruimte: Vec2,
    pub schaal: f32,
}

impl Passend {
    fn basis(grootte: f32) -> Passend {
        Passend { basis: grootte, ruimte: Vec2::ZERO, schaal: 1.0 }
    }
}

/// De nieuwe schaal van iets dat zo `groot` is bij schaal `nu` en in `ruimte` moet passen: een
/// stapje kleiner zolang het niet past, nooit groter dan 1 en nooit kleiner dan de helft.
pub fn krimpschaal(nu: f32, groot: Vec2, ruimte: Vec2) -> f32 {
    if groot.x <= ruimte.x + 0.5 && groot.y <= ruimte.y + 0.5 {
        return nu;
    }
    (nu * 0.96).max(0.5)
}

/// Laat de teksten in elk [`Passend`] krimpen tot het in zijn ruimte past. Na een paar beelden
/// staat het stil; de teksten vervagen pas daarna in beeld.
pub fn pas_in(
    mut houders: Query<(Entity, &mut Passend, &ComputedNode), Without<Text>>,
    kinderen: Query<&Children>,
    mut teksten: Query<(&Passend, &mut TextFont), With<Text>>,
) {
    for (e, mut p, node) in &mut houders {
        let groot = node.size * node.inverse_scale_factor;
        let nieuw = krimpschaal(p.schaal, groot, p.ruimte);
        if nieuw == p.schaal {
            continue;
        }
        p.schaal = nieuw;
        for k in kinderen.iter_descendants(e) {
            if let Ok((t, mut font)) = teksten.get_mut(k) {
                font.font_size = bevy::text::FontSize::Px((t.basis * nieuw).round());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_labels_staan_onder_hun_kant_van_de_balk_tenzij_een_kant_dan_niet_past() {
        let (b, h, g) = (1632.0, 112.0, 30.0);
        // de mockup: tien labels voor, vijf tegen; past bij de wissel van 101 tegen 49
        let voor = [82.0, 100.0, 156.0, 100.0, 118.0, 100.0, 64.0, 64.0, 136.0, 82.0];
        let tegen = [82.0, 100.0, 82.0, 82.0, 82.0];
        let w = b * 101.0 / 150.0;
        assert_eq!(verdeel_onder_balk(b, h, 101.0 / 150.0, &voor, &tegen, g), w);
        // zeven tegen met twee lange namen passen niet in twee rijen van 513 breed: tegen krijgt
        // meer ruimte, zodat elke kant in twee rijen past
        let tegen = [82.0, 100.0, 82.0, 332.0, 82.0, 82.0, 212.0];
        let w = verdeel_onder_balk(b, h, 101.0 / 150.0, &voor, &tegen, g);
        assert!(rijen(&voor, w - 30.0) <= 2 && rijen(&tegen, b - w - 20.0) <= 2, "{w}");
    }

    #[test]
    fn wat_niet_past_krimpt_in_stapjes_tot_de_helft_en_wat_past_blijft_staan() {
        let ruimte = Vec2::new(1632.0, 664.0);
        assert_eq!(krimpschaal(1.0, Vec2::new(1632.0, 600.0), ruimte), 1.0);
        assert_eq!(krimpschaal(1.0, Vec2::new(1632.0, 800.0), ruimte), 0.96);
        assert_eq!(krimpschaal(0.51, Vec2::new(1632.0, 800.0), ruimte), 0.5);
        assert_eq!(krimpschaal(0.5, Vec2::new(1632.0, 800.0), ruimte), 0.5);
    }
}
