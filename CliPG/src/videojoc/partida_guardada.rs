use std::ffi::OsString;
use std::path::PathBuf;
use std::fs;
use normalized_hash::Hasher;
use filetime::FileTime;
use crate::videojoc::Videojoc;

pub struct PartidaGuarda {
    pub nom: OsString,
    pub path: PathBuf,
    pub timestamp: u32,
    pub hash: String
}

impl PartidaGuarda {
    pub fn new(path: String) -> Self {
        let full_path = PathBuf::from(path.clone());
        PartidaGuarda {
            nom: full_path.file_name().unwrap_or_else(|| { panic!("La ruta {path} no és correcte!") }).to_os_string(),
            hash: Hasher::new().hash_file(&full_path, None::<PathBuf>),
            path: full_path,
            timestamp: FileTime::from_last_modification_time(&fs::metadata(path).unwrap()).nanoseconds(),
        }
    }

    pub fn update_metadata(&mut self) {
        self.hash = Hasher::new().hash_file(&self.path, None::<PathBuf>);
        self.timestamp = FileTime::from_last_modification_time(&fs::metadata(&self.path).unwrap()).nanoseconds();
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    fn get_partida_path_w40k_s1() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/path a videojocs/Total War 40k/save1.txt").to_str().unwrap().to_string()
    }

    fn get_partida_w40k_s1() -> PartidaGuarda {
        PartidaGuarda::new(get_partida_path_w40k_s1())
    }
    #[test]
    fn test_new() {
        let test_file_path = get_partida_path_w40k_s1();
        let pg = get_partida_w40k_s1();
        assert_eq!(pg.nom, "save1.txt");
        assert_eq!(pg.timestamp, 245528886);
        assert_eq!(pg.hash, "02d47a22e09f46731a58dbe7cb299c0315c6760aec7557e8ca6e87090fc85dfd");
        assert_eq!(pg.path.to_str().unwrap(), test_file_path);
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
}