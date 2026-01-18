mod utils;

use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use repeater_atlas::dao;
use repeater_atlas::schema::repeater_system;

#[tokio::test]
async fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = utils::pool().await;
    let mut c = pool.get().await?;

    let new_repeater = dao::repeater::NewRepeater::new("LA1ABC");

    let repeater = dao::repeater::insert(&mut c, new_repeater).await?;

    let tx_hz = 145_775_000;
    let rx_hz = tx_hz - 600_000;
    dao::repeater_port::insert(
        &mut c,
        dao::repeater_port::NewRepeaterPort::new(repeater.id, "VHF", rx_hz, tx_hz),
    )
    .await?;

    let count: i64 = repeater_system::table.count().get_result(&mut c).await?;
    assert!(count >= 1, "expected at least one repeater system row");

    sql_query("DELETE FROM repeater_system")
        .execute(&mut c)
        .await?;

    Ok(())
}
