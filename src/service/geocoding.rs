use crate::RepeaterAtlasError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
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
    cache: Mutex<HashMap<String, GeocodedLocation>>,
    cache_path: PathBuf,
    cache_write: Mutex<()>,
}

impl NominatimGeocoder {
    pub fn from_env() -> Result<Self, RepeaterAtlasError> {
        let user_agent = user_agent_from_env();
        let client = reqwest::Client::builder().user_agent(user_agent).build()?;
        let cache_path = PathBuf::from("data/geocoder.csv");
        let cache = Self::load_cache(&cache_path)?;
        Ok(Self {
            client,
            base_url: base_url_from_env(),
            next_allowed: Mutex::new(Instant::now()),
            cache: Mutex::new(cache),
            cache_path,
            cache_write: Mutex::new(()),
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

    fn load_cache(path: &Path) -> Result<HashMap<String, GeocodedLocation>, RepeaterAtlasError> {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(HashMap::new());
            }
            Err(error) => return Err(error.into()),
        };

        let mut reader = csv::Reader::from_reader(file);
        let mut cache = HashMap::new();
        for record in reader.deserialize::<GeocoderCacheRow>() {
            let record = record?;
            let query = record.query.trim();
            if query.is_empty() {
                continue;
            }

            if !record.latitude.is_finite() || !record.longitude.is_finite() {
                warn!(
                    latitude = %record.latitude,
                    longitude = %record.longitude,
                    "Skipping non-finite cached geocode coordinates"
                );
                continue;
            }

            cache.insert(
                query.to_string(),
                GeocodedLocation {
                    latitude: record.latitude,
                    longitude: record.longitude,
                },
            );
        }

        Ok(cache)
    }

    async fn append_cache(
        &self,
        query: &str,
        location: GeocodedLocation,
    ) -> Result<(), RepeaterAtlasError> {
        let _guard = self.cache_write.lock().await;
        let file_exists = self.cache_path.exists();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.cache_path)?;
        let write_headers = !file_exists || file.metadata()?.len() == 0;
        let mut writer = csv::WriterBuilder::new()
            .has_headers(write_headers)
            .from_writer(file);
        writer.serialize(GeocoderCacheRow {
            query: query.to_string(),
            latitude: location.latitude,
            longitude: location.longitude,
        })?;
        writer.flush()?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct NominatimSearchResult {
    lat: String,
    lon: String,
    #[allow(dead_code)]
    display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeocoderCacheRow {
    query: String,
    latitude: f64,
    longitude: f64,
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

        if let Some(location) = self.cache.lock().await.get(query).copied() {
            return Ok(Some(location));
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

        let location = GeocodedLocation {
            latitude,
            longitude,
        };

        let mut cache = self.cache.lock().await;
        let should_write = match cache.entry(query.to_string()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(location);
                true
            }
            std::collections::hash_map::Entry::Occupied(_) => false,
        };
        drop(cache);

        if should_write {
            self.append_cache(query, location).await?;
        }

        Ok(Some(location))
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
