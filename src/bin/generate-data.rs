#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = repeater_atlas::init().await;

    let mut c = pool.get().await?;

    repeater_atlas::test_data::generate(&mut c).await?;

    Ok(())
}
