use bb8::ManageConnection;
use diesel::query_builder::QueryFragment;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::PoolError;
use diesel_async::pooled_connection::PoolableConnection;
use diesel_async::{AsyncConnection, AsyncPgConnection};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tracing::{debug, warn};

// Diesel's bb8 manager does not expose enough lifecycle visibility for the
// kind of connection debugging we need when startup checks or test migrations
// fail. Keep the app on bb8, but route connection creation through a local
// manager so those events show up in normal tracing output.
pub type AppPool = bb8::Pool<LoggingConnectionManager<AsyncPgConnection>>;

pub struct LoggingConnectionManager<C> {
    inner: AsyncDieselConnectionManager<C>,
    next_connection_id: AtomicU64,
}

impl<C> LoggingConnectionManager<C>
where
    C: AsyncConnection + 'static,
{
    pub fn new(connection_url: impl Into<String>) -> Self {
        Self {
            inner: AsyncDieselConnectionManager::new(connection_url),
            next_connection_id: AtomicU64::new(1),
        }
    }
}

impl<C> fmt::Debug for LoggingConnectionManager<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LoggingConnectionManager<{}>",
            std::any::type_name::<C>()
        )
    }
}

// A stable synthetic ID makes it possible to correlate "connect", "broken",
// and "dropped" events for a single physical database connection across the
// lifetime of the process. Without that, the logs are much less useful when
// diagnosing flaky connection handling.
pub struct LoggedConnection<C> {
    id: u64,
    connected_at: Instant,
    inner: C,
}

impl<C> LoggedConnection<C> {
    fn new(id: u64, inner: C) -> Self {
        Self {
            id,
            connected_at: Instant::now(),
            inner,
        }
    }
}

impl<C> Deref for LoggedConnection<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> DerefMut for LoggedConnection<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<C> Drop for LoggedConnection<C> {
    fn drop(&mut self) {
        debug!(
            connection_id = self.id,
            lifetime_ms = self.connected_at.elapsed().as_millis(),
            "Database connection dropped"
        );
    }
}

impl<C> ManageConnection for LoggingConnectionManager<C>
where
    C: PoolableConnection + 'static,
    diesel::dsl::select<diesel::dsl::AsExprOf<i32, diesel::sql_types::Integer>>:
        diesel_async::methods::ExecuteDsl<C>,
    diesel::query_builder::SqlQuery: QueryFragment<C::Backend>,
{
    type Connection = LoggedConnection<C>;
    type Error = PoolError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let connection_id = self.next_connection_id.fetch_add(1, Ordering::Relaxed);
        let started_at = Instant::now();
        debug!(connection_id, "Connecting to database");

        match self.inner.connect().await {
            Ok(connection) => {
                let connect_time_ms = started_at.elapsed().as_millis();
                debug!(
                    connection_id,
                    connect_time_ms, "Database connection established in {connect_time_ms} ms"
                );
                Ok(LoggedConnection::new(connection_id, connection))
            }
            Err(error) => {
                let connect_time_ms = started_at.elapsed().as_millis();
                warn!(
                    connection_id,
                    connect_time_ms,
                    error = ?error,
                    "Failed to connect to database in {connect_time_ms} ms"
                );
                Err(error)
            }
        }
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        match self.inner.is_valid(&mut conn.inner).await {
            Ok(()) => Ok(()),
            Err(error) => {
                warn!(
                    connection_id = conn.id,
                    error = ?error,
                    "Database connection validation failed"
                );
                Err(error)
            }
        }
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        let broken = self.inner.has_broken(&mut conn.inner);

        if broken {
            warn!(connection_id = conn.id, "Database connection marked broken");
        }

        broken
    }
}
