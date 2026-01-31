use crate::MaidenheadLocator;
use crate::dao;

pub struct ResolvedSite {
    pub maidenhead: String,
    pub location: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

pub fn resolve_site_fields(repeater: &dao::repeater_system::RepeaterSystem) -> ResolvedSite {
    let grid: Option<MaidenheadLocator> = repeater.maidenhead.clone();
    let mut latitude = repeater.latitude;
    let mut longitude = repeater.longitude;

    if latitude.is_none() || longitude.is_none() {
        if let Some(ref grid) = grid {
            let (grid_longitude, grid_latitude) = grid.longlat();
            latitude = latitude.or(Some(grid_latitude));
            longitude = longitude.or(Some(grid_longitude));
        }
    }

    let location = match (latitude, longitude) {
        (Some(latitude), Some(longitude)) => format!("{latitude}, {longitude}"),
        _ => "-".to_string(),
    };

    ResolvedSite {
        maidenhead: grid
            .map(|value| format!("{value}"))
            .unwrap_or_else(|| "-".to_string()),
        location,
        latitude,
        longitude,
    }
}

pub fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_km = 6_371.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_km * c
}
