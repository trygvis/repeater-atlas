mod utils;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum_extra::routing::RouterExt;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use http_body_util::BodyExt;
use repeater_atlas::dao;
use repeater_atlas::schema::entity;
use repeater_atlas::web::{AppState, index};
use tower::util::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn call_sign_routes_resolve_repeater_and_contact()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = utils::pool().await;
    let cleanup_pool = pool.clone();
    let mut c = pool.get().await?;

    // Use a random suffix so rerunning the test doesn't trip the global unique index.
    let suffix = Uuid::new_v4().simple().to_string().to_uppercase();
    let contact_call_sign = format!("RAO{}", &suffix[..8]);
    let repeater_call_sign = format!("RAR{}", &suffix[..8]);

    let contact_entity = dao::entity::insert(
        &mut c,
        dao::entity::NewEntity {
            kind: dao::entity::EntityKind::Contact,
            call_sign: Some(contact_call_sign.clone()),
        },
    )
    .await?;

    let contact = dao::contact::insert(
        &mut c,
        dao::contact::NewContact {
            entity: contact_entity.id,
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

    let repeater_entity = dao::entity::insert(
        &mut c,
        dao::entity::NewEntity {
            kind: dao::entity::EntityKind::Repeater,
            call_sign: Some(repeater_call_sign.clone()),
        },
    )
    .await?;

    let repeater = dao::repeater_system::insert(
        &mut c,
        dao::repeater_system::NewRepeaterSystem::new(repeater_entity.id).owner(contact.id),
    )
    .await?;

    drop(c);

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = Router::new()
        .typed_get(index::home)
        .typed_get(index::repeaters)
        .typed_get(index::organizations)
        .typed_get(index::call_sign)
        .typed_get(index::detail)
        .with_state(state);

    let repeater_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/call-sign/{repeater_call_sign}"))
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
                .uri(format!("/call-sign/{contact_call_sign}"))
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
                .uri("/organization")
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
        .oneshot(Request::builder().uri("/repeater").body(Body::empty())?)
        .await?;
    assert_eq!(repeaters_response.status(), StatusCode::OK);

    let legacy_detail_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/repeater/{repeater_call_sign}"))
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(legacy_detail_response.status(), StatusCode::OK);

    // Cleanup: delete only the entity rows we created. This cascades to
    // (entity -> contact/repeater_system -> dependent rows).
    let mut c = cleanup_pool.get().await?;
    diesel::delete(entity::table.filter(entity::id.eq(repeater_entity.id)))
        .execute(&mut c)
        .await?;
    diesel::delete(entity::table.filter(entity::id.eq(contact_entity.id)))
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
