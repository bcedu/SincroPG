use crate::cli_pg::*;
use eframe::egui;
use std::path::PathBuf;

pub fn start_pg_gui(clipg_config_path: Option<PathBuf>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("CliPG: Sincronitzacio de partides guardades", options, Box::new(|_cc| Ok(Box::new(PgGUI::new(clipg_config_path)))))
}

pub struct PgGUI {
    clipg_config_path: Option<PathBuf>,
}
impl PgGUI {
    pub fn new(clipg_config_path: Option<PathBuf>) -> Self {
        Self { clipg_config_path }
    }
}
impl eframe::App for PgGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CliPG");
        });
    }
}
