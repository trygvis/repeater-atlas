use crate::AppPool;
use askama::Template;
use axum::Router;
use axum_extra::routing::RouterExt;

pub mod auth;
pub mod export;
pub mod map;
pub mod my_page;
pub mod organization_list;
pub mod repeater;
pub mod repeater_list;
pub mod search;
pub mod user_location;
pub mod utils;

#[derive(Clone)]
pub struct AppState {
    pub pool: AppPool,
    pub jwt_secret: String,
}

#[derive(Clone)]
pub struct AuthHeader {
    pub logged_in: bool,
    pub call_sign: String,
}

impl AuthHeader {
    pub fn anonymous() -> Self {
        Self {
            logged_in: false,
            call_sign: String::new(),
        }
    }

    pub fn logged_in(call_sign: String) -> Self {
        Self {
            logged_in: true,
            call_sign,
        }
    }
}

#[derive(Template)]
#[template(path = "pages/500.html")]
struct ErrorTemplate {
    auth: AuthHeader,
}

#[derive(Template)]
#[template(path = "pages/404.html")]
struct NotFoundTemplate {
    auth: AuthHeader,
}

pub fn render_500() -> String {
    ErrorTemplate {
        auth: AuthHeader::anonymous(),
    }
    .render()
    .unwrap_or_else(|_| "<h1>Server Error</h1>".to_string())
}

pub fn render_404() -> String {
    NotFoundTemplate {
        auth: AuthHeader::anonymous(),
    }
    .render()
    .unwrap_or_else(|_| "<h1>Not Found</h1>".to_string())
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .typed_get(map::home)
        .typed_get(repeater_list::repeaters)
        .typed_get(organization_list::organizations)
        .typed_get(repeater::call_sign)
        .typed_get(search::call_sign_search)
        .typed_get(auth::login_form)
        .typed_post(auth::login_submit)
        .typed_post(auth::signup_submit)
        .typed_get(auth::logout)
        .typed_get(my_page::my_page)
        .typed_post(user_location::add_location)
        .typed_get(user_location::edit_location_form)
        .typed_put(user_location::update_location)
        .typed_delete(user_location::delete_location)
        .typed_get(export::chirp_export)
        .with_state(state)
}
