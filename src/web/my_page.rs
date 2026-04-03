use super::auth::auth_header;
use super::{AppState, AuthHeader};
use crate::dao::user_location::UserLocation;
use crate::{RepeaterAtlasError, dao};
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
    locations: Vec<UserLocation>,
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

    let mut c = state.pool.get().await?;
    let user = dao::user::find_by_call_sign(&mut c, auth.call_sign.clone())
        .await?
        .ok_or(RepeaterAtlasError::NotFound)?;
    let locations = dao::user_location::list_by_user(&mut c, user.id).await?;

    let template = MyPageTemplate { auth, locations };
    Ok(Html(template.render()?).into_response())
}
