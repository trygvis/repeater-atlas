use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_site)]
pub struct NewRepeaterSite {
    pub name: Option<String>,
    pub address: Option<String>,
    pub maidenhead: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation_m: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
}

impl NewRepeaterSite {
    pub fn address(address: impl Into<String>) -> Self {
        Self {
            name: None,
            address: Some(address.into()),
            maidenhead: None,
            latitude: None,
            longitude: None,
            elevation_m: None,
            country: None,
            region: None,
        }
    }
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::repeater_site)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterSite {
    pub id: i64,
    pub name: Option<String>,
    pub address: Option<String>,
    pub maidenhead: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation_m: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
}

pub async fn insert(c: &mut AsyncPgConnection, site: NewRepeaterSite) -> QueryResult<RepeaterSite> {
    use crate::schema::repeater_site::dsl as s;

    diesel::insert_into(s::repeater_site)
        .values(&site)
        .returning(RepeaterSite::as_returning())
        .get_result(c)
        .await
}

pub async fn get(c: &mut AsyncPgConnection, site_id: i64) -> QueryResult<RepeaterSite> {
    use crate::schema::repeater_site::dsl as s;

    s::repeater_site
        .filter(s::id.eq(site_id))
        .select(RepeaterSite::as_select())
        .first(c)
        .await
}
