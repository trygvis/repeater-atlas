use super::AppState;
use super::auth::auth_header;
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct MapRepeater {
    pub call_sign: String,
    pub point: MapPoint,
    pub status: String,
    pub services: Vec<String>,
    pub is_external: bool,
}

#[derive(Clone, Serialize)]
pub struct MapPoint {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize)]
pub struct MapContext {
    pub center: MapPoint,
    pub radius_meters: u32,
    pub repeaters: Vec<MapRepeater>,
}

#[derive(Serialize)]
pub struct MapLink {
    pub from: MapPoint,
    pub to: MapPoint,
    pub from_call_sign: String,
    pub to_call_sign: String,
}

#[derive(Serialize)]
pub struct OrganizationMapContext {
    pub repeaters: Vec<MapRepeater>,
    pub links: Vec<MapLink>,
}

#[derive(Serialize)]
pub struct LinkedMapContext {
    pub repeaters: Vec<MapRepeater>,
    pub links: Vec<MapLink>,
}

#[derive(TypedPath)]
#[typed_path("/")]
pub struct MapPath;

#[derive(Template)]
#[template(path = "pages/map.html")]
struct MapTemplate {
    auth: super::AuthHeader,
    repeater_data: Vec<MapRepeater>,
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
        if let (Some(latitude), Some(longitude)) = (repeater.latitude, repeater.longitude) {
            candidates.push((
                repeater.id,
                repeater.call_sign,
                repeater.status,
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
            point: MapPoint {
                latitude,
                longitude,
            },
            status,
            services,
            is_external: false,
        });
    }

    let auth = auth_header(&jar, &state);
    let template = MapTemplate {
        auth,
        repeater_data: map_repeaters,
    };
    let body = template.render()?;

    Ok(Html(body))
}
