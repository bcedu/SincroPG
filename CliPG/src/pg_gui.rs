use crate::cli_pg::*;
use eframe::egui;
use std::path::PathBuf;

pub fn start_pg_gui(clipg_config_path: Option<PathBuf>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("CliPG: Sincronitzacio de partides guardades", options, Box::new(|_cc| Ok(Box::new(PgGUI::default()))))
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum AppMode {
    Dashboard,
    EditarJoc,
    Configuracio,
    Sincronitzacio,
}
pub struct PgGUI {
    clipg_config_path: Option<PathBuf>,
    current_mode: AppMode,
}
impl Default for PgGUI {
    fn default() -> Self {
        Self {
            clipg_config_path: None,
            current_mode: AppMode::Dashboard,
        }
    }
}
impl PgGUI {
    fn setup_top_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Fitxer", |ui| {
                if ui.button("Afegir joc").clicked() {
                    // TODO -> Posar la vista AppMode::EditarJoc
                }
                if ui.button("Surt").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Edita", |ui| {
                if ui.button("Preferències").clicked() {
                    // TODO -> Posar la vista AppMode::Configuracio
                }
            });
        });
    }
    fn setup_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("VIEW DASHBOARD");
        });
    }
    fn setup_configuracio(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("VIEW CONFIGURACIO");
        });
    }
    fn setup_editar_joc(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("VIEW EDITAR JOC");
        });
    }
    fn setup_sincronitzacio(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("VIEW SINCRONITZACIÓ");
        });
    }
}
impl eframe::App for PgGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Menu Bar using Top Panel ---
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.setup_top_panel(ctx, ui);
        });
        // --- Central Panel for the Main Content ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Mode: {:?}", self.current_mode));
            ui.separator();
            match self.current_mode {
                AppMode::Dashboard => {
                    self.setup_dashboard(ui);
                }
                AppMode::Configuracio => {
                    self.setup_configuracio(ui);
                }
                AppMode::EditarJoc => {
                    self.setup_editar_joc(ui);
                }
                AppMode::Sincronitzacio => {
                    self.setup_sincronitzacio(ui);
                }
            }
        });
    }
}
