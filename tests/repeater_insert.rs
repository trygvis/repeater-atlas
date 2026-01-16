use diesel::prelude::*;
use diesel::sql_query;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use repeater_atlas::dao;
use repeater_atlas::schema::repeater;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[test]
fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set to run database integration tests");

    let mut conn = PgConnection::establish(&database_url)?;
    conn.run_pending_migrations(MIGRATIONS)?;

    let new_repeater = dao::repeater::NewRepeater {
        call_sign: "LA1ABC".to_string(),
        frequency: 145775,
        rx_offset: 600,
    };

    dao::repeater::insert(&mut conn, new_repeater)?;

    let count: i64 = repeater::table.count().get_result(&mut conn)?;
    assert!(count >= 1, "expected at least one repeater row");

    sql_query("DELETE FROM repeater").execute(&mut conn)?;

    Ok(())
}
