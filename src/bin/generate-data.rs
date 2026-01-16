use diesel::prelude::*;
use repeater_atlas::dao;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = repeater_atlas::init().await;

    let mut conn = pool.get().await?;

    let repeaters = [
        // LA4O
        dao::repeater::NewRepeater {
            call_sign: "LA5OR".to_string(),
            frequency: 145_600_000,
            rx_offset: -600_000,
        },
    ];

    for r in repeaters {
        dao::repeater::insert(&mut conn, r).await?;
    }

    Ok(())
}
