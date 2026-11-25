//! De basis van het beeld: de Zaal (zie `zaal.rs`) door de hoofdcamera en, voor proeven, de
//! regel met rol en verbinding. De Graphics tekent de Graphic-laag ([`crate::graphic`]).

use bevy::camera::{Exposure, Hdr, RenderTarget};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::post_process::effect_stack::Vignette;
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection};

use crate::Rolinfo;
use crate::graphic::Bron;
use crate::volging::{Standen, Verbinding};

pub struct BeeldPlugin;

impl Plugin for BeeldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb_u8(0x0d, 0x12, 0x20)))
            .add_systems(Startup, bouw)
            .add_systems(Update, toon_info);
    }
}

/// Alle instelwaarden voor licht en beeld van de Zaal op één plek: de camera hier, de lampen in
/// `zaal.rs`. Bloom, toonweergave en kleurcorrectie werken alleen op de Zaal; de Graphics komen
/// er daarna overheen en veranderen niet.
#[derive(Clone, Debug)]
pub struct Beeldinstellingen {
    /// Belichting van de camera. De bak is in Blender-eenheden.
    pub ev100: f32,
    pub tonemapping: Tonemapping,
    /// Sterkte van de gloed om felle plekken (lampen, daklicht).
    pub bloom: f32,
    /// Vanaf deze helderheid (na belichting) gloeit iets.
    pub bloom_drempel: f32,
    /// Kleurtemperatuur: negatief is koeler (de bak is warm, zoals gloeilampen).
    pub temperatuur: f32,
    /// Verzadiging van de middentonen.
    pub verzadiging: f32,
    /// Gamma van de schaduwen: boven 1 dieper, voor diepte onder banken en tussen de rijen.
    pub schaduw_gamma: f32,
    /// Hoe ver de randen van het beeld donker worden (0 tot 1), boven en onder sterker dan
    /// opzij: daar staan de Graphics, en de Stoelen zitten in de band ertussen.
    pub vignet: f32,
    /// Verlichtingssterkte van het daglicht door het glas, live en met schaduw.
    pub daglicht_lux: f32,
    /// Zacht licht van alle kanten, alleen voor wat niet gebakken is.
    pub omgeving: f32,
    /// Hoe ver de lampen uit de bak live reiken (glans en wat niet gebakken is), in meter.
    pub lamp_bereik: f32,
}

/// De instellingen van de uitzending, gekozen naast foto's van de plenaire zaal (#69): licht en
/// fris zoals de zaal op tv, met mintgroen tapijt, honingkleurig hout en een wit daklicht.
pub const BEELD: Beeldinstellingen = Beeldinstellingen {
    // een halve stop lichter dan EV100 0,6 uit #8, en daarna een fractie terug: de zaal is licht
    // en open, maar de Graphics staan op een rustige, iets donkerder achtergrond
    ev100: 0.3,
    // TonyMcMapface houdt felle plekken (daklicht, spots) wit zonder uit te bijten
    tonemapping: Tonemapping::TonyMcMapface,
    // een zachte gloed om de spots en het daklicht, niet meer
    bloom: 0.05,
    bloom_drempel: 1.2,
    // de bak is warm (lampen van 3000 K); koeler maakt wit weer wit en geeft de rustige,
    // koele studiokleur achter de Graphics (council-ronde 1)
    temperatuur: -0.035,
    verzadiging: 1.05,
    schaduw_gamma: 1.12,
    vignet: 0.45,
    // alleen voor een oudere Zaal met live daglicht; de nieuwe Zaal heeft het daglicht in de bak
    daglicht_lux: 6.0,
    omgeving: 0.35,
    // de spots hangen 5 m hoog; tot 12 m ziet niemand verschil, en elke pixel rekent zo met minder
    // lampen (0,5 ms per beeld minder dan 20 m)
    lamp_bereik: 12.0,
};

