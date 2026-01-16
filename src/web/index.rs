use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html};

use super::AppState;
use crate::dao;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    repeaters: Vec<dao::repeater::Repeater>,
}

pub async fn index(State(state): State<AppState>) -> Result<Html<String>, (StatusCode, String)> {
    let mut conn = state
        .pool
        .get()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("db pool error: {}", err)))?;

    let repeaters = dao::repeater::select(&mut conn)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("db query error: {}", err)))?;

    let template = IndexTemplate { repeaters };
    let body = template
        .render()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Ok(Html(body))
}
