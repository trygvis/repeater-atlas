use crate::MaidenheadLocator;
use crate::RepeaterAtlasError;
use crate::dao::repeater_system::NewRepeaterSystem;
use crate::service::geocoding::Geocoder;
use tracing::{info, warn};

const DEFAULT_MAIDENHEAD_LEN: usize = 6;

pub async fn enrich_location<G: Geocoder + ?Sized>(
    geocoder: &G,
    call_sign: &str,
    repeater: &mut NewRepeaterSystem,
) -> Result<bool, RepeaterAtlasError> {
    if repeater.maidenhead.is_some() {
        return Ok(false);
    }

    let Some(address) = repeater.address.as_ref() else {
        return Ok(false);
    };
    if address.trim().is_empty() {
        return Ok(false);
    }

    let Some(location) = geocoder.geocode_one(address).await? else {
        warn!(%call_sign, %address, "Failed to geocode repeater address");
        return Ok(false);
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
    let locator = MaidenheadLocator::new(grid).map_err(|e| {
        RepeaterAtlasError::Other(Box::new(e), "failed to canonicalize maidenhead".to_string())
    })?;

    info!("Resolved '{address}' to '{location}', {locator}");

    repeater.latitude = Some(location.latitude);
    repeater.longitude = Some(location.longitude);
    repeater.maidenhead = Some(locator);

    Ok(true)
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

    #[tokio::test]
    async fn enriches_location_when_maidenhead_missing() {
        let geocoder = FakeGeocoder {
            calls: AtomicUsize::new(0),
            // Trondheim-ish.
            location: Some(crate::service::geocoding::GeocodedLocation {
                latitude: 63.4305,
                longitude: 10.3951,
            }),
        };

        let mut repeater = NewRepeaterSystem::new("LA0ZZZ");
        repeater.address = Some("Trondheim, Norway".to_string());
        assert!(repeater.maidenhead.is_none());

        let changed = enrich_location(&geocoder, "LA0ZZZ", &mut repeater)
            .await
            .unwrap();
        assert!(changed);
        assert!(repeater.maidenhead.is_some());
        assert!(repeater.latitude.is_some());
        assert!(repeater.longitude.is_some());
        assert_eq!(geocoder.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn does_nothing_when_maidenhead_present() {
        let geocoder = FakeGeocoder {
            calls: AtomicUsize::new(0),
            location: Some(crate::service::geocoding::GeocodedLocation {
                latitude: 63.4305,
                longitude: 10.3951,
            }),
        };

        let mut repeater = NewRepeaterSystem::new("LA0ZZZ");
        repeater.address = Some("Trondheim, Norway".to_string());
        repeater.maidenhead = Some(MaidenheadLocator::new("JP53fi").unwrap());

        let changed = enrich_location(&geocoder, "LA0ZZZ", &mut repeater)
            .await
            .unwrap();
        assert!(!changed);
        assert_eq!(geocoder.calls.load(Ordering::SeqCst), 0);
    }
}
