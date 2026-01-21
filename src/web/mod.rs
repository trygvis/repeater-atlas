use askama::Template;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;

pub mod auth;
pub mod index;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<AsyncPgConnection>,
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
