use crate::dao;
use crate::dao::ham_club::{HamClub, NewHamClub};
use crate::dao::repeater_port::{NewRepeaterPort, RepeaterPort};
use crate::dao::repeater_service::{NewRepeaterService, RepeaterServiceKind};
use crate::dao::repeater_service_aprs::{AprsMode, NewRepeaterServiceAprs};
use crate::dao::repeater_service_dstar::{DstarMode, NewRepeaterServiceDstar};
use crate::dao::repeater_service_fm::{FmBandwidth, NewRepeaterServiceFm, ToneKind};
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use diesel::QueryResult;
use diesel_async::AsyncPgConnection;

async fn ham_club(c: &mut AsyncPgConnection, call_sign: impl Into<String>) -> QueryResult<HamClub> {
    dao::ham_club::insert(c, NewHamClub::new(call_sign.into())).await
}

async fn repeater(
    c: &mut AsyncPgConnection,
    club: &Option<HamClub>,
    call_sign: impl Into<String>,
) -> QueryResult<RepeaterSystem> {
    let mut r = NewRepeaterSystem::new(call_sign);
    if let Some(club) = club {
        r = r.ham_club_id(club.id);
    }

    dao::repeater_system::insert(c, r).await
}

pub async fn narrow_fm(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
    subtone: Option<f32>,
) -> QueryResult<RepeaterPort> {
    let port = NewRepeaterPort {
        repeater_id: r.id,
        label: label.into(),
        rx_hz: tx_frequency - offset,
        tx_hz: tx_frequency,
        note: None,
    };

    let port = dao::repeater_port::insert(c, port).await?;
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id: r.id,
            port_id: Some(port.id),
            kind: RepeaterServiceKind::Fm,
            enabled: true,
        },
    )
    .await?;
    let mut fm = NewRepeaterServiceFm {
        service_id: service.id,
        bandwidth: FmBandwidth::Narrow,
        access_tone_kind: ToneKind::None,
        access_ctcss_hz: None,
        access_dcs_code: None,
        tx_tone_kind: ToneKind::None,
        tx_ctcss_hz: None,
        tx_dcs_code: None,
    };

    if let Some(subtone) = subtone {
        fm = NewRepeaterServiceFm {
            access_tone_kind: ToneKind::CTCSS,
            access_ctcss_hz: Some(subtone),
            tx_tone_kind: ToneKind::CTCSS,
            tx_ctcss_hz: Some(subtone),
            ..fm
        }
    }

    dao::repeater_service_fm::insert(c, fm).await?;

    Ok(port)
}

pub async fn dstar(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
) -> QueryResult<RepeaterPort> {
    let port = NewRepeaterPort {
        repeater_id: r.id,
        label: label.into(),
        rx_hz: tx_frequency - offset,
        tx_hz: tx_frequency,
        note: None,
    };

    let port = dao::repeater_port::insert(c, port).await?;
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id: r.id,
            port_id: Some(port.id),
            kind: RepeaterServiceKind::Dstar,
            enabled: true,
        },
    )
    .await?;
    dao::repeater_service_dstar::insert(
        c,
        NewRepeaterServiceDstar {
            service_id: service.id,
            mode: DstarMode::Dv,
            gateway_call_sign: None,
            reflector: None,
        },
    )
    .await?;

    Ok(port)
}

pub async fn igate(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    frequency: i64,
) -> QueryResult<RepeaterPort> {
    let port = NewRepeaterPort {
        repeater_id: r.id,
        label: label.into(),
        rx_hz: frequency,
        tx_hz: frequency,
        note: None,
    };

    let port = dao::repeater_port::insert(c, port).await?;
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id: r.id,
            port_id: Some(port.id),
            kind: RepeaterServiceKind::Aprs,
            enabled: true,
        },
    )
    .await?;
    dao::repeater_service_aprs::insert(
        c,
        NewRepeaterServiceAprs {
            service_id: service.id,
            mode: AprsMode::Igate,
            path: None,
        },
    )
    .await?;

    Ok(port)
}

pub async fn digipeater(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    frequency: i64,
) -> QueryResult<RepeaterPort> {
    let port = NewRepeaterPort {
        repeater_id: r.id,
        label: label.into(),
        rx_hz: frequency,
        tx_hz: frequency,
        note: None,
    };

    let port = dao::repeater_port::insert(c, port).await?;
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id: r.id,
            port_id: Some(port.id),
            kind: RepeaterServiceKind::Aprs,
            enabled: true,
        },
    )
    .await?;
    dao::repeater_service_aprs::insert(
        c,
        NewRepeaterServiceAprs {
            service_id: service.id,
            mode: AprsMode::Digipeater,
            path: None,
        },
    )
    .await?;

    Ok(port)
}

pub async fn generate(c: &mut AsyncPgConnection) -> QueryResult<()> {
    let club = ham_club(c, "LA4O").await?;
    let club = &Some(club);

    let r = &repeater(c, &club, "LA5OR").await?;
    narrow_fm(c, r, "VHF", 145_600_000, -600_000, Some(123.0)).await?;

    let r = &repeater(c, &club, "LA7OR").await?;
    narrow_fm(c, r, "UHF", 434_775_000, -2_000_000, Some(123.0)).await?;

    let r = &repeater(c, &club, "LD1OA").await?;
    narrow_fm(c, r, "UHF", 434_887_500, -2_000_000, Some(123.0)).await?;

    let r = &repeater(c, &club, "LD1OT").await?;
    dstar(c, r, "A", 1_297_100_000, -6_000_000).await?;
    dstar(c, r, "B", 434_862_500, -2_000_000).await?;

    let r = &repeater(c, &club, "LD1OS").await?;
    igate(c, r, "A", 144_800_000).await?;

    let r = &repeater(c, &club, "LD1OE").await?;
    digipeater(c, r, "A", 144_800_000).await?;

    Ok(())
}
