use crate::dao::call_sign::NewCallSign;
use crate::dao::contact::Contact;
use crate::dao::repeater_service::{AprsMode, DstarMode, FmBandwidth, SsbSideband};
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystemDao};
use crate::service::repeater_service::RepeaterService;
use crate::{Frequency, MaidenheadLocator, Point, RepeaterAtlasError, dao, service};
use diesel_async::AsyncPgConnection;
use tracing::info;

pub async fn create_repeater_system(
    c: &mut AsyncPgConnection,
    repeater: NewRepeaterSystem,
) -> Result<RepeaterSystemDao, RepeaterAtlasError> {
    let call_sign = repeater.call_sign.clone();
    let new_call_sign = NewCallSign::new_repeater(call_sign.clone());

    dao::call_sign::insert(c, new_call_sign)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(
                e,
                format!("call_sign kind=repeater value={call_sign}"),
            )
        })?;

    info!("Creating repeater system call sign {call_sign}");

    let system = dao::repeater_system::insert(c, repeater)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(e, format!("repeater system call_sign={call_sign}"))
        })?;

    Ok(system)
}

pub(crate) async fn update(
    c: &mut AsyncPgConnection,
    repeater_system: RepeaterSystemDao,
) -> Result<RepeaterSystemDao, RepeaterAtlasError> {
    info!(
        call_sign = repeater_system.call_sign,
        "Updating RepeaterSystem {}", repeater_system.call_sign
    );

    dao::repeater_system::update(c, repeater_system)
        .await
        .map_err(RepeaterAtlasError::from)
}

pub struct Repeater {
    pub id: i64,
    pub call_sign: String,
    pub owner: Option<Contact>,
    pub tech_contact: Option<Contact>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub address: Option<String>,
    pub maidenhead: Option<MaidenheadLocator>,
    pub point: Option<Point>,
    pub elevation_m: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub status: String,

    pub services: ServiceItems,
}

pub struct ServiceItems {
    pub fm_services: Vec<FmServiceItem>,
    pub dmr_services: Vec<DmrServiceItem>,
    pub dstar_services: Vec<DstarServiceItem>,
    pub c4fm_services: Vec<C4fmServiceItem>,
    pub aprs_services: Vec<AprsServiceItem>,
    pub ssb_services: Vec<SsbServiceItem>,
    pub am_services: Vec<AmServiceItem>,
}

pub struct FmServiceItem {
    pub label: String,
    pub enabled: bool,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub bandwidth: FmBandwidth,
    pub rx_tone: service::repeater_service::Tone,
    pub tx_tone: service::repeater_service::Tone,
    pub note: String,
}

pub struct DmrServiceItem {
    pub label: String,
    pub enabled: bool,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub color_code: i16,
    pub dmr_repeater_id: Option<i64>,
    pub network: String,
    pub note: String,
}

pub struct DstarServiceItem {
    pub label: String,
    pub enabled: bool,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub mode: DstarMode,
    pub gateway_call_sign: Option<String>,
    pub reflector: Option<String>,
    pub note: String,
}

pub struct C4fmServiceItem {
    pub label: String,
    pub enabled: bool,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub wires_x_node_id: Option<i32>,
    pub room: Option<String>,
    pub note: String,
}

pub struct AprsServiceItem {
    pub label: String,
    pub enabled: bool,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub mode: Option<AprsMode>,
    pub path: Option<String>,
    pub note: String,
}

pub struct SsbServiceItem {
    pub label: String,
    pub enabled: bool,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub sideband: Option<SsbSideband>,
    pub note: String,
}

pub struct AmServiceItem {
    pub label: String,
    pub enabled: bool,
    pub rx_hz: Frequency,
    pub tx_hz: Frequency,
    pub note: String,
}

