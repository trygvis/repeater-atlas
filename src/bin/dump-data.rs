use repeater_atlas::RepeaterAtlasError;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), RepeaterAtlasError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .init();

    let pool = repeater_atlas::init().await;

    let mut c = pool.get().await?;

    repeater_atlas::test_data::dump_data(&mut c).await?;

    Ok(())
}
