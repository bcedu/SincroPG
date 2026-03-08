pub mod partida_guardada;
use crate::cli_pg::PartidaGuardadaConfig;
use crate::pg_api::{PartidesGuardadesAPI, PgAPI};
use chrono::Local;
use partida_guardada::*;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub struct Videojoc {
    pub nom: OsString,
    pub local_folder: PathBuf,
    pub partides_locals: Vec<PartidaGuardada>,
    pub partides_remotes: Vec<PartidaGuardada>,
    pub partides_guardades: HashMap<String, PartidaGuardadaConfig>,
}
impl Videojoc {
    pub fn new(path: String) -> Self {
        let local_folder = PathBuf::from(path.clone());
        let nom = local_folder.file_name().unwrap_or_else(|| panic!("La ruta {path} no és correcte!")).to_os_string();
        Videojoc {
            nom,
            local_folder,
            partides_locals: Vec::new(),
            partides_remotes: Vec::new(),
            partides_guardades: HashMap::new(),
        }
    }
    pub fn from(videojoc: &Videojoc) -> Self {
        Videojoc::new(videojoc.local_folder.to_str().unwrap().to_string()).with_partides_guardades(videojoc.partides_guardades.clone())
    }
    pub fn with_nom(mut self, nom: String) -> Self {
        self.nom = OsString::from(nom);
        self
    }
    pub fn with_partides_guardades(mut self, partides_guardades: HashMap<String, PartidaGuardadaConfig>) -> Self {
        self.partides_guardades = partides_guardades;
        self
    }
    pub fn with_partides_guardades_list(mut self, partides_guardades: &Vec<PartidaGuardadaConfig>) -> Self {
        self.partides_guardades.clear();
        for p in partides_guardades.iter() {
            self.partides_guardades.insert(p.path.clone(), p.clone());
        }
        self
    }
    pub fn load_partides_locals(&mut self) {
        self.partides_locals.clear();

        let Ok(entries) = self.local_folder.read_dir() else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                self.partides_locals.push(PartidaGuardada::new(path.to_str().unwrap().to_string()).with_videojoc(self));
            }
        }
    }
    pub fn fetch_partides_remotes<A: PartidesGuardadesAPI>(&mut self, api: &A) {
        self.partides_remotes.clear();
        for partida_remota in api.get_partides_guardades(&self) {
            self.partides_remotes.push(partida_remota)
        }
    }
    pub fn sync<A: PartidesGuardadesAPI>(&mut self, api: &A, test_mode: bool) -> String {
        self.load_partides_locals();
        self.fetch_partides_remotes(api);
        let locals: HashMap<_, _> = self.partides_locals.iter().map(|p| (p.nom.to_str().unwrap().to_string(), p)).collect();
        let remotes: HashMap<_, _> = self.partides_remotes.iter().map(|p| (p.nom.to_str().unwrap().to_string(), p)).collect();
        let guardades: HashMap<_, _> = self
            .partides_guardades
            .iter()
            .map(|(k, v)| (PathBuf::from(k).file_name().unwrap().to_str().unwrap().to_string(), v))
            .collect();
        let mut noms: Vec<_> = locals.keys().chain(remotes.keys()).cloned().collect();
        noms.sort();
        noms.dedup();
        let mut resultat = String::new();
        for nom in noms {
            let local = locals.get(&nom);
            let remote = remotes.get(&nom);
            let last_sync_hash = guardades.get(&nom).map(|p| p.hash.as_str()).unwrap_or("");
            let msg = match (local, remote) {
                // només local
                (Some(local), None) => {
                    if last_sync_hash == local.hash {
                        if !test_mode {
                            local.eliminar_partida_guardada();
                        }
                        format!("❌ Eliminar local: {}", nom)
                    } else {
                        if !test_mode {
                            local.pujar_partida_guardada(api);
                        }
                        format!("⬆️ Pujar partida local: {}", nom)
                    }
                }
                // només remot
                (None, Some(remote)) => {
                    if last_sync_hash == remote.hash {
                        if !test_mode {
                            api.delete_partida_guardada(&remote);
                        }
                        format!("❌ Eliminar remot: {}", nom)
                    } else {
                        if !test_mode {
                            remote.descarregar_partida_guardada(api);
                        }
                        format!("⬇️ Descarregar partida remota: {}", nom)
                    }
                }
                // existeixen tots dos
                (Some(local), Some(remote)) => {
                    if local.hash == remote.hash {
                        format!("✔️ Partida OK: {}", nom)
                    } else if local.hash == last_sync_hash {
                        if !test_mode {
                            remote.descarregar_partida_guardada(api);
                        }
                        format!("⬇️ Descarregar (remot modificat): {}", nom)
                    } else if remote.hash == last_sync_hash {
                        if !test_mode {
                            local.pujar_partida_guardada(api);
                        }
                        format!("⬆️ Pujar partida local (local modificat): {}", nom)
                    } else {
                        if !test_mode {
                            self.resoldre_conflicte(local, remote, api);
                        }
                        format!("⚠️ Conflicte: {}", nom)
                    }
                }
                _ => continue,
            };
            println!("{}", msg);
            resultat.push_str(&msg);
            resultat.push('\n');
        }
        self.actualitzar_partides_guardades();
        resultat
    }
    pub fn actualitzar_partides_guardades(&mut self) {
        // Actualitzem les partides que han quedat al local per obtenir els seus hash i actualitzar el partides_guardades
        self.load_partides_locals();
        self.partides_guardades = HashMap::new();
        for partida in &self.partides_locals {
            let key = partida.path.to_str().unwrap().to_string();
            self.partides_guardades.insert(
                key.clone(),
                PartidaGuardadaConfig {
                    path: key,
                    hash: partida.hash.clone(),
                },
            );
        }
    }
    pub fn resoldre_conflicte<A: PartidesGuardadesAPI>(&self, local: &PartidaGuardada, remot: &PartidaGuardada, api: &A) {
        // Donarem prioritat al que tingui el timestamp mes recent. El que tingui el timestamp
        // mes antic es renombara posant a davant del nom "bck_yyyymmddhhss_"
        let nou_nom = format!("bck_{0}_{1}", Local::now().format("%Y%m%d%H%M%S"), remot.nom.to_str().unwrap());
        if local.timestamp >= remot.timestamp {
            // Pujem la partida remot pero renombrada al servidor;
            let mut remot = PartidaGuardada::from_partida_guardada(remot);
            remot.nom = OsString::from(nou_nom);
            api.post_partida_guardada(&remot);
            // Pujem la partida local al servidor (aixo sobreescriu la que hi havia)
            api.post_partida_guardada(&local);
        } else {
            // Creem una nova partida local amb el nom nou
            local.duplicar_fitxer(nou_nom);
            // Descarreguem la remota per actualitzar la original
            remot.descarregar_partida_guardada(api);
        }
    }
    pub fn get_partides_guardades_list(&self) -> Vec<PartidaGuardadaConfig> {
        self.partides_guardades.values().cloned().collect()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::videojoc::partida_guardada::tests::get_partida_ntw_s1;
    pub struct FakeAPI;
    impl PartidesGuardadesAPI for FakeAPI {
        fn probar_connexio(&self) -> bool {
            true
        }
        fn get_videojocs(&self) -> Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: &Videojoc) -> Vec<PartidaGuardada> {
            let mut v = Vec::new();
            let p1 = PartidaGuardada {
                videojoc: "".to_string(),
                nom: OsString::from("save1.txt"),
                path: PathBuf::new(),
                timestamp: 245528886,
                hash: "02d47a22e09f46731a58dbe7cb299c0315c6760aec7557e8ca6e87090fc85dfd".to_string(),
            };
            let p2 = PartidaGuardada {
                videojoc: "".to_string(),
                nom: OsString::from("save_test_2"),
                path: PathBuf::new(),
                timestamp: 0,
                hash: "1".to_string(),
            };
            let p3 = PartidaGuardada {
                videojoc: "".to_string(),
                nom: OsString::from("save3.txt"),
                path: PathBuf::new(),
                timestamp: 0,
                hash: "2".to_string(),
            };
            let p4 = PartidaGuardada {
                videojoc: "".to_string(),
                nom: OsString::from("save4.txt"),
                path: PathBuf::new(),
                timestamp: 0,
                hash: "patata".to_string(),
            };
            let p5 = PartidaGuardada {
                videojoc: "".to_string(),
                nom: OsString::from("save_remote_modified.txt"),
                path: PathBuf::new(),
                timestamp: 0,
                hash: "nou_hash".to_string(),
            };
            let p6 = PartidaGuardada {
                videojoc: "".to_string(),
                nom: OsString::from("save_deleted_local.txt"),
                path: PathBuf::new(),
                timestamp: 0,
                hash: "xyz".to_string(),
            };
            v.push(p1);
            v.push(p2);
            v.push(p3);
            v.push(p4);
            v.push(p5);
            v.push(p6);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn delete_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            if partida_guardada.nom == "save_remot.txt" {
                "Pastanaga Bullida\nPartida remota\n@#áçñÑ%".to_string()
            } else {
                "Contingut @ctualitzat!".to_string()
            }
        }
    }
    fn get_videojoc_path_w40k() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures_videojoc/path a videojocs/Total War 40k/")
            .to_str()
            .unwrap()
            .to_string()
    }
    fn get_videojoc_w40k() -> Videojoc {
        Videojoc::new(get_videojoc_path_w40k())
    }
    pub fn get_fake_api() -> FakeAPI {
        FakeAPI
    }
    #[test]
    fn test_new() {
        let v = get_videojoc_w40k();
        assert_eq!(v.nom, "Total War 40k");
        assert_eq!(v.local_folder.to_str().unwrap(), get_videojoc_path_w40k());
    }
    #[test]
    fn test_with_nom() {
        let v = get_videojoc_w40k().with_nom("Pastanaga bullida".to_string());
        assert_eq!(v.nom, "Pastanaga bullida");
        assert_eq!(v.local_folder.to_str().unwrap(), get_videojoc_path_w40k());
    }
    #[test]
    fn test_from() {
        let v = get_videojoc_w40k();
        let v2 = Videojoc::from(&v);
        assert_eq!(v2.nom, "Total War 40k");
        assert_eq!(v2.local_folder.to_str().unwrap(), get_videojoc_path_w40k());
    }
    #[test]
    fn test_load_partides_locals() {
        let mut v = get_videojoc_w40k();
        assert_eq!(v.partides_locals.len(), 0);
        v.load_partides_locals();
        assert_eq!(v.partides_locals.len(), 6);
        assert_eq!(v.partides_locals[5].nom, "save_remote_modified.txt");
        assert_eq!(v.partides_locals[4].nom, "save_deleted_remote.txt");
        assert_eq!(v.partides_locals[3].nom, "save3.txt");
        assert_eq!(v.partides_locals[2].nom, "save1.txt");
        assert_eq!(v.partides_locals[1].nom, "save4.txt");
        assert_eq!(v.partides_locals[0].nom, "save2.txt");
    }
    #[test]
    fn test_fetch_partides_remotes() {
        let mut v = get_videojoc_w40k();
        let s = get_fake_api();
        v.fetch_partides_remotes(&s);
        assert_eq!(v.partides_remotes.len(), 6);
        assert_eq!(v.partides_remotes[0].nom, "save1.txt");
        assert_eq!(v.partides_remotes[1].nom, "save_test_2");
        assert_eq!(v.partides_remotes[2].nom, "save3.txt");
        assert_eq!(v.partides_remotes[3].nom, "save4.txt");
    }
    #[test]
    fn test_resoldre_conflicte() {
        let mut local = get_partida_ntw_s1();
        let mut remot = get_partida_ntw_s1();
        let videojoc = get_videojoc_w40k();
        let api = get_fake_api();
        let contingut_original = local.read_file_sync();
        // Cas en que la local es la mes recent. No s'ha de crear cap fitxer local nou
        local.timestamp = 1;
        remot.timestamp = 0;
        let nfitxers_abans = fs::read_dir(local.path.parent().unwrap()).iter().count();
        assert_eq!(nfitxers_abans, 1);
        videojoc.resoldre_conflicte(&local, &remot, &api);
        let nfitxers_despres = fs::read_dir(local.path.parent().unwrap()).iter().count();
        assert_eq!(nfitxers_despres, 1);
        // Cas en que el remot es mes recent. Es farà una copia
        local.timestamp = 0;
        remot.timestamp = 1;
        let nfitxers_abans = fs::read_dir(local.path.parent().unwrap()).iter().count();
        assert_eq!(nfitxers_abans, 1);
        videojoc.resoldre_conflicte(&local, &remot, &api);
        let mut nfitxers_despres = 0;
        for entry in fs::read_dir(local.path.parent().unwrap()).unwrap().flatten() {
            nfitxers_despres += 1;
            let path = entry.path();
            if path.file_name().unwrap() != local.nom.to_str().unwrap() {
                // Haurien de tindre el mateix contingut
                let content2 = fs::read_to_string(&path).unwrap();
                let content1 = fs::read_to_string(&local.path).unwrap();
                assert_ne!(content1, content2);
                assert_eq!(content2, contingut_original);
                // Aprofitem per eliminarlo
                fs::remove_file(path).unwrap();
            }
        }
        assert_eq!(nfitxers_despres, 2);
        local.write_file_sync(contingut_original.as_str());
    }
    #[test]
    fn test_sync() {
        let mut partides_guardades = Vec::new();
        // remot eliminat
        partides_guardades.push(PartidaGuardadaConfig {
            path: format!("{}save_deleted_remote.txt", get_videojoc_path_w40k()),
            hash: "dd4857f6cd556600cb629caf0acdcd94666543dfdb1d1001cac26b7f12e9b6ca".to_string(),
        });
        // local eliminat
        partides_guardades.push(PartidaGuardadaConfig {
            path: format!("{}save_deleted_local.txt", get_videojoc_path_w40k()),
            hash: "xyz".to_string(),
        });
        // remot modificat
        partides_guardades.push(PartidaGuardadaConfig {
            path: format!("{}save_remote_modified.txt", get_videojoc_path_w40k()),
            hash: "c0badec7d321935a94a42b9601512ebf655c64a577dd7711255fac1b112ac795".to_string(),
        });
        // local modificat
        partides_guardades.push(PartidaGuardadaConfig {
            path: format!("{}save4.txt", get_videojoc_path_w40k()),
            hash: "patata".to_string(),
        });
        let mut v = get_videojoc_w40k().with_partides_guardades_list(&partides_guardades);
        let resultat = v.sync(&get_fake_api(), true);
        let resultat_esperat = "✔️ Partida OK: save1.txt
⬆️ Pujar partida local: save2.txt
⚠️ Conflicte: save3.txt
⬆️ Pujar partida local (local modificat): save4.txt
❌ Eliminar remot: save_deleted_local.txt
❌ Eliminar local: save_deleted_remote.txt
⬇️ Descarregar (remot modificat): save_remote_modified.txt
⬇️ Descarregar partida remota: save_test_2
";
        assert_eq!(resultat_esperat, resultat);
    }
}
