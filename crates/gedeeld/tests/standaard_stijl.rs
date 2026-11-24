//! Het dashboard krijgt de standaard-Stijl en de grenzen van de Stijl uit deze crate, zoals de
//! typen: `cargo test -p gedeeld` schrijft ze naar `dashboard/src/typen/standaardStijl.ts`.

use gedeeld::sjabloon::{INGEBOUWD, MAX_LOGOTEKST, MAX_SNELHEID, MIN_SNELHEID, Stijl};

fn inhoud() -> String {
    let stijl = serde_json::to_string_pretty(&Stijl::default()).unwrap();
    let letters = serde_json::to_string(&INGEBOUWD).unwrap();
    format!(
        "// Gemaakt door `cargo test -p gedeeld` (crates/gedeeld/tests/standaard_stijl.rs). Niet met de hand wijzigen.\n\
         import type {{ Stijl }} from \"./Stijl\";\n\n\
         /** De standaard-Stijl (docs/vormgeving.md). */\n\
         export const STANDAARD_STIJL: Stijl = {stijl};\n\n\
         /** De lettertypes die in elke Renderer zitten. */\n\
         export const INGEBOUWDE_LETTERS: readonly string[] = {letters};\n\n\
         /** De langzaamste en snelste snelheid van de Stijl, in procent. */\n\
         export const MIN_SNELHEID = {MIN_SNELHEID};\n\
         export const MAX_SNELHEID = {MAX_SNELHEID};\n\n\
         /** Langste tekst van het programmalogo zonder beeld, in tekens. */\n\
         export const MAX_LOGOTEKST = {MAX_LOGOTEKST};\n"
    )
}

#[test]
fn schrijf_de_standaard_stijl_voor_het_dashboard() {
    let Some(map) = std::env::var_os("TS_RS_EXPORT_DIR") else { return };
    let pad = std::path::Path::new(&map).join("standaardStijl.ts");
    let nieuw = inhoud();
    if std::fs::read_to_string(&pad).ok().as_deref() != Some(nieuw.as_str()) {
        std::fs::write(&pad, nieuw).unwrap();
    }
}
