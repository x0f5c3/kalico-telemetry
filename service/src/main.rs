use axum::{routing::get, Router, extract::Path, response::Json};
use core::MeshDB;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(MeshDB::new("/var/lib/kalico/mesh.duckdb").unwrap()));

    let app = Router::new()
        .route("/v1/mesh/:kind", get({
            let db = db.clone();
            move |Path(kind): Path<String>| {
                let db = db.clone();
                async move {
                    // TODO: Query mesh points for kind
                    Json(vec![]) // Placeholder
                }
            }
        }));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
