use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::DstarMode)]
pub enum DstarMode {
    Dv,
    Dd,
}

impl ToSql<crate::schema::sql_types::DstarMode, Pg> for DstarMode {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            DstarMode::Dv => "dv",
            DstarMode::Dd => "dd",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::DstarMode, Pg> for DstarMode {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"dv" => Ok(DstarMode::Dv),
            b"dd" => Ok(DstarMode::Dd),
            _ => Err("unrecognized dstar_mode variant".into()),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_service_dstar)]
pub struct NewRepeaterServiceDstar {
    pub service_id: i64,
    pub mode: DstarMode,
    pub gateway_call_sign: Option<String>,
    pub reflector: Option<String>,
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_dstar: NewRepeaterServiceDstar,
) -> QueryResult<usize> {
    use crate::schema::repeater_service_dstar::dsl as d;

    diesel::insert_into(d::repeater_service_dstar)
        .values(&new_dstar)
        .execute(c)
        .await
}
