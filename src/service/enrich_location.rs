use crate::MaidenheadLocator;
use crate::RepeaterAtlasError;
use crate::service::geocoding::Geocoder;
use tracing::{info, warn};

const DEFAULT_MAIDENHEAD_LEN: usize = 6;

pub struct EnrichedLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub maidenhead: MaidenheadLocator,
}

pub async fn enrich_location<G: Geocoder + ?Sized>(
    geocoder: &G,
    call_sign: &str,
    address: Option<&str>,
    maidenhead: Option<&MaidenheadLocator>,
) -> Result<Option<EnrichedLocation>, RepeaterAtlasError> {
    if maidenhead.is_some() {
        return Ok(None);
    }

    let Some(address) = address else {
        return Ok(None);
    };
    if address.trim().is_empty() {
        return Ok(None);
    }

    let Some(location) = geocoder.geocode_one(address).await? else {
        warn!(%call_sign, %address, "Failed to geocode repeater address");
        return Ok(None);
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

    info!("Resolved {address} to {location}, {locator}");

    Ok(Some(EnrichedLocation {
        latitude: location.latitude,
        longitude: location.longitude,
        maidenhead: locator,
    }))
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

        let changed = enrich_location(&geocoder, "LA0ZZZ", Some("Trondheim, Norway"), None)
            .await
            .unwrap();
        let changed = changed.expect("location should be enriched");
        assert!(changed.maidenhead.as_str().len() >= DEFAULT_MAIDENHEAD_LEN);
        assert_eq!(changed.latitude, 63.4305);
        assert_eq!(changed.longitude, 10.3951);
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
        let maidenhead = MaidenheadLocator::new("JP53fi").unwrap();

        let changed = enrich_location(
            &geocoder,
            "LA0ZZZ",
            Some("Trondheim, Norway"),
            Some(&maidenhead),
        )
        .await
        .unwrap();
        assert!(changed.is_none());
        assert_eq!(geocoder.calls.load(Ordering::SeqCst), 0);
    }
}
