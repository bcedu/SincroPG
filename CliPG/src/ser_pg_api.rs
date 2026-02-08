use eframe::egui::TextBuffer;
use crate::videojoc::partida_guardada::PartidaGuardada;
use crate::videojoc::Videojoc;
use serde::Deserialize;
use urlencoding::encode;

pub trait PartidesGuardadesAPI {
    fn probar_connexio(&self) -> bool;
    // GET /api/v1/test
    fn get_videojocs(&self) -> Vec<String>;
    // GET /api/v1/videojocs
    fn get_partides_guardades(&self, nom_videojoc: String) -> Vec<PartidaGuardada>;
    // GET /api/v1/videojocs/{videojoc_id}/partides
    fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada);
    // POST /api/v1/videojocs/{videojoc_id}/partides
    fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String;
    // GET /api/v1/videojocs/{videojoc_id}/partides/{partida_id}/contingut
}

pub struct SerPGAPI {
    url: String,
    usuari: String,
    contrassenya: String,
    client: reqwest::blocking::Client,
}

#[derive(Debug, Deserialize)]
struct VideojocAPI {
    id: String,
    nom: String,
}
#[derive(Debug, Deserialize)]
struct PartidaGuardadaAPI {
    nom: String,
    hash: String
}

impl SerPGAPI {
    pub fn new(url: String, usuari: String, contrassenya: String) -> Self {
        SerPGAPI {
            url,
            usuari,
            contrassenya,
            client: reqwest::blocking::Client::new()
        }
    }

    fn make_request(&self, endpoint: &str) -> reqwest::blocking::Response {
        let encoded = encode(&endpoint);
        let request_url = format!("{url}/api/v1/{encoded}", url = self.url.clone());
        println!("REQUEST {}", request_url);
        let response = self.client
            .get(request_url)
            .basic_auth(self.usuari.clone(), Some(self.contrassenya.clone()))
            .send();
        response.unwrap()
    }
}

impl PartidesGuardadesAPI for SerPGAPI{

    fn probar_connexio(&self) -> bool {
        // GET /api/v1/test
        self.make_request("test").status().is_success()
    }
    fn get_videojocs(&self) -> Vec<String> {
        // GET /api/v1/videojocs
        let mut videojocs = Vec::new();
        let response = self.make_request("videojocs");
        let videjocs_server:  Vec<VideojocAPI>  = response.json().unwrap();
        for v in videjocs_server {
            videojocs.push(v.nom);
        }
        videojocs
    }
    fn get_partides_guardades(&self, nom_videojoc: String) -> Vec<PartidaGuardada> {
        // GET /api/v1/videojocs/{videojoc_id}/partides
        let mut partides = Vec::new();
        let request_url = format!("videojocs/{nom_videojoc}/partides");
        let response = self.make_request(request_url.as_str());
        let partides_server:  Vec<PartidaGuardadaAPI>  = response.json().unwrap();
        for p in partides_server {
            let pg = PartidaGuardada::new(p.nom).with_hash(p.hash);
            partides.push(pg);
        }
        partides
    }
    fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {

    }
    fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
        "".to_string()
    }
}

#[cfg(test)]
pub mod tests {
    use eframe::egui::TextBuffer;
    use mockito::{Mock, Server};
    use urlencoding::encode;
    use crate::ser_pg_api::{PartidesGuardadesAPI, SerPGAPI};

    fn get_pg_api(url: String) -> SerPGAPI {
        SerPGAPI::new(url, String::from("admin"), String::from("pastanagabullida"))
    }
    fn get_fake_server(url: &str) -> (mockito::ServerGuard, Mock) {
        let mut server = Server::new();
        let encoded = encode(&url);
        let _mock = server
            .mock("GET", format!("/api/v1/{encoded}").as_str())
            .match_header("authorization", "Basic YWRtaW46cGFzdGFuYWdhYnVsbGlkYQ==")
            .with_status(200).expect(1);
        (server, _mock)
    }
    fn setup_fake_server_probar_connexio() -> mockito::ServerGuard {
        let (server, _mock) = get_fake_server("test");
        _mock.create();
        server
    }
    fn setup_fake_server_get_videojocs() -> mockito::ServerGuard {
        let (server, _mock) = get_fake_server("videojocs");
        _mock.with_header("content-type", "application/json").with_body(r#"[
            { "id": "m", "nom": "Mount & blade Warband 2" },
            { "id": "n", "nom": "Napoleón TW HD" },
            { "id": "t", "nom": "Total War 40k" }
        ]"#).create();
        server
    }
    fn setup_fake_server_get_partides_guardades(nom_videojoc: String) -> mockito::ServerGuard {
        let (server, _mock) = get_fake_server(format!("videojocs/{nom_videojoc}/partides").as_str());
        _mock.with_header("content-type", "application/json").with_body(r#"[
            { "nom": "save1.txt", "hash": "patata" },
            { "nom": "save", "hash": "pastanaga" },
            { "nom": "1234@.xml,1", "hash": "@@" }
        ]"#).create();
        server
    }
    #[test]
    fn test_probar_connexio() {
        let server = setup_fake_server_probar_connexio();
        let pgapi = get_pg_api(server.url().clone());
        let check = pgapi.probar_connexio();
        assert!(check);
    }
    #[test]
    fn test_get_videojocs() {
        let server = setup_fake_server_get_videojocs();
        let pgapi = get_pg_api(server.url().clone());
        let videojocs = pgapi.get_videojocs();
        assert_eq!(videojocs.len(), 3);
        assert_eq!(videojocs.get(0).unwrap().to_string(), "Mount & blade Warband 2".to_string());
        assert_eq!(videojocs.get(1).unwrap().to_string(), "Napoleón TW HD".to_string());
        assert_eq!(videojocs.get(2).unwrap().to_string(), "Total War 40k".to_string());
    }
    #[test]
    fn test_get_partides_guardades() {
        let nom_videojoc = "Napoleón TW HD";
        let server = setup_fake_server_get_partides_guardades(nom_videojoc.to_string());
        let pgapi = get_pg_api(server.url().clone());
        let videojocs = pgapi.get_partides_guardades(nom_videojoc.to_string());
        assert_eq!(videojocs.len(), 3);
        assert_eq!(videojocs.get(0).unwrap().nom.to_str().unwrap().to_string(), "save1.txt".to_string());
        assert_eq!(videojocs.get(0).unwrap().hash.to_string(), "patata".to_string());
        assert_eq!(videojocs.get(1).unwrap().nom.to_str().unwrap().to_string(), "save".to_string());
        assert_eq!(videojocs.get(1).unwrap().hash.to_string(), "pastanaga".to_string());
        assert_eq!(videojocs.get(2).unwrap().nom.to_str().unwrap().to_string(), "1234@.xml,1".to_string());
        assert_eq!(videojocs.get(2).unwrap().hash.to_string(), "@@".to_string());
    }
}