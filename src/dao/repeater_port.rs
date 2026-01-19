use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_port)]
pub struct NewRepeaterPort {
    pub repeater_id: i64,
    pub label: String,
    #[diesel(column_name = rx_hz)]
    pub rx_frequency: i64,
    #[diesel(column_name = tx_hz)]
    pub tx_frequency: i64,
    pub note: Option<String>,
}

impl NewRepeaterPort {
    pub fn new(
        repeater_id: i64,
        label: impl Into<String>,
        rx_hz: i64,
        tx_hz: i64,
    ) -> Self {
        Self {
            repeater_id,
            label: label.into(),
            rx_frequency: rx_hz,
            tx_frequency: tx_hz,
            note: None,
        }
    }
}

pub async fn insert(c: &mut AsyncPgConnection, new_port: NewRepeaterPort) -> QueryResult<RepeaterPort> {
    use crate::schema::repeater_port::dsl as p;

    diesel::insert_into(p::repeater_port)
        .values(&new_port)
        .returning(RepeaterPort::as_returning())
        .get_result(c)
        .await
}

#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::repeater_port)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterPort {
    pub id: i64,
    pub repeater_id: i64,
    pub label: String,
    pub rx_hz: i64,
    pub tx_hz: i64,
    pub note: Option<String>,
}

pub async fn select_by_repeater_id(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
) -> QueryResult<Vec<RepeaterPort>> {
    use crate::schema::repeater_port::dsl as p;

    p::repeater_port
        .filter(p::repeater_id.eq(repeater_id))
        .select(RepeaterPort::as_select())
        .order_by(p::label.asc())
        .get_results(c)
        .await
}
