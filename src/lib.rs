use crate::logging_connection_manager::LoggingConnectionManager;
use crate::pg_tls_connection_manager::{PgTlsConnectionManager, establish_tls};
use bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::ManagerConfig;

pub mod auth;
pub mod dao;
pub mod error;
pub mod frequency;
pub mod logging_connection_manager;
pub mod maidenhead_locator;
pub mod pg_tls_connection_manager;
pub mod point;
pub mod schema;
pub mod service;
pub mod test_data;
pub mod web;

pub use error::RepeaterAtlasError;
pub use frequency::Frequency;
pub use maidenhead_locator::MaidenheadLocator;
pub use point::Point;

pub type AppPool = bb8::Pool<LoggingConnectionManager>;

pub async fn init() -> AppPool {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run the server");
    let manager = build_manager(database_url);
    Pool::builder()
        .build(manager)
        .await
        .expect("failed to build database connection pool")
}

pub fn build_manager(database_url: impl Into<String>) -> LoggingConnectionManager {
    let tls = PgTlsConnectionManager::new(database_url);

    let mut manager_config = ManagerConfig::default();
    manager_config.custom_setup = Box::new(|database_url| Box::pin(establish_tls(database_url)));

    let inner = AsyncDieselConnectionManager::new_with_config(tls.database_url(), manager_config);
    LoggingConnectionManager::new(inner)
}

pub fn hello() -> &'static str {
    "hello world"
}
