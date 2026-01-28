use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::EntityKind)]
pub enum EntityKind {
    Repeater,
    Contact,
}

impl ToSql<crate::schema::sql_types::EntityKind, Pg> for EntityKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            EntityKind::Repeater => "repeater",
            EntityKind::Contact => "contact",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::EntityKind, Pg> for EntityKind {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"repeater" => Ok(EntityKind::Repeater),
            b"contact" => Ok(EntityKind::Contact),
            _ => Err("unrecognized entity_kind variant".into()),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::entity)]
pub struct NewEntity {
    pub kind: EntityKind,
    pub call_sign: Option<String>,
}

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::entity)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Entity {
    pub id: i64,
    pub kind: EntityKind,
    pub call_sign: Option<String>,
}

pub async fn insert(c: &mut AsyncPgConnection, new_entity: NewEntity) -> QueryResult<Entity> {
    use crate::schema::entity::dsl as e;

    diesel::insert_into(e::entity)
        .values(&new_entity)
        .returning(Entity::as_returning())
        .get_result(c)
        .await
}

pub async fn get(c: &mut AsyncPgConnection, entity_id: i64) -> QueryResult<Entity> {
    use crate::schema::entity::dsl as e;

    e::entity
        .filter(e::id.eq(entity_id))
        .select(Entity::as_select())
        .first(c)
        .await
}

pub async fn find_by_call_sign(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> QueryResult<Option<Entity>> {
    use crate::schema::entity::dsl as e;

    e::entity
        .filter(e::call_sign.eq(call_sign))
        .select(Entity::as_select())
        .first(c)
        .await
        .optional()
}
