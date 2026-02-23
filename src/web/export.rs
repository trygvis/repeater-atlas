use super::AppState;
use super::auth::auth_header;
use crate::RepeaterAtlasError;
use crate::service::export::{ExportOptions, chirp};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;

#[derive(TypedPath)]
#[typed_path("/-/export/chirp.csv")]
pub struct ChirpExportPath;

pub async fn chirp_export(
    _: ChirpExportPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Response, RepeaterAtlasError> {
    let auth = auth_header(&jar, &state);
    if !auth.logged_in {
        return Ok(Redirect::to("/-/login").into_response());
    }

    let mut c = state.pool.get().await?;
    let mut body = Vec::new();
    chirp::chirp_export(&mut c, ExportOptions::default(), &mut body).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"repeater-atlas-chirp.csv\""),
    );

    Ok((StatusCode::OK, headers, Body::from(body)).into_response())
}
