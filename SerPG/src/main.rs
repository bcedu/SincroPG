use axum::{
    extract::{Path, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use std::fs;
use std::fs::File;
use std::io::{Write, Read};
use std::path::PathBuf;
use axum::extract::State;
use normalized_hash::Hasher;
#[derive(Clone)]
struct SerPGState {
    videojocs_path: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct VideojocAPI {
    id: String,
    nom: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct PartidaGuardadaAPI {
    nom: String,
    hash: String
}
#[derive(Debug, Deserialize, Serialize)]
struct PartidaGuardadaContingutAPI {
    nom: String,
    contingut: String
}
struct SerPG {
    pub router: Router,
}
impl SerPG {
    fn new(path: String) -> Self {
        let state = SerPGState {
            videojocs_path: path,
        };
        let r = Router::new()
            .route("/api/v1/test", get(Self::test))
            .route("/api/v1/videojocs", get(Self::get_videojocs))
            .route("/api/v1/videojocs/{videojoc_id}/partides", get(Self::get_partides_guardades).post(Self::post_partida_guardada))
            .route("/api/v1/videojocs/{videojoc_id}/partides/{partida_id}/contingut", get(Self::get_partida_guardada))
            .with_state(state)
        ;
        SerPG {router: r}
    }
    async fn start(self, port: Option<String>) {
        let port = port.unwrap_or_else(|| String::from("3000"));
        let addr = format!("0.0.0.0:{port}");
        println!("🚀 SerPG escoltant a http://{}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, self.router).await.unwrap();
    }
    async fn test() -> &'static str {
        "OK"
    }
    async fn get_videojocs(State(spg_state): State<SerPGState>) -> Json<Vec<VideojocAPI>> {
        let mut videojocs_list = Vec::new();
        for path in fs::read_dir(spg_state.videojocs_path).unwrap() {
            let videojoc = path.unwrap().file_name().to_str().unwrap().to_string();
            videojocs_list.push(
                VideojocAPI {
                    id: videojoc.clone(),
                    nom: videojoc.clone(),
                }
            );
        }
        Json(videojocs_list)
    }
    async fn get_partides_guardades(State(spg_state): State<SerPGState>, Path(videojoc_id): Path<String>) -> Json<Vec<PartidaGuardadaAPI>> {
        let mut partides_list = Vec::new();
        let videojoc_path = format!("{}/{}", spg_state.videojocs_path, videojoc_id);
        if !PathBuf::from(&videojoc_path).exists() {
            fs::create_dir(videojoc_path.clone()).unwrap();
        }
        for entry in fs::read_dir(videojoc_path).unwrap() {
            let entry = entry.unwrap();
            let full_path = entry.path();
            let partida_hash = Hasher::new().hash_file(&full_path, None::<PathBuf>);
            let partida = entry.file_name().to_str().unwrap().to_string();
            partides_list.push(
                PartidaGuardadaAPI {
                    nom: partida,
                    hash: partida_hash,
                }
            );
        }
        Json(partides_list)
    }
    async fn get_partida_guardada(State(spg_state): State<SerPGState>, Path((videojoc_id, partida_id)): Path<(String, String)>) -> Json<PartidaGuardadaContingutAPI> {
        let partida_path = format!("{}/{}/{}", spg_state.videojocs_path, videojoc_id, partida_id);
        let mut contingut = String::new();
        let mut f = File::open(partida_path).unwrap();
        f.read_to_string(&mut contingut).unwrap();
        drop(f);
        Json(PartidaGuardadaContingutAPI{nom: partida_id, contingut})
    }
    async fn post_partida_guardada(State(spg_state): State<SerPGState>, Path(videojoc_id): Path<String>, Json(partida_nova): Json<PartidaGuardadaContingutAPI>) {
        let partida_path = format!("{}/{}/{}", spg_state.videojocs_path, videojoc_id, partida_nova.nom.clone());
        let videojoc_path = format!("{}/{}", spg_state.videojocs_path, videojoc_id);
        if !PathBuf::from(&videojoc_path).exists() {
            fs::create_dir(videojoc_path).unwrap();
        }
        let mut f = File::create(partida_path).unwrap();
        f.write_all(partida_nova.contingut.as_bytes()).unwrap();
        f.sync_all().unwrap();
        drop(f);
    }
}
#[tokio::main]
async fn main() {
    SerPG::new("".to_string()).start(None).await;
}

#[cfg(test)]
pub mod tests {
    use crate::{PartidaGuardadaContingutAPI, SerPG};
    use urlencoding::encode;
    use std::path::{PathBuf, Path};
    use std::fs::{remove_dir_all, File};
    use std::io::{Write, Read};
    use tokio::fs::remove_dir;
    async fn setup_server() {
        // Fem neteja decoses que no haurien de existir
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pastanaga bullida").to_str().unwrap().to_string();
        remove_dir_all(&test_path);
        // Fem el servidor
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").to_str().unwrap().to_string();
        tokio::spawn(async {
            SerPG::new(test_path).start(Some("3001".to_string())).await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    async fn make_get_request(endpoint: &str) -> String {
        let mut request_url = "http://127.0.0.1:3001/api/v1".to_string();
        for endpoint_part in endpoint.split('/') {
            request_url = format!("{}/{}", request_url, encode(endpoint_part));
        }
        println!("GET: {}", request_url.clone());
        let res = reqwest::Client::new()
            .get(request_url)
            .basic_auth("admin", Some("admin"))
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let body = res.text().await.unwrap();
        body
    }
    async fn make_post_request(endpoint: &str, payload: PartidaGuardadaContingutAPI) {
        let mut request_url = "http://127.0.0.1:3001/api/v1".to_string();
        for endpoint_part in endpoint.split('/') {
            request_url = format!("{}/{}", request_url, encode(endpoint_part));
        }
        println!("POST: {}", request_url.clone());
        let res = reqwest::Client::new()
            .post(request_url)
            .basic_auth("admin", Some("admin"))
            .json(&payload)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }
    #[tokio::test]
    async fn test_api_get_test() {
        let server = setup_server().await;
        let res = make_get_request("test").await;
        assert!(res.starts_with("OK"));
    }
    #[tokio::test]
    async fn test_api_get_videojocs() {
        let server = setup_server().await;
        let res = make_get_request("videojocs").await;
        let expected_res = "[{\"id\":\"Napoleón TW HD\",\"nom\":\"Napoleón TW HD\"},{\"id\":\"Warhammer 50k\",\"nom\":\"Warhammer 50k\"}]";
        assert!(res.starts_with(expected_res));
    }
    #[tokio::test]
    async fn test_api_get_partides_guardades() {
        let server = setup_server().await;
        // Joc que no existeix
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pastanaga bullida").to_str().unwrap().to_string();
        let res = make_get_request("videojocs/pastanaga bullida/partides").await;
        assert_eq!(res, "[]");
        let path = Path::new(&test_path);
        assert!(Path::new(&path).exists());
        // Joc sense partides
        let res = make_get_request("videojocs/Warhammer 50k/partides").await;
        assert_eq!(res, "[]");
        // Joc amb partides
        let res = make_get_request("videojocs/Napoleón TW HD/partides").await;
        let expected_res = "[{\"nom\":\"save1.txt\",\"hash\":\"02d47a22e09f46731a58dbe7cb299c0315c6760aec7557e8ca6e87090fc85dfd\"},{\"nom\":\"save3.txt\",\"hash\":\"158ed8c255d81393d423bc01c4993eceb3bb20a2596659ebc7f14ae82cbde4c8\"}]";
        assert_eq!(res, expected_res);
    }
    #[tokio::test]
    async fn test_api_get_partida_guardada() {
        let server = setup_server().await;
        let res = make_get_request("videojocs/Napoleón TW HD/partides/save3.txt/contingut").await;
        let expected_res = "{\"nom\":\"save3.txt\",\"contingut\":\"Soc una partida guardada del Total War 40k\\nPartida 3\"}";
        assert_eq!(res, expected_res);
    }
    #[tokio::test]
    async fn test_post_partida_guardada() {
        let server = setup_server().await;
        // Joc que no existeix
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pastanaga bullida").to_str().unwrap().to_string();
        let test_partida_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pastanaga bullida/save.txt").to_str().unwrap().to_string();
        let partida_test = PartidaGuardadaContingutAPI {
            nom: "save.txt".to_string(),
            contingut: "@@".to_string(),
        };
        make_post_request("videojocs/pastanaga bullida/partides", partida_test).await;
        let path = Path::new(&test_path);
        assert!(Path::new(&path).exists());
        let path = Path::new(&test_partida_path);
        assert!(Path::new(&path).exists());
        let mut contingut = String::new();
        let mut f = File::open(test_partida_path.clone()).unwrap();
        f.read_to_string(&mut contingut).unwrap();
        drop(f);
        assert_eq!(contingut, "@@");
        // Joc amb partides
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/Napoleón TW HD").to_str().unwrap().to_string();
        let test_partida_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/Napoleón TW HD/save3.txt").to_str().unwrap().to_string();
        let partida_test = PartidaGuardadaContingutAPI {
            nom: "save3.txt".to_string(),
            contingut: "Partida4".to_string(),
        };
        let original_content = "Soc una partida guardada del Total War 40k\nPartida 3";
        make_post_request("videojocs/Napoleón TW HD/partides", partida_test).await;
        let path = Path::new(&test_path);
        assert!(Path::new(&path).exists());
        let path = Path::new(&test_partida_path);
        assert!(Path::new(&path).exists());
        let mut contingut = String::new();
        let mut f = File::open(test_partida_path.clone()).unwrap();
        f.read_to_string(&mut contingut).unwrap();
        drop(f);
        assert_eq!(contingut, "Partida4");
        let mut f = File::create(test_partida_path).unwrap();
        f.write_all(original_content.as_bytes()).unwrap();
        f.sync_all().unwrap();
        drop(f)
    }
}