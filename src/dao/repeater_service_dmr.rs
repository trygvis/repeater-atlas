use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_service_dmr)]
pub struct NewRepeaterServiceDmr {
    pub service_id: i64,
    pub color_code: Option<i16>,
    pub dmr_repeater_id: Option<i64>,
    pub network: Option<String>,
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_dmr: NewRepeaterServiceDmr,
) -> QueryResult<usize> {
    use crate::schema::repeater_service_dmr::dsl as d;

    diesel::insert_into(d::repeater_service_dmr)
        .values(&new_dmr)
        .execute(c)
        .await
}
