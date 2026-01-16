use diesel::prelude::*;

use crate::schema::repeater;

#[derive(Insertable)]
#[diesel(table_name = repeater)]
pub struct NewRepeater {
    pub call_sign: String,
    pub frequency: i64,
    pub rx_offset: i64,
}

pub fn insert(conn: &mut PgConnection, new_repeater: NewRepeater) -> QueryResult<usize> {
    diesel::insert_into(repeater::table)
        .values(&new_repeater)
        .execute(conn)
}
