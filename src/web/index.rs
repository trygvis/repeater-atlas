use super::AppState;
use super::auth::auth_header;
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use maidenhead::grid_to_longlat;
use serde::{Deserialize, Serialize};

#[derive(TypedPath)]
#[typed_path("/")]
pub struct HomePath;

#[derive(Template)]
#[template(path = "pages/index.html")]
struct HomeTemplate {
    auth: super::AuthHeader,
    repeater_data: Vec<MapRepeater>,
}

#[derive(TypedPath)]
#[typed_path("/repeater")]
pub struct RepeatersPath;

#[derive(Template)]
#[template(path = "pages/repeaters.html")]
struct RepeatersTemplate {
    auth: super::AuthHeader,
    repeaters: Vec<RepeaterListItem>,
}

struct RepeaterListItem {
    call_sign: String,
    status: String,
    description: String,
    maidenhead: String,
    location: String,
}

#[derive(Serialize)]
struct MapRepeater {
    call_sign: String,
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize)]
struct MapPoint {
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize)]
struct MapContext {
    center: MapPoint,
    radius_meters: u32,
    repeaters: Vec<MapRepeater>,
}

struct ResolvedSite {
    maidenhead: String,
    location: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

fn resolve_site_fields(repeater: &dao::repeater_system::RepeaterSystem) -> ResolvedSite {
    let grid = repeater.maidenhead.clone();
    let mut latitude = repeater.latitude;
    let mut longitude = repeater.longitude;

    if latitude.is_none() || longitude.is_none() {
        if let Some(ref grid) = grid {
            if let Ok((grid_longitude, grid_latitude)) = grid_to_longlat(grid) {
                latitude = latitude.or(Some(grid_latitude));
                longitude = longitude.or(Some(grid_longitude));
            }
        }
    }

    let location = match (latitude, longitude) {
        (Some(latitude), Some(longitude)) => format!("{latitude}, {longitude}"),
        _ => "-".to_string(),
    };

    ResolvedSite {
        maidenhead: grid.unwrap_or_else(|| "-".to_string()),
        location,
        latitude,
        longitude,
    }
}

fn distance_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let earth_radius_km = 6_371.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius_km * c
}

pub async fn home(
    _: HomePath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;
    let repeaters = dao::repeater_system::select(&mut c).await?;
    let mut map_repeaters = Vec::new();

    for repeater in repeaters {
        let call_sign = repeater.call_sign.clone();
        let resolved = resolve_site_fields(&repeater);
        if let (Some(latitude), Some(longitude)) = (resolved.latitude, resolved.longitude) {
            map_repeaters.push(MapRepeater {
                call_sign,
                latitude,
                longitude,
            });
        }
    }

    let auth = auth_header(&jar, &state);
    let template = HomeTemplate {
        auth,
        repeater_data: map_repeaters,
    };
    let body = template.render()?;

    Ok(Html(body))
}

pub async fn repeaters(
    _: RepeatersPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeaters = dao::repeater_system::select(&mut c).await?;
    let mut items = Vec::with_capacity(repeaters.len());
    for repeater in repeaters {
        let resolved = resolve_site_fields(&repeater);
        let call_sign = repeater.call_sign.clone();
        let status = repeater.status.clone();
        let description = repeater.description.clone();

        items.push(RepeaterListItem {
            call_sign,
            status,
            description: description.unwrap_or_else(|| "-".to_string()),
            maidenhead: resolved.maidenhead,
            location: resolved.location,
        });
    }

    let auth = auth_header(&jar, &state);
    let template = RepeatersTemplate {
        auth,
        repeaters: items,
    };
    let body = template.render()?;

    Ok(Html(body))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/repeater/{call_sign}")]
pub struct RepeaterDetailPath {
    pub call_sign: String,
}

#[derive(Template)]
#[template(path = "pages/repeater_detail.html")]
struct DetailTemplate {
    auth: super::AuthHeader,
    repeater: dao::repeater_system::RepeaterSystem,
    ports: Vec<dao::repeater_port::RepeaterPort>,
    status: String,
    description: String,
    maidenhead: String,
    location: String,
    map_context: Option<MapContext>,
}

pub async fn detail(
    RepeaterDetailPath { call_sign }: RepeaterDetailPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeater = match dao::repeater_system::find_by_call_sign(&mut c, call_sign).await? {
        Some(row) => row,
        None => return Err(RepeaterAtlasError::NotFound),
    };

    let ports = dao::repeater_port::select_by_repeater_id(&mut c, repeater.id).await?;
    let status = repeater.status.clone();
    let description = repeater
        .description
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let resolved = resolve_site_fields(&repeater);
    let map_context =
        if let (Some(center_lat), Some(center_lon)) = (resolved.latitude, resolved.longitude) {
            let all_repeaters = dao::repeater_system::select(&mut c).await?;
            let mut nearby_repeaters = Vec::new();

            for candidate in all_repeaters {
                let candidate_resolved = resolve_site_fields(&candidate);
                if let (Some(lat), Some(lon)) =
                    (candidate_resolved.latitude, candidate_resolved.longitude)
                {
                    if distance_km(center_lat, center_lon, lat, lon) <= 50.0 {
                        nearby_repeaters.push(MapRepeater {
                            call_sign: candidate.call_sign,
                            latitude: lat,
                            longitude: lon,
                        });
                    }
                }
            }

            Some(MapContext {
                center: MapPoint {
                    latitude: center_lat,
                    longitude: center_lon,
                },
                radius_meters: 50_000,
                repeaters: nearby_repeaters,
            })
        } else {
            None
        };

    let auth = auth_header(&jar, &state);
    let template = DetailTemplate {
        auth,
        repeater,
        ports,
        status,
        description,
        maidenhead: resolved.maidenhead,
        location: resolved.location,
        map_context,
    };
    let body = template.render()?;

    Ok(Html(body))
}
