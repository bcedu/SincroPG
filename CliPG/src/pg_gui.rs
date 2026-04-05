use crate::cli_pg::CliPG;
use crate::videojoc::Videojoc;
use eframe::egui;
use rfd::FileDialog;
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
    estat_servidor: String,
    activitat: String,
    joc_afegit: String,
    joc_afegit_nom: String,
}
impl Default for PgGUI {
    fn default() -> Self {
        Self {
            clipg_config_path: None,
            current_mode: AppMode::Dashboard,
            estat_servidor: String::new(),
            activitat: String::new(),
            joc_afegit: String::new(),
            joc_afegit_nom: String::new(),
        }
    }
}
impl PgGUI {
    fn get_estat_servidor(&mut self) -> String {
        if self.estat_servidor.is_empty() {
            let clipg = CliPG::default(self.clipg_config_path.clone());
            let conectat = if clipg.api.probar_connexio() { "✔ Conectat" } else { "❌ Desconectat" };
            self.estat_servidor = format!("{} ({})", conectat, clipg.config.server.url,);
        }
        self.estat_servidor.clone()
    }
    fn get_activitat(&mut self) -> String {
        self.activitat.clone()
    }
    fn sincronitzar_tots(&mut self) {
        let mut clipg = CliPG::default(self.clipg_config_path.clone());
        let res = clipg.sync_all(false);
        self.activitat = res;
    }
    fn sincronitzar_joc(&mut self, joc: &mut Videojoc) {
        let mut clipg = CliPG::default(self.clipg_config_path.clone());
        let res = clipg.sync_joc(joc, false);
        self.activitat = res;
    }
    fn eliminar_joc(&mut self, joc: &mut Videojoc) {
        let mut clipg = CliPG::default(self.clipg_config_path.clone());
        let nom_joc = joc.nom.to_str().unwrap().to_string();
        clipg.eliminar_joc(nom_joc.clone());
        self.activitat = format!("'{nom_joc}' eliminat correctament");
    }
    fn afegir_joc(&mut self, path_joc: String, nom_joc: String) {
        let mut clipg = CliPG::default(self.clipg_config_path.clone());
        clipg.afegir_joc(path_joc, Some(nom_joc.clone()));
        self.activitat = format!("'{nom_joc}' afegit correctament");
    }
    fn setup_top_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Fitxer", |ui| {
                if ui.button("Afegir joc").clicked() {
                    self.current_mode = AppMode::EditarJoc;
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
        self.setup_dashboard_videojocs_habilitats(centered_ui);
        self.setup_dashboard_servidor_status(centered_ui);
        self.setup_dashboard_activitat(centered_ui);
    }
    fn setup_dashboard_videojocs_habilitats(&mut self, centered_ui: &mut egui::Ui) {
        centered_ui.add_space(10.0);
        egui::Frame::group(centered_ui.style()).show(centered_ui, |group_ui| {
            egui::ScrollArea::vertical().show(group_ui, |scroll_ui| {
                scroll_ui.heading("🎮 Videojocs habilitats");
                scroll_ui.add_space(10.0);
                scroll_ui.horizontal(|row_ui| {
                    if row_ui.button("+ Afegir joc").clicked() {
                        self.current_mode = AppMode::EditarJoc;
                    }
                    row_ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |right_ui| {
                        if right_ui.button("🔄 Sincronitzar tots").clicked() {
                            self.sincronitzar_tots();
                        }
                    });
                });
                scroll_ui.add_space(10.0);
                let mut clipg = CliPG::default(self.clipg_config_path.clone());
                clipg.load_local_jocs();
                for joc in clipg.vjocs.iter_mut() {
                    scroll_ui.horizontal(|row_ui| {
                        row_ui.label(joc.nom.clone().to_str().unwrap());
                        row_ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |right_ui| {
                            if right_ui.button("🗑 Eliminar").clicked() {
                                self.eliminar_joc(joc);
                            }
                            if right_ui.button("🔄 Sincronitzar").clicked() {
                                self.sincronitzar_joc(joc);
                            }
                        });
                    });
                    scroll_ui.separator();
                }
            });
        });
    }
    fn setup_dashboard_servidor_status(&mut self, centered_ui: &mut egui::Ui) {
        centered_ui.add_space(10.0);
        egui::Frame::group(centered_ui.style()).show(centered_ui, |group_ui| {
            group_ui.label(format!("Estat servidor: {}", self.get_estat_servidor()));
        });
    }
    fn setup_dashboard_activitat(&mut self, centered_ui: &mut egui::Ui) {
        centered_ui.add_space(10.0);
        egui::Frame::group(centered_ui.style()).show(centered_ui, |group_ui| {
            group_ui.vertical_centered_justified(|vertical_ui| {
                vertical_ui.vertical(|ui| {
                    egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |scroll_ui| {
                        let res = self.get_activitat();
                        for line in res.split("\n") {
                            scroll_ui.label(line);
                        }
                    });
                });
            });
        });
    }
    fn setup_configuracio(&mut self, centered_ui: &mut egui::Ui) {}
    fn setup_editar_joc(&mut self, centered_ui: &mut egui::Ui) {
        centered_ui.add_space(10.0);
        egui::Frame::group(centered_ui.style()).show(centered_ui, |group_ui| {
            group_ui.heading("🎮 Afegir Joc");
            group_ui.add_space(10.0);
            group_ui.vertical_centered_justified(|vui| {
                vui.horizontal(|hui| {
                    hui.label("Directori de partides guardades:");
                    if self.joc_afegit.is_empty() {
                        if hui.button("Seleccionar carpeta").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                let folder_path = Some(path.display().to_string());
                                let v = Videojoc::new(folder_path.unwrap());
                                self.joc_afegit = v.local_folder.display().to_string();
                                self.joc_afegit_nom = v.nom.into_string().unwrap();
                            }
                        }
                    } else {
                        hui.add(egui::Label::new(&self.joc_afegit).wrap());
                    }
                });
                vui.horizontal_top(|ui| {
                    ui.label("Nom:");
                    ui.add(egui::TextEdit::singleline(&mut self.joc_afegit_nom));
                });
                vui.add_space(10.0);
                vui.horizontal(|hui| {
                    hui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Afegir").clicked() {
                            self.afegir_joc(self.joc_afegit.clone(), self.joc_afegit_nom.clone());
                            self.joc_afegit = String::new();
                            self.current_mode = AppMode::Dashboard;
                        }
                        if ui.button("Cancel·lar").clicked() {
                            self.joc_afegit = String::new();
                            self.current_mode = AppMode::Dashboard;
                        }
                    });
                });
            });
            group_ui.add_space(10.0);
        });
    }
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
