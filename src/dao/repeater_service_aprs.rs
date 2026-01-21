use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::AprsMode)]
pub enum AprsMode {
    Igate,
    Digipeater,
}

impl ToSql<crate::schema::sql_types::AprsMode, Pg> for AprsMode {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            AprsMode::Igate => "igate",
            AprsMode::Digipeater => "digipeater",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::AprsMode, Pg> for AprsMode {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"igate" => Ok(AprsMode::Igate),
            b"digipeater" => Ok(AprsMode::Digipeater),
            _ => Err("unrecognized aprs_mode variant".into()),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_service_aprs)]
pub struct NewRepeaterServiceAprs {
    pub service_id: i64,
    pub mode: AprsMode,
    pub path: Option<String>,
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_aprs: NewRepeaterServiceAprs,
) -> QueryResult<usize> {
    use crate::schema::repeater_service_aprs::dsl as a;

    diesel::insert_into(a::repeater_service_aprs)
        .values(&new_aprs)
        .execute(c)
        .await
}
