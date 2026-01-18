use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::routing::TypedPath;
use serde::Deserialize;

use super::AppState;
use crate::{dao, RepeaterAtlasError};

#[derive(TypedPath)]
#[typed_path("/")]
pub struct IndexPath;

#[derive(Template)]
#[template(path = "pages/index.html")]
struct IndexTemplate {
    repeaters: Vec<RepeaterListItem>,
}

struct RepeaterListItem {
    repeater: dao::repeater_system::RepeaterSystem,
    ports: Vec<dao::repeater_port::RepeaterPort>,
}

pub async fn index(
    _: IndexPath,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeaters = dao::repeater_system::select(&mut c).await?;
    let mut items = Vec::with_capacity(repeaters.len());
    for repeater in repeaters {
        let ports = dao::repeater_port::select_by_repeater_id(&mut c, repeater.id).await?;
        items.push(RepeaterListItem { repeater, ports });
    }

    let template = IndexTemplate { repeaters: items };
    let body = template.render()?;

    Ok(Html(body))
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/repeaters/{call_sign}")]
pub struct RepeaterDetailPath {
    pub call_sign: String,
}

#[derive(Template)]
#[template(path = "pages/repeater_detail.html")]
struct DetailTemplate {
    repeater: dao::repeater_system::RepeaterSystem,
    ports: Vec<dao::repeater_port::RepeaterPort>,
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

    let template = DetailTemplate { repeater, ports };
    let body = template.render()?;

    Ok(Html(body))
}
