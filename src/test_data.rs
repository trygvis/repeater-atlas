use crate::dao::ham_club::{HamClub, NewHamClub};
use crate::dao::repeater_port::{NewRepeaterPort, RepeaterPort};
use crate::dao::repeater_service::{NewRepeaterService, RepeaterServiceKind};
use crate::dao::repeater_service_aprs::{AprsMode, NewRepeaterServiceAprs};
use crate::dao::repeater_service_c4fm::NewRepeaterServiceC4fm;
use crate::dao::repeater_service_dmr::NewRepeaterServiceDmr;
use crate::dao::repeater_service_dstar::{DstarMode, NewRepeaterServiceDstar};
use crate::dao::repeater_service_fm::{FmBandwidth, NewRepeaterServiceFm, ToneKind};
use crate::dao::repeater_site::NewRepeaterSite;
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use crate::{RepeaterAtlasError, dao};
use csv::StringRecord;
use diesel::QueryResult;
use diesel_async::AsyncPgConnection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

async fn repeater_with_site(
    c: &mut AsyncPgConnection,
    club: &Option<HamClub>,
    call_sign: impl Into<String> + std::fmt::Display,
    address: impl Into<String>,
    maidenhead: Option<&str>,
) -> Result<RepeaterSystem, RepeaterAtlasError> {
    let call_sign = call_sign.into();
    let mut site = NewRepeaterSite::address(address);
    site.maidenhead = maidenhead.map(|value| value.to_string());
    let site = dao::repeater_site::insert(c, site).await?;

    let mut repeater = NewRepeaterSystem::new(call_sign.clone());
    if let Some(club) = club {
        repeater = repeater.ham_club_id(club.id);
    }
    repeater.site_id = Some(site.id);

    info!("Creating repeater system call sign {call_sign}");

    dao::repeater_system::insert(c, repeater)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(e, format!("repeater system call_sign={call_sign}"))
        })
}

async fn create_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    label: impl Into<String>,
    tx_frequency: i64,
    rx_frequency: i64,
) -> Result<RepeaterPort, RepeaterAtlasError> {
    let label = label.into();

    let port = NewRepeaterPort {
        repeater_id,
        label: label.clone(),
        rx_frequency,
        tx_frequency,
        note: None,
    };

    dao::repeater_port::insert(c, port).await.map_err(|e| {
        RepeaterAtlasError::DatabaseOther(e, format!("Error adding port with label {label}"))
    })
}

async fn fm_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
    bandwidth: FmBandwidth,
    subtone: Option<f32>,
) -> Result<(), RepeaterAtlasError> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
            kind: RepeaterServiceKind::Fm,
            enabled: true,
        },
    )
    .await?;
    let mut fm = NewRepeaterServiceFm {
        service_id: service.id,
        bandwidth,
        access_tone_kind: ToneKind::None,
        access_ctcss_frequency: None,
        access_dcs_code: None,
        tx_tone_kind: ToneKind::None,
        tx_ctcss_frequency: None,
        tx_dcs_code: None,
    };

    if let Some(subtone) = subtone {
        fm = NewRepeaterServiceFm {
            access_tone_kind: ToneKind::CTCSS,
            access_ctcss_frequency: Some(subtone),
            tx_tone_kind: ToneKind::CTCSS,
            tx_ctcss_frequency: Some(subtone),
            ..fm
        }
    }

    dao::repeater_service_fm::insert(c, fm).await?;

    Ok(())
}

async fn dstar_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
) -> QueryResult<()> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
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

    Ok(())
}

async fn dmr_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
    dmr_id: Option<i64>,
) -> QueryResult<()> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
            kind: RepeaterServiceKind::Dmr,
            enabled: true,
        },
    )
    .await?;

    dao::repeater_service_dmr::insert(
        c,
        NewRepeaterServiceDmr {
            service_id: service.id,
            color_code: None,
            dmr_repeater_id: dmr_id,
            network: None,
        },
    )
    .await?;

    Ok(())
}

async fn c4fm_service_on_port(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    port_id: i64,
) -> QueryResult<()> {
    let service = dao::repeater_service::insert(
        c,
        NewRepeaterService {
            repeater_id,
            port_id: Some(port_id),
            kind: RepeaterServiceKind::C4fm,
            enabled: true,
        },
    )
    .await?;

    dao::repeater_service_c4fm::insert(
        c,
        NewRepeaterServiceC4fm {
            service_id: service.id,
            wires_x_node_id: None,
            room: None,
        },
    )
    .await?;

    Ok(())
}

pub async fn narrow_fm(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
    subtone: Option<f32>,
) -> Result<RepeaterPort, RepeaterAtlasError> {
    let port = create_port(c, r.id, label, tx_frequency, tx_frequency + offset).await?;
    fm_service_on_port(c, r.id, port.id, FmBandwidth::Narrow, subtone).await?;

    Ok(port)
}

