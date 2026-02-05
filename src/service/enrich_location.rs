use crate::MaidenheadLocator;
use crate::RepeaterAtlasError;
use crate::service::geocoding::Geocoder;
use tracing::{info, warn};

const DEFAULT_MAIDENHEAD_LEN: usize = 6;

pub struct EnrichedLocation {
    pub address: Option<String>,
    pub maidenhead: Option<MaidenheadLocator>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl EnrichedLocation {
    const NONE: EnrichedLocation = EnrichedLocation {
        address: None,
        maidenhead: None,
        latitude: None,
        longitude: None,
    };
}

pub async fn enrich_location<G: Geocoder + ?Sized>(
    geocoder: &G,
    call_sign: &str,
    address: Option<String>,
    maidenhead: Option<MaidenheadLocator>,
) -> Result<EnrichedLocation, RepeaterAtlasError> {
    // Clean the address
    let address = address
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // If we have a maidenhead, resolve to lat/lon and return everything
    if let Some(maidenhead) = maidenhead {
        let (long, lat) = maidenhead.clone().longlat();
        return Ok(EnrichedLocation {
            address: address.map(|s| s.to_string()),
            maidenhead: Some(maidenhead),
            latitude: Some(lat),
            longitude: Some(long),
        });
    }

    match address {
        None => Ok(EnrichedLocation::NONE),
        Some(address) => {
            let Some(location) = geocoder.geocode_one(address.as_str()).await? else {
                warn!(%call_sign, %address, "Failed to geocode repeater address");
                return Ok(EnrichedLocation::NONE);
            };

            // maidenhead crate uses (lon, lat).
            let grid = maidenhead::longlat_to_grid(
                location.longitude,
                location.latitude,
                DEFAULT_MAIDENHEAD_LEN,
            )
            .map_err(|e| {
                RepeaterAtlasError::Other(Box::new(e), "failed to compute maidenhead".to_string())
            })?;

            let locator = MaidenheadLocator::new(grid)
                .map_err(|e| {
                    warn!(%call_sign, %address, "Failed to compute maidenhead location: {e}");
                })
                .ok();

            info!("Resolved {address} to {location}");

            Ok(EnrichedLocation {
                address: Some(address.to_string()),
                maidenhead: locator,
                latitude: Some(location.latitude),
                longitude: Some(location.longitude),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeGeocoder {
        calls: AtomicUsize,
        location: Option<crate::service::geocoding::GeocodedLocation>,
    }

    #[async_trait]
    impl Geocoder for FakeGeocoder {
        async fn geocode_one(
            &self,
            _query: &str,
        ) -> Result<Option<crate::service::geocoding::GeocodedLocation>, RepeaterAtlasError>
        {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.location)
        }
    }

    // Trondheim-ish.
    const TRONDHEIM: crate::service::geocoding::GeocodedLocation = crate::service::geocoding::GeocodedLocation {
        latitude: 63.4305,
        longitude: 10.3951,
    };

    #[tokio::test]
    async fn enriches_location_when_maidenhead_missing() {
        let geocoder = FakeGeocoder {
            calls: AtomicUsize::new(0),
            location: Some(TRONDHEIM),
        };

        let changed = enrich_location(
            &geocoder,
            "LA0ZZZ",
            Some("Trondheim, Norway".to_string()),
            None,
        )
        .await
        .unwrap();
        assert_eq!(changed.address, Some("Trondheim, Norway".to_string()));
        assert_eq!(changed.maidenhead, Some(MaidenheadLocator::new("JP53ek").unwrap()));
        assert_eq!(changed.latitude, Some(63.4305));
        assert_eq!(changed.longitude, Some(10.3951));
        assert_eq!(geocoder.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn does_nothing_when_maidenhead_present() {
        let geocoder = FakeGeocoder {
            calls: AtomicUsize::new(0),
            location: Some(TRONDHEIM),
        };
        let maidenhead = MaidenheadLocator::new("JP53fi").unwrap();

        let changed = enrich_location(
            &geocoder,
            "LA0ZZZ",
            Some("Trondheim, Norway".to_string()),
            Some(maidenhead.clone()),
        )
        .await
        .unwrap();
        assert_eq!(changed.address, Some("Trondheim, Norway".to_string()));
        assert_eq!(changed.maidenhead, Some(maidenhead));
        assert_eq!(changed.latitude, Some(63.354166666666686));
        assert_eq!(changed.longitude, Some(10.458333333333314));
        assert_eq!(geocoder.calls.load(Ordering::SeqCst), 0);
    }
}
