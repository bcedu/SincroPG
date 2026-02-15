use crate::pg_api::*;
use crate::videojoc::*;
use eframe::egui;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub struct CliPG {
    api: PgAPI,
    vjocs: Vec<Videojoc>,
    config: CliPgConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliPgConfig {
    pub server: ServerConfig,
    pub videojocs_habilitats: VideojocConfigList,
}
impl CliPgConfig {
    fn default() -> Self {
        CliPgConfig {
            server: ServerConfig {
                url: "http://localhost:8000".to_string(),
                usuari: "admin".to_string(),
                contrasenya: "admin".to_string()
            },
            videojocs_habilitats: VideojocConfigList {
                list: Vec::new(),
            },
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub usuari: String,
    pub contrasenya: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VideojocConfigList {
    pub list: Vec<VideojocConfig>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VideojocConfig {
    pub nom: String,
    pub path: String,
}

impl CliPG {
    fn get_credentials(sconf: &ServerConfig) -> (String, String, String) {
        (
            sconf.url.clone(),
            sconf.usuari.clone(),
            sconf.contrasenya.clone()
        )
    }
    pub fn default() -> Self {
        // Obtenim les credencials per el client
        let config = Self::load_or_create_config(None);
        let credencials: (String, String, String) = Self::get_credentials(&config.server);
        CliPG {
            api: PgAPI::new(credencials.0, credencials.1, credencials.2),
            vjocs: Vec::new(),
            config
        }
    }
    fn get_config_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("com", "bcedu", "CliPG").expect("No s'han pogut obtenir els directoris");
        proj_dirs.config_dir().join("config.toml")
    }
    fn load_or_create_config(path: Option<PathBuf>) -> CliPgConfig {
        let cpath;
        if path.is_none() {
            cpath = Self::get_config_path();
        } else {
            cpath = path.unwrap();
        }
        if !cpath.exists() {
            let config = CliPgConfig::default();
            Self::save_config(config, None);
        }
        let content = fs::read_to_string(&cpath).unwrap();
        let config = toml::from_str(&content).unwrap();
        config
    }
    fn save_config(config: CliPgConfig, path: Option<PathBuf>) {
        let cpath;
        if path.is_none() {
            cpath = Self::get_config_path();
        } else {
            cpath = path.unwrap();
        }
        if let Some(dir) = cpath.parent() {
            fs::create_dir_all(dir).unwrap();
        }
        let toml = toml::to_string_pretty(&config).unwrap();
        fs::write(cpath, toml).unwrap();
    }
    fn load_local_jocs(&mut self) -> Vec<VideojocConfig> {
        self.vjocs = Vec::new();
        let mut error_jocs = Vec::new();
        for v in self.config.videojocs_habilitats.list.iter() {
            let path = PathBuf::from(&v.path);
            if path.exists() {
                self.vjocs.push(Videojoc::new(v.path.clone()).with_nom(v.nom.clone()))
            } else {
                error_jocs.push(VideojocConfig {
                    nom: v.nom.clone(),
                    path: v.path.clone(),
                })
            }
        }
        error_jocs
    }
}

impl eframe::App for CliPG {

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CliPG");
        });
    }
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;
    use std::io::{Write, Read};
    use std::path::PathBuf;
    use crate::cli_pg::{CliPG, CliPgConfig, VideojocConfig};
    use crate::pg_api::PgAPI;
    fn get_dummy_cli_pg() -> CliPG {
        let url = "http://localhost:8000".to_string();
        let usuari = "admin".to_string();
        let contrassenya = "pass".to_string();
        let mut config = CliPgConfig::default();
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Napoleon TW".to_string(),
            path: "/home/patata/Napoleon TW".to_string()
        });
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Space Marine 3".to_string(),
            path: "/home/patata/Space Marine 3".to_string()
        });
        CliPG {
            api: PgAPI::new(url.clone(), usuari.clone(), contrassenya.clone()),
            vjocs: Vec::new(),
            config: config,
        }
    }
    fn get_correct_dummy_cli_pg() -> CliPG {
        let url = "http://localhost:8000".to_string();
        let usuari = "admin".to_string();
        let contrassenya = "pass".to_string();
        let mut config = CliPgConfig::default();
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/path a videojocs");
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Mount & blade Warband 2".to_string(),
            path: format!("{}/Mount & blade Warband 2", test_path.to_str().unwrap().clone())
        });
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Napoleón TW HD".to_string(),
            path: format!("{}/Napoleón TW HD", test_path.to_str().unwrap().clone())
        });
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Total War 40k".to_string(),
            path: format!("{}/Total War 40k", test_path.to_str().unwrap().clone())
        });
        CliPG {
            api: PgAPI::new(url.clone(), usuari.clone(), contrassenya.clone()),
            vjocs: Vec::new(),
            config: config,
        }
    }
    fn read_file_sync(path: String) -> String {
        let mut contingut = String::new();
        let mut f = File::open(path).unwrap();
        f.read_to_string(&mut contingut).unwrap();
        drop(f);
        contingut
    }
    fn get_save_config_fixture_conf_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/conf.toml")
    }
    fn get_load_or_create_fixture_conf_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/conf_test.toml")
    }
    #[test]
    fn test_get_config_path() {
        let p = CliPG::get_config_path();
        assert!(p.to_str().unwrap().contains(".config/clipg/config.toml"));
    }
    #[test]
    fn test_save_config() {
        let mut conf = get_dummy_cli_pg().config;
        conf.server.url = "patata".to_string();
        let test_path = get_save_config_fixture_conf_path();
        CliPG::save_config(conf, Some(test_path.clone()));
        let c = read_file_sync(test_path.to_str().unwrap().to_string());
        let expected = "[server]\nurl = \"patata\"\nusuari = \"admin\"\ncontrasenya = \"admin\"\n\n[[videojocs_habilitats.list]]\nnom = \"Napoleon TW\"\npath = \"/home/patata/Napoleon TW\"\n\n[[videojocs_habilitats.list]]\nnom = \"Space Marine 3\"\npath = \"/home/patata/Space Marine 3\"\n";
        assert_eq!(c, expected);
    }
    #[test]
    fn test_load_or_create_config() {
        let test_path = get_load_or_create_fixture_conf_path();
        let c = CliPG::load_or_create_config(Some(test_path));
        assert_eq!(c.server.url, "patata".to_string());
        assert_eq!(c.server.usuari, "demo".to_string());
        assert_eq!(c.server.contrasenya, "demo".to_string());
        assert_eq!(c.videojocs_habilitats.list[0].nom, "Napoleon TW".to_string());
        assert_eq!(c.videojocs_habilitats.list[1].path, "/home/patata/Space Marine 3".to_string());
    }
    #[test]
    fn test_load_local_jocs() {
        let mut cli = get_dummy_cli_pg();
        let res = cli.load_local_jocs();
        assert_eq!(res.len(), 2);
        assert_eq!(res.get(0).unwrap().nom, "Napoleon TW");
        assert_eq!(res.get(1).unwrap().nom, "Space Marine 3");
        assert_eq!(cli.vjocs.len(), 0);
        let mut cli = get_correct_dummy_cli_pg();
        let res = cli.load_local_jocs();
        assert_eq!(res.len(), 0);
        assert_eq!(cli.vjocs.len(), 3);
        assert_eq!(cli.vjocs[0].nom, "Mount & blade Warband 2");
        assert_eq!(cli.vjocs[1].nom, "Napoleón TW HD");
        assert_eq!(cli.vjocs[2].nom, "Total War 40k");
    }
}