use diesel::prelude::*;
use diesel::{QueryableByName, sql_query};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::{MaidenheadLocator, Point};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_system)]
pub struct NewRepeaterSystem {
    pub call_sign: String,
    pub owner: Option<i64>,
    pub tech_contact: Option<i64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub maidenhead: Option<MaidenheadLocator>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation_m: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub status: String,
}

impl NewRepeaterSystem {
    pub fn new(call_sign: impl Into<String>) -> Self {
        Self {
            call_sign: call_sign.into(),
            owner: None,
            tech_contact: None,
            name: None,
            description: None,
            address: None,
            maidenhead: None,
            latitude: None,
            longitude: None,
            elevation_m: None,
            country: None,
            region: None,
            status: "".to_string(),
        }
    }

    pub fn owner(self, owner: i64) -> Self {
        Self {
            owner: Some(owner),
            ..self
        }
    }

    pub fn tech_contact(self, tech_contact: i64) -> Self {
        Self {
            tech_contact: Some(tech_contact),
            ..self
        }
    }
}

// When updating this, remember to update RepeaterSystemRow (the CSV dump structure).
#[derive(Clone, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::repeater_system)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterSystemDao {
    pub id: i64,
    pub call_sign: String,
    pub owner: Option<i64>,
    pub tech_contact: Option<i64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub maidenhead: Option<MaidenheadLocator>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation_m: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub status: String,
}

impl RepeaterSystemDao {
    pub fn location(&self) -> Option<Point> {
        Point::from_optional(self.latitude, self.longitude)
    }
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_repeater: NewRepeaterSystem,
) -> QueryResult<RepeaterSystemDao> {
    use crate::schema::repeater_system::dsl as r;

    diesel::insert_into(r::repeater_system)
        .values(&new_repeater)
        .returning(RepeaterSystemDao::as_returning())
        .get_result(c)
        .await
}

pub async fn update(
    c: &mut AsyncPgConnection,
    repeater: RepeaterSystemDao,
) -> QueryResult<RepeaterSystemDao> {
    use crate::schema::repeater_system::dsl as r;

    diesel::update(r::repeater_system.filter(r::id.eq(repeater.id)))
        .set(&repeater)
        .returning(RepeaterSystemDao::as_returning())
        .get_result(c)
        .await
}

pub async fn get(c: &mut AsyncPgConnection, repeater_id: i64) -> QueryResult<RepeaterSystemDao> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .filter(r::id.eq(repeater_id))
        .select(RepeaterSystemDao::as_select())
        .first(c)
        .await
}

pub async fn find_by_call_sign(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> QueryResult<Option<RepeaterSystemDao>> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .filter(r::call_sign.eq(call_sign))
        .select(RepeaterSystemDao::as_select())
        .first(c)
        .await
        .optional()
}

pub async fn select_with_call_sign(
    c: &mut AsyncPgConnection,
) -> QueryResult<Vec<RepeaterSystemDao>> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .select(RepeaterSystemDao::as_select())
        .order_by(r::call_sign.asc())
        .get_results(c)
        .await
}

pub async fn select_by_ids(
    c: &mut AsyncPgConnection,
    repeater_ids: &[i64],
) -> QueryResult<Vec<RepeaterSystemDao>> {
    use crate::schema::repeater_system::dsl as r;

    if repeater_ids.is_empty() {
        return Ok(Vec::new());
    }

    r::repeater_system
        .filter(r::id.eq_any(repeater_ids))
        .select(RepeaterSystemDao::as_select())
        .order_by(r::call_sign.asc())
        .get_results(c)
        .await
}

pub async fn select_by_call_signs(
    c: &mut AsyncPgConnection,
    call_signs: &[String],
) -> QueryResult<Vec<RepeaterSystemDao>> {
    use crate::schema::repeater_system::dsl as r;

    if call_signs.is_empty() {
        return Ok(Vec::new());
    }

    r::repeater_system
        .filter(r::call_sign.eq_any(call_signs))
        .select(RepeaterSystemDao::as_select())
        .order_by(r::call_sign.asc())
        .get_results(c)
        .await
}

#[derive(QueryableByName)]
struct RepeaterSystemRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    id: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    call_sign: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    owner: Option<i64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::BigInt>)]
    tech_contact: Option<i64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    address: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    maidenhead: Option<MaidenheadLocator>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    latitude: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
    longitude: Option<f64>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
    elevation_m: Option<i32>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    country: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    region: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
}

impl From<RepeaterSystemRow> for RepeaterSystemDao {
    fn from(row: RepeaterSystemRow) -> Self {
        Self {
            id: row.id,
            call_sign: row.call_sign,
            owner: row.owner,
            tech_contact: row.tech_contact,
            name: row.name,
            description: row.description,
            address: row.address,
            maidenhead: row.maidenhead,
            latitude: row.latitude,
            longitude: row.longitude,
            elevation_m: row.elevation_m,
            country: row.country,
            region: row.region,
            status: row.status,
        }
    }
}

pub async fn select_within_radius(
    c: &mut AsyncPgConnection,
    center: Point,
    radius_meters: f64,
) -> QueryResult<Vec<RepeaterSystemDao>> {
    let rows: Vec<RepeaterSystemRow> = sql_query(
        r#"
        SELECT
            id,
            call_sign,
            owner,
            tech_contact,
            name,
            description,
            address,
            maidenhead,
            latitude,
            longitude,
            elevation_m,
            country,
            region,
            status
        FROM repeater_system
        WHERE latitude IS NOT NULL
          AND longitude IS NOT NULL
          AND ST_DWithin(
                geography(ST_MakePoint(longitude, latitude)),
                geography(ST_MakePoint($1, $2)),
                $3
              )
        ORDER BY call_sign ASC
        "#,
    )
    .bind::<diesel::sql_types::Double, _>(center.longitude)
    .bind::<diesel::sql_types::Double, _>(center.latitude)
    .bind::<diesel::sql_types::Double, _>(radius_meters)
    .get_results(c)
    .await?;

    Ok(rows.into_iter().map(RepeaterSystemDao::from).collect())
}

// TODO: rename to by_owner
pub async fn select_with_call_sign_by_owner(
    c: &mut AsyncPgConnection,
    contact_id: i64,
) -> QueryResult<Vec<RepeaterSystemDao>> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .filter(r::owner.eq(contact_id))
        .select(RepeaterSystemDao::as_select())
        .order_by(r::call_sign.asc())
        .get_results(c)
        .await
}

// TODO: rename to by_tech_contact
pub async fn select_with_call_sign_by_tech_contact(
    c: &mut AsyncPgConnection,
    contact_id: i64,
) -> QueryResult<Vec<RepeaterSystemDao>> {
    use crate::schema::repeater_system::dsl as r;

    r::repeater_system
        .filter(r::tech_contact.eq(contact_id))
        .select(RepeaterSystemDao::as_select())
        .order_by(r::call_sign.asc())
        .get_results(c)
        .await
}
