//! De gamepad aan de regie-pc (#44): een Xbox-controller met snoer, uitgelezen door de server
//! (niet door de browser), zodat de camera van de Ether niet afhangt van een open dashboard.
//!
//! - Linker stick: vooruit, achteruit en opzij. Rechter stick: rondkijken.
//! - Rechter trigger omhoog, linker trigger omlaag. Rechter bumper inzoomen, linker uitzoomen.
//! - A wisselt langzaam en snel. Start zet de Vrije camera aan of uit, net als V op het dashboard.
//! - B start en stopt de opname van een Camerarit, net als R op het dashboard (alleen tijdens
//!   Oefenen, #45).
//!
//! Zonder controller start de server gewoon; een controller die later in de pc gaat, telt meteen
//! mee. Met de Vrije camera uit doet de stick niets (dat regelt de camera, zie
//! [`crate::camera::VrijeCamera`]).

use std::sync::Arc;
use std::time::Duration;

use gedeeld::camera::Camerakeuze;
use gedeeld::{Afzender, Opdracht};

use crate::Gedeeld;
use crate::camera::Stuur;

/// Hoe vaak de server de gamepad uitleest: vaker dan de camera rekent.
const ELKE: Duration = Duration::from_millis(5);

/// Wat de server van een gamepad leest, los van gilrs zodat een test het kan nabootsen.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Gamepadstand {
    pub links: [f32; 2],
    pub rechts: [f32; 2],
    pub linker_trigger: f32,
    pub rechter_trigger: f32,
    pub linker_bumper: bool,
    pub rechter_bumper: bool,
}

/// Een knop die iets doet als je hem indrukt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Druk {
    A,
    B,
    Start,
}

/// Wat een druk op een knop in de Stand verandert.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gevolg {
    WisselVrijeCamera,
    WisselOpnemen,
}

/// Vertaalt de gamepad naar [`Stuur`] en knoppen naar Opdrachten.
#[derive(Clone, Copy, Debug, Default)]
pub struct Gamepad {
    snel: bool,
}

impl Gamepad {
    pub fn druk(&mut self, d: Druk) -> Option<Gevolg> {
        match d {
            Druk::A => {
                self.snel = !self.snel;
                None
            }
            Druk::B => Some(Gevolg::WisselOpnemen),
            Druk::Start => Some(Gevolg::WisselVrijeCamera),
        }
    }

    pub fn stuur(&self, g: &Gamepadstand) -> Stuur {
        Stuur {
            links: g.links,
            rechts: g.rechts,
            omhoog: g.rechter_trigger,
            omlaag: g.linker_trigger,
            inzoomen: g.rechter_bumper,
            uitzoomen: g.linker_bumper,
            snel: self.snel,
        }
    }
}

/// De Opdracht bij een druk op de knop, in de Stand van nu.
pub fn opdracht_voor(gevolg: Gevolg, camera: &Camerakeuze) -> Opdracht {
    match gevolg {
        Gevolg::WisselVrijeCamera => Opdracht::ZetVrijeCamera { aan: !camera.vrij },
        // stoppen: de server vult in wat hij opnam
        Gevolg::WisselOpnemen => Opdracht::ZetOpnemen { aan: camera.opname.is_none(), voor: None, opname: None },
    }
}

/// Leest de gamepad in een eigen draad, tot de server stopt.
pub(crate) fn volg(g: Arc<Gedeeld>) {
    let runtime = tokio::runtime::Handle::current();
    let gelukt = std::thread::Builder::new()
        .name("gamepad".into())
        .spawn(move || lees(g, runtime));
    if let Err(e) = gelukt {
        tracing::warn!("gamepad niet gestart: {e}");
    }
}

