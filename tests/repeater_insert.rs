mod utils;

use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use repeater_atlas::dao;
use repeater_atlas::schema::repeater;

#[tokio::test]
async fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = utils::pool().await;
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
