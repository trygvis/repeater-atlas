mod utils;

use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use repeater_atlas::dao;
use repeater_atlas::repeater_service::RepeaterService;
use repeater_atlas::schema::repeater_system;

#[tokio::test]
async fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = utils::pool().await;
    let mut c = pool.get().await?;

    let new_repeater = dao::repeater_system::NewRepeaterSystem::new("LA1ABC");

    let repeater = dao::repeater_system::insert(&mut c, new_repeater).await?;

    let tx_hz = 145_775_000;
    let rx_hz = tx_hz - 600_000;
    let service = RepeaterService::Fm {
        label: "VHF".to_string(),
        rx_hz,
        tx_hz,
        bandwidth: repeater_atlas::dao::repeater_service::FmBandwidth::Narrow,
        rx_tone: repeater_atlas::repeater_service::Tone::None,
        tx_tone: repeater_atlas::repeater_service::Tone::None,
        note: None,
    };
    dao::repeater_service::insert(&mut c, service.to_new_dao(repeater.id)).await?;

    let count: i64 = repeater_system::table.count().get_result(&mut c).await?;
    assert!(count >= 1, "expected at least one repeater system row");

    sql_query("DELETE FROM repeater_system")
        .execute(&mut c)
        .await?;

    Ok(())
}
