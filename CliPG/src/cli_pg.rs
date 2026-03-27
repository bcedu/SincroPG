use crate::pg_api::*;
use crate::videojoc::*;
use directories::ProjectDirs;
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct CliPG {
    pub api: Box<dyn PartidesGuardadesAPI>,
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
                contrasenya: "admin".to_string(),
            },
            videojocs_habilitats: VideojocConfigList { list: Vec::new() },
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub url: String,
    pub usuari: String,
    pub contrasenya: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct VideojocConfigList {
    pub list: Vec<VideojocConfig>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct VideojocConfig {
    pub nom: String,
    pub path: String,
    pub partides_guardades: Vec<PartidaGuardadaConfig>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PartidaGuardadaConfig {
    pub path: String,
    pub hash: String,
}
impl CliPG {
    fn get_credentials(sconf: &ServerConfig) -> (String, String, String) {
        (sconf.url.clone(), sconf.usuari.clone(), sconf.contrasenya.clone())
    }
    pub fn default(config_path: Option<PathBuf>) -> Self {
        // Obtenim les credencials per el client
        let config_path = config_path.unwrap_or_else(Self::get_config_path);
        let config = Self::load_or_create_config(Some(config_path.clone()));
        let credencials: (String, String, String) = Self::get_credentials(&config.server);
        CliPG {
            api: Box::new(PgAPI::new(credencials.0, credencials.1, credencials.2)),
            vjocs: Vec::new(),
            config,
            config_path: config_path.to_str().unwrap().to_string(),
        }
    }
    pub fn afegir_joc(&mut self, path: String) -> Result<(), String> {
        let pbuf = PathBuf::from(&path);
        if pbuf.exists() {
            let v = VideojocConfig {
                nom: pbuf.file_name().unwrap().to_str().unwrap().to_string(),
                path,
                partides_guardades: Vec::new(),
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
        if trobat { Ok(()) } else { Err(format!("\"{}\" no era un joc habilitat.", videojoc_id)) }
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
        let config;
        if !cpath.exists() {
            config = CliPgConfig::default();
            Self::save_config(&config, Some(cpath));
        } else {
            let mut content = String::new();
            let mut f = File::open(&cpath).unwrap();
            f.read_to_string(&mut content).unwrap();
            drop(f);
            if content == "" {
                config = CliPgConfig::default();
                Self::save_config(&config, None);
            } else {
                config = toml::from_str(&content).unwrap();
            }
        }
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
        let mut f = File::create(&cpath).unwrap();
        f.write_all(toml.as_bytes()).unwrap();
        f.sync_all().unwrap();
        drop(f);
    }
    fn load_local_jocs(&mut self) -> Vec<VideojocConfig> {
        self.vjocs = Vec::new();
        let mut error_jocs = Vec::new();
        for v in self.config.videojocs_habilitats.list.iter() {
            let path = PathBuf::from(&v.path);
            if path.exists() {
                self.vjocs.push(
                    Videojoc::new(path.to_str().unwrap().to_string())
                        .with_nom(v.nom.clone())
                        .with_partides_guardades_list(&v.partides_guardades),
                )
            } else {
                error_jocs.push(VideojocConfig {
                    nom: v.nom.clone(),
                    path: v.path.clone(),
                    partides_guardades: Vec::new(),
                })
            }
        }
        error_jocs
    }
    pub fn sync_joc(&self, joc: &mut Videojoc, test_mode: bool) -> String {
        let joc_res = joc.sync(&self.api, test_mode);
        format!("* {}:\n{joc_res}", joc.nom.clone().to_str().unwrap())
    }
    pub fn sync_all(&mut self, test_mode: bool) -> String {
        let mut res = String::new();
        let mut new_config = CliPgConfig {
            server: self.config.server.clone(),
            videojocs_habilitats: VideojocConfigList { list: Vec::new() },
        };
        self.load_local_jocs();
        for v in self.vjocs.iter() {
            let mut updated_v = Videojoc::from(v);
            let joc_res = self.sync_joc(&mut updated_v, test_mode);
            res.push_str(&format!("\n{}", joc_res.as_str()));
            new_config.videojocs_habilitats.list.push(VideojocConfig {
                nom: updated_v.nom.to_str().unwrap().to_string().clone(),
                path: updated_v.local_folder.to_str().unwrap().to_string().clone(),
                partides_guardades: updated_v.get_partides_guardades_list(),
            });
        }
        Self::save_config(&new_config, Some(PathBuf::from(self.config_path.clone())));
        self.config = new_config;
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
    use crate::cli_pg::{CliPG, CliPgConfig, Videojoc, VideojocConfig};
    use crate::pg_api::{PartidesGuardadesAPI, PgAPI};
    use crate::videojoc::partida_guardada::PartidaGuardada;
    use std::ffi::OsString;
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::PathBuf;
    pub struct FakeAPI_fase1;
    impl PartidesGuardadesAPI for FakeAPI_fase1 {
        fn probar_connexio(&self) -> bool {
            true
        }
        fn get_videojocs(&self) -> Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: &Videojoc) -> Vec<PartidaGuardada> {
            let mut v = Vec::new();
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save1.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save1.txt"),
                path: path,
                timestamp: 245528886,
                hash: "acbbaa798a883fb0be7534092b20f5188fb07799a1c175c28f8fb1b03bc63ae2".to_string(),
            };
            v.push(p1);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {
            PgAPI::new("url".to_string(), "usuari".to_string(), "contrassenya".to_string())._post_partida_guardada(partida_guardada);
        }
        fn delete_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            "Pastanaga bullida@".to_string()
        }
    }
    pub struct FakeAPI_fase2;
    impl PartidesGuardadesAPI for FakeAPI_fase2 {
        fn probar_connexio(&self) -> bool {
            true
        }
        fn get_videojocs(&self) -> Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: &Videojoc) -> Vec<PartidaGuardada> {
            let mut v = Vec::new();
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save1.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save1.txt"),
                path: path,
                timestamp: 245528886,
                hash: "3b136fcad41f6a8fb66b38cae89aaba00f30ac7f79797fcd8a46bc13a733811a".to_string(),
            };
            v.push(p1);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {
            PgAPI::new("url".to_string(), "usuari".to_string(), "contrassenya".to_string())._post_partida_guardada(partida_guardada);
        }
        fn delete_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            "Pastanaga bullida@2 la venganza".to_string()
        }
    }
    pub struct FakeAPI_fase4;
    impl PartidesGuardadesAPI for FakeAPI_fase4 {
        fn probar_connexio(&self) -> bool {
            true
        }
        fn get_videojocs(&self) -> Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: &Videojoc) -> Vec<PartidaGuardada> {
            let mut v = Vec::new();
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save3.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save3.txt"),
                path: path,
                timestamp: 245528886,
                hash: "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4".to_string(),
            };
            v.push(p1);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {
            PgAPI::new("url".to_string(), "usuari".to_string(), "contrassenya".to_string())._post_partida_guardada(partida_guardada);
        }
        fn delete_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            "Pastanaga bullida@ 3 sl retrno".to_string()
        }
    }
    pub struct FakeAPI_fase5;
    impl PartidesGuardadesAPI for FakeAPI_fase5 {
        fn probar_connexio(&self) -> bool {
            true
        }
        fn get_videojocs(&self) -> Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: &Videojoc) -> Vec<PartidaGuardada> {
            let mut v = Vec::new();
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save3.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save3.txt"),
                path: path,
                timestamp: 245528886,
                hash: "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4".to_string(),
            };
            v.push(p1);
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save4.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save4.txt"),
                path: path,
                timestamp: 0,
                hash: "12ee21760f19253fca62f5d0cdf480d1477c37300e56c2af141bcf35226a89b3".to_string(),
            };
            v.push(p1);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {
            PgAPI::new("url".to_string(), "usuari".to_string(), "contrassenya".to_string())._post_partida_guardada(partida_guardada);
        }
        fn delete_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            "save 4 alt 2".to_string()
        }
    }
    pub struct FakeAPI_fase6 {
        pub bck_name: String,
    }
    impl PartidesGuardadesAPI for FakeAPI_fase6 {
        fn probar_connexio(&self) -> bool {
            true
        }
        fn get_videojocs(&self) -> Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: &Videojoc) -> Vec<PartidaGuardada> {
            let mut v = Vec::new();
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save3.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save3.txt"),
                path: path,
                timestamp: 245528886,
                hash: "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4".to_string(),
            };
            v.push(p1);
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save4.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save4.txt"),
                path: path,
                timestamp: 999999999,
                hash: "d623d87b3ef0b2b93f99637081a29ca70fa78c527f1f28a9242a7e93910fb194".to_string(),
            };
            v.push(p1);
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures_cli_pg/test_sync/Joc/{}.txt", self.bck_name.clone()));
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from(self.bck_name.clone()),
                path: path,
                timestamp: 999999999,
                hash: "12ee21760f19253fca62f5d0cdf480d1477c37300e56c2af141bcf35226a89b3".to_string(),
            };
            v.push(p1);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {
            PgAPI::new("url".to_string(), "usuari".to_string(), "contrassenya".to_string())._post_partida_guardada(partida_guardada);
        }
        fn delete_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            "save 4 alt 2".to_string()
        }
    }
    pub struct FakeAPI_fase7 {
        pub bck_name: String,
    }
    impl PartidesGuardadesAPI for FakeAPI_fase7 {
        fn probar_connexio(&self) -> bool {
            true
        }
        fn get_videojocs(&self) -> Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: &Videojoc) -> Vec<PartidaGuardada> {
            let mut v = Vec::new();
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save3.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save3.txt"),
                path: path,
                timestamp: 245528886,
                hash: "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4".to_string(),
            };
            v.push(p1);
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures_cli_pg/test_sync/Joc/{}.txt", self.bck_name.clone()));
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from(self.bck_name.clone()),
                path: path,
                timestamp: 999999999,
                hash: "12ee21760f19253fca62f5d0cdf480d1477c37300e56c2af141bcf35226a89b3".to_string(),
            };
            v.push(p1);
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc/save4.txt");
            let p1 = PartidaGuardada {
                videojoc: "Joc".to_string(),
                nom: OsString::from("save4.txt"),
                path: path,
                timestamp: 999999999,
                hash: "patata".to_string(),
            };
            v.push(p1);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {
            PgAPI::new("url".to_string(), "usuari".to_string(), "contrassenya".to_string())._post_partida_guardada(partida_guardada);
        }
        fn delete_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            "save 4 alt 22222222".to_string()
        }
    }
    fn get_dummy_cli_pg() -> CliPG {
        let url = "http://localhost:8000".to_string();
        let usuari = "admin".to_string();
        let contrassenya = "pass".to_string();
        let mut config = CliPgConfig::default();
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/dummy_conf.toml");
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Napoleon TW".to_string(),
            path: "/home/patata/Napoleon TW".to_string(),
            partides_guardades: Vec::new(),
        });
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Space Marine 3".to_string(),
            path: "/home/patata/Space Marine 3".to_string(),
            partides_guardades: Vec::new(),
        });
        CliPG {
            api: Box::new(PgAPI::new(url.clone(), usuari.clone(), contrassenya.clone())),
            vjocs: Vec::new(),
            config: config,
            config_path: test_path.to_str().unwrap().to_string(),
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
            path: format!("{}/Mount & blade Warband 2", test_path.to_str().unwrap().clone()),
            partides_guardades: Vec::new(),
        });
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Napoleón TW HD".to_string(),
            path: format!("{}/Napoleón TW HD", test_path.to_str().unwrap().clone()),
            partides_guardades: Vec::new(),
        });
        config.videojocs_habilitats.list.push(VideojocConfig {
            nom: "Total War 40k".to_string(),
            path: format!("{}/Total War 40k", test_path.to_str().unwrap().clone()),
            partides_guardades: Vec::new(),
        });
        CliPG {
            api: Box::new(PgAPI::new(url.clone(), usuari.clone(), contrassenya.clone())),
            vjocs: Vec::new(),
            config: config,
            config_path: conf_path.to_str().unwrap().to_string(),
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
        let expected = r#"[server]
url = "patata"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Napoleon TW"
path = "/home/patata/Napoleon TW"
partides_guardades = []

[[videojocs_habilitats.list]]
nom = "Space Marine 3"
path = "/home/patata/Space Marine 3"
partides_guardades = []
"#;
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
    fn test_afegir_joc() {
        let mut cli = get_dummy_cli_pg();
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 2);
        CliPG::save_config(&cli.config, Some(PathBuf::from(cli.config_path.clone())));
        // Un path fictici no afegeix res
        let err = cli.afegir_joc("/home/patata/Napoleon TW".to_string());
        assert!(err.is_err());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 2);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(
            res_cont,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Napoleon TW"
path = "/home/patata/Napoleon TW"
partides_guardades = []

[[videojocs_habilitats.list]]
nom = "Space Marine 3"
path = "/home/patata/Space Marine 3"
partides_guardades = []
"#
        );
        // Un path real si que afageix
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2");
        cli.afegir_joc(test_path.to_str().unwrap().to_string());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 3);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(
            res_cont,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Napoleon TW"
path = "/home/patata/Napoleon TW"
partides_guardades = []

[[videojocs_habilitats.list]]
nom = "Space Marine 3"
path = "/home/patata/Space Marine 3"
partides_guardades = []

[[videojocs_habilitats.list]]
nom = "Mount & blade Warband 2"
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2"
partides_guardades = []
"#
        );
        // Un path repetit no fa res
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2");
        cli.afegir_joc(test_path.to_str().unwrap().to_string());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 3);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(
            res_cont,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Napoleon TW"
path = "/home/patata/Napoleon TW"
partides_guardades = []

[[videojocs_habilitats.list]]
nom = "Space Marine 3"
path = "/home/patata/Space Marine 3"
partides_guardades = []

[[videojocs_habilitats.list]]
nom = "Mount & blade Warband 2"
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/path a videojocs/Mount & blade Warband 2"
partides_guardades = []
"#
        );
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
        assert_eq!(
            res_cont,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Napoleon TW"
path = "/home/patata/Napoleon TW"
partides_guardades = []

[[videojocs_habilitats.list]]
nom = "Space Marine 3"
path = "/home/patata/Space Marine 3"
partides_guardades = []
"#
        );
        // Un videojoc_id que elimina
        cli.eliminar_joc("Napoleon TW".to_string());
        assert_eq!(cli.config.videojocs_habilitats.list.len(), 1);
        let res_cont = read_file_sync(cli.config_path.clone());
        assert_eq!(
            res_cont,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Space Marine 3"
path = "/home/patata/Space Marine 3"
partides_guardades = []
"#
        );
    }
    #[test]
    fn test_sync_joc() {
        // NO testegem res ja que el metode sync crida només el sync del videjoc
        assert!(true);
    }
    #[test]
    fn test_full_process() {
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync");
        let conf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/conf.toml");
        let joc_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_cli_pg/test_sync/Joc");
        let mut clipg = test_full_process_fase_0(&test_path, &conf_path, &joc_path);
        test_full_process_fase_1(&mut clipg, &joc_path, &conf_path);
        test_full_process_fase_2(&mut clipg, &joc_path, &conf_path);
        test_full_process_fase_3(&mut clipg, &joc_path, &conf_path);
        test_full_process_fase_4(&mut clipg, &joc_path, &conf_path);
        let back_file = test_full_process_fase_5(&mut clipg, &joc_path, &conf_path);
        test_full_process_fase_6(&mut clipg, &joc_path, &conf_path, &back_file);
        test_full_process_fase_7(&mut clipg, &joc_path, &conf_path, &back_file);
    }
    fn test_full_process_fase_0(test_path: &PathBuf, conf_path: &PathBuf, joc_path: &PathBuf) -> CliPG {
        /*
         * PRE:
         * El fitxer de configuracio no existeix.
         * En local no tenim cap fitxer a "Joc".
         * POST:
         * Inicialitzem un CliPG.
         * Aixo crea un primer conf.toml sense cap joc habilitat.
         * Habilitem el "Joc". Aixo actualitza el conf.toml amb el joc habilitat.
         */
        // Fem neteja: eliminem tots els fitxers de test_sync
        fs::remove_dir_all(test_path).unwrap();
        fs::create_dir(test_path).unwrap();
        fs::create_dir(joc_path).unwrap();
        // Comencem el test
        let mut clipg = CliPG::default(Some(conf_path.clone()));
        let config_content = read_file_sync(clipg.config_path.clone());
        assert_eq!(
            config_content,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[videojocs_habilitats]
list = []
"#
        );
        let result = clipg.afegir_joc(joc_path.to_str().unwrap().to_string());
        assert!(result.is_ok());
        let config_content = read_file_sync(clipg.config_path.clone());
        assert_eq!(
            config_content,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Joc"
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc"
partides_guardades = []
"#
        );
        clipg
    }
    fn test_full_process_fase_1(clipg: &mut CliPG, joc_path: &PathBuf, conf_path: &PathBuf) {
        /*
         * PRE:
         * El fitxer de configuracio te habilitat "Joc" sense cap partida.
         * En local no tenim cap fitxer a "Joc".
         * En remot tenim el save1.txt.
         * POST:
         * Fem una primera sincronitzacio que crearà el fitxer save1.txt a local.
         * S'actualitzarà el conf.toml amb el save1.txt.
         */
        // Fem sincronitzacio amb el fake_apiq ue ens diu que hi ha el save1.txt a remot
        clipg.api = Box::new(FakeAPI_fase1 {});
        let result = clipg.sync_all(false);
        // Revisem que el resum que ens retornen indica que s'ha descarregat el save1.txt
        assert!(result.contains("⬇ Descarregar partida remota: save1.txt"));
        // Revisem que el fitxer save1.txt existeix a local i el seu contingut
        let save_path = joc_path.join("save1.txt");
        assert!(save_path.exists());
        let save_content = read_file_sync(save_path.to_str().unwrap().to_string());
        assert_eq!(save_content, r#"Pastanaga bullida@"#);
        // Revisem el contingut del conf.toml
        let config_content = read_file_sync(conf_path.to_str().unwrap().to_string());
        assert_eq!(
            config_content,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Joc"
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc"

[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save1.txt"
hash = "acbbaa798a883fb0be7534092b20f5188fb07799a1c175c28f8fb1b03bc63ae2"
"#
        );
    }
    fn test_full_process_fase_2(clipg: &mut CliPG, joc_path: &PathBuf, conf_path: &PathBuf) {
        /*
         * PRE:
         * El fitxer de configuracio te habilitat "Joc" amb 1 partida (save1.txt).
         * En local tenim el save1.txt a "Joc".
         * En remot tenim el save1.txt.
         * POST:
         * Fem una primera sincronitzacio que no fa res perque no ha canviat res.
         * En remot, canviem el hash del save1.txt per simular que l'han canviat.
         * Tornem a sincronitzar i s'ha de actuialitzar el save1.txt local
         * i també el conf.toml amb el nou hash.
         */
        // NO ha canviat res
        clipg.api = Box::new(FakeAPI_fase1 {});
        let result = clipg.sync_all(false);
        assert_eq!(
            result,
            r#"
* Joc:
    ✔ Partida OK: save1.txt
"#
        );
        clipg.api = Box::new(FakeAPI_fase2 {});
        // Ha canviat el save1.txt remot
        let result = clipg.sync_all(false);
        assert_eq!(
            result,
            r#"
* Joc:
    ⬇ Descarregar (remot modificat): save1.txt
"#
        );
        // Revisem el contingut del conf.toml
        let config_content = read_file_sync(conf_path.to_str().unwrap().to_string());
        assert_eq!(
            config_content,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Joc"
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc"

[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save1.txt"
hash = "3b136fcad41f6a8fb66b38cae89aaba00f30ac7f79797fcd8a46bc13a733811a"
"#
        );
    }
    fn test_full_process_fase_3(clipg: &mut CliPG, joc_path: &PathBuf, conf_path: &PathBuf) {
        /*
         * PRE:
         * El fitxer de configuracio te habilitat "Joc" amb 1 partida (save1.txt).
         * En local tenim el save1.txt a "Joc".
         * En remot tenim el save1.txt.
         * POST:
         * Creem el save2.txt en local i eliminem el save1.txt.
         * Sincronitzem: s'elimina el save1.txt de remot i es crea el save2.txt.
         * S'actualitza el conf.toml per mostrar que ja nomes tenim el save2.txt
         */
        // Creem el save2.txt
        std::fs::write(&joc_path.join("save2.txt"), "save2.txt content").unwrap();
        // Eliminem el save1.txt de local
        fs::remove_file(&joc_path.join("save1.txt")).unwrap();
        // Sincronitzem
        clipg.api = Box::new(FakeAPI_fase2 {});
        let result = clipg.sync_all(false);
        assert_eq!(
            result,
            r#"
* Joc:
    ❌ Eliminar remot: save1.txt
    ⬆ Pujar partida local: save2.txt
"#
        );
        // Verifiquem que s'ha actualitzat el conf.toml
        let config_content = read_file_sync(conf_path.to_str().unwrap().to_string());
        assert_eq!(
            config_content,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Joc"
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc"

[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save2.txt"
hash = "47a1e807edaccae9bdc6d5f5eb1da36becbd1484b8d394e8a572597af65302b2"
"#
        );
    }
    fn test_full_process_fase_4(clipg: &mut CliPG, joc_path: &PathBuf, conf_path: &PathBuf) {
        /*
         * PRE:
         * El fitxer de configuracio te habilitat "Joc" amb 1 partida (save2.txt).
         * En local tenim el save2.txt a "Joc".
         * En remot tenim el save2.txt.
         * POST:
         * Creem el save3.txt en remot i eliminem el save2.txt del remot.
         * Sincronitzem: s'elimina el save2.txt de local i es crea el save3.txt al local.
         * S'actualitza el conf.toml per mostrar que ja nomes tenim el save3.txt
         */
        // Sincronitzem. En remote s'ha eliminat save2 i creat save3
        clipg.api = Box::new(FakeAPI_fase4 {});
        let result = clipg.sync_all(false);
        assert_eq!(
            result,
            r#"
* Joc:
    ❌ Eliminar local: save2.txt
    ⬇ Descarregar partida remota: save3.txt
"#
        );
        // Verifiquem el contingut del conf.toml
        let config_content = read_file_sync(conf_path.to_str().unwrap().to_string());
        assert_eq!(
            config_content,
            r#"[server]
url = "http://localhost:8000"
usuari = "admin"
contrasenya = "admin"

[[videojocs_habilitats.list]]
nom = "Joc"
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc"

[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save3.txt"
hash = "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4"
"#
        );
    }
    fn test_full_process_fase_5(clipg: &mut CliPG, joc_path: &PathBuf, conf_path: &PathBuf) -> PathBuf {
        /*
         * PRE:
         * El fitxer de configuracio te habilitat "Joc" amb 1 partida (save3.txt).
         * En local tenim el save3.txt a "Joc".
         * En remot tenim el save3.txt.
         * POST:
         * Eliminaem el fitxer conf.toml.
         * Creem el save4.txt a local i remot. Sincronitzem:
         * EL save3.txt ha de seguir igual al tindre els amteixos hash.
         * El save4.txt al tindre hash diferents en local i remot -> s'han de duplicar.
         * El que es renombra a "bck_" es el remot ja que te un timestamp més petit.
         */
        // Eliminem el fitxer conf.toml
        std::fs::remove_file(conf_path).unwrap();
        // Creem el save4.txt a local i remot
        let save4_path = joc_path.join("save4.txt");
        std::fs::write(&save4_path, "save 4").unwrap();
        clipg.api = Box::new(FakeAPI_fase5 {});
        let result = clipg.sync_all(false);
        assert_eq!(
            result,
            r#"
* Joc:
    ✔ Partida OK: save3.txt
    ⚠ Conflicte: save4.txt
"#
        );
        // Verifiquem el contingut del conf.toml
        let config_content = read_file_sync(conf_path.to_str().unwrap().to_string());
        assert!(config_content.contains(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save4.txt"
hash = "d623d87b3ef0b2b93f99637081a29ca70fa78c527f1f28a9242a7e93910fb194"
"#
        ));
        assert!(config_content.contains(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save3.txt"
hash = "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4"
"#
        ));
        // Verifiquem que s'ha creat el bck_save4.txt
        let mut fitxer_trobat: Option<PathBuf> = None;
        for entry in joc_path.read_dir().unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();
            if file_name.contains("bck_") {
                fitxer_trobat = Some(entry.path());
                break;
            }
        }
        assert!(fitxer_trobat.is_some(), "No s'ha trobat cap fitxer 'bck_'");
        let content = read_file_sync(fitxer_trobat.clone().unwrap().to_str().unwrap().to_string());
        assert_eq!(content, "save 4 alt 2");
        let back_name = fitxer_trobat.unwrap().clone();
        let expect1 = format!(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "{}"
hash = "12ee21760f19253fca62f5d0cdf480d1477c37300e56c2af141bcf35226a89b3"
"#,
            back_name.to_str().unwrap()
        );
        assert!(config_content.contains(expect1.as_str()));
        // si ho has de retornar:
        back_name
    }
    fn test_full_process_fase_6(clipg: &mut CliPG, joc_path: &PathBuf, conf_path: &PathBuf, back_file: &PathBuf) {
        /*
         * PRE:
         * El fitxer de configuracio te habilitat "Joc" amb 3 partides (save3.txt, save4.txt i "bck_save4.txt").
         * En local tenim el save3.txt, el save4.txt i el bck_save4.txt a "Joc".
         * En remot tenim el save3.txt i el save4.txt i el bck_save4.txt.
         * POST:
         * Fem una sincronitzacio que no fa res.
         * Verifiquem que efectivament no ha cambiat res en el conf.toml
         */
        // Verifiquem que a joc_path hi ha 3 fitxers: save3.txt, save4.txt i bck_save4.txt
        let bck_name = back_file.file_name().clone().unwrap();
        assert!(joc_path.join("save3.txt").exists(), "save3.txt no existeix a joc_path");
        assert!(joc_path.join("save4.txt").exists(), "save4.txt no existeix a joc_path");
        assert!(bck_name.to_str().unwrap().contains("bck_"));
        assert!(joc_path.join(bck_name.clone()).exists(), "bck_save4.txt no existeix a joc_path");
        // Sincronitzem sense haver canviat res per asegurar que no passa res
        clipg.api = Box::new(FakeAPI_fase6 {
            bck_name: bck_name.clone().to_str().unwrap().to_string(),
        });
        let result = clipg.sync_all(false);
        assert_eq!(
            result,
            format!(
                r#"
* Joc:
    ✔ Partida OK: {}
    ✔ Partida OK: save3.txt
    ✔ Partida OK: save4.txt
"#,
                bck_name.to_str().unwrap()
            ),
        );
        let config_content = read_file_sync(conf_path.to_str().unwrap().to_string());
        // El back_save4 original
        let expect1 = format!(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/{}"
hash = "12ee21760f19253fca62f5d0cdf480d1477c37300e56c2af141bcf35226a89b3"
"#,
            bck_name.to_str().unwrap()
        );
        assert!(config_content.contains(expect1.as_str()));
        // El save3.txt original
        assert!(config_content.contains(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save3.txt"
hash = "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4"
"#
        ));
        // El save4.txt original
        assert!(config_content.contains(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save4.txt"
hash = "d623d87b3ef0b2b93f99637081a29ca70fa78c527f1f28a9242a7e93910fb194"
"#
        ));
    }
    fn test_full_process_fase_7(clipg: &mut CliPG, joc_path: &PathBuf, conf_path: &PathBuf, back_file: &PathBuf) {
        /*
         * PRE:
         * El fitxer de configuracio te habilitat "Joc" amb 3 partides (save3.txt, save4.txt i "bck_save4.txt").
         * En local tenim el save3.txt, el save4.txt i el bck_save4.txt a "Joc".
         * En remot tenim el save3.txt i el save4.txt i el bck_save4.txt.
         * POST:
         * Modifiquem el save4.txt a local i remot. Sincronitzem:
         * EL save3.txt ha de seguir igual al tindre els amteixos hash.
         * EL bck_save4.txt ha de seguir igual al tindre els amteixos hash.
         * El save4.txt al tindre hash diferents en local i remot -> s'han de duplicar.
         * El que es renombra a "bck_" es el local ja que te un timestamp més petit.
         */
        let bck_name = back_file.file_name().clone().unwrap();
        // Modifiquem el save4.txt a local
        let save4_path = joc_path.join("save4.txt");
        std::fs::write(&save4_path, "44 save 44444").unwrap();
        // Modifiquem el save4.txt a remot: retorna un hash diferent i un timestamp molt gran
        clipg.api = Box::new(FakeAPI_fase7 {
            bck_name: bck_name.clone().to_str().unwrap().to_string(),
        });
        let result = clipg.sync_all(false);
        assert_eq!(
            result,
            format!(
                r#"
* Joc:
    ✔ Partida OK: {}
    ✔ Partida OK: save3.txt
    ⚠ Conflicte: save4.txt
"#,
                bck_name.to_str().unwrap()
            ),
        );
        // Verifiquem el contingut del conf.toml
        let config_content = read_file_sync(conf_path.to_str().unwrap().to_string());
        // El save3.txt original
        assert!(config_content.contains(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save3.txt"
hash = "fa7f7d6422a91afca0eedfc15dbb4f27286f14253624c5758314af03c786afc4"
"#
        ));
        // El bck_save4 original
        let expect1 = format!(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/{}"
hash = "12ee21760f19253fca62f5d0cdf480d1477c37300e56c2af141bcf35226a89b3"
"#,
            bck_name.to_str().unwrap()
        );
        assert!(config_content.contains(expect1.as_str()));
        // El save4.txt te un hash diferent, ja que es el nou
        let expect2 = format!(
            r#"[[videojocs_habilitats.list.partides_guardades]]
path = "/home/bcedu/Documents/Projectes/SincroPG/CliPG/tests/fixtures_cli_pg/test_sync/Joc/save4.txt"
hash = "d163dc016eb3e5640cd2b9e8a5364c35cf6faf3a9b7f7b0c53bdef37dca31d74"
"#,
        );
        assert!(config_content.contains(expect2.as_str()));
        // El altre fitxer bck_ te un altre hash
        let expect3 = format!(r#"hash = "39b8270f6c2bceb8824a7525d0cfc6daed0b521cb2a25b25932eedee6249e2f4""#);
        assert!(config_content.contains(expect3.as_str()));
    }
}
