mod serpg_api;
mod videojoc;

use serpg_api::*;
use videojoc::*;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "CliPG",
        options,
        Box::new(|_cc| Ok(Box::new(CliPG::default()))),
    )
}

struct CliPG {
    api: SerPGAPI,
    vjocs: Vec<Videojoc>,
}

impl CliPG {

    fn get_credentials() -> (String, String, String) {
        // TODO
        (
            String::from("http://pastanagabullida"),
            String::from("pastanaga"),
            String::from("buillida")
        )
    }

    fn default() -> Self {
        // Obtenim les credencials per el client
        let credencials: (String, String, String) = Self::get_credentials();
        CliPG {
            api: SerPGAPI::new(credencials.0, credencials.1, credencials.2),
            vjocs: Vec::new(),
        }
    }
}

impl eframe::App for CliPG {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CliPG");
        });
    }
}
