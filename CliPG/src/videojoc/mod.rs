mod partida_guardada;

use partida_guardada::*;
use crate::ser_pg_api::SerPGAPI;
use std::path::{Path, PathBuf};
use std::ffi::OsString;

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
        // TODO: emplenar partides locals
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = Videojoc::new("/avi/pare/fill/fitxer.txt".to_string());
        assert_eq!(v.nom, "fitxer.txt");
        assert_eq!(v.local_folder.to_str().unwrap(), "/avi/pare/fill/fitxer.txt");
        // TODO: testejar partides locals emplenades
    }
}