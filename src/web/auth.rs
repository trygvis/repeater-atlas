use super::{AppState, AuthHeader};
use crate::auth;
use crate::{RepeaterAtlasError, dao};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use askama::Template;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::Deserialize;

#[derive(TypedPath)]
#[typed_path("/-/login")]
pub struct LoginPagePath;

#[derive(Deserialize)]
pub struct LoginForm {
    pub call_sign: String,
    pub password: String,
}

#[derive(Template)]
#[template(path = "pages/login.html")]
struct LoginTemplate {
    auth: AuthHeader,
    error: Option<String>,
    call_sign: String,
}

pub async fn login_form(
    _: LoginPagePath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    let auth = auth_header(&jar, &state);
    let template = LoginTemplate {
        auth,
        error: None,
        call_sign: String::new(),
    };
    Ok(Html(template.render()?))
}

pub async fn login_submit(
    _: LoginPagePath,
    jar: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Response, RepeaterAtlasError> {
    let call_sign = auth::normalize_call_sign(&form.call_sign);
    let mut c = state.pool.get().await?;

    let user = match dao::user::find_by_call_sign(&mut c, call_sign.clone()).await? {
        Some(user) => user,
        None => return Ok(login_error(jar, &state, call_sign, "Invalid credentials").await?),
    };

    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(_) => return Ok(login_error(jar, &state, call_sign, "Invalid credentials").await?),
    };
    if Argon2::default()
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(login_error(jar, &state, call_sign, "Invalid credentials").await?);
    }

    let token = auth::encode_token(&user.call_sign, &state.jwt_secret)?;
    let jar = jar.add(auth::build_auth_cookie(token));

    Ok((jar, Redirect::to("/")).into_response())
}

pub async fn logout(_: LogoutActionPath, jar: CookieJar) -> Result<Response, RepeaterAtlasError> {
    let jar = jar.add(auth::build_logout_cookie());
    Ok((jar, Redirect::to("/")).into_response())
}

#[derive(TypedPath)]
#[typed_path("/-/logout")]
pub struct LogoutActionPath;

async fn login_error(
    jar: CookieJar,
    state: &AppState,
    call_sign: String,
    message: &str,
) -> Result<Response, RepeaterAtlasError> {
    let auth = auth_header(&jar, state);
    let template = LoginTemplate {
        auth,
        error: Some(message.to_string()),
        call_sign,
    };
    Ok(Html(template.render()?).into_response())
}

pub fn auth_header(jar: &CookieJar, state: &AppState) -> AuthHeader {
    let Some(cookie) = jar.get(auth::AUTH_COOKIE_NAME) else {
        return AuthHeader::anonymous();
    };

    let claims = match auth::decode_token(cookie.value(), &state.jwt_secret) {
        Ok(claims) => claims,
        Err(_) => return AuthHeader::anonymous(),
    };

    AuthHeader::logged_in(claims.sub)
}
