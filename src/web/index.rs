use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::routing::TypedPath;
use maidenhead::grid_to_longlat;
use serde::Deserialize;

use super::AppState;
use crate::{dao, RepeaterAtlasError};

#[derive(TypedPath)]
#[typed_path("/")]
pub struct HomePath;

#[derive(Template)]
#[template(path = "pages/index.html")]
struct HomeTemplate;

#[derive(TypedPath)]
#[typed_path("/repeater")]
pub struct RepeatersPath;

#[derive(Template)]
#[template(path = "pages/repeaters.html")]
struct RepeatersTemplate {
    repeaters: Vec<RepeaterListItem>,
}

struct RepeaterListItem {
    call_sign: String,
    status: String,
    description: String,
    maidenhead: String,
    location: String,
}

struct ResolvedSite {
    maidenhead: String,
    location: String,
}

fn resolve_site_fields(site: Option<dao::repeater_site::RepeaterSite>) -> ResolvedSite {
    if let Some(site) = site {
        let grid = site.maidenhead.clone();
        let mut latitude = site.latitude;
        let mut longitude = site.longitude;

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

        return ResolvedSite {
            maidenhead: grid.unwrap_or_else(|| "-".to_string()),
            location,
        };
    }

    ResolvedSite {
        maidenhead: "-".to_string(),
        location: "-".to_string(),
    }
}

pub async fn home(_: HomePath) -> Result<Html<String>, RepeaterAtlasError> {
    let template = HomeTemplate;
    let body = template.render()?;

    Ok(Html(body))
}

pub async fn repeaters(
    _: RepeatersPath,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeaters = dao::repeater_system::select(&mut c).await?;
    let mut items = Vec::with_capacity(repeaters.len());
    for repeater in repeaters {
        let dao::repeater_system::RepeaterSystem {
            call_sign,
            status,
            description,
            site_id,
            ..
        } = repeater;

        let site = match site_id {
            Some(site_id) => Some(dao::repeater_site::get(&mut c, site_id).await?),
            None => None,
        };

        let resolved = resolve_site_fields(site);

        items.push(RepeaterListItem {
            call_sign,
            status,
            description: description.unwrap_or_else(|| "-".to_string()),
            maidenhead: resolved.maidenhead,
            location: resolved.location,
        });
    }

    let template = RepeatersTemplate { repeaters: items };
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
    repeater: dao::repeater_system::RepeaterSystem,
    ports: Vec<dao::repeater_port::RepeaterPort>,
    status: String,
    description: String,
    maidenhead: String,
    location: String,
}

pub async fn detail(
    RepeaterDetailPath { call_sign }: RepeaterDetailPath,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeater = match dao::repeater_system::find_by_call_sign(&mut c, call_sign).await? {
        Some(row) => row,
        None => return Err(RepeaterAtlasError::NotFound),
    };

    let ports = dao::repeater_port::select_by_repeater_id(&mut c, repeater.id).await?;
    let status = repeater.status.clone();
    let description = repeater.description.clone().unwrap_or_else(|| "-".to_string());
    let site = match repeater.site_id {
        Some(site_id) => Some(dao::repeater_site::get(&mut c, site_id).await?),
        None => None,
    };
    let resolved = resolve_site_fields(site);

    let template = DetailTemplate {
        repeater,
        ports,
        status,
        description,
        maidenhead: resolved.maidenhead,
        location: resolved.location,
    };
    let body = template.render()?;

    Ok(Html(body))
}
