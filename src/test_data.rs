use crate::dao::contact::{Contact, ContactKind, NewContact};
use crate::dao::entity::{EntityKind, NewEntity};
use crate::dao::repeater_service::{AprsMode, FmBandwidth};
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use crate::repeater_service::{RepeaterService, Tone};
use crate::{Frequency, MaidenheadLocator, RepeaterAtlasError, dao};
use csv::StringRecord;
use diesel::QueryResult;
use diesel_async::AsyncPgConnection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Clone)]
struct RepeaterFixture {
    call_sign: String,
    system: RepeaterSystem,
}

fn row_from_record(
    headers: &StringRecord,
    record: &StringRecord,
) -> Result<HashMap<String, String>, RepeaterAtlasError> {
    let mut row = HashMap::new();

    for (header, value) in headers.iter().zip(record.iter()) {
        let key = header.trim().to_string();
        if row.contains_key(&key) {
            return Err(RepeaterAtlasError::Other(
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "duplicate header in csv",
                )),
                format!("duplicate header {key}"),
            ));
        }
        row.insert(key, value.to_string());
    }

    Ok(row)
}

fn parse_ctcss(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Accept "123", "123.0", "123.0 Hz".
    let first = trimmed.split_whitespace().next().unwrap_or(trimmed);
    first.parse::<f32>().ok()
}

async fn repeater_with_site(
    c: &mut AsyncPgConnection,
    call_sign: impl Into<String> + std::fmt::Display,
    owner: Option<&Contact>,
    address: impl Into<String>,
    maidenhead: Option<&str>,
) -> Result<RepeaterFixture, RepeaterAtlasError> {
    let call_sign = call_sign.into();

    let entity = dao::entity::insert(
        c,
        NewEntity {
            kind: EntityKind::Repeater,
            call_sign: Some(call_sign.clone()),
        },
    )
    .await
    .map_err(|e| {
        RepeaterAtlasError::DatabaseOther(e, format!("entity kind=repeater call_sign={call_sign}"))
    })?;

    let mut repeater = NewRepeaterSystem::new(entity.id);
    if let Some(owner) = owner {
        repeater = repeater.owner(owner.id);
    }
    let address = address.into();
    if !address.trim().is_empty() {
        repeater.address = Some(address);
    }
    repeater.maidenhead = maidenhead
        .map(MaidenheadLocator::new)
        .transpose()
        .map_err(|e| {
            RepeaterAtlasError::Other(
                Box::new(e),
                format!("invalid maidenhead locator for call_sign={call_sign}"),
            )
        })?;

    info!("Creating repeater system call sign {call_sign}");

    let system = dao::repeater_system::insert(c, repeater)
        .await
        .map_err(|e| {
            RepeaterAtlasError::DatabaseOther(e, format!("repeater system call_sign={call_sign}"))
        })?;

    Ok(RepeaterFixture { call_sign, system })
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

async fn narrow_fm(
    c: &mut AsyncPgConnection,
    r: &RepeaterFixture,
    label: impl Into<String>,
    tx_frequency: i64,
    offset: i64,
    subtone: Option<f32>,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let tone = subtone.map(Tone::CTCSS).unwrap_or(Tone::None);
    let tx_hz = Frequency::new_hz(tx_frequency).map_err(|e| {
        RepeaterAtlasError::Other(
            Box::new(e),
            format!("invalid tx frequency for call_sign={}", r.call_sign),
        )
    })?;
    let rx_hz = Frequency::new_hz(tx_frequency + offset).map_err(|e| {
        RepeaterAtlasError::Other(
            Box::new(e),
            format!("invalid rx frequency for call_sign={}", r.call_sign),
        )
    })?;
    let service = RepeaterService::Fm {
        label,
        rx_hz,
        tx_hz,
        bandwidth: FmBandwidth::Narrow,
        rx_tone: tone.clone(),
        tx_tone: tone,
        note: None,
    };
    create_service(c, r.system.id, service).await
}

async fn igate(
    c: &mut AsyncPgConnection,
    r: &RepeaterFixture,
    label: impl Into<String>,
    frequency: i64,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let hz = Frequency::new_hz(frequency).map_err(|e| {
        RepeaterAtlasError::Other(
            Box::new(e),
            format!("invalid aprs frequency for call_sign={}", r.call_sign),
        )
    })?;
    let service = RepeaterService::Aprs {
        label,
        rx_hz: hz,
        tx_hz: hz,
        mode: Some(AprsMode::Igate),
        path: None,
        note: None,
    };
    create_service(c, r.system.id, service).await
}

async fn digipeater(
    c: &mut AsyncPgConnection,
    r: &RepeaterFixture,
    label: impl Into<String>,
    frequency: i64,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let hz = Frequency::new_hz(frequency).map_err(|e| {
        RepeaterAtlasError::Other(
            Box::new(e),
            format!("invalid aprs frequency for call_sign={}", r.call_sign),
        )
    })?;
    let service = RepeaterService::Aprs {
        label,
        rx_hz: hz,
        tx_hz: hz,
        mode: Some(AprsMode::Digipeater),
        path: None,
        note: None,
    };
    create_service(c, r.system.id, service).await
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
    let contacts = load_contacts(c, PathBuf::from("data/ham_clubs.csv")).await?;

    let mut repeater_files = Vec::new();
    for d in Path::new("data").read_dir()? {
        let d = d?;
        let name = d.file_name();
        let name: &str = name.to_str().unwrap_or("");
        if name.starts_with("repeaters-") {
            repeater_files.push(d.path());
        }
    }

    repeater_files.sort();
    for path in repeater_files {
        load_repeaters(c, &contacts, path).await?;
    }

    let links_path = PathBuf::from("data/repeater-links.csv");
    if links_path.exists() {
        load_repeater_links(c, links_path).await?;
    }

    Ok(())
}

pub async fn load_repeater_links(
    c: &mut AsyncPgConnection,
    path: PathBuf,
) -> Result<(), RepeaterAtlasError> {
    let (headers, records) = load_csv(&path)?;

    let mut call_sign_a_index = None;
    let mut call_sign_b_index = None;
    for (idx, header) in headers.iter().enumerate() {
        match header.trim() {
            "call_sign_a" => call_sign_a_index = Some(idx),
            "call_sign_b" => call_sign_b_index = Some(idx),
            _ => {}
        }
    }
    let (Some(call_sign_a_index), Some(call_sign_b_index)) = (call_sign_a_index, call_sign_b_index)
    else {
        return Err(RepeaterAtlasError::Other(
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing required headers call_sign_a/call_sign_b",
            )),
            format!("invalid repeater links csv: {}", path.to_string_lossy()),
        ));
    };

    let mut imported = 0usize;
    for (row_index, record) in records.iter().enumerate() {
        let row_index = row_index + 2;
        let call_sign_a_raw = record.get(call_sign_a_index).unwrap_or("").trim();
        let call_sign_b_raw = record.get(call_sign_b_index).unwrap_or("").trim();
        if call_sign_a_raw.is_empty() || call_sign_b_raw.is_empty() {
            info!(row = row_index, reason = "missing call signs", "Skipping link row");
            continue;
        }

        let (call_sign_a, _) = split_call_sign(call_sign_a_raw);
        let (call_sign_b, _) = split_call_sign(call_sign_b_raw);

        let Some(a) = dao::repeater_system::find_by_call_sign(c, call_sign_a.clone()).await? else {
            info!(
                row = row_index,
                call_sign = call_sign_a,
                reason = "unknown repeater",
                "Skipping link row"
            );
            continue;
        };
        let Some(b) = dao::repeater_system::find_by_call_sign(c, call_sign_b.clone()).await? else {
            info!(
                row = row_index,
                call_sign = call_sign_b,
                reason = "unknown repeater",
                "Skipping link row"
            );
            continue;
        };

        let (repeater_a_id, repeater_b_id) = if a.id < b.id {
            (a.id, b.id)
        } else {
            (b.id, a.id)
        };
        dao::repeater_link::insert(
            c,
            dao::repeater_link::NewRepeaterLink::new(repeater_a_id, repeater_b_id),
        )
        .await?;
        imported += 1;
    }

    info!(
        file = path.to_string_lossy().as_ref(),
        imported = imported,
        "Imported repeater links"
    );

    Ok(())
}

