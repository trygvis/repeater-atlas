use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::Serialize;
use std::io::Write;

use crate::Frequency;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, AsExpression, FromSqlRow, Serialize)]
#[diesel(sql_type = crate::schema::sql_types::RepeaterServiceKind)]
pub enum RepeaterServiceKind {
    Fm,
    Am,
    Ssb,
    Dstar,
    Dmr,
    C4fm,
    Aprs,
}

impl RepeaterServiceKind {
    pub fn label(&self) -> &'static str {
        match self {
            RepeaterServiceKind::Fm => "FM",
            RepeaterServiceKind::Am => "AM",
            RepeaterServiceKind::Ssb => "SSB",
            RepeaterServiceKind::Dstar => "D-STAR",
            RepeaterServiceKind::Dmr => "DMR",
            RepeaterServiceKind::C4fm => "C4FM",
            RepeaterServiceKind::Aprs => "APRS",
        }
    }
}

impl ToSql<crate::schema::sql_types::RepeaterServiceKind, Pg> for RepeaterServiceKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            RepeaterServiceKind::Fm => "fm",
            RepeaterServiceKind::Am => "am",
            RepeaterServiceKind::Ssb => "ssb",
            RepeaterServiceKind::Dstar => "dstar",
            RepeaterServiceKind::Dmr => "dmr",
            RepeaterServiceKind::C4fm => "c4fm",
            RepeaterServiceKind::Aprs => "aprs",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::RepeaterServiceKind, Pg> for RepeaterServiceKind {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"fm" => Ok(RepeaterServiceKind::Fm),
            b"am" => Ok(RepeaterServiceKind::Am),
            b"ssb" => Ok(RepeaterServiceKind::Ssb),
            b"dstar" => Ok(RepeaterServiceKind::Dstar),
            b"dmr" => Ok(RepeaterServiceKind::Dmr),
            b"c4fm" => Ok(RepeaterServiceKind::C4fm),
            b"aprs" => Ok(RepeaterServiceKind::Aprs),
            _ => Err("unrecognized repeater_service_kind variant".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow, Serialize)]
#[diesel(sql_type = crate::schema::sql_types::FmBandwidth)]
pub enum FmBandwidth {
    Narrow,
    Wide,
}

impl std::fmt::Display for FmBandwidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FmBandwidth::Narrow => write!(f, "Narrow"),
            FmBandwidth::Wide => write!(f, "Wide"),
        }
    }
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow, Serialize)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow, Serialize)]
#[diesel(sql_type = crate::schema::sql_types::DstarMode)]
pub enum DstarMode {
    Dv,
    Dd,
}

impl std::fmt::Display for DstarMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DstarMode::Dv => write!(f, "DV"),
            DstarMode::Dd => write!(f, "DD"),
        }
    }
}

impl ToSql<crate::schema::sql_types::DstarMode, Pg> for DstarMode {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            DstarMode::Dv => "dv",
            DstarMode::Dd => "dd",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::DstarMode, Pg> for DstarMode {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"dv" => Ok(DstarMode::Dv),
            b"dd" => Ok(DstarMode::Dd),
            _ => Err("unrecognized dstar_mode variant".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow, Serialize)]
#[diesel(sql_type = crate::schema::sql_types::AprsMode)]
pub enum AprsMode {
    Igate,
    Digipeater,
}

impl std::fmt::Display for AprsMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AprsMode::Igate => write!(f, "Igate"),
            AprsMode::Digipeater => write!(f, "Digipeater"),
        }
    }
}

impl ToSql<crate::schema::sql_types::AprsMode, Pg> for AprsMode {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            AprsMode::Igate => "igate",
            AprsMode::Digipeater => "digipeater",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::AprsMode, Pg> for AprsMode {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"igate" => Ok(AprsMode::Igate),
            b"digipeater" => Ok(AprsMode::Digipeater),
            _ => Err("unrecognized aprs_mode variant".into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, AsExpression, FromSqlRow, Serialize)]
#[diesel(sql_type = crate::schema::sql_types::SsbSideband)]
pub enum SsbSideband {
    Lsb,
    Usb,
}

impl std::fmt::Display for SsbSideband {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsbSideband::Lsb => write!(f, "LSB"),
            SsbSideband::Usb => write!(f, "USB"),
        }
    }
}

impl ToSql<crate::schema::sql_types::SsbSideband, Pg> for SsbSideband {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match self {
            SsbSideband::Lsb => "lsb",
            SsbSideband::Usb => "usb",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::SsbSideband, Pg> for SsbSideband {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"lsb" => Ok(SsbSideband::Lsb),
            b"usb" => Ok(SsbSideband::Usb),
            _ => Err("unrecognized ssb_sideband variant".into()),
        }
    }
}

#[derive(Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::repeater_service)]
pub struct NewRepeaterServiceDao {
    pub repeater_id: i64,
    pub kind: RepeaterServiceKind,
    pub enabled: bool,
    pub label: String,
    pub note: String,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub fm_bandwidth: Option<FmBandwidth>,
    pub rx_tone_kind: Option<ToneKind>,
    pub rx_ctcss_hz: Option<f32>,
    pub rx_dcs_code: Option<i32>,
    pub tx_tone_kind: Option<ToneKind>,
    pub tx_ctcss_hz: Option<f32>,
    pub tx_dcs_code: Option<i32>,
    pub dmr_color_code: Option<i16>,
    pub dmr_repeater_id: Option<i64>,
    pub dmr_network: Option<String>,
    pub dstar_mode: Option<DstarMode>,
    pub dstar_gateway_call_sign: Option<String>,
    pub dstar_reflector: Option<String>,
    pub c4fm_wires_x_node_id: Option<i32>,
    pub c4fm_room: Option<String>,
    pub aprs_mode: Option<AprsMode>,
    pub aprs_path: Option<String>,
    pub ssb_sideband: Option<SsbSideband>,
}

