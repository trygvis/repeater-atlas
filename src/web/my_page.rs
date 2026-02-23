use super::auth::auth_header;
use super::{AppState, AuthHeader};
use crate::RepeaterAtlasError;
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;

#[derive(TypedPath)]
#[typed_path("/-/my")]
pub struct MyPagePath;

#[derive(Template)]
#[template(path = "pages/my_page.html")]
struct MyPageTemplate {
    auth: AuthHeader,
}

pub async fn my_page(
    _: MyPagePath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Response, RepeaterAtlasError> {
    let auth = auth_header(&jar, &state);
    if !auth.logged_in {
        return Ok(Redirect::to("/-/login").into_response());
    }

    let template = MyPageTemplate { auth };
    Ok(Html(template.render()?).into_response())
}
