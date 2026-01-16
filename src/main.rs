use std::net::SocketAddr;

use axum::{Router, routing::get};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;

use repeater_atlas::web::AppState;
use repeater_atlas::web::index;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run the server");
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .await
        .expect("failed to build database connection pool");

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
