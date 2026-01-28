mod utils;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use repeater_atlas::Frequency;
use repeater_atlas::dao;
use repeater_atlas::repeater_service::RepeaterService;
use repeater_atlas::schema::{entity, repeater_service, repeater_system};

#[tokio::test]
async fn inserts_repeater_row() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = utils::pool().await;
    let mut c = pool.get().await?;

    let repeater_entity = dao::entity::insert(
        &mut c,
        dao::entity::NewEntity {
            kind: dao::entity::EntityKind::Repeater,
            call_sign: Some("LA1ABC".to_string()),
        },
    )
    .await?;

    let new_repeater = dao::repeater_system::NewRepeaterSystem::new(repeater_entity.id);

    let repeater = dao::repeater_system::insert(&mut c, new_repeater).await?;

    let tx_hz = Frequency::new_hz(145_775_000)?;
    let rx_hz = tx_hz.offset(-600_000)?;
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

    // Delete only the entity row we created; this exercises ON DELETE CASCADE
    // (entity -> repeater_system -> repeater_service).
    diesel::delete(entity::table.filter(entity::id.eq(repeater_entity.id)))
        .execute(&mut c)
        .await?;

    let repeater_exists: i64 = repeater_system::table
        .filter(repeater_system::id.eq(repeater.id))
        .count()
        .get_result(&mut c)
        .await?;
    assert_eq!(repeater_exists, 0, "expected repeater_system row to be removed");

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
