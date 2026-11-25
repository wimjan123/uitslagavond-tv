//! Het dashboard krijgt de vaste Sjablonen van deze versie en hun vingerafdruk uit deze crate,
//! zoals de typen: `cargo test -p gedeeld` schrijft ze naar
//! `dashboard/src/typen/standaardSjablonen.ts`. Zo ziet de vormgevingstab of een aangepast vast
//! Sjabloon op een oudere standaard gebaseerd is.

use gedeeld::sjabloon::{DONKER, EIGEN, GROEP_EIGEN, MAX_BLOKKEN, MAX_SJABLOONNAAM, vaste_sjablonen, vingerafdruk};

fn inhoud() -> String {
    let vast = vaste_sjablonen();
    let regels: Vec<String> = vast.iter().map(|s| format!("  {},", serde_json::to_string(s).unwrap())).collect();
    let afdrukken: Vec<String> = vast.iter().map(|s| format!("  {:?}: {:?},", s.id, vingerafdruk(s))).collect();
    format!(
        "// Gemaakt door `cargo test -p gedeeld` (crates/gedeeld/tests/standaard_sjablonen.rs). Niet met de hand wijzigen.\n\
         import type {{ Sjabloon }} from \"./Sjabloon\";\n\n\
         /** De vaste Sjablonen van deze versie van de software: de standaard. (De JSON heeft `null` waar het type een optioneel veld heeft.) */\n\
         export const STANDAARD_SJABLONEN = [\n{}\n] as unknown as readonly Sjabloon[];\n\n\
         /** De vingerafdruk van elke standaard (`sjabloon::vingerafdruk`), op `id`. */\n\
         export const VINGERAFDRUKKEN: Readonly<Record<string, string>> = {{\n{}\n}};\n\n\
         /** Het begin van het `id` van een eigen Sjabloon. */\n\
         export const EIGEN = {EIGEN:?};\n\
         /** De groep van een eigen Sjabloon. */\n\
         export const GROEP_EIGEN = {GROEP_EIGEN:?};\n\
         /** Langste naam van een Sjabloon, in tekens. */\n\
         export const MAX_SJABLOONNAAM = {MAX_SJABLOONNAAM};\n\
         /** Het grootste aantal Blokken in een Sjabloon. */\n\
         export const MAX_BLOKKEN = {MAX_BLOKKEN};\n\
         /** De dekking in procent van het zwarte vlak over de Zaal bij de achtergrond Zaal donker. */\n\
         export const DONKER = {DONKER};\n",
        regels.join("\n"),
        afdrukken.join("\n"),
    )
}

#[test]
fn schrijf_de_standaard_sjablonen_voor_het_dashboard() {
    let Some(map) = std::env::var_os("TS_RS_EXPORT_DIR") else { return };
    let pad = std::path::Path::new(&map).join("standaardSjablonen.ts");
    let nieuw = inhoud();
    if std::fs::read_to_string(&pad).ok().as_deref() != Some(nieuw.as_str()) {
        std::fs::write(&pad, nieuw).unwrap();
    }
}
