use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let pool = repeater_atlas::init().await;

    let mut c = pool.get().await?;

    repeater_atlas::test_data::generate(&mut c).await?;

    repeater_atlas::test_data::generate_users(&mut c).await?;

    Ok(())
}
