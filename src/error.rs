use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use diesel_async::pooled_connection::PoolError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepeaterAtlasError {
    #[error("database error")]
    Database(#[from] diesel::result::Error),
    #[error("database pool error")]
    Pool(#[from] bb8::RunError<PoolError>),
    #[error("template error")]
    Template(#[from] askama::Error),
    #[error("not found")]
    NotFound,
}

impl RepeaterAtlasError {
    fn render_500() -> (StatusCode, Html<String>) {
        let body = crate::web::render_500();
        (StatusCode::INTERNAL_SERVER_ERROR, Html(body))
    }

    fn render_404() -> (StatusCode, Html<String>) {
        let body = crate::web::render_404();
        (StatusCode::NOT_FOUND, Html(body))
    }
}

impl IntoResponse for RepeaterAtlasError {
    fn into_response(self) -> Response {
        match self {
            RepeaterAtlasError::NotFound => Self::render_404().into_response(),
            _ => Self::render_500().into_response(),
        }
    }
}
