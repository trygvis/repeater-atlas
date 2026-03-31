mod utils;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use dao::call_sign::NewCallSign;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use http_body_util::BodyExt;
use repeater_atlas::dao;
use repeater_atlas::schema::app_user;
use repeater_atlas::schema::call_sign;
use repeater_atlas::web::{AppState, create_router};
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tower::util::ServiceExt;
use uuid::Uuid;

static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[tokio::test]
async fn call_sign_routes_resolve_repeater_and_contact()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = TEST_MUTEX.lock().await;
    let pool = utils::pool().await;
    let cleanup_pool = pool.clone();
    let mut c = pool.get().await?;

    // Use a random suffix so rerunning the test doesn't trip the global unique index.
    let suffix = Uuid::new_v4().simple().to_string().to_uppercase();
    let contact_call_sign = format!("RAO{}", &suffix[..8]);
    let repeater_call_sign = format!("RAR{}", &suffix[..8]);

    let contact_call_sign_row =
        dao::call_sign::insert(&mut c, NewCallSign::new_contact(contact_call_sign.clone())).await?;

    let contact = dao::contact::insert(
        &mut c,
        dao::contact::NewContact {
            call_sign: Some(contact_call_sign_row.value),
            kind: dao::contact::ContactKind::Organization,
            display_name: "RA-09f3 Org".to_string(),
            description: None,
            web_url: None,
            email: None,
            phone: None,
            address: None,
        },
    )
    .await?;

    let repeater_call_sign_row =
        dao::call_sign::insert(&mut c, NewCallSign::new_repeater(&repeater_call_sign)).await?;

    let repeater = dao::repeater_system::insert(
        &mut c,
        dao::repeater_system::NewRepeaterSystem::new(repeater_call_sign_row.value)
            .owner(contact.id),
    )
    .await?;

    drop(c);

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let repeater_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{repeater_call_sign}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(repeater_response.status(), StatusCode::OK);
    let repeater_body = repeater_response.into_body().collect().await?.to_bytes();
    let repeater_html = String::from_utf8_lossy(&repeater_body);
    assert!(
        repeater_html.contains(format!("<h1>{repeater_call_sign}</h1>").as_str()),
        "expected repeater detail page to render call sign"
    );

    let contact_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/{contact_call_sign}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(contact_response.status(), StatusCode::OK);
    let contact_body = contact_response.into_body().collect().await?.to_bytes();
    let contact_html = String::from_utf8_lossy(&contact_body);
    assert!(
        contact_html.contains(format!("<h1>{contact_call_sign}</h1>").as_str()),
        "expected contact page to render call sign"
    );

    let orgs_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/-/organization")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(orgs_response.status(), StatusCode::OK);
    let orgs_body = orgs_response.into_body().collect().await?.to_bytes();
    let orgs_html = String::from_utf8_lossy(&orgs_body);
    assert!(
        orgs_html.contains(contact_call_sign.as_str()),
        "expected organizations list to include the contact call sign"
    );

    let repeaters_response = app
        .clone()
        .oneshot(Request::builder().uri("/-/repeater").body(Body::empty())?)
        .await?;
    assert_eq!(repeaters_response.status(), StatusCode::OK);

    // Cleanup: delete only the call_sign rows we created. This cascades to
    // (call_sign -> contact/repeater_system -> dependent rows).
    let mut c = cleanup_pool.get().await?;
    diesel::delete(call_sign::table.filter(call_sign::value.eq(repeater_call_sign)))
        .execute(&mut c)
        .await?;
    diesel::delete(call_sign::table.filter(call_sign::value.eq(contact_call_sign)))
        .execute(&mut c)
        .await?;

    // Sanity: ensure the test repeater row is gone.
    let repeater_exists: i64 = repeater_atlas::schema::repeater_system::table
        .filter(repeater_atlas::schema::repeater_system::id.eq(repeater.id))
        .count()
        .get_result(&mut c)
        .await?;
    assert_eq!(repeater_exists, 0);

    Ok(())
}

#[tokio::test]
async fn signup_creates_user_and_rejects_invalid_email()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = TEST_MUTEX.lock().await;
    let pool = utils::pool().await;
    let cleanup_pool = pool.clone();
    let suffix = Uuid::new_v4().simple().to_string().to_uppercase();
    let call_sign = format!("LA{}", &suffix[..6]);
    let email = format!("{}@example.org", suffix[..8].to_lowercase());

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let signup_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/-/signup")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(format!(
                    "call_sign={call_sign}&email={email}&password=password123"
                )))?,
        )
        .await?;

    assert_eq!(signup_response.status(), StatusCode::SEE_OTHER);
    let set_cookie = signup_response
        .headers()
        .get("set-cookie")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    assert!(
        set_cookie.contains("ra_auth="),
        "expected signup to set the auth cookie"
    );

    let mut c = cleanup_pool.get().await?;
    let created_user: i64 = app_user::table
        .filter(app_user::call_sign.eq(&call_sign))
        .count()
        .get_result(&mut c)
        .await?;
    assert_eq!(created_user, 1);

    let invalid_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/-/signup")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "call_sign=LA1ABC&email=not-an-email&password=password123",
                ))?,
        )
        .await?;

    assert_eq!(invalid_response.status(), StatusCode::OK);
    let invalid_body = invalid_response.into_body().collect().await?.to_bytes();
    let invalid_html = String::from_utf8_lossy(&invalid_body);
    assert!(
        invalid_html.contains("Invalid email address"),
        "expected signup validation message for invalid email"
    );

    diesel::delete(app_user::table.filter(app_user::call_sign.eq(&call_sign)))
        .execute(&mut c)
        .await?;

    Ok(())
}
