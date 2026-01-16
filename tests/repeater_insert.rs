use diesel::prelude::*;
use diesel::sql_query;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use repeater_atlas::schema::repeaters;
use uuid::Uuid;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Insertable)]
#[diesel(table_name = repeaters)]
struct NewRepeater {
    id: Uuid,
    callsign: String,
}

#[test]
fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set to run database integration tests");

    let mut conn = PgConnection::establish(&database_url)?;
    conn.run_pending_migrations(MIGRATIONS)?;

    let new_repeater = NewRepeater {
        id: Uuid::new_v4(),
        callsign: "LA1ABC".to_string(),
    };

    diesel::insert_into(repeaters::table)
        .values(&new_repeater)
        .execute(&mut conn)?;

    let count: i64 = repeaters::table.count().get_result(&mut conn)?;
    assert!(count >= 1, "expected at least one repeater row");

    sql_query("DELETE FROM repeaters WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(new_repeater.id)
        .execute(&mut conn)?;

    Ok(())
}
