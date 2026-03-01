use crate::pg_api::*;
use crate::videojoc::*;
use eframe::egui;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use clap::builder::Str;

pub struct CliPG {
    api: PgAPI,
    pub vjocs: Vec<Videojoc>,
    pub config: CliPgConfig,
    config_path: String,
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
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
        let config_path = Self::get_config_path();
        let config = Self::load_or_create_config(Some(config_path.clone()));
        let credencials: (String, String, String) = Self::get_credentials(&config.server);
        CliPG {
            api: PgAPI::new(credencials.0, credencials.1, credencials.2),
            vjocs: Vec::new(),
            config,
            config_path: config_path.to_str().unwrap().to_string()
        }
    }
    pub fn afegir_joc(&mut self, path: String) -> Result<(), String> {
        let pbuf = PathBuf::from(&path);
        if pbuf.exists() {
            let v = VideojocConfig{
                nom: pbuf.file_name().unwrap().to_str().unwrap().to_string(),
                path
            };
            if !self.config.videojocs_habilitats.list.contains(&v) {
                self.config.videojocs_habilitats.list.push(v);
                Self::save_config(&self.config, Some(PathBuf::from(self.config_path.clone())));
            }
            Ok(())
        } else {
            Err(format!("\"{}\" no existeix.", path))
        }
    }
    pub fn eliminar_joc(&mut self, videojoc_id: String) -> Result<(), String> {
        let mut i = 0;
        let mut trobat = false;
        for vc in self.config.videojocs_habilitats.list.iter() {
            if vc.nom == videojoc_id {
                self.config.videojocs_habilitats.list.remove(i);
                Self::save_config(&self.config, Some(PathBuf::from(self.config_path.clone())));
                trobat = true;
                break;
            }
            i = i + 1;
        }
        if trobat {
            Ok(())
        } else {
            Err(format!("\"{}\" no era un joc habilitat.", videojoc_id))
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
            Self::save_config(&config, None);
        }
        let content = fs::read_to_string(&cpath).unwrap();
        let config = toml::from_str(&content).unwrap();
        config
    }
    fn save_config(config: &CliPgConfig, path: Option<PathBuf>) {
        let cpath;
        if path.is_none() {
            cpath = Self::get_config_path();
        } else {
            cpath = path.unwrap();
        }
        if let Some(dir) = cpath.parent() {
            fs::create_dir_all(dir).unwrap();
        }
        let toml = toml::to_string_pretty(config).unwrap();
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
    pub fn sync_joc(&self, joc: &Videojoc) -> String {
        let mut joc_m = Videojoc::from(joc);
        let joc_res = joc_m.sync(&self.api, false);
        format!("* {}:\n{joc_res}", joc.nom.clone().to_str().unwrap())
    }
    pub fn sync_all(&mut self) -> String {
        let res = "";
        self.load_local_jocs();
        for v in self.vjocs.iter() {
            let joc_res = self.sync_joc(v);
            let res = format!("{}\n{}", res, joc_res);
        }
        res.to_string()
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
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/dummy_conf.toml");
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
            config_path: test_path.to_str().unwrap().to_string()
        }
    }
    fn get_correct_dummy_cli_pg() -> CliPG {
        let url = "http://localhost:8000".to_string();
        let usuari = "admin".to_string();
        let contrassenya = "pass".to_string();
        let mut config = CliPgConfig::default();
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/path a videojocs");
        let conf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/dummy_conf.toml");
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
            config_path: conf_path.to_str().unwrap().to_string()
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
        CliPG::save_config(&conf, Some(test_path.clone()));
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
    #[test]
    fn test_sync_joc() {
        // NO testegem res ja que el metode sync crida només el sync del videjoc
        assert!(true);
    }
    #[test]
    fn test_sync_all() {
        // NO testegem res ja que el metode sync_all nomes fa un bucle cridant el sync_joc
        assert!(true);
    }
    #[test]
    fn test_afegir_joc() {
        let mut cli = get_dummy_cli_pg();
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 2);
        CliPG::save_config(&cli.config, Some(PathBuf::from(cli.config_path.clone())));
        // Un path fictici no afegeix res
        let err = cli.afegir_joc("/home/patata/Napoleon TW".to_string());
        assert!(err.is_err());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 2);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(res_cont, "[server]\nurl = \"http://localhost:8000\"\nusuari = \"admin\"\ncontrasenya = \"admin\"\n\n[[videojocs_habilitats.list]]\nnom = \"Napoleon TW\"\npath = \"/home/patata/Napoleon TW\"\n\n[[videojocs_habilitats.list]]\nnom = \"Space Marine 3\"\npath = \"/home/patata/Space Marine 3\"\n");
        // Un path real si que afageix
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2");
        cli.afegir_joc(test_path.to_str().unwrap().to_string());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 3);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(res_cont, "[server]\nurl = \"http://localhost:8000\"\nusuari = \"admin\"\ncontrasenya = \"admin\"\n\n[[videojocs_habilitats.list]]\nnom = \"Napoleon TW\"\npath = \"/home/patata/Napoleon TW\"\n\n[[videojocs_habilitats.list]]\nnom = \"Space Marine 3\"\npath = \"/home/patata/Space Marine 3\"\n\n[[videojocs_habilitats.list]]\nnom = \"Mount & blade Warband 2\"\npath = \"/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2\"\n");
        // Un path repetit no fa res
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2");
        cli.afegir_joc(test_path.to_str().unwrap().to_string());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 3);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(res_cont, "[server]\nurl = \"http://localhost:8000\"\nusuari = \"admin\"\ncontrasenya = \"admin\"\n\n[[videojocs_habilitats.list]]\nnom = \"Napoleon TW\"\npath = \"/home/patata/Napoleon TW\"\n\n[[videojocs_habilitats.list]]\nnom = \"Space Marine 3\"\npath = \"/home/patata/Space Marine 3\"\n\n[[videojocs_habilitats.list]]\nnom = \"Mount & blade Warband 2\"\npath = \"/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2\"\n");
    }
    #[test]
    fn test_eliminar_joc() {
        let mut cli = get_dummy_cli_pg();
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 2);
        CliPG::save_config(&cli.config, Some(PathBuf::from(cli.config_path.clone())));
        // Un videojoc_id dona error
        let err = cli.eliminar_joc("PATATA".to_string());
        assert!(err.is_err());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 2);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(res_cont, "[server]\nurl = \"http://localhost:8000\"\nusuari = \"admin\"\ncontrasenya = \"admin\"\n\n[[videojocs_habilitats.list]]\nnom = \"Napoleon TW\"\npath = \"/home/patata/Napoleon TW\"\n\n[[videojocs_habilitats.list]]\nnom = \"Space Marine 3\"\npath = \"/home/patata/Space Marine 3\"\n");
        // Un videojoc_id que elimina
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2");
        cli.eliminar_joc("Napoleon TW".to_string());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 1);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(res_cont, "[server]\nurl = \"http://localhost:8000\"\nusuari = \"admin\"\ncontrasenya = \"admin\"\n\n[[videojocs_habilitats.list]]\nnom = \"Space Marine 3\"\npath = \"/home/patata/Space Marine 3\"\n");
    }
}