impl Beeldinstellingen {
    pub fn kleurcorrectie(&self) -> ColorGrading {
        ColorGrading {
            global: ColorGradingGlobal {
                temperature: self.temperatuur,
                ..default()
            },
            midtones: ColorGradingSection {
                saturation: self.verzadiging,
                ..default()
            },
            shadows: ColorGradingSection {
                saturation: self.verzadiging,
                gamma: self.schaduw_gamma,
                ..default()
            },
            ..default()
        }
    }

    /// Een lichte vignet: de Stoelen in de band in het midden blijven zoals ze zijn, het plafond
    /// en de voorgrond achter de Graphics worden rustig donker.
    pub fn vignet(&self) -> Vignette {
        Vignette {
            intensity: self.vignet,
            radius: 1.0,
            smoothness: 1.5,
            // boven en onder sterker dan opzij
            roundness: 0.6,
            edge_compensation: 1.0,
            ..default()
        }
    }

    pub fn gloed(&self) -> Bloom {
        Bloom {
            intensity: self.bloom,
            prefilter: BloomPrefilter {
                threshold: self.bloom_drempel,
                threshold_softness: 0.5,
            },
            composite_mode: BloomCompositeMode::Additive,
            ..Bloom::NATURAL
        }
    }
}

/// De camera die de Zaal en de graphics tekent.
#[derive(Component)]
pub struct HoofdCamera;

/// Een camera op de Zaal: toonweergave, belichting, gloed, kleurcorrectie, vignet en MSAA (zie
/// [`BEELD`]).
pub fn zaalcamera() -> impl Bundle {
    let b = &BEELD;
    (
        Camera3d::default(),
        Hdr,
        b.tonemapping,
        Exposure { ev100: b.ev100 },
        b.gloed(),
        b.kleurcorrectie(),
        b.vignet(),
        Msaa::Sample4,
    )
}

/// Doel van de camera als er geen venster is.
#[derive(Resource, Clone)]
pub struct Doel(pub RenderTarget);

#[derive(Component)]
struct Info;

pub fn bouw(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    doel: Option<Res<Doel>>,
    rolinfo: Res<Rolinfo>,
) {
    let gewoon = fonts.add(Font::from_bytes(
        include_bytes!("../assets/fonts/DejaVuSans.ttf").to_vec(),
    ));
    // Het Voorbeeld toont de klaargezette Graphic, de andere rollen de Ether.
    let bron = if rolinfo.rol == gedeeld::Rol::Voorbeeld { Bron::Klaar } else { Bron::Ether };
    // tot de eerste stand van de server: Totaal vanaf voorzitter
    let a = gedeeld::camera::Standpunt::vaste()[0].stand;
    let mut camera = commands.spawn((
        HoofdCamera,
        zaalcamera(),
        bron,
        crate::camera::transform(&a),
        crate::camera::projectie(&a),
    ));
    if let Some(d) = doel {
        camera.insert(d.0.clone());
    }
    let camera = camera.id();

    let wortel = commands
        .spawn((
            UiTargetCamera(camera),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .id();
    if rolinfo.info {
        commands.spawn((
            ChildOf(wortel),
            Info,
            Text::new(""),
            TextFont::from_font_size(24.0).with_font(gewoon),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(48.0),
                top: Val::Px(36.0),
                ..default()
            },
        ));
    }
}

fn toon_info(
    standen: Res<Standen>,
    verbinding: Res<Verbinding>,
    rolinfo: Res<Rolinfo>,
    info: Option<Single<&mut Text, With<Info>>>,
) {
    let Some(mut info) = info else { return };
    if !standen.is_changed() && !verbinding.is_changed() {
        return;
    }
    let volgnummer = standen
        .huidig
        .as_ref()
        .map_or("-".to_string(), |s| s.volgnummer.to_string());
    let staat = match &*verbinding {
        Verbinding::Zoeken => "zoekt de server".to_string(),
        Verbinding::Verbonden => "verbonden".to_string(),
        Verbinding::Geweigerd(r) => format!("geweigerd: {r}"),
        Verbinding::Verbroken(_) => "geen verbinding".to_string(),
    };
    info.0 = format!("{:?} · volgnummer {volgnummer} · {staat}", rolinfo.rol);
}
