use super::AppState;
use super::auth::auth_header;
use super::map::{MapContext, MapPoint, MapRepeater, OrganizationMapContext};
use super::utils::distance_km;
use crate::service;
use crate::service::repeater_system::{Repeater, ServiceItems};
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::Deserialize;

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
    repeater: Repeater,
    links: Vec<LinkedRepeaterItem>,
    nearby_repeaters: Vec<NearbyRepeaterItem>,
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

    let owned_repeaters =
        dao::repeater_system::select_with_call_sign_by_owner(&mut c, contact.id).await?;
    let tech_contact_repeaters =
        dao::repeater_system::select_with_call_sign_by_tech_contact(&mut c, contact.id).await?;

    // let mut repeaters_by_call_sign = std::collections::BTreeMap::new();
    let mut repeaters = Vec::with_capacity(owned_repeaters.len() + tech_contact_repeaters.len());
    let mut map_repeaters = Vec::new();

    for repeater in owned_repeaters
        .into_iter()
        .chain(tech_contact_repeaters.into_iter())
    {
        let repeater = service::repeater_system::load(&mut c, repeater).await?;

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
            services: repeater.services,
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

    let repeater = service::repeater_system::load_by_call_sign(&mut c, call_sign.clone()).await?;

    let links = dao::repeater_link::select_with_other_call_sign(&mut c, repeater.id)
        .await?
        .into_iter()
        .map(|row| LinkedRepeaterItem {
            call_sign: row.other_call_sign,
            note: row.note,
        })
        .collect();

    let status = repeater.status.clone();
    let description = repeater.description.clone();
    let maidenhead = repeater.maidenhead.as_ref().map(|value| value.to_string());
    let latitude = repeater.latitude.clone();
    let longitude = repeater.longitude.clone();
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
        links,
        nearby_repeaters,
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
