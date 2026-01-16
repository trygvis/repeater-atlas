use repeater_atlas::dao;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = repeater_atlas::init().await;

    let mut c = pool.get().await?;

    let repeaters = [
        // LA4O
        dao::repeater::NewRepeater {
            call_sign: "LA5OR".to_string(),
            frequency: 145_600_000,
            rx_offset: -600_000,
            modulation: Some(dao::repeater::Modulation::FmNarrow),
            subtone_mode: dao::repeater::SubtoneMode::None,
            tx_subtone: None,
            rx_subtone: None,
            has_dmr: false,
            dmr_id: None,
            has_aprs: false,
            maidenhead_locator: Some("JO59ix".to_string()),
            latitude: None,
            longitude: None,
            address: Some("Tryvann, Oslo".to_string()),
        },
    ];

    for r in repeaters {
        dao::repeater::insert(&mut c, r).await?;
    }

    Ok(())
}
