use eframe::egui::TextBuffer;
use reqwest::Response;
use crate::videojoc::partida_guardada::PartidaGuardada;
use crate::videojoc::Videojoc;
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Deserialize, Serialize)]
struct PartidaGuardadaContingutAPI {
    nom: String,
    contingut: String
}
#[derive(Debug)]
enum RTYPE {GET, POST}

impl SerPGAPI {
    pub fn new(url: String, usuari: String, contrassenya: String) -> Self {
        SerPGAPI {
            url,
            usuari,
            contrassenya,
            client: reqwest::blocking::Client::new()
        }
    }
    fn make_get_request(&self, endpoint: &str) -> reqwest::blocking::Response {
        self.make_request(RTYPE::GET, endpoint, None)
    }
    fn make_post_request(&self, endpoint: &str, body: PartidaGuardadaContingutAPI) -> reqwest::blocking::Response {
        self.make_request(RTYPE::POST, endpoint, Some(body))
    }
    fn make_request(&self, rtype: RTYPE, endpoint: &str, body: Option<PartidaGuardadaContingutAPI>) -> reqwest::blocking::Response {
        let encoded = encode(&endpoint);
        let request_url = format!("{url}/api/v1/{encoded}", url = self.url.clone());
        println!("REQUEST {:?} {request_url}", rtype);
        let response;
        match rtype {
            RTYPE::GET => {
                response = self.client
                    .get(request_url)
                    .basic_auth(self.usuari.clone(), Some(self.contrassenya.clone()))
                    .send();
            }
            RTYPE::POST => {
                match body {
                    Some(body) => {
                        response = self.client
                            .post(request_url)
                            .basic_auth(self.usuari.clone(), Some(self.contrassenya.clone()))
                            .json(&body)
                            .send();
                    }
                    None => {
                        response = self.client
                            .post(request_url)
                            .basic_auth(self.usuari.clone(), Some(self.contrassenya.clone()))
                            .send();
                    }
                }
            }
        }
        response.unwrap()
    }
}

impl PartidesGuardadesAPI for SerPGAPI{

    fn probar_connexio(&self) -> bool {
        // GET /api/v1/test
        self.make_get_request("test").status().is_success()
    }
    fn get_videojocs(&self) -> Vec<String> {
        // GET /api/v1/videojocs
        let mut videojocs = Vec::new();
        let response = self.make_get_request("videojocs");
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
        let response = self.make_get_request(request_url.as_str());
        let partides_server:  Vec<PartidaGuardadaAPI>  = response.json().unwrap();
        for p in partides_server {
            let pg = PartidaGuardada::new(p.nom).with_hash(p.hash);
            partides.push(pg);
        }
        partides
    }
    fn post_partida_guardada(&self, partida_guardada: &PartidaGuardada) {
        // POST /api/v1/videojocs/{videojoc_id}/partides
        if partida_guardada.videojoc.is_empty() {
            panic!("No es pot pujar la partida {} si no te el videjoc definit.", partida_guardada.nom.to_str().unwrap());
        }
        let request_url = format!("videojocs/{}/partides", partida_guardada.videojoc);
        let content = partida_guardada.read_file_sync();
        let pa = PartidaGuardadaContingutAPI{
            nom: partida_guardada.nom.to_str().unwrap().to_string(),
            contingut: content
        };
        self.make_post_request(request_url.as_str(), pa);
    }
    fn get_partida_guardada(&self, partida_guardada: &PartidaGuardada) -> String {
        "".to_string()
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;
    use eframe::egui::TextBuffer;
    use mockito::{Mock, Server};
    use urlencoding::encode;
    use crate::ser_pg_api::{PartidesGuardadesAPI, SerPGAPI};
    use crate::videojoc::partida_guardada::PartidaGuardada;

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
    fn setup_fake_server_post_partida_guardada(nom_videojoc: String) -> (mockito::ServerGuard, Mock) {
        let url = format!("videojocs/{nom_videojoc}/partides");
        let mut server = Server::new();
        let encoded = encode(&url);
        let _mock = server
            .mock("POST", format!("/api/v1/{encoded}").as_str())
            .match_header("authorization", "Basic YWRtaW46cGFzdGFuYWdhYnVsbGlkYQ==")
            .match_body(r#"{"nom":"save1.txt","contingut":"Soc una partida guardada del Napoleón"}"#)
            .with_status(201)
            .expect(1)
            .create();
        (server, _mock)
    }
    fn get_partida_path_ntw_s1() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_pg_api/path a videojocs/Napoleón TW HD/save1.txt").to_str().unwrap().to_string()
    }
    pub fn get_partida_ntw_s1() -> PartidaGuardada {
        let mut pg = PartidaGuardada::new(get_partida_path_ntw_s1());
        pg.videojoc = "Napoleón TW HD".to_string();
        pg
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
    #[test]
    fn test_post_partida_guardada() {
        let nom_videojoc = "Napoleón TW HD";
        let (server, _mock) = setup_fake_server_post_partida_guardada(nom_videojoc.to_string());
        let pgapi = get_pg_api(server.url().clone());
        let partida = get_partida_ntw_s1();
        pgapi.post_partida_guardada(&partida);
        _mock.assert();
    }
}