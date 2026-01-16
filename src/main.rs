use std::net::SocketAddr;

use axum::{routing::get, Router};

use repeater_atlas::web::index;
use repeater_atlas::web::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let pool = repeater_atlas::init().await;

    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(index::index))
        .with_state(state);

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("BIND_ADDR must be a valid socket address");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    println!("Listening on {}", addr);

    axum::serve(listener, app).await.expect("server error");
}
