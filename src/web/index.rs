use super::AppState;
use super::auth::auth_header;
use crate::Frequency;
use crate::MaidenheadLocator;
use crate::dao::repeater_service::{AprsMode, DstarMode, FmBandwidth, SsbSideband};
use crate::repeater_service::RepeaterService;
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(TypedPath)]
#[typed_path("/")]
pub struct MapPath;

#[derive(Template)]
#[template(path = "pages/map.html")]
struct HomeTemplate {
    auth: super::AuthHeader,
    repeater_data: Vec<MapRepeater>,
}

#[derive(TypedPath)]
#[typed_path("/repeater")]
pub struct RepeaterListPath;

#[derive(Template)]
#[template(path = "pages/repeater_list.html")]
struct RepeatersTemplate {
    auth: super::AuthHeader,
    repeaters: Vec<RepeaterListItem>,
}

#[derive(TypedPath)]
#[typed_path("/organization")]
pub struct OrganizationListPath;

#[derive(Template)]
#[template(path = "pages/organization_list.html")]
struct OrganizationsTemplate {
    auth: super::AuthHeader,
    organizations: Vec<OrganizationListItem>,
}

struct OrganizationListItem {
    call_sign: Option<String>,
    display_name: String,
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
    status: String,
    services: Vec<String>,
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
        maidenhead: grid
            .map(|value| format!("{value}"))
            .unwrap_or_else(|| "-".to_string()),
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
    _: MapPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;
    let repeaters = dao::repeater_system::select_with_call_sign(&mut c).await?;
    let mut candidates = Vec::new();

    for repeater in repeaters {
        let resolved = resolve_site_fields(&repeater.system);
        if let (Some(latitude), Some(longitude)) = (resolved.latitude, resolved.longitude) {
            candidates.push((
                repeater.system.id,
                repeater.call_sign,
                repeater.system.status,
                latitude,
                longitude,
            ));
        }
    }

    let repeater_ids: Vec<i64> = candidates.iter().map(|(id, _, _, _, _)| *id).collect();
    let mut kinds_by_id: HashMap<i64, Vec<String>> = HashMap::new();
    for (repeater_id, kind) in
        dao::repeater_service::select_kinds_by_repeater_ids(&mut c, &repeater_ids).await?
    {
        kinds_by_id
            .entry(repeater_id)
            .or_default()
            .push(kind.label().to_string());
    }

