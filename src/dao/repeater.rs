use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater)]
pub struct NewRepeater {
    pub call_sign: String,
    pub frequency: i64,
    pub rx_offset: i64,
}

#[derive(HasQuery)]
#[diesel(table_name = crate::schema::repeater)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Repeater {
    pub id: i64,
    pub call_sign: String,
    pub frequency: i64,
    pub rx_offset: i64,
}

pub async fn insert(conn: &mut AsyncPgConnection, new_repeater: NewRepeater) -> QueryResult<usize> {
    use crate::schema::repeater::dsl as r;

    diesel::insert_into(r::repeater)
        .values(&new_repeater)
        .execute(conn)
        .await
}

pub async fn select(conn: &mut AsyncPgConnection) -> QueryResult<Vec<Repeater>> {
    use crate::schema::repeater::dsl as r;

    r::repeater
        .select(Repeater::as_select())
        .get_results(conn)
        .await
}
