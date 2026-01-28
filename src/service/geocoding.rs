use crate::RepeaterAtlasError;
use async_trait::async_trait;
use serde::Deserialize;
use std::fmt;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

#[derive(Debug, Clone, Copy)]
pub struct GeocodedLocation {
    pub latitude: f64,
    pub longitude: f64,
}

impl fmt::Display for GeocodedLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.latitude, self.longitude)
    }
}

#[async_trait]
pub trait Geocoder: Send + Sync {
    async fn geocode_one(
        &self,
        query: &str,
    ) -> Result<Option<GeocodedLocation>, RepeaterAtlasError>;
}

pub struct NullGeocoder;

#[async_trait]
impl Geocoder for NullGeocoder {
    async fn geocode_one(
        &self,
        _query: &str,
    ) -> Result<Option<GeocodedLocation>, RepeaterAtlasError> {
        Ok(None)
    }
}

pub fn nominatim_enabled_from_env() -> bool {
    let Ok(value) = std::env::var("NOMINATIM_ENABLED") else {
        return true;
    };

    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => {
            warn!(value = %value, "Invalid NOMINATIM_ENABLED value; treating as disabled");
            false
        }
    }
}

fn base_url_from_env() -> String {
    std::env::var("NOMINATIM_BASE_URL")
        .unwrap_or_else(|_| "https://nominatim.openstreetmap.org".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn user_agent_from_env() -> String {
    std::env::var("NOMINATIM_USER_AGENT").unwrap_or_else(|_| "Repeater Atlas".to_string())
}

pub struct NominatimGeocoder {
    client: reqwest::Client,
    base_url: String,
    // Simple single-process rate limiter to avoid hammering the public instance.
    next_allowed: Mutex<Instant>,
}

impl NominatimGeocoder {
    pub fn from_env() -> Result<Self, reqwest::Error> {
        let user_agent = user_agent_from_env();
        let client = reqwest::Client::builder().user_agent(user_agent).build()?;
        Ok(Self {
            client,
            base_url: base_url_from_env(),
            next_allowed: Mutex::new(Instant::now()),
        })
    }

    async fn wait_turn(&self) {
        // Respect ~1 req/sec guideline for public Nominatim.
        let mut next_allowed = self.next_allowed.lock().await;
        let now = Instant::now();
        if *next_allowed > now {
            tokio::time::sleep(*next_allowed - now).await;
        }
        *next_allowed = Instant::now() + Duration::from_secs(1);
    }
}

#[derive(Debug, Deserialize)]
struct NominatimSearchResult {
    lat: String,
    lon: String,
    #[allow(dead_code)]
    display_name: Option<String>,
}

#[async_trait]
impl Geocoder for NominatimGeocoder {
    async fn geocode_one(
        &self,
        query: &str,
    ) -> Result<Option<GeocodedLocation>, RepeaterAtlasError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(None);
        }

        self.wait_turn().await;

        let url = format!("{}/search", self.base_url);
        debug!(%url, %query, "Nominatim geocode");

        let rows: Vec<NominatimSearchResult> = self
            .client
            .get(url)
            .query(&[
                ("q", query),
                ("format", "jsonv2"),
                ("limit", "1"),
                ("addressdetails", "0"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let Some(row) = rows.first() else {
            return Ok(None);
        };

        let latitude: f64 = row.lat.parse().map_err(|e| {
            RepeaterAtlasError::Other(Box::new(e), "invalid lat value from nominatim".to_string())
        })?;
        let longitude: f64 = row.lon.parse().map_err(|e| {
            RepeaterAtlasError::Other(Box::new(e), "invalid lon value from nominatim".to_string())
        })?;

        if !latitude.is_finite() || !longitude.is_finite() {
            warn!(%latitude, %longitude, "Nominatim returned non-finite coordinates");
            return Ok(None);
        }

        Ok(Some(GeocodedLocation {
            latitude,
            longitude,
        }))
    }
}

static NULL_GEOCODER: NullGeocoder = NullGeocoder;
static NOMINATIM: OnceLock<Result<NominatimGeocoder, String>> = OnceLock::new();

pub fn nominatim_geocoder_from_env() -> Result<&'static dyn Geocoder, RepeaterAtlasError> {
    if !nominatim_enabled_from_env() {
        return Ok(&NULL_GEOCODER);
    }

    let geocoder = NOMINATIM.get_or_init(|| {
        NominatimGeocoder::from_env().map_err(|e| format!("failed to init Nominatim client: {e}"))
    });

    match geocoder {
        Ok(value) => Ok(value as &dyn Geocoder),
        Err(msg) => Err(RepeaterAtlasError::Other(
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg.clone())),
            "nominatim init error".to_string(),
        )),
    }
}