fn lees(g: Arc<Gedeeld>, runtime: tokio::runtime::Handle) {
    use gilrs::{Axis, Button, EventType, Gilrs};
    let mut gilrs = match Gilrs::new() {
        Ok(gilrs) => gilrs,
        Err(gilrs::Error::NotImplemented(gilrs)) => {
            tracing::warn!("gamepad: dit systeem wordt niet ondersteund");
            gilrs
        }
        Err(e) => {
            tracing::warn!("geen gamepad: {e}. De Vrije camera gaat alleen aan en uit met V.");
            return;
        }
    };
    if gilrs.gamepads().next().is_none() {
        tracing::info!("nog geen gamepad aangesloten");
    }
    let mut gamepad = Gamepad::default();
    loop {
        if *g.gestopt.borrow() {
            return;
        }
        while let Some(ev) = gilrs.next_event() {
            let naam = gilrs.gamepad(ev.id).name().to_owned();
            let druk = match ev.event {
                EventType::Connected => {
                    tracing::info!("gamepad aangesloten: {naam}");
                    None
                }
                EventType::Disconnected => {
                    tracing::warn!("gamepad los: {naam}");
                    None
                }
                EventType::ButtonPressed(Button::South, _) => Some(Druk::A),
                EventType::ButtonPressed(Button::East, _) => Some(Druk::B),
                EventType::ButtonPressed(Button::Start, _) => Some(Druk::Start),
                _ => None,
            };
            if let Some(gevolg) = druk.and_then(|d| gamepad.druk(d)) {
                let g = g.clone();
                runtime.spawn(async move {
                    let camera = g.uitzending.borrow().stand.camera.clone();
                    let opdracht = opdracht_voor(gevolg, &camera);
                    if let Err(e) = g.voer_uit(&Afzender::Gamepad { naam }, &opdracht).await {
                        tracing::warn!("{gevolg:?} op de gamepad: {e}");
                    }
                });
            }
        }
        // De eerste aangesloten gamepad stuurt; zonder gamepad staat alles los.
        let stand = gilrs
            .gamepads()
            .next()
            .map(|(_, gp)| {
                let trigger = |b| gp.button_data(b).map_or(0.0, |d| d.value());
                Gamepadstand {
                    links: [gp.value(Axis::LeftStickX), gp.value(Axis::LeftStickY)],
                    rechts: [gp.value(Axis::RightStickX), gp.value(Axis::RightStickY)],
                    linker_trigger: trigger(Button::LeftTrigger2),
                    rechter_trigger: trigger(Button::RightTrigger2),
                    linker_bumper: gp.is_pressed(Button::LeftTrigger),
                    rechter_bumper: gp.is_pressed(Button::RightTrigger),
                }
            })
            .unwrap_or_default();
        let stuur = gamepad.stuur(&stand);
        g.stuur.send_if_modified(|s| {
            let anders = *s != stuur;
            *s = stuur;
            anders
        });
        std::thread::sleep(ELKE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wisselt_langzaam_en_snel_en_start_wisselt_de_vrije_camera() {
        let mut gp = Gamepad::default();
        let stand = Gamepadstand {
            links: [0.0, 1.0],
            rechter_trigger: 0.5,
            rechter_bumper: true,
            ..Default::default()
        };
        let s = gp.stuur(&stand);
        assert!(!s.snel);
        assert_eq!((s.links, s.omhoog, s.omlaag), ([0.0, 1.0], 0.5, 0.0));
        assert!(s.inzoomen && !s.uitzoomen);
        assert_eq!(gp.druk(Druk::A), None);
        assert!(gp.stuur(&stand).snel);
        assert_eq!(gp.druk(Druk::A), None);
        assert!(!gp.stuur(&stand).snel);
        assert_eq!(gp.druk(Druk::Start), Some(Gevolg::WisselVrijeCamera));
        assert_eq!(gp.stuur(&Gamepadstand::default()), Stuur::LOS);
    }

    #[test]
    fn b_start_en_stopt_de_opname() {
        let mut gp = Gamepad::default();
        let gevolg = gp.druk(Druk::B).unwrap();
        let mut camera = Camerakeuze { oefenen: true, ..Default::default() };
        assert_eq!(opdracht_voor(gevolg, &camera), Opdracht::ZetOpnemen { aan: true, voor: None, opname: None });
        camera.opname = Some(gedeeld::camera::Opname { voor: None, sinds: 4 });
        assert_eq!(opdracht_voor(gevolg, &camera), Opdracht::ZetOpnemen { aan: false, voor: None, opname: None });
        assert!(!gp.stuur(&Gamepadstand::default()).snel, "B wisselt niet langzaam en snel");
    }
}
