use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::app_user)]
pub struct NewUser {
    pub call_sign: String,
    pub email: String,
    pub password_hash: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::app_user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i64,
    pub call_sign: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn find_by_call_sign(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> QueryResult<Option<User>> {
    use crate::schema::app_user::dsl as u;

    u::app_user
        .filter(u::call_sign.eq(call_sign))
        .select(User::as_select())
        .first(c)
        .await
        .optional()
}

pub async fn insert(c: &mut AsyncPgConnection, new_user: NewUser) -> QueryResult<User> {
    use crate::schema::app_user::dsl as u;

    diesel::insert_into(u::app_user)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(c)
        .await
}
