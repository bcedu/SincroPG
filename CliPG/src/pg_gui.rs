use crate::cli_pg::CliPG;
use crate::videojoc::Videojoc;
use eframe::egui::{self, CornerRadius, RichText};
use rfd::FileDialog;
use std::path::PathBuf;

pub fn start_pg_gui(clipg_config_path: Option<PathBuf>) -> Result<(), eframe::Error> {
    let options = PgGUI::get_default_egui_options(&clipg_config_path);
    let res = eframe::run_native("CliPG: Sincronitzacio de partides guardades", options, Box::new(|_cc| Ok(Box::new(PgGUI::default()))));
    res
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum AppMode {
    Dashboard,
    EditarJoc,
    Configuracio,
}
pub struct PgGUI {
    clipg_config_path: Option<PathBuf>,
    current_mode: AppMode,
    estat_servidor: String,
    activitat: String,
    joc_afegit: String,
    joc_afegit_nom: String,
    config_url: String,
    config_usuari: String,
    config_contrasenya: String,
    quit_app: bool,
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
            config_url: String::new(),
            config_usuari: String::new(),
            config_contrasenya: String::new(),
            quit_app: false,
        }
    }
}
// Metodes per configurar opcions de la UI
impl PgGUI {
    fn get_default_egui_options(clipg_config_path: &Option<PathBuf>) -> eframe::NativeOptions {
        let mut res = eframe::NativeOptions::default();
        res.persist_window = true;
        res.persistence_path = clipg_config_path.clone();
        res
    }
    fn setup_style(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Tema Clar/Fosc
        if ctx.system_theme().or_else(|| Some(egui::Theme::Light)) == Some(egui::Theme::Light) {
            ctx.set_theme(egui::Theme::Light);
        } else {
            ctx.set_theme(egui::Theme::Dark);
        }
        ctx.all_styles_mut(|style| {
            // Marge als buttons
            style.spacing.button_padding = egui::vec2(12.0, 4.0);
        });
    }
    fn setup_signals(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_signal_close(ctx, _frame);
    }
    fn setup_signal_close(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            self.quit_app = true; // TODO: fins que no sapiga fer una instancia de app unica no puc fer que s'amagui i ja. S'ha de tancar la app
            if !self.quit_app {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            } else {
                self.quit_app = false;
            }
        }
    }
}
// Metodes amb logica de aplicacio
impl PgGUI {
    fn get_estat_servidor(&mut self) -> String {
        if self.estat_servidor.is_empty() {
            let clipg = CliPG::default(self.clipg_config_path.clone());
            let conectat = if clipg.api.probar_connexio() { "✔ Conectat" } else { "❌ Desconectat" };
            self.estat_servidor = format!("{} ({})", conectat, clipg.config.server.url,);
            self.config_url = clipg.config.server.url;
            self.config_usuari = clipg.config.server.usuari;
            self.config_contrasenya = clipg.config.server.contrasenya;
        }
        self.estat_servidor.clone()
    }
    fn get_activitat(&mut self) -> String {
        self.activitat.clone()
    }
    fn sincronitzar_tots(&mut self) {
        let mut clipg = CliPG::default(self.clipg_config_path.clone());
        let res = clipg.sync_all(false);
        self.activitat = res.trim().to_string();
    }
    fn sincronitzar_joc(&mut self, joc: &mut Videojoc) {
        let mut clipg = CliPG::default(self.clipg_config_path.clone());
        let res = clipg.sync_joc(joc, false);
        self.activitat = res.trim().to_string();
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
    fn guardar_configuracio(&mut self, url: String, usuari: String, contrasenya: String) {
        let mut clipg = CliPG::default(self.clipg_config_path.clone());
        clipg.config.server.url = url;
        clipg.config.server.usuari = usuari;
        clipg.config.server.contrasenya = contrasenya;
        CliPG::save_config(&clipg.config, self.clipg_config_path.clone());
    }
}
// Metodes amb components i construiccio de la UI
impl PgGUI {
    fn ui_card<F: FnOnce(&mut egui::Ui)>(ui: &mut egui::Ui, title: Option<&str>, add: F) {
        egui::Frame::group(ui.style()).corner_radius(CornerRadius::same(8)).inner_margin(12.0).show(ui, |ui| {
            if let Some(title) = title {
                ui.heading(RichText::new(title).strong());
                ui.add_space(10.0);
            }
            add(ui);
        });
    }
    fn ui_primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE).strong().size(14.0))
                .fill(egui::Color32::from_rgb(0, 120, 215))
                .corner_radius(6.0),
        )
    }
    fn ui_danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE).strong().size(14.0))
                .fill(egui::Color32::from_rgb(230, 90, 90))
                .corner_radius(6.0),
        )
    }
    fn ui_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(text).color(egui::Color32::BLACK).strong().size(14.0))
                .fill(egui::Color32::LIGHT_GRAY)
                .corner_radius(6.0),
        )
    }
    fn ui_primary_secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(text).color(egui::Color32::from_rgb(0, 120, 215)).strong())
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 120, 215)))
                .corner_radius(6.0),
        )
    }
    fn ui_danger_secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(text).color(egui::Color32::from_rgb(230, 90, 90)).strong())
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 90, 90)))
                .corner_radius(6.0),
        )
    }
    fn ui_secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
        ui.add(
            egui::Button::new(egui::RichText::new(text).color(egui::Color32::DARK_GRAY).strong())
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::new(1.0, egui::Color32::DARK_GRAY))
                .corner_radius(6.0),
        )
    }
    fn setup_top_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Fitxer", |ui| {
                if ui.button("Afegir joc").clicked() {
                    self.current_mode = AppMode::EditarJoc;
                }
                if ui.button("Tancar i sortir").clicked() {
                    self.quit_app = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Edita", |ui| {
                if ui.button("Preferències").clicked() {
                    self.current_mode = AppMode::Configuracio;
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
        Self::ui_card(centered_ui, Some("🎮 Videojocs a sincronitzar"), |group_ui| {
            egui::ScrollArea::vertical().show(group_ui, |scroll_ui| {
                scroll_ui.horizontal(|row_ui| {
                    if Self::ui_button(row_ui, "+ Afegir joc").clicked() {
                        self.current_mode = AppMode::EditarJoc;
                    }
                    row_ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |right_ui| {
                        if Self::ui_primary_button(right_ui, "🔄 Sincronitzar tots").clicked() {
                            self.sincronitzar_tots();
                        }
                    });
                });
                scroll_ui.add_space(10.0);
                let mut clipg = CliPG::default(self.clipg_config_path.clone());
                clipg.load_local_jocs();
                for joc in clipg.vjocs.iter_mut() {
                    scroll_ui.horizontal(|row_ui| {
                        row_ui.strong(joc.nom.clone().to_str().unwrap());
                        row_ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |right_ui| {
                            if Self::ui_danger_secondary_button(right_ui, "🗑").clicked() {
                                self.eliminar_joc(joc);
                            }
                            if Self::ui_secondary_button(right_ui, "🛠").clicked() {
                                self.joc_afegit = joc.local_folder.clone().display().to_string();
                                self.joc_afegit_nom = joc.nom.clone().into_string().unwrap();
                                self.current_mode = AppMode::EditarJoc;
                            }
                            if Self::ui_primary_secondary_button(right_ui, "🔄").clicked() {
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
        Self::ui_card(centered_ui, None, |ui| {
            ui.horizontal(|ui| {
                let estat_servidor = self.get_estat_servidor();
                let color;
                if estat_servidor.contains("Desconectat") {
                    color = egui::Color32::DARK_RED;
                } else {
                    color = egui::Color32::DARK_GREEN;
                }
                ui.label("Estat servidor:");
                ui.colored_label(color, format!("{}", estat_servidor));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if Self::ui_button(ui, "⚙ Editar").clicked() {
                        self.current_mode = AppMode::Configuracio;
                    }
                });
            });
        });
    }
    fn setup_dashboard_activitat(&mut self, centered_ui: &mut egui::Ui) {
        centered_ui.add_space(10.0);
        Self::ui_card(centered_ui, None, |group_ui| {
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
    fn setup_configuracio(&mut self, centered_ui: &mut egui::Ui) {
        centered_ui.add_space(10.0);
        Self::ui_card(centered_ui, Some("⚙ Configuració"), |group_ui| {
            group_ui.vertical_centered_justified(|vui| {
                vui.add_space(4.0);
                vui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.add(egui::TextEdit::singleline(&mut self.config_url).code_editor().desired_width(f32::INFINITY));
                });
                vui.add_space(4.0);
                vui.horizontal(|ui| {
                    ui.label("Usuari:");
                    ui.add(egui::TextEdit::singleline(&mut self.config_usuari).code_editor().desired_width(f32::INFINITY));
                });
                vui.add_space(4.0);
                vui.horizontal(|ui| {
                    ui.label("Contrasenya:");
                    ui.add(egui::TextEdit::singleline(&mut self.config_contrasenya).code_editor().desired_width(f32::INFINITY));
                });
                vui.add_space(10.0);
                vui.horizontal(|hui| {
                    hui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Self::ui_primary_button(ui, "Desar").clicked() {
                            self.guardar_configuracio(self.config_url.clone(), self.config_usuari.clone(), self.config_contrasenya.clone());
                            self.estat_servidor = String::new();
                            self.current_mode = AppMode::Dashboard;
                        }
                        if Self::ui_danger_button(ui, "Cancel·lar").clicked() {
                            self.estat_servidor = String::new();
                            self.current_mode = AppMode::Dashboard;
                        }
                    });
                });
            });
            group_ui.add_space(10.0);
        });
    }
    fn setup_editar_joc(&mut self, centered_ui: &mut egui::Ui) {
        centered_ui.add_space(10.0);
        Self::ui_card(centered_ui, Some("🎮 Afegir Joc"), |group_ui| {
            group_ui.vertical_centered_justified(|vui| {
                vui.add_space(4.0);
                vui.horizontal(|hui| {
                    hui.label("Directori de partides guardades:");
                    if self.joc_afegit.is_empty() {
                        if Self::ui_button(hui, "Seleccionar carpeta").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                let folder_path = Some(path.display().to_string());
                                let v = Videojoc::new(folder_path.unwrap());
                                self.joc_afegit = v.local_folder.display().to_string();
                                self.joc_afegit_nom = v.nom.into_string().unwrap();
                            }
                        }
                    } else {
                        hui.add(egui::TextEdit::singleline(&mut self.joc_afegit).desired_width(f32::INFINITY));
                        if Self::ui_button(hui, "🛠").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                let folder_path = Some(path.display().to_string());
                                let v = Videojoc::new(folder_path.unwrap());
                                self.joc_afegit = v.local_folder.display().to_string();
                                self.joc_afegit_nom = v.nom.into_string().unwrap();
                            }
                        }
                    }
                });
                vui.add_space(4.0);
                vui.horizontal_top(|ui| {
                    ui.label("Nom:");
                    ui.add(egui::TextEdit::singleline(&mut self.joc_afegit_nom).desired_width(f32::INFINITY));
                });
                vui.add_space(10.0);
                vui.horizontal(|hui| {
                    hui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if Self::ui_primary_button(ui, "Afegir").clicked() {
                            self.afegir_joc(self.joc_afegit.clone(), self.joc_afegit_nom.clone());
                            self.joc_afegit = String::new();
                            self.joc_afegit_nom = String::new();
                            self.current_mode = AppMode::Dashboard;
                        }
                        if Self::ui_danger_button(ui, "Cancel·lar").clicked() {
                            self.joc_afegit = String::new();
                            self.joc_afegit_nom = String::new();
                            self.current_mode = AppMode::Dashboard;
                        }
                    });
                });
            });
            group_ui.add_space(10.0);
        });
    }
}
// Bucle principal de egui
impl eframe::App for PgGUI {
    fn ui(&mut self, egui_ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = egui_ui.ctx();
        self.setup_signals(ctx, _frame);
        self.setup_style(ctx, _frame);
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
                    };
                });
            });
        });
    }
}
