use diesel::ConnectionError;
use diesel::ConnectionResult;
use diesel_async::AsyncPgConnection;
use rustls::ClientConfig;
use std::sync::OnceLock;
use tokio_postgres_rustls::MakeRustlsConnect;
use tracing::warn;

/// Provides TLS-enabled PostgreSQL connection establishment using rustls and
/// the system's native certificate store.
pub struct PgTlsConnectionManager {
    database_url: String,
}

impl PgTlsConnectionManager {
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
        }
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}

pub async fn establish_tls(database_url: &str) -> ConnectionResult<AsyncPgConnection> {
    let tls = tls_connector()?;
    let (client, connection) = tokio_postgres::connect(database_url, tls)
        .await
        .map_err(|error| ConnectionError::BadConnection(error.to_string()))?;

    AsyncPgConnection::try_from_client_and_connection(client, connection).await
}

fn tls_connector() -> ConnectionResult<MakeRustlsConnect> {
    static TLS_CONNECTOR: OnceLock<Result<MakeRustlsConnect, String>> = OnceLock::new();

    TLS_CONNECTOR
        .get_or_init(build_tls_connector)
        .as_ref()
        .map(Clone::clone)
        .map_err(|error| ConnectionError::BadConnection(error.clone()))
}

fn build_tls_connector() -> Result<MakeRustlsConnect, String> {
    let mut roots = rustls::RootCertStore::empty();
    let cert_result = rustls_native_certs::load_native_certs();

    if !cert_result.errors.is_empty() {
        warn!(
            errors = ?cert_result.errors,
            "Errors occurred while loading native TLS certificates"
        );
    }

    for cert in cert_result.certs {
        roots.add(cert).map_err(|error| error.to_string())?;
    }

    if roots.is_empty() {
        return Err("no TLS root certificates were loaded for PostgreSQL".to_string());
    }

    let config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    Ok(MakeRustlsConnect::new(config))
}
