use super::AppState;
use super::auth::auth_header;
use crate::service::logbook::{LogbookOptions, PageSize, generate_pdf, render_typst};
use crate::{RepeaterAtlasError, dao};
use axum::{
    Form,
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::Deserialize;

#[derive(TypedPath)]
#[typed_path("/-/my/logbook.pdf")]
pub struct LogbookPdfPath;

#[derive(TypedPath)]
#[typed_path("/-/my/logbook.typ")]
pub struct LogbookTypPath;

#[derive(Deserialize)]
pub struct LogbookForm {
    pub page_size: String,
    pub log_pages: Option<u32>,
    pub title_page: Option<String>,
    pub locations_page: Option<String>,
    pub phonetic_alphabet_page: Option<String>,
    pub frequency_bands_page: Option<String>,
}

pub async fn logbook_pdf(
    _: LogbookPdfPath,
    jar: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<LogbookForm>,
) -> Result<Response, RepeaterAtlasError> {
    let opts = build_opts(form, &jar, &state).await?;
    let opts = match opts {
        Ok(opts) => opts,
        Err(redirect) => return Ok(redirect),
    };

    let filename = format!("logbook-{}.pdf", opts.call_sign.to_lowercase());
    let pdf = generate_pdf(&opts).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("filename=\"{filename}\""))
            .unwrap_or(HeaderValue::from_static("")),
    );
    Ok((StatusCode::OK, headers, Body::from(pdf)).into_response())
}

pub async fn logbook_typ(
    _: LogbookTypPath,
    jar: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<LogbookForm>,
) -> Result<Response, RepeaterAtlasError> {
    let opts = build_opts(form, &jar, &state).await?;
    let opts = match opts {
        Ok(opts) => opts,
        Err(redirect) => return Ok(redirect),
    };

    let filename = format!("logbook-{}.typ", opts.call_sign.to_lowercase());
    let source = render_typst(&opts)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("filename=\"{filename}\""))
            .unwrap_or(HeaderValue::from_static("")),
    );
    Ok((StatusCode::OK, headers, Body::from(source)).into_response())
}

async fn build_opts(
    form: LogbookForm,
    jar: &CookieJar,
    state: &AppState,
) -> Result<Result<LogbookOptions, Response>, RepeaterAtlasError> {
    let auth = auth_header(jar, state);
    if !auth.logged_in {
        return Ok(Err(Redirect::to("/-/login").into_response()));
    }

    let page_size = match form.page_size.as_str() {
        "a5" => PageSize::A5,
        _ => PageSize::A4,
    };

    let mut c = state.pool.get().await?;
    let user = dao::user::find_by_call_sign(&mut c, auth.call_sign.clone())
        .await?
        .ok_or(RepeaterAtlasError::NotFound)?;
    let locations = dao::user_location::list_by_user(&mut c, user.id).await?;

    Ok(Ok(LogbookOptions {
        call_sign: auth.call_sign.clone(),
        page_size,
        log_pages: form.log_pages.unwrap_or(10).max(1).min(100),
        title_page: form.title_page.is_some(),
        locations_page: form.locations_page.is_some(),
        phonetic_alphabet_page: form.phonetic_alphabet_page.is_some(),
        frequency_bands_page: form.frequency_bands_page.is_some(),
        locations,
    }))
}
