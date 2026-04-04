use axum::Router;
use repeater_atlas::AppPool;
use repeater_atlas::RepeaterAtlasError;
use repeater_atlas::dao;
use repeater_atlas::web::{AppState, create_router};
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::process::exit;
use tower_http::services::ServeDir;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let pool = repeater_atlas::init().await;

    let db_check = env::var("DB_CHECK").ok().unwrap_or("1".to_owned());

    if db_check == "1" {
        check_db(&pool)
            .await
            .expect("Bad database connection and/or schema");
    }

    check_typst();

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

    let state = AppState { pool, jwt_secret };

    let path = "static";

    if !Path::new(path).is_dir() {
        eprintln!("Not a directory: {}", path);
        exit(1);
    }

    let app = Router::new()
        .nest_service("/static", ServeDir::new(path))
        .merge(create_router(state));

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string())
        .parse()
        .expect("BIND_ADDR must be a valid socket address");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    info!("Listening on {}", addr);

    axum::serve(listener, app).await.expect("server error");
}

fn check_typst() {
    match std::process::Command::new("typst")
        .arg("--version")
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr).trim().to_string();
            if output.status.success() {
                info!("typst ok: {}", combined);
            } else {
                warn!("typst --version failed: {}", combined);
            }
        }
        Err(e) => {
            warn!("typst not found: {}", e);
        }
    }
}

async fn check_db(pool: &AppPool) -> Result<(), RepeaterAtlasError> {
    let mut c = pool.get().await?;
    let call_signs = dao::call_sign::search_by_prefix(&mut c, "LA".to_string(), 1).await?;
    info!("DB ok, contains {} call-signs", call_signs.len());

    Ok(())
}
