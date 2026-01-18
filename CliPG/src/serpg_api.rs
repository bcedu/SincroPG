
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