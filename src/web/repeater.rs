use super::AppState;
use super::auth::auth_header;
use super::map::{MapContext, MapPoint, MapRepeater, OrganizationMapContext};
use super::utils::distance_km;
use crate::dao::repeater_service::{AprsMode, DstarMode, FmBandwidth, SsbSideband};
use crate::service::repeater_service::RepeaterService;
use crate::{Frequency, service};
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::Deserialize;

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

struct NearbyRepeaterItem {
    call_sign: String,
    distance_km: f64,
    distance_label: String,
}

struct OrganizationRepeaterItem {
    call_sign: String,
    status: String,
    description: Option<String>,
    services: ServiceItems,
}

struct ServiceItems {
    fm_services: Vec<FmServiceItem>,
    dmr_services: Vec<DmrServiceItem>,
    dstar_services: Vec<DstarServiceItem>,
    c4fm_services: Vec<C4fmServiceItem>,
    aprs_services: Vec<AprsServiceItem>,
    ssb_services: Vec<SsbServiceItem>,
    am_services: Vec<AmServiceItem>,
}

struct FmServiceItem {
    label: String,
    enabled: bool,
    rx_hz: Frequency,
    tx_hz: Frequency,
    bandwidth: FmBandwidth,
    rx_tone: service::repeater_service::Tone,
    tx_tone: service::repeater_service::Tone,
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

impl From<dao::contact::Contact> for ContactItem {
    fn from(row: dao::contact::Contact) -> Self {
        Self {
            display_name: row.display_name,
            call_sign: row.call_sign,
        }
    }
}

fn build_service_items(services: Vec<dao::repeater_service::RepeaterServiceDao>) -> ServiceItems {
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

    ServiceItems {
        fm_services,
        dmr_services,
        dstar_services,
        c4fm_services,
        aprs_services,
        ssb_services,
        am_services,
    }
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/repeater/{call_sign}")]
pub struct RepeaterDetailPagePath {
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
    nearby_repeaters: Vec<NearbyRepeaterItem>,
    fm_services: Vec<FmServiceItem>,
    dmr_services: Vec<DmrServiceItem>,
    dstar_services: Vec<DstarServiceItem>,
    c4fm_services: Vec<C4fmServiceItem>,
    aprs_services: Vec<AprsServiceItem>,
    ssb_services: Vec<SsbServiceItem>,
    am_services: Vec<AmServiceItem>,
    status: String,
    description: Option<String>,
    maidenhead: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    map_context: Option<MapContext>,
}

pub async fn detail(
    RepeaterDetailPagePath { call_sign }: RepeaterDetailPagePath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let call_sign = call_sign.to_uppercase();
    render_repeater_detail(call_sign, jar, state).await
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/call-sign/{call_sign}")]
pub struct CallSignDetailPath {
    pub call_sign: String,
}

pub async fn call_sign(
    CallSignDetailPath { call_sign }: CallSignDetailPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let call_sign = call_sign.to_uppercase();

    let kind = {
        let mut c = state.pool.get().await?;
        dao::call_sign::find_by_call_sign(&mut c, call_sign.clone())
            .await?
            .map(|row| row.kind)
    };

    match kind {
        Some(dao::call_sign::CallSignKind::Repeater) => {
            render_repeater_detail(call_sign, jar, state).await
        }
        Some(dao::call_sign::CallSignKind::Contact) => {
            render_contact_detail(call_sign, jar, state).await
        }
        None => Err(RepeaterAtlasError::NotFound),
    }
}

#[derive(Template)]
#[template(path = "pages/contact.html")]
struct ContactDetailTemplate {
    auth: super::AuthHeader,
    call_sign: String,
    contact: dao::contact::Contact,
    repeaters: Vec<OrganizationRepeaterItem>,
    map_context: Option<OrganizationMapContext>,
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

    let mut repeaters_by_call_sign = std::collections::BTreeMap::new();
    for repeater in dao::repeater_system::select_with_call_sign_by_owner(&mut c, contact.id).await?
    {
        repeaters_by_call_sign
            .entry(repeater.call_sign.clone())
            .or_insert(repeater);
    }
    for repeater in
        dao::repeater_system::select_with_call_sign_by_tech_contact(&mut c, contact.id).await?
    {
        repeaters_by_call_sign
            .entry(repeater.call_sign.clone())
            .or_insert(repeater);
    }

    let mut repeaters = Vec::with_capacity(repeaters_by_call_sign.len());
    let mut map_repeaters = Vec::new();

    for (_call_sign, repeater) in repeaters_by_call_sign {
        let services = dao::repeater_service::select_by_repeater_id(&mut c, repeater.id).await?;
        let service_items = build_service_items(services);
        if let (Some(latitude), Some(longitude)) = (repeater.latitude, repeater.longitude) {
            map_repeaters.push(MapRepeater {
                call_sign: repeater.call_sign.clone(),
                latitude,
                longitude,
                status: repeater.status.clone(),
                services: Vec::new(),
            });
        }

        repeaters.push(OrganizationRepeaterItem {
            call_sign: repeater.call_sign.clone(),
            status: repeater.status.clone(),
            description: repeater.description.clone(),
            services: service_items,
        });
    }

    let map_context = if map_repeaters.is_empty() {
        None
    } else {
        Some(OrganizationMapContext {
            repeaters: map_repeaters,
        })
    };

    let auth = auth_header(&jar, &state);
    let template = ContactDetailTemplate {
        auth,
        call_sign,
        contact,
        repeaters,
        map_context,
    };
    let body = template.render()?;

    Ok(Html(body))
}

async fn render_repeater_detail(
    call_sign: String,
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    const NEARBY_RADIUS_METERS: f64 = 50_000.0;
    let mut c = state.pool.get().await?;

    let Some(repeater) = dao::repeater_system::find_by_call_sign(&mut c, call_sign.clone()).await?
    else {
        return Err(RepeaterAtlasError::NotFound);
    };

    let owner = match repeater.owner {
        Some(contact_id) => Some(dao::contact::get(&mut c, contact_id).await?.into()),
        None => None,
    };

    let tech_contact = match repeater.tech_contact {
        Some(contact_id) => Some(dao::contact::get(&mut c, contact_id).await?.into()),
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
    let service_items = build_service_items(services);
    let ServiceItems {
        fm_services,
        dmr_services,
        dstar_services,
        c4fm_services,
        aprs_services,
        ssb_services,
        am_services,
    } = service_items;
    let status = repeater.status.clone();
    let description = repeater.description.clone();
    let maidenhead = repeater.maidenhead.as_ref().map(|value| value.to_string());
    let latitude = repeater.latitude;
    let longitude = repeater.longitude;
    let (map_context, nearby_repeaters) =
        if let (Some(center_lat), Some(center_lon)) = (latitude, longitude) {
            let all_repeaters = dao::repeater_system::select_within_radius(
                &mut c,
                center_lat,
                center_lon,
                NEARBY_RADIUS_METERS,
            )
            .await?;
            let mut nearby_repeaters = Vec::new();
            let mut nearby_list = Vec::new();

            for candidate in all_repeaters {
                if let (Some(lat), Some(lon)) = (candidate.latitude, candidate.longitude) {
                    let distance = distance_km(center_lat, center_lon, lat, lon);
                    nearby_list.push(NearbyRepeaterItem {
                        call_sign: candidate.call_sign.clone(),
                        distance_km: distance,
                        distance_label: format!("{distance:.1} km"),
                    });

                    nearby_repeaters.push(MapRepeater {
                        call_sign: candidate.call_sign,
                        latitude: lat,
                        longitude: lon,
                        status: candidate.status,
                        services: Vec::new(),
                    });
                }
            }

            nearby_list.sort_by(|a, b| {
                a.distance_km
                    .partial_cmp(&b.distance_km)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.call_sign.cmp(&b.call_sign))
            });

            (
                Some(MapContext {
                    center: MapPoint {
                        latitude: center_lat,
                        longitude: center_lon,
                    },
                    radius_meters: NEARBY_RADIUS_METERS as u32,
                    repeaters: nearby_repeaters,
                }),
                nearby_list,
            )
        } else {
            (None, Vec::new())
        };

    let auth = auth_header(&jar, &state);
    let template = DetailTemplate {
        auth,
        call_sign,
        repeater,
        owner,
        tech_contact,
        links,
        nearby_repeaters,
        fm_services,
        dmr_services,
        dstar_services,
        c4fm_services,
        aprs_services,
        ssb_services,
        am_services,
        status,
        description,
        maidenhead,
        latitude,
        longitude,
        map_context,
    };
    let body = template.render()?;

    Ok(Html(body))
}
