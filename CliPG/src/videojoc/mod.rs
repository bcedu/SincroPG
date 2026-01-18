mod partida_guardada;
use partida_guardada::*;

pub struct Videojoc {
    nom: String,
    local_folder: String,
    partides_locals: Vec<PartidaGuarda>,
    partides_remotes: Vec<PartidaGuarda>,
}