// When updating this, remember to update RepeaterServiceRow (the CSV dump structure).
#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::repeater_service)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RepeaterServiceDao {
    pub id: i64,
    pub repeater_id: i64,
    pub kind: RepeaterServiceKind,
    pub enabled: bool,
    pub label: String,
    pub note: String,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub fm_bandwidth: Option<FmBandwidth>,
    pub rx_tone_kind: Option<ToneKind>,
    pub rx_ctcss_hz: Option<f32>,
    pub rx_dcs_code: Option<i32>,
    pub tx_tone_kind: Option<ToneKind>,
    pub tx_ctcss_hz: Option<f32>,
    pub tx_dcs_code: Option<i32>,
    pub dmr_color_code: Option<i16>,
    pub dmr_repeater_id: Option<i64>,
    pub dmr_network: Option<String>,
    pub dstar_mode: Option<DstarMode>,
    pub dstar_gateway_call_sign: Option<String>,
    pub dstar_reflector: Option<String>,
    pub c4fm_wires_x_node_id: Option<i32>,
    pub c4fm_room: Option<String>,
    pub aprs_mode: Option<AprsMode>,
    pub aprs_path: Option<String>,
    pub ssb_sideband: Option<SsbSideband>,
}

pub async fn insert(
    c: &mut AsyncPgConnection,
    new_service: NewRepeaterServiceDao,
) -> QueryResult<RepeaterServiceDao> {
    use crate::schema::repeater_service::dsl as s;

    diesel::insert_into(s::repeater_service)
        .values(&new_service)
        .returning(RepeaterServiceDao::as_returning())
        .get_result(c)
        .await
}

pub async fn update_by_label_kind(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    label: &str,
    kind: RepeaterServiceKind,
    updated_service: &NewRepeaterServiceDao,
) -> QueryResult<RepeaterServiceDao> {
    use crate::schema::repeater_service::dsl as s;

    diesel::update(
        s::repeater_service
            .filter(s::repeater_id.eq(repeater_id))
            .filter(s::label.eq(label))
            .filter(s::kind.eq(kind)),
    )
    .set((
        s::enabled.eq(updated_service.enabled),
        s::note.eq(&updated_service.note),
        s::rx_hz.eq(updated_service.rx_hz),
        s::tx_hz.eq(updated_service.tx_hz),
        s::fm_bandwidth.eq(updated_service.fm_bandwidth),
        s::rx_tone_kind.eq(updated_service.rx_tone_kind),
        s::rx_ctcss_hz.eq(updated_service.rx_ctcss_hz),
        s::rx_dcs_code.eq(updated_service.rx_dcs_code),
        s::tx_tone_kind.eq(updated_service.tx_tone_kind),
        s::tx_ctcss_hz.eq(updated_service.tx_ctcss_hz),
        s::tx_dcs_code.eq(updated_service.tx_dcs_code),
        s::dmr_color_code.eq(updated_service.dmr_color_code),
        s::dmr_repeater_id.eq(updated_service.dmr_repeater_id),
        s::dmr_network.eq(updated_service.dmr_network.as_deref()),
        s::dstar_mode.eq(updated_service.dstar_mode),
        s::dstar_gateway_call_sign.eq(updated_service.dstar_gateway_call_sign.as_deref()),
        s::dstar_reflector.eq(updated_service.dstar_reflector.as_deref()),
        s::c4fm_wires_x_node_id.eq(updated_service.c4fm_wires_x_node_id),
        s::c4fm_room.eq(updated_service.c4fm_room.as_deref()),
        s::aprs_mode.eq(updated_service.aprs_mode),
        s::aprs_path.eq(updated_service.aprs_path.as_deref()),
        s::ssb_sideband.eq(updated_service.ssb_sideband),
    ))
    .returning(RepeaterServiceDao::as_returning())
    .get_result(c)
    .await
}

pub async fn select_by_repeater_id(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
) -> QueryResult<Vec<RepeaterServiceDao>> {
    use crate::schema::repeater_service::dsl as s;

    s::repeater_service
        .filter(s::repeater_id.eq(repeater_id))
        .select(RepeaterServiceDao::as_select())
        .order_by(s::kind.asc())
        .order_by(s::id.asc())
        .get_results(c)
        .await
}

pub async fn select_kinds_by_repeater_ids(
    c: &mut AsyncPgConnection,
    repeater_ids: &[i64],
) -> QueryResult<Vec<(i64, RepeaterServiceKind)>> {
    use crate::schema::repeater_service::dsl as s;

    if repeater_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows: Vec<(i64, RepeaterServiceKind)> = s::repeater_service
        .filter(s::repeater_id.eq_any(repeater_ids))
        .select((s::repeater_id, s::kind))
        .distinct()
        .get_results(c)
        .await?;

    Ok(rows)
}
