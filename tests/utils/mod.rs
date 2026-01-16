use diesel::{Connection, PgConnection};
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tokio::sync::OnceCell;
use tracing_subscriber::EnvFilter;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
type AsyncPool = Pool<AsyncPgConnection>;

static DB_POOL: OnceCell<AsyncPool> = OnceCell::const_new();
static LOGGING: std::sync::Once = std::sync::Once::new();

fn init_logging() {
    LOGGING.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
            )
            .init();
    });
}

pub(crate) async fn pool() -> AsyncPool {
    dotenvy::dotenv().ok();
    init_logging();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run database integration tests");

    DB_POOL
        .get_or_init(|| async move {
            let migrate_url = database_url.clone();
            tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let mut c = PgConnection::establish(&migrate_url)?;
                c.run_pending_migrations(MIGRATIONS)?;
                Ok(())
            })
            .await
            .expect("migration task failed")
            .expect("migration failed");

            let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
            Pool::builder()
                .build(manager)
                .await
                .expect("failed to build async db pool")
        })
        .await
        .clone()
}
