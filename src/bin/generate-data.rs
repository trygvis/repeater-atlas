use repeater_atlas::dao;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = repeater_atlas::init().await;

    let mut c = pool.get().await?;

    for r in repeater_atlas::test_data::repeaters() {
        dao::repeater::insert(&mut c, r).await?;
    }

    Ok(())
}
