use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_service_c4fm)]
pub struct NewRepeaterServiceC4fm {
    pub service_id: i64,
    pub wires_x_node_id: Option<i32>,
    pub room: Option<String>,
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_c4fm: NewRepeaterServiceC4fm,
) -> QueryResult<usize> {
    use crate::schema::repeater_service_c4fm::dsl as c4fm;

    diesel::insert_into(c4fm::repeater_service_c4fm)
        .values(&new_c4fm)
        .execute(c)
        .await
}
