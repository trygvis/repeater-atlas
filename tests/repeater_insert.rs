use diesel::prelude::*;
use diesel::sql_query;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use repeater_atlas::schema::repeater;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Insertable)]
#[diesel(table_name = repeater)]
struct NewRepeater {
    call_sign: Option<String>,
}

#[test]
fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set to run database integration tests");

    let mut conn = PgConnection::establish(&database_url)?;
    conn.run_pending_migrations(MIGRATIONS)?;

    let new_repeater = NewRepeater {
        call_sign: Some("LA1ABC".to_string()),
    };

    diesel::insert_into(repeater::table)
        .values(&new_repeater)
        .execute(&mut conn)?;

    let count: i64 = repeater::table.count().get_result(&mut conn)?;
    assert!(count >= 1, "expected at least one repeater row");

    sql_query("DELETE FROM repeater")
        .execute(&mut conn)?;

    Ok(())
}