pub async fn dstar(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
) -> Result<RepeaterPort, RepeaterAtlasError> {
    let port = create_port(c, r.id, label, tx_frequency, tx_frequency + offset).await?;
    dstar_service_on_port(c, r.id, port.id).await?;

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
        rx_frequency: frequency,
        tx_frequency: frequency,
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
        rx_frequency: frequency,
        tx_frequency: frequency,
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

fn label_for_frequency(frequency: i64) -> &'static str {
    if frequency < 200_000_000 {
        "VHF"
    } else if frequency < 1_000_000_000 {
        "UHF"
    } else {
        "SHF"
    }
}

fn split_call_sign(input: &str) -> (String, Option<String>) {
    let trimmed = input.trim();
    if let Some((head, tail)) = trimmed.split_once('-') {
        let label = tail.trim();
        let label = if label.is_empty() {
            None
        } else {
            Some(label.to_ascii_uppercase())
        };
        (head.trim().to_ascii_uppercase(), label)
    } else {
        (trimmed.to_ascii_uppercase(), None)
    }
}

fn normalize_call_sign(input: &str) -> String {
    input.trim().to_ascii_uppercase()
}

fn load_csv(path: &Path) -> Result<(StringRecord, Vec<StringRecord>), RepeaterAtlasError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)?;

    let headers = reader.headers()?.clone();

    let records: Result<Vec<StringRecord>, csv::Error> = reader.records().collect();

    Ok((headers, records?))
}

pub async fn generate(c: &mut AsyncPgConnection) -> Result<(), RepeaterAtlasError> {
    let clubs = load_ham_clubs(c, PathBuf::from("data/ham_clubs.csv")).await?;

    let dir = Path::new("data").read_dir()?;
    for d in dir {
        let d = d?;
        let name = d.file_name();
        let name: &str = name.to_str().unwrap_or("");
        if name.starts_with("repeaters-") {
            load_repeaters(c, &clubs, d.path()).await?;
        }
    }

    Ok(())
}

pub async fn load_ham_clubs(
    c: &mut AsyncPgConnection,
    path: PathBuf,
) -> Result<HashMap<String, HamClub>, RepeaterAtlasError> {
    let (headers, records) = load_csv(&path)?;
    let mut clubs = HashMap::new();

    for (row_index, record) in records.iter().enumerate() {
        let row_index = row_index + 2;
        let mut row = HashMap::new();
        for (header, value) in headers.iter().zip(record.iter()) {
            row.insert(header.to_string(), value.to_string());
        }

        let call_sign_raw = match row
            .get("call_sign")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            Some(value) => value.to_string(),
            None => {
                info!(
                    row = row_index,
                    reason = "missing call_sign",
                    "Skipping club row"
                );
                continue;
            }
        };
        let call_sign = normalize_call_sign(&call_sign_raw);
        if call_sign.is_empty() {
            info!(
                row = row_index,
                reason = "empty call_sign",
                "Skipping club row"
            );
            continue;
        }

        if clubs.contains_key(&call_sign) {
            info!(
                row = row_index,
                call_sign = call_sign,
                "Skipping duplicate club row"
            );
            continue;
        }

        let name = row
            .get("name")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());
        let web_url = row
            .get("web_url")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());
        let email = row
            .get("email")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());

        let club = dao::ham_club::insert(
            c,
            NewHamClub {
                name: call_sign.clone(),
                description: name,
                web_url,
                email,
            },
        )
        .await?;

        clubs.insert(call_sign, club);
    }

    Ok(clubs)
}

