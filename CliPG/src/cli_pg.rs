use crate::pg_api::*;
use crate::videojoc::*;
use eframe::egui;

pub struct CliPG {
    api: PgAPI,
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

    pub fn default() -> Self {
        // Obtenim les credencials per el client
        let credencials: (String, String, String) = Self::get_credentials();
        CliPG {
            api: PgAPI::new(credencials.0, credencials.1, credencials.2),
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
