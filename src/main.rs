use std::net::SocketAddr;

use axum::Router;
use axum_extra::routing::RouterExt;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

use repeater_atlas::web::index;
use repeater_atlas::web::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let pool = repeater_atlas::init().await;

    let state = AppState { pool };

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .typed_get(index::home)
        .typed_get(index::repeaters)
        .typed_get(index::detail)
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
