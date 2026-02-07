use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::ContactKind)]
pub enum ContactKind {
    Organization,
    Individual,
}

impl ToSql<crate::schema::sql_types::ContactKind, Pg> for ContactKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            ContactKind::Organization => "organization",
            ContactKind::Individual => "individual",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::ContactKind, Pg> for ContactKind {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"organization" => Ok(ContactKind::Organization),
            b"individual" => Ok(ContactKind::Individual),
            _ => Err("unrecognized contact_kind variant".into()),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::contact)]
pub struct NewContact {
    pub call_sign: Option<String>,
    pub kind: ContactKind,
    pub display_name: String,
    pub description: Option<String>,
    pub web_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::contact)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Contact {
    pub id: i64,
    pub call_sign: Option<String>,
    pub kind: ContactKind,
    pub display_name: String,
    pub description: Option<String>,
    pub web_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}

pub async fn insert(c: &mut AsyncPgConnection, new_contact: NewContact) -> QueryResult<Contact> {
    use crate::schema::contact::dsl as ct;

    diesel::insert_into(ct::contact)
        .values(&new_contact)
        .returning(Contact::as_returning())
        .get_result(c)
        .await
}

pub async fn get(c: &mut AsyncPgConnection, contact_id: i64) -> QueryResult<Contact> {
    use crate::schema::contact::dsl as ct;

    ct::contact
        .filter(ct::id.eq(contact_id))
        .select(Contact::as_select())
        .first(c)
        .await
}

pub async fn find_by_call_sign(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> QueryResult<Option<Contact>> {
    use crate::schema::contact::dsl as ct;

    ct::contact
        .filter(ct::call_sign.eq(call_sign))
        .select(Contact::as_select())
        .first(c)
        .await
        .optional()
}

pub async fn select_organizations_with_call_sign(
    c: &mut AsyncPgConnection,
) -> QueryResult<Vec<Contact>> {
    use crate::schema::contact::dsl as ct;

    ct::contact
        .filter(ct::kind.eq(ContactKind::Organization))
        .select(Contact::as_select())
        .order_by(ct::call_sign.asc())
        .get_results(c)
        .await
}
