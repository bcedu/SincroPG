use std::ffi::OsString;
use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::{Write, Read};
use normalized_hash::Hasher;
use filetime::FileTime;
use crate::ser_pg_api::PartidesGuardadesAPI;
use crate::videojoc::Videojoc;

pub struct PartidaGuardada {
    pub nom: OsString,
    pub path: PathBuf,
    pub timestamp: u32,
    pub hash: String
}

impl PartidaGuardada {
    pub fn new(path: String) -> Self {
        let full_path = PathBuf::from(path.clone());
        let hash: String;
        let timestamp: u32;
        if full_path.exists() {
            hash = Hasher::new().hash_file(&full_path, None::<PathBuf>);
            timestamp = FileTime::from_last_modification_time(&fs::metadata(path.clone()).unwrap()).nanoseconds();
        } else {
            hash = "".to_string();
            timestamp = 0;
        }
        PartidaGuardada {
            nom: full_path.file_name().unwrap_or_else(|| { panic!("La ruta {path} no és correcte!") }).to_os_string(),
            hash: hash,
            path: full_path,
            timestamp: timestamp,
        }
    }

    pub fn from_partida_guardada(partida_guardada: &PartidaGuardada) -> Self {
        PartidaGuardada {
            nom: partida_guardada.nom.clone(),
            hash: partida_guardada.hash.clone(),
            path: PathBuf::from(partida_guardada.path.to_str().unwrap()),
            timestamp: partida_guardada.timestamp,
        }
    }

    pub fn update_metadata(&mut self) {
        if self.path.exists() {
            self.hash = Hasher::new().hash_file(&self.path, None::<PathBuf>);
            self.timestamp = FileTime::from_last_modification_time(&fs::metadata(&self.path).unwrap()).nanoseconds();
        }
    }

    pub fn pujar_partida_guardada<A: PartidesGuardadesAPI>(&self, api: &A) {
        api.post_partida_guardada(&self);
    }

    pub fn descarregar_partida_guardada<A: PartidesGuardadesAPI>(&self, api: &A) {
        let contingut = api.get_partida_guardada(&self);
        self.write_file_sync(contingut.as_str());
    }

    pub fn duplicar_fitxer(&self, nou_nom: String) {
        let dir = self.path.parent().unwrap();
        let nou_path = dir.join(nou_nom);
        fs::copy(&self.path, &nou_path).unwrap();
    }

    pub fn write_file_sync(&self, content: &str) {
        let mut f = fs::File::create(self.path.as_path()).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.sync_all().unwrap();
        drop(f);
    }

    pub fn read_file_sync(&self) -> String {
        let mut contingut = String::new();
        let mut f = File::open(self.path.clone().as_path()).unwrap();
        f.read_to_string(&mut contingut).unwrap();
        drop(f);
        contingut
    }
}


#[cfg(test)]
pub mod tests {
    use crate::videojoc::tests::get_fake_server;
    use super::*;

