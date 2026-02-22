pub mod partida_guardada;

use std::collections::HashMap;
use partida_guardada::*;
use crate::pg_api::{PartidesGuardadesAPI, PgAPI};
use std::path::PathBuf;
use std::ffi::OsString;
use std::fs;
use chrono::Local;

pub struct Videojoc {
    pub nom: OsString,
    local_folder: PathBuf,
    partides_locals: Vec<PartidaGuardada>,
    partides_remotes: Vec<PartidaGuardada>,
}

impl Videojoc {
    pub fn new(path: String) -> Self {
        let local_folder = PathBuf::from(path.clone());
        let nom = local_folder.file_name().unwrap_or_else(|| {
            panic!("La ruta {path} no és correcte!")
        }).to_os_string();
        Videojoc {
            nom,
            local_folder,
            partides_locals: Vec::new(),
            partides_remotes: Vec::new()
        }
    }
    pub fn from(videojoc: &Videojoc) -> Self {
        Videojoc::new(videojoc.local_folder.to_str().unwrap().to_string())
    }
    pub fn with_nom(mut self, nom: String) -> Self {
        self.nom = OsString::from(nom);
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
                self.partides_locals.push(
                    PartidaGuardada::new(path.to_str().unwrap().to_string()).with_videojoc(self)
                );
            }
        }
    }
    pub fn fetch_partides_remotes<A: PartidesGuardadesAPI>(&mut self, api: &A) {
        self.partides_remotes.clear();
        for partida_remota in api.get_partides_guardades(self.nom.to_str().unwrap().to_string()) {
            self.partides_remotes.push(partida_remota)
        }
    }
    pub fn sync<A: PartidesGuardadesAPI>(&mut self, api: &A, test_mode: bool) -> String {
        // Assegurem dades actualitzades
        self.load_partides_locals();
        self.fetch_partides_remotes(api);
        // Indexem per nom
        let locals: HashMap<String, &PartidaGuardada> = self
            .partides_locals
            .iter()
            .map(|p| (p.nom.to_str().unwrap().to_string(), p))
            .collect();
        let remotes: HashMap<String, &PartidaGuardada> = self
            .partides_remotes
            .iter()
            .map(|p| (p.nom.to_str().unwrap().to_string(), p))
            .collect();
        // Unió de totes les partides
        let mut noms: Vec<String> = locals
            .keys()
            .chain(remotes.keys())
            .cloned()
            .collect();
        // Eliminem duplicats
        noms.sort();
        noms.dedup();
        // Revisem cada partida guardada i fem la sincronitzacio
        let mut resultat = String::new();
        for nom in noms {
            let aux;
            match (locals.get(&nom), remotes.get(&nom)) {
                // ⬆️ Només local
                (Some(local), None) => {
                    aux = format!("⬆️ Pujar partida local: {}", local.nom.to_str().unwrap().to_string());
                    println!("{}", aux);
                    if !test_mode {
                        local.pujar_partida_guardada(api);
                    }
                }
                // ⬇️ Només servidor
                (None, Some(remote)) => {
                    aux = format!("⬇️ Descarregar partida remota: {}", remote.nom.to_str().unwrap().to_string());
                    println!("{}", aux);
                    if !test_mode {
                        remote.descarregar_partida_guardada(api);
                    }
                }
                // ✔️ Existeixen les dues
                (Some(local), Some(remote)) => {
                    if local.hash == remote.hash {
                        // Iguals → no fer res
                        aux = format!("✔️ Partida OK: {}", local.nom.to_str().unwrap().to_string());
                        println!("{}", aux);
                    } else {
                        // Diferents → conflicte
                        aux = format!("⚠️ Conflicte: {}", local.nom.to_str().unwrap().to_string());
                        println!("{}", aux);
                        if !test_mode {
                            self.resoldre_conflicte(local, remote, api);
                        }
                    }
                }
                // Cas impossible
                (None, None) => {
                    aux = String::new();
                }
            }
            resultat.push_str(aux.as_str());
            resultat.push('\n');
        }
        resultat
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
        }  else {
            // Creem una nova partida local amb el nom nou
            local.duplicar_fitxer(nou_nom);
            // Descarreguem la remota per actualitzar la original
            remot.descarregar_partida_guardada(api);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::videojoc::partida_guardada::tests::get_partida_ntw_s1;
    pub struct FakeAPI;
    impl PartidesGuardadesAPI for FakeAPI {
        fn probar_connexio(&self) -> bool{
            true
        }
        fn get_videojocs(&self) ->Vec<String> {
            Vec::new()
        }
        fn get_partides_guardades(&self, _: String) -> Vec<PartidaGuardada> {
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
                hash: "".to_string(),
            };
            let p3 = PartidaGuardada {
                videojoc: "".to_string(),
                nom: OsString::from("save3.txt"),
                path: PathBuf::new(),
                timestamp: 0,
                hash: "".to_string(),
            };
            v.push(p1);
            v.push(p2);
            v.push(p3);
            v
        }
        fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {}
        fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
            if partida_guardada.nom == "save_remot.txt" {
                "Pastanaga Bullida\nPartida remota\n@#áçñÑ%".to_string()
            } else {
                "Contingut @ctualitzat!".to_string()
            }
        }
    }
    fn get_videojoc_path_w40k() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_videojoc/path a videojocs/Total War 40k/").to_str().unwrap().to_string()
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
        assert_eq!(v.partides_locals.len(), 3);
        assert_eq!(v.partides_locals[2].nom, "save3.txt");
        assert_eq!(v.partides_locals[1].nom, "save1.txt");
        assert_eq!(v.partides_locals[0].nom, "save2.txt");
    }
    #[test]
    fn test_fetch_partides_remotes() {
        let mut v = get_videojoc_w40k();
        let s = get_fake_api();
        v.fetch_partides_remotes(&s);
        assert_eq!(v.partides_remotes.len(), 3);
        assert_eq!(v.partides_remotes[0].nom, "save1.txt");
        assert_eq!(v.partides_remotes[1].nom, "save_test_2");
        assert_eq!(v.partides_remotes[2].nom, "save3.txt");
    }
    #[test]
    fn test_sync() {
        let mut v = get_videojoc_w40k();
        let resultat = v.sync(&get_fake_api(), true);
        let resultat_esperat = "✔️ Partida OK: save1.txt
⬆️ Pujar partida local: save2.txt
⚠️ Conflicte: save3.txt
⬇️ Descarregar partida remota: save_test_2\n";
        assert_eq!(resultat_esperat, resultat);
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
}