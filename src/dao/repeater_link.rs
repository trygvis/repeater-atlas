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

#[derive(Clone)]
pub struct RepeaterLinkWithOtherCallSign {
    pub other_repeater_id: i64,
    pub other_call_sign: String,
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
        .filter(
            l::repeater_a_id
                .eq(repeater_id)
                .or(l::repeater_b_id.eq(repeater_id)),
        )
        .select(RepeaterLink::as_select())
        .order_by((l::repeater_a_id.asc(), l::repeater_b_id.asc()))
        .get_results(c)
        .await
}

pub async fn select_with_other_call_sign(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
) -> QueryResult<Vec<RepeaterLinkWithOtherCallSign>> {
    use crate::schema::repeater_link::dsl as l;
    use crate::schema::repeater_system::dsl as rs;

    // Two queries keeps this simple in Diesel: one for "A is self", one for "B is self".
    let a_rows: Vec<(i64, String, String)> = l::repeater_link
        .filter(l::repeater_a_id.eq(repeater_id))
        .inner_join(rs::repeater_system.on(rs::id.eq(l::repeater_b_id)))
        .select((l::repeater_b_id, rs::call_sign, l::note))
        .get_results(c)
        .await?;

    let b_rows: Vec<(i64, String, String)> = l::repeater_link
        .filter(l::repeater_b_id.eq(repeater_id))
        .inner_join(rs::repeater_system.on(rs::id.eq(l::repeater_a_id)))
        .select((l::repeater_a_id, rs::call_sign, l::note))
        .get_results(c)
        .await?;

    let mut out = Vec::with_capacity(a_rows.len() + b_rows.len());
    for (other_repeater_id, call_sign, note) in a_rows.into_iter().chain(b_rows) {
        out.push(RepeaterLinkWithOtherCallSign {
            other_repeater_id,
            other_call_sign: call_sign,
            note,
        });
    }

    out.sort_by(|a, b| a.other_call_sign.cmp(&b.other_call_sign));
    Ok(out)
}
