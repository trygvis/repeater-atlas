use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::MaidenheadLocator;
use crate::dao::entity::{EntityKind};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_system)]
pub struct NewRepeaterSystem {
    pub entity: i64,
    pub owner: Option<i64>,
    pub tech_contact: Option<i64>,
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
    pub fn new(entity: i64) -> Self {
        Self {
            entity,
            owner: None,
            tech_contact: None,
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

    pub fn owner(self, owner: i64) -> Self {
        Self {
            owner: Some(owner),
            ..self
        }
    }

    pub fn tech_contact(self, tech_contact: i64) -> Self {
        Self {
            tech_contact: Some(tech_contact),
            ..self
        }
    }
}

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::repeater_system)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterSystem {
    pub id: i64,
    pub entity: i64,
    pub owner: Option<i64>,
    pub tech_contact: Option<i64>,
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

#[derive(Clone)]
pub struct RepeaterSystemWithCallSign {
    pub system: RepeaterSystem,
    pub call_sign: String,
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
    use crate::schema::entity::dsl as e;
    use crate::schema::repeater_system::dsl as r;

    let entity_id: Option<i64> = e::entity
        .filter(e::call_sign.eq(call_sign))
        .filter(e::kind.eq(EntityKind::Repeater))
        .select(e::id)
        .first::<i64>(c)
        .await
        .optional()?;
    let Some(entity_id) = entity_id else {
        return Ok(None);
    };

    r::repeater_system
        .filter(r::entity.eq(entity_id))
        .select(RepeaterSystem::as_select())
        .first(c)
        .await
        .optional()
}

pub async fn select_with_call_sign(
    c: &mut AsyncPgConnection,
) -> QueryResult<Vec<RepeaterSystemWithCallSign>> {
    use crate::schema::entity::dsl as e;
    use crate::schema::repeater_system::dsl as r;

    let rows: Vec<(RepeaterSystem, Option<String>)> = r::repeater_system
        .inner_join(e::entity.on(e::id.eq(r::entity)))
        .filter(e::kind.eq(EntityKind::Repeater))
        .select((RepeaterSystem::as_select(), e::call_sign))
        .order_by(e::call_sign.asc())
        .get_results(c)
        .await?;

    Ok(rows
        .into_iter()
        .map(|(system, call_sign)| RepeaterSystemWithCallSign {
            system,
            call_sign: call_sign.unwrap_or_else(|| "<missing>".to_string()),
        })
        .collect())
}
