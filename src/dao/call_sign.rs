use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::CallSignKind)]
pub enum CallSignKind {
    Repeater,
    Contact,
}

impl ToSql<crate::schema::sql_types::CallSignKind, Pg> for CallSignKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            CallSignKind::Repeater => "repeater",
            CallSignKind::Contact => "contact",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::CallSignKind, Pg> for CallSignKind {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"repeater" => Ok(CallSignKind::Repeater),
            b"contact" => Ok(CallSignKind::Contact),
            _ => Err("unrecognized call_sign_kind variant".into()),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::call_sign)]
pub struct NewCallSign {
    pub value: String,
    pub kind: CallSignKind,
}

impl NewCallSign {
    pub fn new_repeater(call_sign: impl Into<String>) -> Self {
        Self {
            kind: CallSignKind::Repeater,
            value: call_sign.into(),
        }
    }

    pub fn new_contact(call_sign: impl Into<String>) -> Self {
        Self {
            kind: CallSignKind::Contact,
            value: call_sign.into(),
        }
    }
}

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::call_sign)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CallSign {
    pub value: String,
    pub kind: CallSignKind,
}

pub async fn insert(c: &mut AsyncPgConnection, call_sign: NewCallSign) -> QueryResult<CallSign> {
    use crate::schema::call_sign::dsl as cs;

    diesel::insert_into(cs::call_sign)
        .values(&call_sign)
        .returning(CallSign::as_returning())
        .get_result(c)
        .await
}

pub async fn find_by_call_sign(
    c: &mut AsyncPgConnection,
    value: String,
) -> QueryResult<Option<CallSign>> {
    use crate::schema::call_sign::dsl as cs;

    cs::call_sign
        .filter(cs::value.eq(value))
        .select(CallSign::as_select())
        .first(c)
        .await
        .optional()
}
