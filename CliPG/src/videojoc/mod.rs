mod partida_guardada;

use partida_guardada::*;
use crate::ser_pg_api::SerPGAPI;
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::fs;
use std::fs::File;

pub struct Videojoc {
    nom: OsString,
    local_folder: PathBuf,
    partides_locals: Vec<PartidaGuarda>,
    partides_remotes: Vec<PartidaGuarda>,
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
                    PartidaGuarda::new(path.to_str().unwrap().to_string())
                );
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_videojoc_path_w40k() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/path a videojocs/Total War 40k/").to_str().unwrap().to_string()
    }

    fn get_videojoc_w40k() -> Videojoc {
        Videojoc::new(get_videojoc_path_w40k())
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
    fn test_load_partides_locals() {
        let mut v = get_videojoc_w40k();
        assert_eq!(v.partides_locals.len(), 0);
        v.load_partides_locals();
        assert_eq!(v.partides_locals.len(), 2);
        assert_eq!(v.partides_locals[1].nom, "save1.txt");
        assert_eq!(v.partides_locals[0].nom, "save2.txt");
    }
}