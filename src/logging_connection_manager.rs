use bb8::ManageConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::PoolError;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tracing::{debug, warn};

// Diesel's bb8 manager does not expose enough lifecycle visibility for the
// kind of connection debugging we need when startup checks or test migrations
// fail. Keep the app on bb8, but route connection creation through a local
// manager so those events show up in normal tracing output.
pub struct LoggingConnectionManager {
    inner: AsyncDieselConnectionManager<AsyncPgConnection>,
    next_connection_id: AtomicU64,
}

impl LoggingConnectionManager {
    pub fn new(inner: AsyncDieselConnectionManager<AsyncPgConnection>) -> Self {
        Self {
            inner,
            next_connection_id: AtomicU64::new(1),
        }
    }
}

impl fmt::Debug for LoggingConnectionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoggingConnectionManager<AsyncPgConnection>")
    }
}

// A stable synthetic ID makes it possible to correlate "connect", "broken",
// and "dropped" events for a single physical database connection across the
// lifetime of the process. Without that, the logs are much less useful when
// diagnosing flaky connection handling.
pub struct LoggedConnection<C> {
    id: u64,
    connected_at: Instant,
    pub(crate) inner: C,
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

impl ManageConnection for LoggingConnectionManager {
    type Connection = LoggedConnection<AsyncPgConnection>;
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
