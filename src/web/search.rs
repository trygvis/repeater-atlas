use super::AppState;
use crate::{RepeaterAtlasError, dao};
use axum::{Json, extract::Query, extract::State};
use axum_extra::routing::TypedPath;
use serde::{Deserialize, Serialize};

const SEARCH_LIMIT: i64 = 50;

#[derive(TypedPath)]
#[typed_path("/-/search")]
pub struct CallSignSearchPath;

#[derive(Deserialize)]
pub struct CallSignSearchQuery {
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct CallSignSearchResult {
    pub call_sign: String,
    pub kind: String,
}

#[derive(Serialize)]
pub struct CallSignSearchResponse {
    pub results: Vec<CallSignSearchResult>,
}

pub async fn call_sign_search(
    _: CallSignSearchPath,
    State(state): State<AppState>,
    Query(query): Query<CallSignSearchQuery>,
) -> Result<Json<CallSignSearchResponse>, RepeaterAtlasError> {
    let Some(raw_query) = query.q else {
        return Ok(Json(CallSignSearchResponse {
            results: Vec::new(),
        }));
    };

    let normalized = raw_query.trim();
    if normalized.is_empty() {
        return Ok(Json(CallSignSearchResponse {
            results: Vec::new(),
        }));
    }

    let prefix = normalized.to_uppercase();
    let mut c = state.pool.get().await?;
    let rows = dao::call_sign::search_by_prefix(&mut c, prefix, SEARCH_LIMIT).await?;
    let results = rows
        .into_iter()
        .map(|row| CallSignSearchResult {
            call_sign: row.value,
            kind: call_sign_kind_label(row.kind).to_string(),
        })
        .collect();

    Ok(Json(CallSignSearchResponse { results }))
}

fn call_sign_kind_label(kind: dao::call_sign::CallSignKind) -> &'static str {
    match kind {
        dao::call_sign::CallSignKind::Repeater => "repeater",
        dao::call_sign::CallSignKind::Contact => "organization",
    }
}