pub async fn load_repeaters(
    c: &mut AsyncPgConnection,
    clubs: &HashMap<String, HamClub>,
    path: PathBuf,
) -> Result<(), RepeaterAtlasError> {
    let mut imported = 0usize;
    let mut repeaters = HashMap::<String, RepeaterSystem>::new();

    let (headers, records) = load_csv(&path)?;

    for (row_index, record) in records.iter().enumerate() {
        let row_index = row_index + 2;
        let mut row = HashMap::new();
        for (header, value) in headers.iter().zip(record.iter()) {
            row.insert(header.to_string(), value.to_string());
        }

        let call_sign_raw = match row
            .get("call_sign")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            Some(value) => value.to_string(),
            None => {
                info!(
                    row = row_index,
                    reason = "missing call_sign",
                    "Skipping repeater row"
                );
                continue;
            }
        };
        let (call_sign, port_label) = split_call_sign(&call_sign_raw);

        let owner = row
            .get("owner")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        let club = owner
            .map(normalize_call_sign)
            .and_then(|value| clubs.get(&value).cloned());

        let repeater = if let Some(existing) = repeaters.get(&call_sign) {
            existing.clone()
        } else {
            let address = row
                .get("address")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .unwrap_or("");
            let maidenhead = row
                .get("maidenhead")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty());
            let mut repeater =
                repeater_with_site(c, &club, call_sign.clone(), address, maidenhead).await?;

            if let Some(status) = row
                .get("status")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
            {
                repeater.status = status.to_string();
                repeater = dao::repeater_system::update(c, repeater).await?;
            }

            repeaters.insert(call_sign.clone(), repeater.clone());
            repeater
        };

        let service = row
            .get("service")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        let tx_frequency = parse_tx_frequency(&row);
        let offset = parse_offset(&row).or_else(|| tx_frequency.and_then(default_offset));
        let ctcss = row
            .get("ctcss_tx")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .and_then(|value| value.parse::<f32>().ok())
            .or_else(|| {
                row.get("ctcss_rx")
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .and_then(|value| value.parse::<f32>().ok())
            });
        let dmr_id = row
            .get("dmr_id")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .and_then(|value| value.parse::<i64>().ok());

        match service {
            Some("FM_NARROW") => {
                let (Some(tx_frequency), Some(offset)) = (tx_frequency, offset) else {
                    info!(
                        row = row_index,
                        call_sign = call_sign,
                        reason = "missing tx/offset",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label
                    .as_deref()
                    .unwrap_or(label_for_frequency(tx_frequency));
                narrow_fm(c, &repeater, label, tx_frequency, offset, ctcss).await?;
                imported += 1;
            }
            Some("APRS_IGATE") => {
                let Some(tx_frequency) = tx_frequency else {
                    info!(
                        row = row_index,
                        call_sign = call_sign,
                        reason = "missing tx",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label
                    .as_deref()
                    .unwrap_or(label_for_frequency(tx_frequency));
                igate(c, &repeater, label, tx_frequency).await?;
                imported += 1;
            }
            Some("APRS_DIGIPEATER") => {
                let Some(tx_frequency) = tx_frequency else {
                    info!(
                        row = row_index,
                        call_sign = call_sign,
                        reason = "missing tx",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label
                    .as_deref()
                    .unwrap_or(label_for_frequency(tx_frequency));
                digipeater(c, &repeater, label, tx_frequency).await?;
                imported += 1;
            }
            Some("DMR") => {
                let (Some(tx_frequency), Some(offset)) = (tx_frequency, offset) else {
                    info!(
                        row = row_index,
                        call_sign = call_sign,
                        reason = "missing tx/offset",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label
                    .as_deref()
                    .unwrap_or(label_for_frequency(tx_frequency));
                let port =
                    create_port(c, repeater.id, label, tx_frequency, tx_frequency + offset).await?;
                dmr_service_on_port(c, repeater.id, port.id, dmr_id).await?;
                imported += 1;
            }
            Some("C4FM") => {
                let (Some(tx_frequency), Some(offset)) = (tx_frequency, offset) else {
                    info!(
                        row = row_index,
                        call_sign = call_sign,
                        reason = "missing tx/offset",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label
                    .as_deref()
                    .unwrap_or(label_for_frequency(tx_frequency));
                let port =
                    create_port(c, repeater.id, label, tx_frequency, tx_frequency + offset).await?;
                c4fm_service_on_port(c, repeater.id, port.id).await?;
                imported += 1;
            }
            None => {
                info!(
                    row = row_index,
                    call_sign = call_sign,
                    reason = "missing service",
                    "Skipping repeater row"
                );
            }
            Some(service) => {
                info!(
                    row = row_index,
                    call_sign = call_sign,
                    service = service,
                    reason = "unsupported service",
                    "Skipping repeater row"
                );
            }
        }
    }

    info!(
        file = path.to_string_lossy().as_ref(),
        imported = imported,
        "Imported repeater data"
    );

    Ok(())
}

fn parse_offset(row: &HashMap<String, String>) -> Option<i64> {
    row.get("offset")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<i64>().ok())
}

fn parse_tx_frequency(row: &HashMap<String, String>) -> Option<i64> {
    let tx_hz = row
        .get("tx")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<i64>().ok());
    if tx_hz.is_some() {
        return tx_hz;
    }

    row.get("tx_mhz")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| (value * 1_000_000.0).round() as i64)
}

fn default_offset(tx_hz: i64) -> Option<i64> {
    if (144_000_000..148_000_000).contains(&tx_hz) {
        Some(-600_000)
    } else if (430_000_000..450_000_000).contains(&tx_hz) {
        Some(-2_000_000)
    } else {
        None
    }
}

pub async fn generate_users(c: &mut AsyncPgConnection) -> QueryResult<()> {
    crate::service::user::create_user(c, "LA8PV", "la8pv@example.org", "la8pv").await?;
    Ok(())
}
