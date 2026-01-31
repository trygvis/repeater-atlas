use super::AppState;
use super::auth::auth_header;
use crate::{RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;

struct OrganizationListItem {
    call_sign: Option<String>,
    display_name: String,
}

#[derive(TypedPath)]
#[typed_path("/organization")]
pub struct OrganizationListPath;

#[derive(Template)]
#[template(path = "pages/organization_list.html")]
struct OrganizationsTemplate {
    auth: super::AuthHeader,
    organizations: Vec<OrganizationListItem>,
}

pub async fn organizations(
    _: OrganizationListPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    render_organizations_list(jar, state).await
}

async fn render_organizations_list(
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let organizations = dao::contact::select_organizations_with_call_sign(&mut c)
        .await?
        .into_iter()
        .map(|row| OrganizationListItem {
            call_sign: row.call_sign,
            display_name: row.display_name,
        })
        .collect::<Vec<_>>();

    let auth = auth_header(&jar, &state);
    let template = OrganizationsTemplate {
        auth,
        organizations,
    };
    let body = template.render()?;

    Ok(Html(body))
}
