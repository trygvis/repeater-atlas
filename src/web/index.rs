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
    repeaters: Vec<dao::repeater::Repeater>,
}

pub async fn index(
    _: IndexPath,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeaters = dao::repeater::select(&mut c).await?;

    let template = IndexTemplate { repeaters };
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
    repeater: dao::repeater::Repeater,
}

pub async fn detail(
    RepeaterDetailPath { call_sign }: RepeaterDetailPath,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeater = match dao::repeater::find_by_call_sign(&mut c, call_sign).await? {
        Some(row) => row,
        None => return Err(RepeaterAtlasError::NotFound),
    };

    let template = DetailTemplate { repeater };
    let body = template.render()?;

    Ok(Html(body))
}
