use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::user_location)]
pub struct NewUserLocation {
    pub user_id: i64,
    pub address: Option<String>,
    pub maidenhead: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::user_location)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserLocation {
    pub id: i64,
    pub user_id: i64,
    pub address: Option<String>,
    pub maidenhead: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_by_user(
    c: &mut AsyncPgConnection,
    user_id: i64,
) -> QueryResult<Vec<UserLocation>> {
    use crate::schema::user_location::dsl as ul;

    ul::user_location
        .filter(ul::user_id.eq(user_id))
        .order(ul::created_at.asc())
        .select(UserLocation::as_select())
        .load(c)
        .await
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_location: NewUserLocation,
) -> QueryResult<UserLocation> {
    use crate::schema::user_location::dsl as ul;

    diesel::insert_into(ul::user_location)
        .values(&new_location)
        .returning(UserLocation::as_returning())
        .get_result(c)
        .await
}

pub async fn update(
    c: &mut AsyncPgConnection,
    id: i64,
    user_id: i64,
    address: Option<String>,
    maidenhead: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> QueryResult<UserLocation> {
    use crate::schema::user_location::dsl as ul;

    diesel::update(ul::user_location.filter(ul::id.eq(id).and(ul::user_id.eq(user_id))))
        .set((
            ul::address.eq(address),
            ul::maidenhead.eq(maidenhead),
            ul::latitude.eq(latitude),
            ul::longitude.eq(longitude),
            ul::updated_at.eq(diesel::dsl::now),
        ))
        .returning(UserLocation::as_returning())
        .get_result(c)
        .await
}

pub async fn delete(c: &mut AsyncPgConnection, id: i64, user_id: i64) -> QueryResult<usize> {
    use crate::schema::user_location::dsl as ul;

    diesel::delete(ul::user_location.filter(ul::id.eq(id).and(ul::user_id.eq(user_id))))
        .execute(c)
        .await
}