pub async fn load_contacts(
    c: &mut AsyncPgConnection,
    path: PathBuf,
) -> Result<HashMap<String, Contact>, RepeaterAtlasError> {
    let (headers, records) = load_csv(&path)?;
    let mut contacts = HashMap::new();

    for (row_index, record) in records.iter().enumerate() {
        let row_index = row_index + 2;
        let row = row_from_record(&headers, record)?;

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

        if contacts.contains_key(&call_sign) {
            info!(
                row = row_index,
                call_sign = call_sign,
                "Skipping duplicate contact row"
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

        let entity = dao::entity::insert(
            c,
            NewEntity {
                kind: EntityKind::Contact,
                call_sign: Some(call_sign.clone()),
            },
        )
        .await?;

        let contact = dao::contact::insert(
            c,
            NewContact {
                entity: entity.id,
                kind: ContactKind::Organization,
                display_name: name.unwrap_or_else(|| call_sign.clone()),
                description: None,
                web_url,
                email,
                phone: None,
                address: None,
            },
        )
        .await?;

        contacts.insert(call_sign, contact);
    }

    Ok(contacts)
}

pub async fn load_repeaters(
    c: &mut AsyncPgConnection,
    contacts: &HashMap<String, Contact>,
    path: PathBuf,
) -> Result<(), RepeaterAtlasError> {
    let mut imported = 0usize;
    let mut repeaters = HashMap::<String, RepeaterFixture>::new();

    let (headers, records) = load_csv(&path)?;

    for (row_index, record) in records.iter().enumerate() {
        let row_index = row_index + 2;
        let row = row_from_record(&headers, record)?;

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
        let contact = owner
            .map(normalize_call_sign)
            .and_then(|value| contacts.get(&value).cloned());

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
                repeater_with_site(c, call_sign.clone(), contact.as_ref(), address, maidenhead)
                    .await?;

            if let Some(name) = row
                .get("name")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
            {
                repeater.system.name = Some(name.to_string());
                repeater.system = dao::repeater_system::update(c, repeater.system.clone()).await?;
            }

            if let Some(status) = row
                .get("status")
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
            {
                repeater.system.status = status.to_string();
                repeater.system = dao::repeater_system::update(c, repeater.system.clone()).await?;
            }

            repeaters.insert(call_sign.clone(), repeater.clone());
            repeater
        };

        let service = row
            .get("service")
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        let tx_frequency = parse_tx_frequency(&row);
        let rx_frequency = parse_rx_frequency(&row);
        let offset = parse_offset(&row)
            .or_else(|| match (tx_frequency, rx_frequency) {
                (Some(tx), Some(rx)) => Some(rx.hz() - tx.hz()),
                _ => None,
            })
            .or_else(|| tx_frequency.map(|value| value.hz()).and_then(default_offset));
        let ctcss = row
            .get("ctcss_tx")
            .and_then(|value| parse_ctcss(value))
            .or_else(|| row.get("ctcss_rx").and_then(|value| parse_ctcss(value)))
            .or_else(|| row.get("ctcss").and_then(|value| parse_ctcss(value)))
            .or_else(|| row.get("CTCSS").and_then(|value| parse_ctcss(value)));
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
                    .unwrap_or(label_for_frequency(tx_frequency.hz()));
                narrow_fm(c, &repeater, label, tx_frequency.hz(), offset, ctcss).await?;
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
                    .unwrap_or(label_for_frequency(tx_frequency.hz()));
                igate(c, &repeater, label, tx_frequency.hz()).await?;
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
                    .unwrap_or(label_for_frequency(tx_frequency.hz()));
                digipeater(c, &repeater, label, tx_frequency.hz()).await?;
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
                    .unwrap_or(label_for_frequency(tx_frequency.hz()));
                let rx_hz = match Frequency::new_hz(tx_frequency.hz() + offset) {
                    Ok(value) => value,
                    Err(_) => {
                        info!(
                            row = row_index,
                            call_sign = call_sign,
                            reason = "invalid rx frequency",
                            "Skipping repeater row"
                        );
                        continue;
                    }
                };
                let service = RepeaterService::Dmr {
                    label: label.to_string(),
                    rx_hz,
                    tx_hz: tx_frequency,
                    color_code: 1,
                    dmr_repeater_id: dmr_id,
                    network: "unknown".to_string(),
                    note: None,
                };
                create_service(c, repeater.system.id, service).await?;
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
                    .unwrap_or(label_for_frequency(tx_frequency.hz()));
                let rx_hz = match Frequency::new_hz(tx_frequency.hz() + offset) {
                    Ok(value) => value,
                    Err(_) => {
                        info!(
                            row = row_index,
                            call_sign = call_sign,
                            reason = "invalid rx frequency",
                            "Skipping repeater row"
                        );
                        continue;
                    }
                };
                let service = RepeaterService::C4fm {
                    label: label.to_string(),
                    rx_hz,
                    tx_hz: tx_frequency,
                    wires_x_node_id: None,
                    room: None,
                    note: None,
                };
                create_service(c, repeater.system.id, service).await?;
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

fn parse_hz_field(row: &HashMap<String, String>, key: &str) -> Option<i64> {
    let raw = row.get(key).map(|value| value.trim()).filter(|value| !value.is_empty())?;

    // Prefer explicit Hz integers.
    if let Ok(value) = raw.parse::<i64>() {
        return Some(value);
    }

    // Otherwise interpret as MHz with decimals (e.g. "145.625").
    raw.parse::<f64>()
        .ok()
        .map(|value| (value * 1_000_000.0).round() as i64)
}

fn parse_tx_frequency(row: &HashMap<String, String>) -> Option<Frequency> {
    let tx_hz = parse_hz_field(row, "tx")
        .or_else(|| parse_hz_field(row, "tx_hz"))
        .or_else(|| parse_hz_field(row, "tx_mhz"));
    tx_hz.and_then(|value| Frequency::new_hz(value).ok())
}

fn parse_rx_frequency(row: &HashMap<String, String>) -> Option<Frequency> {
    let rx_hz = parse_hz_field(row, "rx")
        .or_else(|| parse_hz_field(row, "rx_hz"))
        .or_else(|| parse_hz_field(row, "rx_mhz"));
    rx_hz.and_then(|value| Frequency::new_hz(value).ok())
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
