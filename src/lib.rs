use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;

pub mod auth;
pub mod dao;
pub mod error;
pub mod frequency;
pub mod maidenhead_locator;
pub mod point;
pub mod schema;
pub mod service;
pub mod test_data;
pub mod web;

pub use error::RepeaterAtlasError;
pub use frequency::Frequency;
pub use maidenhead_locator::MaidenheadLocator;
pub use point::Point;

pub async fn init() -> bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set to run the server");
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .await
        .expect("failed to build database connection pool");

    pool
}

pub fn hello() -> &'static str {
    "hello world"
}
