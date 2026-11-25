//! Het dashboard krijgt de vaste Cameraritten uit deze crate, zoals de typen: `cargo test -p
//! gedeeld` schrijft ze naar `dashboard/src/typen/vasteRitten.ts`. De opname gaat mee met 10
//! standen per seconde (het dashboard vult de rest recht aan), zodat het bestand klein blijft.

use gedeeld::camera::TIKKEN_PER_SECONDE;
use gedeeld::vaste_ritten;

/// Zoveel tikken tussen twee standen in het bestand.
const STAP: usize = (TIKKEN_PER_SECONDE / 10) as usize;

fn inhoud() -> String {
    let standaard = serde_json::to_string_pretty(&vaste_ritten::vaste()).unwrap();
    let mut opnames = String::new();
    for r in vaste_ritten::vaste() {
        let punten = r.punten();
        assert_eq!(
            (punten.len() - 1) % STAP,
            0,
            "{}: een hele tiende seconde",
            r.naam
        );
        let rond = |v: f32| (v * 1000.0).round() / 1000.0;
        let dun: Vec<[f32; 6]> = punten.iter().step_by(STAP).map(|p| p.map(rond)).collect();
        opnames.push_str(&format!(
            "  {:?}: {},\n",
            r.letter,
            serde_json::to_string(&dun).unwrap()
        ));
    }
    format!(
        "// Gemaakt door `cargo test -p gedeeld` (crates/gedeeld/tests/vaste_ritten.rs). Niet met de hand wijzigen.\n\
         import type {{ Camerarit }} from \"./Camerarit\";\n\n\
         /** De vaste Cameraritten zoals ze meekomen, voor \"Terug naar standaard\". */\n\
         export const STANDAARD_RITTEN: Camerarit[] = {standaard};\n\n\
         /** Tikken tussen twee standen hieronder. */\n\
         export const RITSTAP = {STAP};\n\n\
         /** De opname van elke vaste Camerarit, 10 standen per seconde. */\n\
         export const VASTE_OPNAMES: Record<string, [number, number, number, number, number, number][]> = {{\n{opnames}}};\n"
    )
}

#[test]
fn schrijf_de_vaste_ritten_voor_het_dashboard() {
    let Some(map) = std::env::var_os("TS_RS_EXPORT_DIR") else {
        return;
    };
    let pad = std::path::Path::new(&map).join("vasteRitten.ts");
    let nieuw = inhoud();
    if std::fs::read_to_string(&pad).ok().as_deref() != Some(nieuw.as_str()) {
        std::fs::write(&pad, nieuw).unwrap();
    }
}
