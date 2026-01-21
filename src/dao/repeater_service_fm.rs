use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use std::io::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::FmBandwidth)]
pub enum FmBandwidth {
    Narrow,
    Wide,
}

impl ToSql<crate::schema::sql_types::FmBandwidth, Pg> for FmBandwidth {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            FmBandwidth::Narrow => "narrow",
            FmBandwidth::Wide => "wide",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::FmBandwidth, Pg> for FmBandwidth {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"narrow" => Ok(FmBandwidth::Narrow),
            b"wide" => Ok(FmBandwidth::Wide),
            _ => Err("unrecognized fm_bandwidth variant".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::ToneKind)]
pub enum ToneKind {
    None,
    CTCSS,
    DCS,
}

impl ToSql<crate::schema::sql_types::ToneKind, Pg> for ToneKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            ToneKind::None => "none",
            ToneKind::CTCSS => "ctcss",
            ToneKind::DCS => "dcs",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::ToneKind, Pg> for ToneKind {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"none" => Ok(ToneKind::None),
            b"ctcss" => Ok(ToneKind::CTCSS),
            b"dcs" => Ok(ToneKind::DCS),
            _ => Err("unrecognized tone_kind variant".into()),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::repeater_service_fm)]
pub struct NewRepeaterServiceFm {
    pub service_id: i64,
    pub bandwidth: FmBandwidth,
    pub access_tone_kind: ToneKind,
    #[diesel(column_name = access_ctcss_hz)]
    pub access_ctcss_frequency: Option<f32>,
    pub access_dcs_code: Option<i32>,
    pub tx_tone_kind: ToneKind,
    #[diesel(column_name = tx_ctcss_hz)]
    pub tx_ctcss_frequency: Option<f32>,
    pub tx_dcs_code: Option<i32>,
}

pub async fn insert(c: &mut AsyncPgConnection, new_fm: NewRepeaterServiceFm) -> QueryResult<usize> {
    use crate::schema::repeater_service_fm::dsl as f;

    diesel::insert_into(f::repeater_service_fm)
        .values(&new_fm)
        .execute(c)
        .await
}
