use crate::dao::ham_club::{HamClub, NewHamClub};
use crate::dao::repeater_service::{AprsMode, DstarMode, FmBandwidth};
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use crate::repeater_service::{RepeaterService, Tone};
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
    let mut repeater = NewRepeaterSystem::new(call_sign.clone());
    if let Some(club) = club {
        repeater = repeater.ham_club_id(club.id);
    }
    let address = address.into();
    if !address.trim().is_empty() {
        repeater.address = Some(address);
    }
    repeater.maidenhead = maidenhead.map(|value| value.to_string());

    info!("Creating repeater system call sign {call_sign}");

    dao::repeater_system::insert(c, repeater)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(e, format!("repeater system call_sign={call_sign}"))
        })
}

async fn create_service(
    c: &mut AsyncPgConnection,
    repeater_id: i64,
    service: RepeaterService,
) -> Result<(), RepeaterAtlasError> {
    let label = service.label().to_string();
    dao::repeater_service::insert(c, service.to_new_dao(repeater_id))
        .await
        .map(|_| ())
        .map_err(|e| RepeaterAtlasError::DatabaseOther(e, format!("Error adding service {label}")))
}

pub async fn narrow_fm(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
    subtone: Option<f32>,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let tone = subtone.map(Tone::CTCSS).unwrap_or(Tone::None);
    let service = RepeaterService::Fm {
        label,
        rx_hz: tx_frequency + offset,
        tx_hz: tx_frequency,
        bandwidth: FmBandwidth::Narrow,
        rx_tone: tone.clone(),
        tx_tone: tone,
        note: None,
    };
    create_service(c, r.id, service).await
}

pub async fn dstar(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let service = RepeaterService::Dstar {
        label,
        rx_hz: tx_frequency + offset,
        tx_hz: tx_frequency,
        mode: DstarMode::Dv,
        gateway_call_sign: None,
        reflector: None,
        note: None,
    };
    create_service(c, r.id, service).await
}

pub async fn igate(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    frequency: i64,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let service = RepeaterService::Aprs {
        label,
        rx_hz: frequency,
        tx_hz: frequency,
        mode: Some(AprsMode::Igate),
        path: None,
        note: None,
    };
    create_service(c, r.id, service).await
}

pub async fn digipeater(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    frequency: i64,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let service = RepeaterService::Aprs {
        label,
        rx_hz: frequency,
        tx_hz: frequency,
        mode: Some(AprsMode::Digipeater),
        path: None,
        note: None,
    };
    create_service(c, r.id, service).await
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
                let service = RepeaterService::Dmr {
                    label: label.to_string(),
                    rx_hz: tx_frequency + offset,
                    tx_hz: tx_frequency,
                    color_code: 1,
                    dmr_repeater_id: dmr_id,
                    network: "unknown".to_string(),
                    note: None,
                };
                create_service(c, repeater.id, service).await?;
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
                let service = RepeaterService::C4fm {
                    label: label.to_string(),
                    rx_hz: tx_frequency + offset,
                    tx_hz: tx_frequency,
                    wires_x_node_id: None,
                    room: None,
                    note: None,
                };
                create_service(c, repeater.id, service).await?;
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
