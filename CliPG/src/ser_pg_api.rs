use crate::videojoc::partida_guardada::PartidaGuardada;

pub trait PartidesGuardadesAPI {
    fn get_partides_guardades(&self, nom: String) -> Vec<PartidaGuardada>;
    fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada);
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
    fn get_partides_guardades(&self, videojoc: String) -> Vec<PartidaGuardada> {
        // TODO
        Vec::new()
    }
    fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {

    }
}