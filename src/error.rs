use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use diesel_async::pooled_connection::PoolError;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum RepeaterAtlasError {
    #[error("database error")]
    Database(#[from] diesel::result::Error),

    #[error("database error")]
    DatabaseOther(#[source] diesel::result::Error, String),

    #[error("database pool error")]
    Pool(#[from] bb8::RunError<PoolError>),

    #[error("template error")]
    Template(#[from] askama::Error),

    #[error("jwt error")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("csv error")]
    Csv(#[from] csv::Error),

    #[error("not found")]
    NotFound,

    #[error("io error")]
    Io(#[from] std::io::Error),

    #[error("other error")]
    Other(#[source] Box<dyn std::error::Error>, String),
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
            RepeaterAtlasError::NotFound => {
                info!(error = ?RepeaterAtlasError::NotFound, "Not found");
                Self::render_404().into_response()
            }
            RepeaterAtlasError::Database(error) => {
                warn!(error = ?error, "Database error");
                Self::render_500().into_response()
            }
            RepeaterAtlasError::DatabaseOther(error, msg) => {
                warn!(error = ?error, "Database error: {msg}");
                Self::render_500().into_response()
            }
            RepeaterAtlasError::Pool(error) => {
                warn!(error = ?error, "Database pool error");
                Self::render_500().into_response()
            }
            RepeaterAtlasError::Template(error) => {
                warn!(error = ?error, "Template error");
                Self::render_500().into_response()
            }
            RepeaterAtlasError::Jwt(error) => {
                warn!(error = ?error, "JWT error");
                Self::render_500().into_response()
            }
            RepeaterAtlasError::Csv(error) => {
                warn!(error = ?error, "Csv error");
                Self::render_500().into_response()
            }
            RepeaterAtlasError::Io(error) => {
                warn!(error = ?error, "IO error");
                Self::render_500().into_response()
            }
            RepeaterAtlasError::Other(error, msg) => {
                warn!(error = ?error, "Other error: {msg}");
                Self::render_500().into_response()
            }
        }
    }
}
