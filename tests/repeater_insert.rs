mod utils;

use dao::call_sign::NewCallSign;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use repeater_atlas::dao;
use repeater_atlas::schema::{call_sign, repeater_service, repeater_system};
use repeater_atlas::service::repeater_service::RepeaterService;
use repeater_atlas::{Frequency, service};

#[tokio::test]
async fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = utils::pool().await;
    let mut c = pool.get().await?;

    let repeater_call_sign_row =
        dao::call_sign::insert(&mut c, NewCallSign::new_repeater("LA1ABC")).await?;

    let new_repeater = dao::repeater_system::NewRepeaterSystem::new(repeater_call_sign_row.value);

    let repeater = dao::repeater_system::insert(&mut c, new_repeater).await?;

    let tx_hz = Frequency::new_hz(145_775_000)?;
    let rx_hz = tx_hz.offset(-600_000)?;
    let service = RepeaterService::Fm {
        label: "VHF".to_string(),
        rx_hz,
        tx_hz,
        bandwidth: dao::repeater_service::FmBandwidth::Narrow,
        rx_tone: service::repeater_service::Tone::None,
        tx_tone: service::repeater_service::Tone::None,
        note: None,
    };
    dao::repeater_service::insert(&mut c, service.to_new_dao(repeater.id)).await?;

    let count: i64 = repeater_system::table.count().get_result(&mut c).await?;
    assert!(count >= 1, "expected at least one repeater system row");

    // Delete only the call_sign row we created; this exercises ON DELETE CASCADE
    // (call_sign -> repeater_system -> repeater_service).
    diesel::delete(call_sign::table.filter(call_sign::value.eq("LA1ABC")))
        .execute(&mut c)
        .await?;

    let repeater_exists: i64 = repeater_system::table
        .filter(repeater_system::id.eq(repeater.id))
        .count()
        .get_result(&mut c)
        .await?;
    assert_eq!(
        repeater_exists, 0,
        "expected repeater_system row to be removed"
    );

    // Ensure the service row is removed via ON DELETE CASCADE.
    let service_count: i64 = repeater_service::table
        .filter(repeater_service::repeater_id.eq(repeater.id))
        .count()
        .get_result(&mut c)
        .await?;
    assert_eq!(
        service_count, 0,
        "expected cascading delete of repeater_service rows"
    );

    Ok(())
}
