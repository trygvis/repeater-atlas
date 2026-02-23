use axum::Router;
use axum_extra::routing::RouterExt;
use repeater_atlas::web::AppState;
use repeater_atlas::web::{
    auth, export, map, my_page, organization_list, repeater, repeater_list, search,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let pool = repeater_atlas::init().await;

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

    let state = AppState { pool, jwt_secret };

    let app = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .typed_get(map::home)
        .typed_get(repeater_list::repeaters)
        .typed_get(organization_list::organizations)
        .typed_get(repeater::call_sign)
        .typed_get(search::call_sign_search)
        .typed_get(auth::login_form)
        .typed_post(auth::login_submit)
        .typed_get(auth::logout)
        .typed_get(my_page::my_page)
        .typed_get(export::chirp_export)
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
