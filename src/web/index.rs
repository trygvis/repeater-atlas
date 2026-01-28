use super::AppState;
use super::auth::auth_header;
use crate::dao::repeater_service::{AprsMode, DstarMode, FmBandwidth, SsbSideband};
use crate::repeater_service::RepeaterService;
use crate::Frequency;
use crate::MaidenheadLocator;
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
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
        maidenhead: grid.map(|value| format!("{value}")).unwrap_or_else(|| "-".to_string()),
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
    fm_services: Vec<FmServiceItem>,
    dmr_services: Vec<DmrServiceItem>,
    dstar_services: Vec<DstarServiceItem>,
    c4fm_services: Vec<C4fmServiceItem>,
    aprs_services: Vec<AprsServiceItem>,
    ssb_services: Vec<SsbServiceItem>,
    am_services: Vec<AmServiceItem>,
    status: String,
    description: String,
    maidenhead: String,
    location: String,
    map_context: Option<MapContext>,
}

struct FmServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    bandwidth: FmBandwidth,
    rx_tone: crate::repeater_service::Tone,
    tx_tone: crate::repeater_service::Tone,
    note: String,
}

struct DmrServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    color_code: i16,
    dmr_repeater_id: Option<i64>,
    network: String,
    note: String,
}

struct DstarServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    mode: DstarMode,
    gateway_call_sign: Option<String>,
    reflector: Option<String>,
    note: String,
}

struct C4fmServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    wires_x_node_id: Option<i32>,
    room: Option<String>,
    note: String,
}

struct AprsServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    mode: Option<AprsMode>,
    path: Option<String>,
    note: String,
}

struct SsbServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    sideband: Option<SsbSideband>,
    note: String,
}

struct AmServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    note: String,
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

    let services = dao::repeater_service::select_by_repeater_id(&mut c, repeater.id).await?;
    let mut fm_services = Vec::new();
    let mut dmr_services = Vec::new();
    let mut dstar_services = Vec::new();
    let mut c4fm_services = Vec::new();
    let mut aprs_services = Vec::new();
    let mut ssb_services = Vec::new();
    let mut am_services = Vec::new();

    for row in services {
        let enabled = row.enabled;
        let note = row.note.clone();
        let service = RepeaterService::from(row);

        match service {
            RepeaterService::Fm {
                label,
                rx_hz,
                tx_hz,
                bandwidth,
                rx_tone,
                tx_tone,
                ..
            } => fm_services.push(FmServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                bandwidth,
                rx_tone,
                tx_tone,
                note,
            }),
            RepeaterService::Dmr {
                label,
                rx_hz,
                tx_hz,
                color_code,
                dmr_repeater_id,
                network,
                ..
            } => dmr_services.push(DmrServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                color_code,
                dmr_repeater_id,
                network,
                note,
            }),
            RepeaterService::Dstar {
                label,
                rx_hz,
                tx_hz,
                mode,
                gateway_call_sign,
                reflector,
                ..
            } => dstar_services.push(DstarServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                mode,
                gateway_call_sign,
                reflector,
                note,
            }),
            RepeaterService::C4fm {
                label,
                rx_hz,
                tx_hz,
                wires_x_node_id,
                room,
                ..
            } => c4fm_services.push(C4fmServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                wires_x_node_id,
                room,
                note,
            }),
            RepeaterService::Aprs {
                label,
                rx_hz,
                tx_hz,
                mode,
                path,
                ..
            } => aprs_services.push(AprsServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                mode,
                path,
                note,
            }),
            RepeaterService::Ssb {
                label,
                rx_hz,
                tx_hz,
                sideband,
                ..
            } => ssb_services.push(SsbServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                sideband,
                note,
            }),
            RepeaterService::Am {
                label, rx_hz, tx_hz, ..
            } => am_services.push(AmServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                note,
            }),
        }
    }
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
        fm_services,
        dmr_services,
        dstar_services,
        c4fm_services,
        aprs_services,
        ssb_services,
        am_services,
        status,
        description,
        maidenhead: resolved.maidenhead,
        location: resolved.location,
        map_context,
    };
    let body = template.render()?;

    Ok(Html(body))
}
