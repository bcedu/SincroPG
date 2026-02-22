use axum::{
    extract::{Path, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Debug, Clone, Serialize)]
struct Partida {
    videojoc: String,
    nom: String,
    path: String,
    contingut: String,
}
#[derive(Debug, Deserialize)]
struct NovaPartida {
    nom: String,
    contingut: String,
}

struct SerPG {
    pub router: Router,
}
impl SerPG {
    fn new() -> Self {
        let r = Router::new()
            .route("/api/v1/test", get(Self::test))
            .route("/api/v1/videojocs", get(Self::llistar_videojocs))
            .route(
                "/api/v1/videojocs/:videojoc_id/partides",
                get(Self::llistar_partides)
                .post(Self::crear_partida),
            );
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
    async fn llistar_videojocs() -> Json<Vec<String>> {
        Json(Vec::new())
    }
    async fn llistar_partides(Path(videojoc_id): Path<String>) -> Json<Vec<Partida>> {
        let _ = videojoc_id;
        Json(Vec::new())
    }
    async fn crear_partida(Path(videojoc_id): Path<String>, Json(nova): Json<NovaPartida>) -> Json<Partida> {
        let partida = Partida {
            videojoc: videojoc_id.clone(),
            nom: nova.nom.clone(),
            path: format!(
                "/api/v1/videojocs/{}/partides/{}",
                videojoc_id, nova.nom
            ),
            contingut: nova.contingut,
        };
        Json(partida)
    }
}
#[tokio::main]
async fn main() {
    SerPG::new().start(None);
}

#[cfg(test)]
pub mod tests {
    fn setup_server() -> SerPG {
        
    }
    #[test]
    fn test_probar_connexio() {
        let server = setup_fake_server_probar_connexio();
        let pgapi = get_pg_api(server.url().clone());
        let check = pgapi.probar_connexio();
        assert!(check);
    }
}