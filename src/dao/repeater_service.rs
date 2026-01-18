use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::RepeaterServiceKind)]
pub enum RepeaterServiceKind {
    Fm,
    Am,
    Ssb,
    Dstar,
    Dmr,
    C4fm,
    Aprs,
}

impl ToSql<crate::schema::sql_types::RepeaterServiceKind, Pg> for RepeaterServiceKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            RepeaterServiceKind::Fm => "fm",
            RepeaterServiceKind::Am => "am",
            RepeaterServiceKind::Ssb => "ssb",
            RepeaterServiceKind::Dstar => "dstar",
            RepeaterServiceKind::Dmr => "dmr",
            RepeaterServiceKind::C4fm => "c4fm",
            RepeaterServiceKind::Aprs => "aprs",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::RepeaterServiceKind, Pg> for RepeaterServiceKind {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"fm" => Ok(RepeaterServiceKind::Fm),
            b"am" => Ok(RepeaterServiceKind::Am),
            b"ssb" => Ok(RepeaterServiceKind::Ssb),
            b"dstar" => Ok(RepeaterServiceKind::Dstar),
            b"dmr" => Ok(RepeaterServiceKind::Dmr),
            b"c4fm" => Ok(RepeaterServiceKind::C4fm),
            b"aprs" => Ok(RepeaterServiceKind::Aprs),
            _ => Err("unrecognized repeater_service_kind variant".into()),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_service)]
pub struct NewRepeaterService {
    pub repeater_id: i64,
    pub port_id: Option<i64>,
    pub kind: RepeaterServiceKind,
    pub enabled: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::repeater_service)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterService {
    pub id: i64,
    pub repeater_id: i64,
    pub port_id: Option<i64>,
    pub kind: RepeaterServiceKind,
    pub enabled: bool,
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_service: NewRepeaterService,
) -> QueryResult<RepeaterService> {
    use crate::schema::repeater_service::dsl as s;

    diesel::insert_into(s::repeater_service)
        .values(&new_service)
        .returning(RepeaterService::as_returning())
        .get_result(c)
        .await
}
