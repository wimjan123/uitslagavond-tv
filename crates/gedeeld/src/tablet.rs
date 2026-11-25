//! De tablet van de Presentator (besluit #10, T2). Hij zet een Preset klaar die de Regie op de
//! tablet zette, en tikt Partijen aan in de Coalitiebouwer. Hij zet zelf niets in de Ether: Toon
//! en alles daarbuiten weigert de server, niet alleen het scherm van de tablet.

use crate::{Afzender, Opdracht, Stand, Weigering};

impl Stand {
    /// Of deze afzender deze Opdracht mag geven. De server vraagt het vóór [`Stand::voer_uit`].
    /// Alleen de tablet heeft grenzen: [`Opdracht::ZetPresetKlaar`] voor een Preset op de tablet, en
    /// [`Opdracht::ZetCoalitie`] (een Coalitie in beeld werkt direct bij, besluit #6).
    pub fn controleer_afzender(&self, afzender: &Afzender, opdracht: &Opdracht) -> Result<(), Weigering> {
        if !matches!(afzender, Afzender::Tablet { .. }) {
            return Ok(());
        }
        match opdracht {
            Opdracht::ZetPresetKlaar { nummer } if !self.preset(*nummer)?.op_tablet => {
                Err(Weigering::NietOpTablet(*nummer))
            }
            Opdracht::ZetPresetKlaar { .. } | Opdracht::ZetCoalitie { .. } => Ok(()),
            _ => Err(Weigering::NietVanafTablet),
        }
    }
}
