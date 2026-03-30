use diesel_async::AsyncConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::scoped_futures::ScopedFutureExt;
use repeater_atlas::RepeaterAtlasError;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), RepeaterAtlasError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let pool = repeater_atlas::init().await;

    let mut c = pool.get().await?;

    let single_tx = true;

    if single_tx {
        c.transaction::<_, RepeaterAtlasError, _>(|c| {
            async move { load_data(c).await }.scope_boxed()
        })
        .await
    } else {
        let mut c = pool.get().await?;

        load_data(&mut c).await
    }
}

async fn load_data(c: &mut AsyncPgConnection) -> Result<(), RepeaterAtlasError> {
    repeater_atlas::test_data::load_data(c).await?;

    repeater_atlas::test_data::generate_users(c).await?;

    repeater_atlas::test_data::dump_data(c).await?;

    Ok(())
}
