use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::MaidenheadLocator;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_system)]
pub struct NewRepeaterSystem {
    pub ham_club_id: Option<i64>,
    pub call_sign: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub maidenhead: Option<MaidenheadLocator>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation_m: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub status: String,
}

impl NewRepeaterSystem {
    pub fn new(call_sign: impl Into<String>) -> Self {
        Self {
            ham_club_id: None,
            call_sign: call_sign.into(),
            name: None,
            description: None,
            address: None,
            maidenhead: None,
            latitude: None,
            longitude: None,
            elevation_m: None,
            country: None,
            region: None,
            status: "active".to_string(),
        }
    }

    pub fn ham_club_id(self, ham_club_id: i64) -> Self {
        Self {
            ham_club_id: Some(ham_club_id),
            ..self
        }
    }
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_repeater: NewRepeaterSystem,
) -> QueryResult<RepeaterSystem> {
    use crate::schema::repeater_system::dsl as r;

    diesel::insert_into(r::repeater_system)
        .values(&new_repeater)
        .returning(RepeaterSystem::as_returning())
        .get_result(c)
        .await
}

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::repeater_system)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterSystem {
    pub id: i64,
    pub ham_club_id: Option<i64>,
    pub call_sign: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub maidenhead: Option<MaidenheadLocator>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation_m: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn select(c: &mut AsyncPgConnection) -> QueryResult<Vec<RepeaterSystem>> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .select(RepeaterSystem::as_select())
        .order_by(r::call_sign.asc())
        .get_results(c)
        .await
}

pub async fn get(c: &mut AsyncPgConnection, repeater_id: i64) -> QueryResult<RepeaterSystem> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .filter(r::id.eq(repeater_id))
        .select(RepeaterSystem::as_select())
        .first(c)
        .await
}

pub async fn find_by_call_sign(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> QueryResult<Option<RepeaterSystem>> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .filter(r::call_sign.eq(call_sign))
        .select(RepeaterSystem::as_select())
        .first(c)
        .await
        .optional()
}

pub async fn update(
    c: &mut AsyncPgConnection,
    repeater: RepeaterSystem,
) -> QueryResult<RepeaterSystem> {
    use crate::schema::repeater_system::dsl as r;

    diesel::update(r::repeater_system.filter(r::id.eq(repeater.id)))
        .set(&repeater)
        .returning(RepeaterSystem::as_returning())
        .get_result(c)
        .await
}
