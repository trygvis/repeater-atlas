use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_link)]
pub struct NewRepeaterLink {
    pub repeater_a_id: i64,
    pub repeater_b_id: i64,
    pub note: String,
}

impl NewRepeaterLink {
    pub fn new(repeater_a_id: i64, repeater_b_id: i64) -> Self {
        Self {
            repeater_a_id,
            repeater_b_id,
            note: String::new(),
        }
    }
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::repeater_link)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterLink {
    pub id: i64,
    pub repeater_a_id: i64,
    pub repeater_b_id: i64,
    pub note: String,
}

pub async fn insert(c: &mut AsyncPgConnection, link: NewRepeaterLink) -> QueryResult<usize> {
    use crate::schema::repeater_link::dsl as l;

    diesel::insert_into(l::repeater_link)
        .values(&link)
        .on_conflict((l::repeater_a_id, l::repeater_b_id))
        .do_nothing()
        .execute(c)
        .await
}

pub async fn select_by_repeater_id(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
) -> QueryResult<Vec<RepeaterLink>> {
    use crate::schema::repeater_link::dsl as l;

    l::repeater_link
        .filter(l::repeater_a_id.eq(repeater_id).or(l::repeater_b_id.eq(repeater_id)))
        .select(RepeaterLink::as_select())
        .order_by((l::repeater_a_id.asc(), l::repeater_b_id.asc()))
        .get_results(c)
        .await
}

