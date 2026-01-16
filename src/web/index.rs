use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};
use axum_extra::routing::TypedPath;
use serde::Deserialize;

use super::AppState;
use crate::dao;

#[derive(TypedPath)]
#[typed_path("/")]
pub struct IndexPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/repeaters/{call_sign}")]
pub struct RepeaterDetailPath {
    pub call_sign: String,
}

#[derive(Template)]
#[template(path = "pages/index.html")]
struct IndexTemplate {
    repeaters: Vec<dao::repeater::Repeater>,
}

#[derive(Template)]
#[template(path = "pages/repeater_detail.html")]
struct DetailTemplate {
    repeater: dao::repeater::Repeater,
}

pub async fn index(
    _: IndexPath,
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db pool error: {}", err),
        )
    })?;

    let repeaters = dao::repeater::select(&mut conn).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db query error: {}", err),
        )
    })?;

    let template = IndexTemplate { repeaters };
    let body = template
        .render()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Html(body))
}

pub async fn detail(
    RepeaterDetailPath { call_sign }: RepeaterDetailPath,
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, String)> {
    let mut conn = state.pool.get().await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db pool error: {}", err),
        )
    })?;

    let repeater = dao::repeater::get_by_call_sign(&mut conn, call_sign).await.map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db query error: {}", err),
        )
    })?;

    let template = DetailTemplate { repeater };
    let body = template
        .render()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Html(body))
}
