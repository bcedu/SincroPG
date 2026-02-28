use axum::{
    extract::{Path, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use std::fs;
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
            .route("/api/v1/videojocs/{videojoc_id}/partides", get(Self::get_partides_guardades))
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
}
#[tokio::main]
async fn main() {
    SerPG::new("".to_string()).start(None).await;
}

#[cfg(test)]
pub mod tests {
    use crate::SerPG;
    use urlencoding::encode;
    use std::path::{PathBuf, Path};
    use tokio::fs::remove_dir;
    async fn setup_server() {
        // Fem neteja decoses que no haurien de existir
        let test_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pastanaga bullida").to_str().unwrap().to_string();
        remove_dir(&test_path).await;
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
        println!("REQUEST: {}", request_url.clone());
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
}