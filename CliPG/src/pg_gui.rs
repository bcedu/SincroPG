use crate::cli_pg::CliPG;
use eframe::egui;
use egui::Vec2;
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
    fn setup_dashboard(&mut self, centered_ui: &mut egui::Ui) {
        self.setup_dashboard_grup_accions_videojocs_habilitats(centered_ui);
        self.setup_dashboard_videojocs_habilitats(centered_ui);
    }
    fn setup_dashboard_grup_accions_videojocs_habilitats(&mut self, centered_ui: &mut egui::Ui) {
        egui::Frame::group(centered_ui.style()).show(centered_ui, |group_ui| {
            group_ui.horizontal(|ui| {
                // Botó esquerra
                if ui.button("Esquerra").clicked() {
                    // acció
                }

                // Ui flexible central
                let available = ui.available_width();
                ui.allocate_ui(Vec2::new(available, 0.0), |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label("🎮 Títol centrat");
                    });
                });

                // Botó dret
                if ui.button("Dreta").clicked() {
                    // acció
                }
            });
        });
    }
    fn setup_dashboard_videojocs_habilitats(&mut self, centered_ui: &mut egui::Ui) {
        egui::Frame::group(centered_ui.style()).show(centered_ui, |group_ui| {
            egui::ScrollArea::vertical().show(group_ui, |scroll_ui| {
                scroll_ui.heading("🎮 Videojocs habilitats");
                scroll_ui.add_space(10.0);
                let mut clipg = CliPG::default(self.clipg_config_path.clone());
                clipg.load_local_jocs();
                for joc in clipg.vjocs.iter() {
                    scroll_ui.horizontal(|row_ui| {
                        row_ui.label(joc.nom.clone().to_str().unwrap());
                        if row_ui.button("⟳ Sincronitzar").clicked() {}
                        if row_ui.button("🗑 Eliminar").clicked() {}
                    });
                    scroll_ui.separator();
                }
            });
        });
    }
    fn setup_configuracio(&mut self, centered_ui: &mut egui::Ui) {}
    fn setup_editar_joc(&mut self, centered_ui: &mut egui::Ui) {}
    fn setup_sincronitzacio(&mut self, centered_ui: &mut egui::Ui) {}
}
impl eframe::App for PgGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.setup_top_panel(ctx, ui);
        });
        egui::CentralPanel::default().show(ctx, |main_ui| {
            // Bloc centrat horitzontalment però mantenint layout vertical
            main_ui.vertical_centered(|main_vertical_ui| {
                main_vertical_ui.with_layout(egui::Layout::top_down(egui::Align::Center), |centered_ui| {
                    centered_ui.set_width(700.0);
                    match self.current_mode {
                        AppMode::Dashboard => {
                            self.setup_dashboard(centered_ui);
                        }
                        AppMode::Configuracio => {
                            self.setup_configuracio(centered_ui);
                        }
                        AppMode::EditarJoc => {
                            self.setup_editar_joc(centered_ui);
                        }
                        AppMode::Sincronitzacio => {
                            self.setup_sincronitzacio(centered_ui);
                        }
                    };
                });
            });
        });
    }
}
