use super::AppState;
use super::AuthHeader;
use super::auth::auth_header;
use crate::dao::user_location::UserLocation;
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared path / template
// ---------------------------------------------------------------------------

#[derive(TypedPath, Deserialize, Serialize)]
#[typed_path("/-/my/location/{id}")]
pub struct UserLocationPath {
    pub id: i64,
}

#[derive(Template)]
#[template(path = "partials/location_list.html")]
struct LocationListTemplate {
    locations: Vec<UserLocation>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub(crate) async fn add_location(
    _: AddLocationPath,
    jar: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<AddLocationForm>,
) -> Result<Response, RepeaterAtlasError> {
    let Some((_, user)) = require_user(&jar, &state).await? else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let address = form.address.filter(|s| !s.trim().is_empty());
    let maidenhead = form.maidenhead.filter(|s| !s.trim().is_empty());
    let latitude = parse_coord(&form.latitude);
    let longitude = parse_coord(&form.longitude);

    let (addr, mh, lat, lon) = resolve_location(address, maidenhead, latitude, longitude).await?;

    let mut c = state.pool.get().await?;
    dao::user_location::insert(
        &mut c,
        dao::user_location::NewUserLocation {
            user_id: user.id,
            address: addr,
            maidenhead: mh,
            latitude: lat,
            longitude: lon,
        },
    )
    .await?;

    let locations = dao::user_location::list_by_user(&mut c, user.id).await?;
    Ok(Html(LocationListTemplate { locations }.render()?).into_response())
}

#[derive(TypedPath)]
#[typed_path("/-/my/location")]
pub(crate) struct AddLocationPath;

#[derive(Deserialize)]
pub(crate) struct AddLocationForm {
    address: Option<String>,
    maidenhead: Option<String>,
    latitude: Option<String>,
    longitude: Option<String>,
}

pub(crate) async fn edit_location_form(
    path: EditLocationPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Response, RepeaterAtlasError> {
    let Some((_, user)) = require_user(&jar, &state).await? else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let mut c = state.pool.get().await?;
    let locations = dao::user_location::list_by_user(&mut c, user.id).await?;
    let Some(location) = locations.into_iter().find(|l| l.id == path.id) else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    Ok(Html(LocationEditFormTemplate { location }.render()?).into_response())
}

#[derive(TypedPath, Deserialize, Serialize)]
#[typed_path("/-/my/location/{id}/edit")]
pub(crate) struct EditLocationPath {
    id: i64,
}

#[derive(Template)]
#[template(path = "partials/location_edit_form.html")]
struct LocationEditFormTemplate {
    location: UserLocation,
}

pub(crate) async fn update_location(
    path: UserLocationPath,
    jar: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<UpdateLocationForm>,
) -> Result<Response, RepeaterAtlasError> {
    let Some((_, user)) = require_user(&jar, &state).await? else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let address = form.address.filter(|s| !s.trim().is_empty());
    let maidenhead = form.maidenhead.filter(|s| !s.trim().is_empty());
    let latitude = parse_coord(&form.latitude);
    let longitude = parse_coord(&form.longitude);

    let (addr, mh, lat, lon) = resolve_location(address, maidenhead, latitude, longitude).await?;

    let mut c = state.pool.get().await?;
    dao::user_location::update(&mut c, path.id, user.id, addr, mh, lat, lon).await?;

    let locations = dao::user_location::list_by_user(&mut c, user.id).await?;
    Ok(Html(LocationListTemplate { locations }.render()?).into_response())
}

#[derive(Deserialize)]
pub(crate) struct UpdateLocationForm {
    address: Option<String>,
    maidenhead: Option<String>,
    latitude: Option<String>,
    longitude: Option<String>,
}

pub(crate) async fn delete_location(
    path: UserLocationPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Response, RepeaterAtlasError> {
    let Some((_, user)) = require_user(&jar, &state).await? else {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    };

    let mut c = state.pool.get().await?;
    dao::user_location::delete(&mut c, path.id, user.id).await?;

    let locations = dao::user_location::list_by_user(&mut c, user.id).await?;
    Ok(Html(LocationListTemplate { locations }.render()?).into_response())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn require_user(
    jar: &CookieJar,
    state: &AppState,
) -> Result<Option<(AuthHeader, dao::user::User)>, RepeaterAtlasError> {
    let auth = auth_header(jar, state);
    if !auth.logged_in {
        return Ok(None);
    }
    let mut c = state.pool.get().await?;
    let user = dao::user::find_by_call_sign(&mut c, auth.call_sign.clone()).await?;
    Ok(user.map(|u| (auth, u)))
}

fn parse_coord(s: &Option<String>) -> Option<f64> {
    s.as_deref()
        .filter(|v| !v.trim().is_empty())
        .and_then(|v| v.trim().parse().ok())
}

async fn resolve_location(
    address: Option<String>,
    maidenhead: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<(Option<String>, Option<String>, Option<f64>, Option<f64>), RepeaterAtlasError> {
    use crate::MaidenheadLocator;
    use crate::service::enrich_location::enrich_location;
    use crate::service::geocoding::nominatim_geocoder_from_env;

    // If lat/lon provided directly, derive maidenhead and return.
    if let (Some(lat), Some(lon)) = (latitude, longitude) {
        let mh = maidenhead::longlat_to_grid(lon, lat, 6)
            .ok()
            .and_then(|g| MaidenheadLocator::new(g).ok())
            .map(|m| m.as_str().to_string());
        return Ok((address, mh, Some(lat), Some(lon)));
    }

    let mh = maidenhead
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(MaidenheadLocator::new)
        .transpose()
        .map_err(|e| RepeaterAtlasError::OtherMsg(format!("invalid maidenhead: {e}")))?;

    let geocoder = nominatim_geocoder_from_env()?;
    let enriched = enrich_location(geocoder, "", address, mh).await?;

    Ok((
        enriched.address,
        enriched.maidenhead.map(|m| m.as_str().to_string()),
        enriched.point.map(|p| p.latitude),
        enriched.point.map(|p| p.longitude),
    ))
}
