use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use repeater_atlas::dao;
use repeater_atlas::schema::repeater;
use tokio::sync::OnceCell;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
type AsyncPool = Pool<AsyncPgConnection>;

static DB_POOL: OnceCell<AsyncPool> = OnceCell::const_new();

async fn pool(database_url: String) -> AsyncPool {
    DB_POOL
        .get_or_init(|| async move {
            let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
            Pool::builder()
                .build(manager)
                .await
                .expect("failed to build async db pool")
        })
        .await
        .clone()
}

#[tokio::test]
async fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set to run database integration tests");

    let migrate_url = database_url.clone();
    tokio::task::spawn_blocking(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = PgConnection::establish(&migrate_url)?;
        conn.run_pending_migrations(MIGRATIONS)?;
        Ok(())
    })
    .await??;

    let pool = pool(database_url).await;
    let mut conn = pool.get().await?;

    let new_repeater = dao::repeater::NewRepeater {
        call_sign: "LA1ABC".to_string(),
        frequency: 145775,
        rx_offset: 600,
    };

    dao::repeater::insert(&mut conn, new_repeater).await?;

    let count: i64 = repeater::table.count().get_result(&mut conn).await?;
    assert!(count >= 1, "expected at least one repeater row");

    sql_query("DELETE FROM repeater").execute(&mut conn).await?;

    Ok(())
}
