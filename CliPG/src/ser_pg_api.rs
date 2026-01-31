use eframe::egui::TextBuffer;
use crate::videojoc::partida_guardada::PartidaGuardada;

pub trait PartidesGuardadesAPI {
    fn get_partides_guardades(&self, nom_videojoc: String) -> Vec<PartidaGuardada>;
    fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada);
    fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String;
}

pub struct SerPGAPI {
    url: String,
    usuari: String,
    contrassenya: String
}

impl SerPGAPI {
    pub fn new(url: String, usuari: String, contrassenya: String) -> Self {
        SerPGAPI {
            url,
            usuari,
            contrassenya
        }
    }
}

impl PartidesGuardadesAPI for SerPGAPI{
    fn get_partides_guardades(&self, nom_videojoc: String) -> Vec<PartidaGuardada> {
        Vec::new()
    }
    fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {

    }
    fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
        "".to_string()
    }
}