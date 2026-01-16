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

#[derive(Template)]
#[template(path = "pages/500.html")]
struct ErrorTemplate;

fn render_500() -> (StatusCode, Html<String>) {
    let body = ErrorTemplate
        .render()
        .unwrap_or_else(|_| "<h1>Server Error</h1>".to_string());
    (StatusCode::INTERNAL_SERVER_ERROR, Html(body))
}

#[derive(Template)]
#[template(path = "pages/404.html")]
struct NotFoundTemplate;

fn render_404() -> (StatusCode, Html<String>) {
    let body = NotFoundTemplate
        .render()
        .unwrap_or_else(|_| "<h1>Not Found</h1>".to_string());
    (StatusCode::NOT_FOUND, Html(body))
}

pub async fn index(
    _: IndexPath,
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let mut conn = match state.pool.get().await {
        Ok(conn) => conn,
        Err(_) => return Err(render_500()),
    };

    let repeaters = match dao::repeater::select(&mut conn).await {
        Ok(rows) => rows,
        Err(_) => return Err(render_500()),
    };

    let template = IndexTemplate { repeaters };
    let body = template
        .render()
        .map_err(|_| render_500())?;

    Ok(Html(body))
}

pub async fn detail(
    RepeaterDetailPath { call_sign }: RepeaterDetailPath,
    State(state): State<AppState>,
) -> Result<Html<String>, (StatusCode, Html<String>)> {
    let mut conn = match state.pool.get().await {
        Ok(conn) => conn,
        Err(_) => return Err(render_500()),
    };

    let repeater = match dao::repeater::find_by_call_sign(&mut conn, call_sign).await {
        Ok(Some(row)) => row,
        Ok(None) => return Err(render_404()),
        Err(_) => return Err(render_500()),
    };

    let template = DetailTemplate { repeater };
    let body = template
        .render()
        .map_err(|_| render_500())?;

    Ok(Html(body))
}