    let mut map_repeaters = Vec::with_capacity(candidates.len());
    for (id, call_sign, status, latitude, longitude) in candidates {
        let mut services = kinds_by_id.remove(&id).unwrap_or_default();
        services.sort();
        services.dedup();

        map_repeaters.push(MapRepeater {
            call_sign,
            latitude,
            longitude,
            status,
            services,
        });
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
    _: RepeaterListPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    render_repeaters_list(jar, state).await
}

async fn render_repeaters_list(
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeaters = dao::repeater_system::select_with_call_sign(&mut c).await?;
    let mut items = Vec::with_capacity(repeaters.len());
    for repeater in repeaters {
        let resolved = resolve_site_fields(&repeater.system);
        let call_sign = repeater.call_sign.clone();
        let status = repeater.system.status.clone();
        let description = repeater.system.description.clone();

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
pub struct RepeaterDetailPagePath {
    pub call_sign: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/call-sign/{call_sign}")]
pub struct CallSignDetailPath {
    pub call_sign: String,
}

#[derive(Template)]
#[template(path = "pages/repeater.html")]
struct DetailTemplate {
    auth: super::AuthHeader,
    call_sign: String,
    repeater: dao::repeater_system::RepeaterSystem,
    owner: Option<ContactItem>,
    tech_contact: Option<ContactItem>,
    links: Vec<LinkedRepeaterItem>,
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

#[derive(Clone)]
struct ContactItem {
    display_name: String,
    call_sign: Option<String>,
}

#[derive(Clone)]
struct LinkedRepeaterItem {
    call_sign: String,
    note: String,
}

impl ContactItem {
    fn from(row: dao::contact::ContactWithCallSign) -> Self {
        Self {
            display_name: row.contact.display_name,
            call_sign: row.call_sign,
        }
    }
}

#[derive(Template)]
#[template(path = "pages/contact.html")]
struct ContactDetailTemplate {
    auth: super::AuthHeader,
    call_sign: String,
    contact: dao::contact::Contact,
    owned_repeaters: Vec<ContactRepeaterItem>,
    tech_contact_repeaters: Vec<ContactRepeaterItem>,
}

struct ContactRepeaterItem {
    call_sign: String,
    status: String,
    description: String,
}

pub async fn organizations(
    _: OrganizationListPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    render_organizations_list(jar, state).await
}

async fn render_organizations_list(
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let organizations = dao::contact::select_organizations_with_call_sign(&mut c)
        .await?
        .into_iter()
        .map(|row| OrganizationListItem {
            call_sign: row.call_sign,
            display_name: row.contact.display_name,
        })
        .collect::<Vec<_>>();

    let auth = auth_header(&jar, &state);
    let template = OrganizationsTemplate {
        auth,
        organizations,
    };
    let body = template.render()?;

    Ok(Html(body))
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
    RepeaterDetailPagePath { call_sign }: RepeaterDetailPagePath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let call_sign = call_sign.to_uppercase();
    render_repeater_detail(call_sign, jar, state).await
}

pub async fn call_sign(
    CallSignDetailPath { call_sign }: CallSignDetailPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let call_sign = call_sign.to_uppercase();

    let kind = {
        let mut c = state.pool.get().await?;
        dao::entity::find_by_call_sign(&mut c, call_sign.clone())
            .await?
            .map(|row| row.kind)
    };

    match kind {
        Some(dao::entity::EntityKind::Repeater) => {
            render_repeater_detail(call_sign, jar, state).await
        }
        Some(dao::entity::EntityKind::Contact) => {
            render_contact_detail(call_sign, jar, state).await
        }
        None => Err(RepeaterAtlasError::NotFound),
    }
}

async fn render_contact_detail(
    call_sign: String,
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let contact = match dao::contact::find_by_call_sign(&mut c, call_sign.clone()).await? {
        Some(row) => row,
        None => return Err(RepeaterAtlasError::NotFound),
    };

    let owned_repeaters = dao::repeater_system::select_with_call_sign_by_owner(&mut c, contact.id)
        .await?
        .into_iter()
        .map(|row| ContactRepeaterItem {
            call_sign: row.call_sign,
            status: row.system.status,
            description: row.system.description.unwrap_or_else(|| "-".to_string()),
        })
        .collect::<Vec<_>>();

    let tech_contact_repeaters =
        dao::repeater_system::select_with_call_sign_by_tech_contact(&mut c, contact.id)
            .await?
            .into_iter()
            .map(|row| ContactRepeaterItem {
                call_sign: row.call_sign,
                status: row.system.status,
                description: row.system.description.unwrap_or_else(|| "-".to_string()),
            })
            .collect::<Vec<_>>();

    let auth = auth_header(&jar, &state);
    let template = ContactDetailTemplate {
        auth,
        call_sign,
        contact,
        owned_repeaters,
        tech_contact_repeaters,
    };
    let body = template.render()?;

    Ok(Html(body))
}

async fn render_repeater_detail(
    call_sign: String,
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeater = match dao::repeater_system::find_by_call_sign(&mut c, call_sign.clone()).await? {
        Some(row) => row,
        None => return Err(RepeaterAtlasError::NotFound),
    };

    let owner = match repeater.owner {
        Some(contact_id) => dao::contact::find_with_call_sign(&mut c, contact_id)
            .await?
            .map(ContactItem::from),
        None => None,
    };

    let tech_contact = match repeater.tech_contact {
        Some(contact_id) => dao::contact::find_with_call_sign(&mut c, contact_id)
            .await?
            .map(ContactItem::from),
        None => None,
    };

    let links = dao::repeater_link::select_with_other_call_sign(&mut c, repeater.id)
        .await?
        .into_iter()
        .map(|row| LinkedRepeaterItem {
            call_sign: row.other_call_sign,
            note: row.note,
        })
        .collect();

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
                label,
                rx_hz,
                tx_hz,
                ..
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
            let all_repeaters = dao::repeater_system::select_with_call_sign(&mut c).await?;
            let mut nearby_repeaters = Vec::new();

            for candidate in all_repeaters {
                let candidate_resolved = resolve_site_fields(&candidate.system);
                if let (Some(lat), Some(lon)) =
                    (candidate_resolved.latitude, candidate_resolved.longitude)
                {
                    if distance_km(center_lat, center_lon, lat, lon) <= 50.0 {
                        nearby_repeaters.push(MapRepeater {
                            call_sign: candidate.call_sign,
                            latitude: lat,
                            longitude: lon,
                            status: candidate.system.status,
                            services: Vec::new(),
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
        call_sign,
        repeater,
        owner,
        tech_contact,
        links,
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