    fn get_partida_path_ntw_s1() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_partida_guardada/path a videojocs/Napoleón TW HD/save1.txt").to_str().unwrap().to_string()
    }
    fn get_partida_path_w40k_s1() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_partida_guardada/path a videojocs/Total War 40k/save1.txt").to_str().unwrap().to_string()
    }
    fn get_partida_path_w40k_sremota() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_partida_guardada/path a videojocs/Total War 40k/save_remot.txt").to_str().unwrap().to_string()
    }
    pub fn get_partida_ntw_s1() -> PartidaGuardada {
        PartidaGuardada::new(get_partida_path_ntw_s1())
    }
    pub fn get_partida_w40k_s1() -> PartidaGuardada {
        PartidaGuardada::new(get_partida_path_w40k_s1())
    }
    pub fn get_partida_w40k_servidor_sremota() -> PartidaGuardada {
        PartidaGuardada::new(get_partida_path_w40k_sremota())
    }
    #[test]
    fn test_new() {
        let test_file_path = get_partida_path_w40k_s1();
        let pg = get_partida_w40k_s1();
        assert_eq!(pg.nom, "save1.txt");
        assert_eq!(pg.timestamp, 288718000);
        assert_eq!(pg.hash, "02d47a22e09f46731a58dbe7cb299c0315c6760aec7557e8ca6e87090fc85dfd");
        assert_eq!(pg.path.to_str().unwrap(), test_file_path);
    }
    #[test]
    fn test_from_partida_guardada() {
        let pg = get_partida_w40k_s1();
        let copia = PartidaGuardada::from_partida_guardada(&pg);
        assert_eq!(copia.nom, "save1.txt");
        assert_eq!(copia.timestamp, 288718000);
        assert_eq!(copia.hash, "02d47a22e09f46731a58dbe7cb299c0315c6760aec7557e8ca6e87090fc85dfd");
        assert_eq!(copia.path.to_str().unwrap(), pg.path.to_str().unwrap());

    }
    #[test]
    fn test_update_metadata() {
        let mut pg = get_partida_w40k_s1();
        let orig_timestamp = pg.timestamp;
        pg.timestamp = 0;
        assert_eq!(pg.timestamp, 0);
        pg.update_metadata();
        assert_eq!(pg.timestamp, orig_timestamp);
    }
    #[test]
    fn test_pujar_partida_guardada() {
        // No podem testejar res, nomes es crida la api
    }
    #[test]
    fn test_descarregar_partida_guardada_fitxer_cas_fitxer_nou() {
        // TEST 1: fitxer nou
        let partida_remota = get_partida_w40k_servidor_sremota();
        // Ens assegurem que no existeix abans (per si de cas algun test ha deixat els fixtures malament)
        if partida_remota.path.exists() {
            fs::remove_file(partida_remota.path).unwrap();
        }
        let partida_remota = get_partida_w40k_servidor_sremota();
        assert!(!partida_remota.path.exists());
        // Descarraguem la partida
        let api = get_fake_server();
        partida_remota.descarregar_partida_guardada(&api);
        // Ara ja hauria de existir
        assert!(partida_remota.path.exists());
        // Verifiquem el contingut
        let content = partida_remota.read_file_sync();
        assert_eq!(content, "Pastanaga Bullida\nPartida remota\n@#áçñÑ%");
        // Tornem a eliminar per deixarho com abans
        fs::remove_file(partida_remota.path).unwrap();
    }
    #[test]
    fn test_descarregar_partida_guardada_fitxer_cas_actulitzar_fitxer() {
        // TEST 2: actualitza fitxer existent
        let partida_ja_existent = get_partida_ntw_s1();
        // Llegim el contingut original i verifiquem que es el que esperem
        let content = partida_ja_existent.read_file_sync();
        assert_eq!(content, "Soc una partida guardada del Napoleon");
        // Descarreguem la nova verZio que hi ha al servidor
        let api = get_fake_server();
        partida_ja_existent.descarregar_partida_guardada(&api);
        assert!(partida_ja_existent.path.exists());
        let content_nou = partida_ja_existent.read_file_sync();
        assert_eq!(content_nou, "Contingut @ctualitzat!");
        // Restaurem el contingut original
        partida_ja_existent.write_file_sync(content.clone().as_str());
        // Ens assegurem que s'hagi restaurat be
        let content2 = partida_ja_existent.read_file_sync();
        assert_eq!(content2, content);
    }
    #[test]
    fn test_duplicar_fitxer() {
        let partida_ja_existent = get_partida_ntw_s1();
        let nou_nom = "pastanaga";
        let nou_path = format!("{}/{}", partida_ja_existent.path.parent().unwrap().to_str().unwrap(), nou_nom);
        assert!(!PathBuf::from(&nou_path).exists());
        partida_ja_existent.duplicar_fitxer(nou_nom.to_string());
        assert!(PathBuf::from(&nou_path).exists());
        fs::remove_file(nou_path).unwrap();
    }
}