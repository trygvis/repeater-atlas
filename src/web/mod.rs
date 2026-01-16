use askama::Template;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::AsyncPgConnection;

pub mod index;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<AsyncPgConnection>,
}

#[derive(Template)]
#[template(path = "pages/500.html")]
struct ErrorTemplate;

#[derive(Template)]
#[template(path = "pages/404.html")]
struct NotFoundTemplate;

pub fn render_500() -> String {
    ErrorTemplate
        .render()
        .unwrap_or_else(|_| "<h1>Server Error</h1>".to_string())
}

pub fn render_404() -> String {
    NotFoundTemplate
        .render()
        .unwrap_or_else(|_| "<h1>Not Found</h1>".to_string())
}
