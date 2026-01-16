use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;

pub mod index;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<AsyncPgConnection>,
}