pub async fn load_by_call_sign(
    c: &mut AsyncPgConnection,
    call_sign: String,
) -> Result<Repeater, RepeaterAtlasError> {
    let Some(repeater) = dao::repeater_system::find_by_call_sign(c, call_sign.clone()).await?
    else {
        return Err(RepeaterAtlasError::NotFound);
    };

    load(c, repeater).await
}

pub async fn load_by_id(
    c: &mut AsyncPgConnection,
    id: i64,
) -> Result<Repeater, RepeaterAtlasError> {
    let repeater = dao::repeater_system::get(c, id).await?;

    load(c, repeater).await
}

pub async fn load(
    c: &mut AsyncPgConnection,
    repeater: RepeaterSystemDao,
) -> Result<Repeater, RepeaterAtlasError> {
    let owner = match repeater.owner {
        Some(contact_id) => Some(dao::contact::get(c, contact_id).await?.into()),
        None => None,
    };

    let tech_contact = match repeater.tech_contact {
        Some(contact_id) => Some(dao::contact::get(c, contact_id).await?.into()),
        None => None,
    };

    let services = dao::repeater_service::select_by_repeater_id(c, repeater.id).await?;
    let service_items = build_service_items(services);

    let point = repeater.location();

    Ok(Repeater {
        id: repeater.id,
        call_sign: repeater.call_sign,
        owner,
        tech_contact,
        name: repeater.name,
        description: repeater.description,
        address: repeater.address,
        maidenhead: repeater.maidenhead,
        point,
        elevation_m: repeater.elevation_m,
        country: repeater.country,
        region: repeater.region,
        status: repeater.status,

        services: service_items,
    })
}

fn build_service_items(services: Vec<dao::repeater_service::RepeaterServiceDao>) -> ServiceItems {
    let mut fm_services = Vec::new();
    let mut dmr_services = Vec::new();
    let mut dstar_services = Vec::new();
    let mut c4fm_services = Vec::new();
    let mut aprs_services = Vec::new();
    let mut ssb_services = Vec::new();
    let mut am_services = Vec::new();

    for row in services {
        let enabled = row.enabled;
        let note = row.note.clone();
        let service = RepeaterService::from(row);

        match service {
            RepeaterService::Fm {
                label,
                rx_hz,
                tx_hz,
                bandwidth,
                rx_tone,
                tx_tone,
                ..
            } => fm_services.push(FmServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                bandwidth,
                rx_tone,
                tx_tone,
                note,
            }),
            RepeaterService::Dmr {
                label,
                rx_hz,
                tx_hz,
                color_code,
                dmr_repeater_id,
                network,
                ..
            } => dmr_services.push(DmrServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                color_code,
                dmr_repeater_id,
                network,
                note,
            }),
            RepeaterService::Dstar {
                label,
                rx_hz,
                tx_hz,
                mode,
                gateway_call_sign,
                reflector,
                ..
            } => dstar_services.push(DstarServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                mode,
                gateway_call_sign,
                reflector,
                note,
            }),
            RepeaterService::C4fm {
                label,
                rx_hz,
                tx_hz,
                wires_x_node_id,
                room,
                ..
            } => c4fm_services.push(C4fmServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                wires_x_node_id,
                room,
                note,
            }),
            RepeaterService::Aprs {
                label,
                rx_hz,
                tx_hz,
                mode,
                path,
                ..
            } => aprs_services.push(AprsServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                mode,
                path,
                note,
            }),
            RepeaterService::Ssb {
                label,
                rx_hz,
                tx_hz,
                sideband,
                ..
            } => ssb_services.push(SsbServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                sideband,
                note,
            }),
            RepeaterService::Am {
                label,
                rx_hz,
                tx_hz,
                ..
            } => am_services.push(AmServiceItem {
                label,
                enabled,
                rx_hz,
                tx_hz,
                note,
            }),
        }
    }

    ServiceItems {
        fm_services,
        dmr_services,
        dstar_services,
        c4fm_services,
        aprs_services,
        ssb_services,
        am_services,
    }
}
