//! Bakt de versie in: de versie uit Cargo.toml plus de commit, zoals `0.1.0+4a46080`. Zo weigert
//! de server een Renderer die uit een andere commit is gebouwd, ook als Cargo.toml gelijk bleef.
//! Zonder git (een bronpakket) blijft het alleen de versie uit Cargo.toml.

use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let uit = Command::new("git").args(args).output().ok()?;
    uit.status.success().then(|| String::from_utf8_lossy(&uit.stdout).trim().to_string())
}

fn main() {
    let pakket = std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION");
    let versie = match git(&["rev-parse", "--short=7", "HEAD"]) {
        Some(commit) if !commit.is_empty() => format!("{pakket}+{commit}"),
        _ => pakket,
    };
    println!("cargo:rustc-env=UITSLAGAVOND_VERSIE={versie}");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    // Opnieuw bij een nieuwe commit: HEAD, de branch waar HEAD naar wijst, en packed-refs.
    if let (Some(map), Some(gemeen)) =
        (git(&["rev-parse", "--git-dir"]), git(&["rev-parse", "--git-common-dir"]))
    {
        let mut paden = vec![Path::new(&map).join("HEAD"), Path::new(&gemeen).join("packed-refs")];
        if let Some(r) = git(&["symbolic-ref", "-q", "HEAD"]) {
            paden.push(Path::new(&gemeen).join(r));
        }
        for p in paden.into_iter().filter(|p| p.exists()) {
            println!("cargo:rerun-if-changed={}", p.display());
        }
    }
}
