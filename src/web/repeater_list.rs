use super::AppState;
use super::auth::auth_header;
use crate::{Point, RepeaterAtlasError, dao};
use askama::Template;
use axum::{extract::State, response::Html};
use axum_extra::extract::cookie::CookieJar;
use axum_extra::routing::TypedPath;

struct RepeaterListItem {
    call_sign: String,
    status: String,
    description: Option<String>,
    maidenhead: Option<String>,
    point: Option<Point>,
}

#[derive(TypedPath)]
#[typed_path("/repeater")]
pub struct RepeaterListPath;

#[derive(Template)]
#[template(path = "pages/repeater_list.html")]
struct RepeatersTemplate {
    auth: super::AuthHeader,
    repeaters: Vec<RepeaterListItem>,
}

pub async fn repeaters(
    _: RepeaterListPath,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<Html<String>, RepeaterAtlasError> {
    render_repeaters_list(jar, state).await
}

async fn render_repeaters_list(
    jar: CookieJar,
    state: AppState,
) -> Result<Html<String>, RepeaterAtlasError> {
    let mut c = state.pool.get().await?;

    let repeaters = dao::repeater_system::select_with_call_sign(&mut c).await?;
    let mut items = Vec::with_capacity(repeaters.len());
    for repeater in repeaters {
        let call_sign = repeater.call_sign.clone();
        let status = repeater.status.clone();

        items.push(RepeaterListItem {
            call_sign,
            status,
            description: repeater.description.clone(),
            maidenhead: repeater.maidenhead.as_ref().map(|value| value.to_string()),
            point: repeater.location(),
        });
    }

    let auth = auth_header(&jar, &state);
    let template = RepeatersTemplate {
        auth,
        repeaters: items,
    };
    let body = template.render()?;

    Ok(Html(body))
}
