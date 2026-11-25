//! Presets: opgeslagen Graphics die de Regie met één klik of cijfertoets klaarzet.
//!
//! Een Preset is een kopie van de Graphic op het moment van bewaren, met haar gegevens. Hij werkt
//! zichzelf nooit bij: verandert daarna een set, de Kamer of een Stemming, dan houdt de Preset de
//! oude gegevens. Elke Preset heeft een vast nummer; de verzameling heeft geen volgorde of
//! draaiboek.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Graphic, GraphicInBeeld, KlaargezetteGraphic};

/// Een opgeslagen Graphic.
#[derive(Serialize, Deserialize, TS, Clone, Debug, PartialEq, Eq)]
#[ts(export)]
pub struct Preset {
    /// Vast nummer, vanaf 1. De cijfertoetsen 1 tot 9 zetten de Preset met dat nummer klaar.
    pub nummer: u16,
    /// De Graphic met de gegevens van het moment van bewaren.
    pub graphic: Graphic,
    /// Een ander Standpunt dan dat van het Sjabloon, als letter. Zie [`Preset::standpunt`].
    #[serde(default)]
    #[ts(optional)]
    pub standpunt: Option<String>,
    /// Of de Presentator deze Preset op de tablet ziet.
    #[serde(default)]
    pub op_tablet: bool,
    /// Wanneer bewaard, in milliseconden sinds 1970 (UTC).
    #[ts(type = "number")]
    pub bewaard_ms: u64,
    pub bewaard_door: String,
    /// Het id van de laatste klaargezette Graphic uit deze Preset. Een nieuwe Graphic krijgt
    /// hetzelfde getal als id en als versie; staat die versie nog klaar, dan krijgt de Preset de
    /// stempel Klaar. Na een wijziging is de Graphic de Preset niet meer.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub klaargezet_als: Option<u64>,
    /// Het id van de laatst getoonde Graphic uit deze Preset: staat die ongewijzigd in de Ether,
    /// dan krijgt de Preset de stempel Live. Apart van `klaargezet_als`, want de Regie kan dezelfde Preset
    /// opnieuw klaarzetten terwijl hij in beeld staat.
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub getoond_als: Option<u64>,
    /// Wanneer een Graphic uit deze Preset het laatst getoond werd ("was live om").
    #[serde(default)]
    #[ts(optional, type = "number")]
    pub laatst_live_ms: Option<u64>,
}

impl Preset {
    /// De letter van het Standpunt: de keuze voor deze Preset, anders die van het Sjabloon.
    pub fn standpunt(&self) -> std::borrow::Cow<'_, str> {
        self.standpunt.as_deref().map_or_else(
            || self.graphic.standaard_standpunt(),
            std::borrow::Cow::Borrowed,
        )
    }

    /// Of deze klaargezette Graphic uit deze Preset komt.
    pub fn is_bron_van_klaar(&self, klaar: &KlaargezetteGraphic) -> bool {
        self.klaargezet_als
            .is_some_and(|k| klaar.id == k && klaar.versie == k)
    }

    /// Of deze Graphic in de Ether uit deze Preset komt.
    pub fn is_bron_van_ether(&self, ether: &GraphicInBeeld) -> bool {
        self.getoond_als
            .is_some_and(|k| ether.id == k && ether.versie == k)
    }

    /// Noteert een Toon: komt de Graphic in de Ether uit deze Preset, dan is hij nu live.
    pub(crate) fn getoond(&mut self, ether: &GraphicInBeeld) {
        if self
            .klaargezet_als
            .is_some_and(|k| ether.id == k && ether.versie == k)
        {
            self.getoond_als = Some(ether.id);
            self.laatst_live_ms = Some(ether.sinds_ms);
        }
    }
}

/// Het eerste nummer dat geen Preset heeft.
pub(crate) fn eerste_vrije_nummer(presets: &[Preset]) -> u16 {
    (1..)
        .find(|n| presets.iter().all(|p| p.nummer != *n))
        .expect("er is altijd een vrij nummer")
}
