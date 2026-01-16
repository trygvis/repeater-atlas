use std::io::Write;

use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater)]
pub struct NewRepeater {
    pub call_sign: String,
    pub maidenhead_locator: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub frequency: i64,
    pub modulation: Option<Modulation>,
    pub rx_offset: i64,
    pub subtone_mode: SubtoneMode,
    pub tx_subtone: Option<f32>,
    pub rx_subtone: Option<f32>,
    pub has_dmr: bool,
    pub dmr_id: Option<i64>,
    pub has_aprs: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::repeater)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Repeater {
    pub id: i64,
    pub call_sign: String,
    pub maidenhead_locator: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address: Option<String>,
    pub frequency: i64,
    pub modulation: Option<Modulation>,
    pub rx_offset: i64,
    pub subtone_mode: SubtoneMode,
    pub tx_subtone: Option<f32>,
    pub rx_subtone: Option<f32>,
    pub has_dmr: bool,
    pub dmr_id: Option<i64>,
    pub has_aprs: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum Modulation {
    FmNarrow,
    FmWide,
    Am,
    Lsb,
    Usb,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum SubtoneMode {
    None,
    CTCSS,
    DCS,
}

impl ToSql<Text, Pg> for Modulation {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            Modulation::FmNarrow => "FmNarrow",
            Modulation::FmWide => "FmWide",
            Modulation::Am => "Am",
            Modulation::Lsb => "Lsb",
            Modulation::Usb => "Usb",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for Modulation {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"FmNarrow" => Ok(Modulation::FmNarrow),
            b"FmWide" => Ok(Modulation::FmWide),
            b"Am" => Ok(Modulation::Am),
            b"Lsb" => Ok(Modulation::Lsb),
            b"Usb" => Ok(Modulation::Usb),
            _ => Err("unrecognized modulation variant".into()),
        }
    }
}

impl ToSql<Text, Pg> for SubtoneMode {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            SubtoneMode::None => "None",
            SubtoneMode::CTCSS => "CTCSS",
            SubtoneMode::DCS => "DCS",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for SubtoneMode {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"None" => Ok(SubtoneMode::None),
            b"CTCSS" => Ok(SubtoneMode::CTCSS),
            b"DCS" => Ok(SubtoneMode::DCS),
            _ => Err("unrecognized subtone_mode variant".into()),
        }
    }
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_repeater: NewRepeater,
) -> QueryResult<usize> {
    use crate::schema::repeater::dsl as r;

    diesel::insert_into(r::repeater)
        .values(&new_repeater)
        .execute(c)
        .await
}

pub async fn select(c: &mut AsyncPgConnection) -> QueryResult<Vec<Repeater>> {
    use crate::schema::repeater::dsl as r;

    r::repeater
        .select(Repeater::as_select())
        .get_results(c)
        .await
}

pub async fn get(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
) -> QueryResult<Repeater> {
    use crate::schema::repeater::dsl as r;

    r::repeater
        .filter(r::id.eq(repeater_id))
        .select(Repeater::as_select())
        .first(c)
        .await
}

pub async fn find_by_call_sign(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> QueryResult<Option<Repeater>> {
    use crate::schema::repeater::dsl as r;

    r::repeater
        .filter(r::call_sign.eq(call_sign))
        .select(Repeater::as_select())
        .first(c)
        .await
        .optional()
}
