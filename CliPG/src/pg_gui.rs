use crate::cli_pg::*;
use std::path::PathBuf;

pub struct PgGUI {
    clipg_config_path: Option<PathBuf>,
}

impl PgGUI {
    pub fn new(clipg_config_path: Option<PathBuf>) -> Self {
        Self { clipg_config_path }
    }
    pub fn start(&self) {
        let options = eframe::NativeOptions::default();
        eframe::run_native("CliPG", options, Box::new(|_cc| Ok(Box::new(CliPG::default(self.clipg_config_path.clone())))));
    }
}
