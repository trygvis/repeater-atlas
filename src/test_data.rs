use crate::dao::call_sign::NewCallSign;
use crate::dao::contact::{Contact, ContactKind, NewContact};
use crate::dao::repeater_service::{AprsMode, FmBandwidth};
use crate::dao::repeater_system::{NewRepeaterSystem, RepeaterSystem};
use crate::service::repeater_service::{RepeaterService, Tone};
use crate::{Frequency, RepeaterAtlasError, dao};
use crate::{MaidenheadLocator, service};
use csv::StringRecord;
use dao::repeater_service::{DstarMode, SsbSideband, ToneKind};
use diesel::QueryResult;
use diesel_async::AsyncPgConnection;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{Level, info, span};

fn parse_ctcss(value: String) -> Option<f32> {
    // Accept "123", "123.0", "123.0 Hz".
    let first = value.split_whitespace().next().unwrap_or(value.as_str());
    first.parse::<f32>().ok()
}

async fn narrow_fm(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    tx_hz: Frequency,
    offset_hz: i64,
    rx_tone: Option<f32>,
    tx_tone: Option<f32>,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let rx_tone = rx_tone.map(Tone::CTCSS).unwrap_or(Tone::None);
    let tx_tone = tx_tone.map(Tone::CTCSS).unwrap_or(Tone::None);
    let rx_hz = tx_hz.offset(offset_hz).map_err(|e| {
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
        rx_tone,
        tx_tone,
        note: None,
    };
    service::repeater_service::create_service(c, r.id, service).await
}

async fn igate(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    hz: Frequency,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let service = RepeaterService::Aprs {
        label,
        rx_hz: hz,
        tx_hz: hz,
        mode: Some(AprsMode::Igate),
        path: None,
        note: None,
    };
    service::repeater_service::create_service(c, r.id, service).await
}

async fn digipeater(
    c: &mut AsyncPgConnection,
    r: &RepeaterSystem,
    label: impl Into<String>,
    hz: Frequency,
) -> Result<(), RepeaterAtlasError> {
    let label = label.into();
    let service = RepeaterService::Aprs {
        label,
        rx_hz: hz,
        tx_hz: hz,
        mode: Some(AprsMode::Digipeater),
        path: None,
        note: None,
    };
    service::repeater_service::create_service(c, r.id, service).await
}

fn split_call_sign(input: String) -> (String, Option<String>) {
    if let Some((head, label)) = input.split_once('-') {
        let label = if label.is_empty() {
            None
        } else {
            Some(label.to_ascii_uppercase())
        };
        (head.trim().to_ascii_uppercase(), label)
    } else {
        (input.to_ascii_uppercase(), None)
    }
}

fn normalize_call_sign(input: String) -> String {
    input.trim().to_ascii_uppercase()
}

fn load_csv(path: &Path) -> Result<CsvFile, RepeaterAtlasError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_path(path)?;

    let headers = reader
        .headers()?
        .clone()
        .iter()
        .map(|header| header.trim().to_lowercase())
        .zip(0usize..)
        .filter(|(header, _)| !header.is_empty())
        .collect();

    let records: Vec<StringRecord> = reader
        .records()
        .collect::<Result<Vec<StringRecord>, csv::Error>>()?;
    let records: Vec<Vec<String>> = records
        .iter()
        .map(|row| row.iter().map(|cell| cell.trim().to_string()).collect())
        .collect();

    Ok(CsvFile {
        headers,
        data: records,
    })
}

pub async fn dump_data(c: &mut AsyncPgConnection) -> Result<(), RepeaterAtlasError> {
    let repeaters = dao::repeater_system::select_with_call_sign(c).await?;

    #[derive(Serialize)]
    struct RepeaterSystemRow {
        call_sign: String,
        owner: Option<String>,
        tech_contact: Option<String>,
        name: Option<String>,
        description: Option<String>,
        address: Option<String>,
        maidenhead: Option<String>,
        latitude: Option<f64>,
        longitude: Option<f64>,
        elevation_m: Option<i32>,
        country: Option<String>,
        region: Option<String>,
        status: String,
    }

    let mut writer = csv::Writer::from_path(PathBuf::from("data/out/repeater-systems.csv"))?;

    for rs in &repeaters {
        let owner = match rs.owner {
            Some(id) => dao::contact::get(c, id).await?.call_sign,
            None => None,
        };
        let tech_contact = match rs.tech_contact {
            Some(id) => dao::contact::get(c, id).await?.call_sign,
            None => None,
        };

        writer.serialize(RepeaterSystemRow {
            call_sign: rs.call_sign.clone(),
            owner,
            tech_contact,
            name: rs.name.clone(),
            description: rs.description.clone(),
            address: rs.address.clone(),
            maidenhead: rs.maidenhead.as_ref().map(|mh| mh.to_string()),
            latitude: rs.latitude,
            longitude: rs.longitude,
            elevation_m: rs.elevation_m,
            country: rs.country.clone(),
            region: rs.region.clone(),
            status: rs.status.clone(),
        })?;
    }

    #[derive(Serialize)]
    struct RepeaterServiceRow {
        repeater_call_sign: String,
        kind: dao::repeater_service::RepeaterServiceKind,
        enabled: bool,
        label: String,
        note: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        fm_bandwidth: Option<FmBandwidth>,
        rx_tone_kind: Option<ToneKind>,
        rx_ctcss_hz: Option<f32>,
        rx_dcs_code: Option<i32>,
        tx_tone_kind: Option<ToneKind>,
        tx_ctcss_hz: Option<f32>,
        tx_dcs_code: Option<i32>,
        dmr_color_code: Option<i16>,
        dmr_repeater_id: Option<i64>,
        dmr_network: Option<String>,
        dstar_mode: Option<DstarMode>,
        dstar_gateway_call_sign: Option<String>,
        dstar_reflector: Option<String>,
        c4fm_wires_x_node_id: Option<i32>,
        c4fm_room: Option<String>,
        aprs_mode: Option<AprsMode>,
        aprs_path: Option<String>,
        ssb_sideband: Option<SsbSideband>,
    }

    let mut writer = csv::Writer::from_path(PathBuf::from("data/out/repeater-services.csv"))?;

    for rs in repeaters {
        let service = dao::repeater_service::select_by_repeater_id(c, rs.id).await?;

        for s in service {
            writer.serialize(RepeaterServiceRow {
                repeater_call_sign: rs.call_sign.clone(),
                kind: s.kind,
                enabled: s.enabled,
                label: s.label,
                note: s.note,
                rx_hz: s.rx_hz,
                tx_hz: s.tx_hz,
                fm_bandwidth: s.fm_bandwidth,
                rx_tone_kind: s.rx_tone_kind,
                rx_ctcss_hz: s.rx_ctcss_hz,
                rx_dcs_code: s.rx_dcs_code,
                tx_tone_kind: s.tx_tone_kind,
                tx_ctcss_hz: s.tx_ctcss_hz,
                tx_dcs_code: s.tx_dcs_code,
                dmr_color_code: s.dmr_color_code,
                dmr_repeater_id: s.dmr_repeater_id,
                dmr_network: s.dmr_network,
                dstar_mode: s.dstar_mode,
                dstar_gateway_call_sign: s.dstar_gateway_call_sign,
                dstar_reflector: s.dstar_reflector,
                c4fm_wires_x_node_id: s.c4fm_wires_x_node_id,
                c4fm_room: s.c4fm_room,
                aprs_mode: s.aprs_mode,
                aprs_path: s.aprs_path,
                ssb_sideband: s.ssb_sideband,
            })?;
        }
    }

    Ok(())
}

pub async fn load_data(c: &mut AsyncPgConnection) -> Result<(), RepeaterAtlasError> {
    let contacts = load_contacts(c, PathBuf::from("data/contacts.csv")).await?;
    info!("Loaded {} contacts", contacts.len());

    // This is a bigger dataset, but of lower quality. Import this first, then let the latter data
    // override them
    // load_nrrl_repeaters(c, &contacts, PathBuf::from("data/NRRL-Relestasjoner.csv")).await?;

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
        let _ = span!(
            Level::INFO,
            "load_repeaters",
            path = path.to_string_lossy().to_string()
        );
        load_repeaters(c, path).await?;
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
    let csv = load_csv(&path)?;

    let call_sign_a_index = csv
        .get_header("call_sign_a")
        .expect("Missing required column: call_sign_a")
        .clone();
    let call_sign_b_index = csv
        .get_header("call_sign_b")
        .expect("Missing required column: call_sign_b")
        .clone();

    let mut imported = 0usize;
    for (row_index, record) in csv.data.iter().enumerate() {
        let row_index = row_index + 2;
        let (call_sign_a_raw, call_sign_b_raw) =
            match (record.get(call_sign_a_index), record.get(call_sign_b_index)) {
                (Some(a), Some(b)) => (a.clone(), b.clone()),
                (_, _) => {
                    info!(
                        row = row_index,
                        reason = "missing call signs",
                        "Skipping link row"
                    );
                    continue;
                }
            };

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
    let csv = load_csv(&path)?;
    let mut contacts = HashMap::new();

    let call_sign_idx = match csv
        .get_header("call_sign")
        .or(csv.get_header("Kallesignal"))
    {
        Ok(idx) => idx,
        Err(e) => return Err(e),
    };

    let name_idx = csv.get_header("name")?;
    let web_url_idx = csv.get_header("web_url")?;
    let email_idx = csv.get_header("email")?;

    for (row, row_index) in csv.data.iter().zip(2..) {
        let call_sign_raw = match row.get(call_sign_idx) {
            Some(value) => value.to_string(),
            None => {
                info!(
                    row = row_index,
                    reason = "missing call_sign",
                    "Skipping contact row"
                );
                continue;
            }
        };
        let call_sign = normalize_call_sign(call_sign_raw);
        if call_sign.is_empty() {
            info!(
                row = row_index,
                reason = "empty call_sign",
                "Skipping contact row"
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

        let name = row.get(name_idx);
        let web_url = row.get(web_url_idx);
        let email = row.get(email_idx);

        let call_sign_row = dao::call_sign::insert(c, NewCallSign::new_contact(&call_sign)).await?;

        let contact = dao::contact::insert(
            c,
            NewContact {
                call_sign: Some(call_sign_row.value),
                kind: ContactKind::Organization,
                display_name: name.unwrap_or_else(|| &call_sign).to_string(),
                description: None,
                web_url: web_url.cloned(),
                email: email.cloned(),
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
    path: PathBuf,
) -> Result<(), RepeaterAtlasError> {
    let mut imported = 0usize;

    let csv = load_csv(&path)?;

    let call_sign_idx = csv.get_header("call_sign")?;
    let owner_idx = csv.get_header("owner")?;
    let address_idx = csv.get_header("address");
    let maidenhead_idx = csv.get_header("maidenhead");
    let name_idx = csv.get_header("name");
    let status_idx = csv.get_header("status");
    let service_idx = csv.get_header("service")?;
    let tx_frequency_idx = csv.get_first_header(vec!["tx", "tx_hz", "tx_mhz"])?;
    let rx_frequency_idx = csv.get_first_header(vec!["rx", "rx_hz", "rx_mhz"]);
    let offset_idx = csv.get_header("offset");
    let ctcss_tx_idx = csv.get_header("ctcss_tx").or(csv.get_header("ctcss"));
    let ctcss_rx_idx = csv.get_header("ctcss_rx").or(csv.get_header("ctcss"));
    let dmr_id_idx = csv.get_header("dmr_id");

    for (_, row) in csv.data.iter().zip(0..) {
        let call_sign_raw = match csv.get(row, call_sign_idx) {
            Some(value) => value.to_string(),
            None => {
                info!(
                    row = row + 2,
                    reason = "missing call_sign",
                    "Skipping repeater row"
                );
                continue;
            }
        };
        let (call_sign, port_label) = split_call_sign(call_sign_raw);

        let owner = csv.get(row, owner_idx).map(normalize_call_sign);
        let owner = match owner {
            Some(owner) => dao::contact::find_by_call_sign(c, owner).await?,
            None => None,
        };

        let name = csv.get_opt(row, &name_idx);

        // All repeaters as assumed to be active if in repeaters-*.csv list.
        let status = csv
            .get_opt(row, &status_idx)
            .unwrap_or("active".to_string());
        let address = csv.get_opt(row, &address_idx);
        let maidenhead = csv.get_opt(row, &maidenhead_idx);

        let repeater = create_repeater(
            c,
            call_sign.clone(),
            owner,
            name,
            status,
            address,
            maidenhead,
        )
        .await?;

        let service = csv.get(row, service_idx);
        let tx_frequency = csv.get(row, tx_frequency_idx).and_then(parse_hz_field);

        let rx_frequency = csv.get_opt(row, &rx_frequency_idx).and_then(parse_hz_field);

        let offset = csv
            .get_opt(row, &offset_idx)
            .and_then(|s| s.parse::<i64>().ok())
            .or_else(|| match (tx_frequency, rx_frequency) {
                (Some(tx), Some(rx)) => Some(rx.hz() - tx.hz()),
                _ => None,
            })
            .or_else(|| tx_frequency.and_then(default_offset));
        let ctcss_tx = csv
            .get_opt(row, &ctcss_tx_idx)
            .and_then(|value| parse_ctcss(value));
        let ctcss_rx = csv
            .get_opt(row, &ctcss_rx_idx)
            .and_then(|value| parse_ctcss(value));
        let dmr_id = csv
            .get_opt(row, &dmr_id_idx)
            .and_then(|value| value.parse::<i64>().ok());

        let service = service.unwrap_or_default();
        let service = service.as_str();
        match service {
            "FM_NARROW" => {
                let (Some(tx_frequency), Some(offset)) = (tx_frequency, offset) else {
                    info!(
                        row = row + 2,
                        call_sign = call_sign,
                        file = path.to_string_lossy().as_ref(),
                        reason = "missing tx/offset",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label.as_deref().unwrap_or(tx_frequency.band_label());
                narrow_fm(
                    c,
                    &repeater,
                    label,
                    tx_frequency,
                    offset,
                    ctcss_rx,
                    ctcss_tx,
                )
                .await?;
                imported += 1;
            }
            "APRS_IGATE" => {
                let Some(tx_frequency) = tx_frequency else {
                    info!(
                        row = row + 2,
                        call_sign = call_sign,
                        reason = "missing tx",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label.as_deref().unwrap_or(tx_frequency.band_label());
                igate(c, &repeater, label, tx_frequency).await?;
                imported += 1;
            }
            "APRS_DIGIPEATER" => {
                let Some(tx_frequency) = tx_frequency else {
                    info!(
                        row = row + 2,
                        call_sign = call_sign,
                        reason = "missing tx",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label.as_deref().unwrap_or(tx_frequency.band_label());
                digipeater(c, &repeater, label, tx_frequency).await?;
                imported += 1;
            }
            "DMR" => {
                let (Some(tx_frequency), Some(offset)) = (tx_frequency, offset) else {
                    info!(
                        row = row + 2,
                        call_sign = call_sign,
                        reason = "missing tx/offset",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label.as_deref().unwrap_or(tx_frequency.band_label());
                let rx_hz = match tx_frequency.offset(offset) {
                    Ok(value) => value,
                    Err(_) => {
                        info!(
                            row = row + 2,
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
                service::repeater_service::create_service(c, repeater.id, service).await?;
                imported += 1;
            }
            "C4FM" => {
                let (Some(tx_frequency), Some(offset)) = (tx_frequency, offset) else {
                    info!(
                        row = row + 2,
                        call_sign = call_sign,
                        reason = "missing tx/offset",
                        "Skipping repeater row"
                    );
                    continue;
                };
                let label = port_label.as_deref().unwrap_or(tx_frequency.band_label());
                let rx_hz = match tx_frequency.offset(offset) {
                    Ok(value) => value,
                    Err(_) => {
                        info!(
                            row = row + 2,
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
                service::repeater_service::create_service(c, repeater.id, service).await?;
                imported += 1;
            }
            "" => {
                info!(
                    row = row + 2,
                    call_sign = call_sign,
                    reason = "missing service",
                    "Skipping repeater row"
                );
            }
            service => {
                info!(
                    row = row + 2,
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

async fn create_repeater(
    c: &mut AsyncPgConnection,
    call_sign: String,
    owner: Option<Contact>,
    name: Option<String>,
    status: String,
    address: Option<String>,
    maidenhead: Option<String>,
) -> Result<RepeaterSystem, RepeaterAtlasError> {
    let existing = dao::repeater_system::find_by_call_sign(c, call_sign.clone()).await?;

    let maidenhead = maidenhead.and_then(|s| MaidenheadLocator::new(s).ok());

    let geocoder = service::geocoding::nominatim_geocoder_from_env()?;
    let enriched = service::enrich_location::enrich_location(
        geocoder,
        &call_sign,
        address.as_deref(),
        maidenhead.as_ref(),
    )
    .await?;

    let longitude = enriched.as_ref().map(|e| e.longitude.clone());
    let latitude = enriched.as_ref().map(|e| e.latitude.clone());
    let maidenhead = maidenhead.or_else(|| enriched.map(|e| e.maidenhead.clone()));

    match existing {
        Some(mut existing) => {
            // Merge in new fields into system
            existing.name = name.or_else(|| existing.name);
            existing.owner = owner.map(|c| c.id).or_else(|| existing.owner);
            existing.address = address.or_else(|| existing.address);
            existing.latitude = latitude.or_else(|| existing.latitude);
            existing.longitude = longitude.or_else(|| existing.longitude);
            existing.maidenhead = maidenhead.or_else(|| existing.maidenhead);
            existing.status = Some(status)
                .filter(|s| !s.is_empty())
                .or(Some(existing.status).filter(|s| !s.is_empty()))
                .unwrap_or_default();

            service::repeater_system::update(c, existing).await
        }
        None => {
            let repeater = NewRepeaterSystem {
                call_sign: call_sign.clone(),
                owner: owner.map(|c| c.id),
                tech_contact: None,
                name,
                description: None,
                address,
                maidenhead,
                latitude,
                longitude,
                elevation_m: None,
                country: None,
                region: None,
                status,
            };

            service::repeater_system::create_repeater_system(c, repeater).await
        }
    }
}

fn parse_hz_field(raw: String) -> Option<Frequency> {
    // Prefer explicit Hz integers.
    if let Ok(value) = raw.parse::<i64>() {
        return Frequency::new_hz(value).ok();
    }

    let raw = raw //
        .replace("Hz", "")
        .replace("hz", "");

    // Otherwise interpret as MHz with decimals (e.g. "145.625").
    raw.trim()
        .parse::<f64>()
        .ok()
        .map(|value| (value * 1_000_000.0).round() as i64)
        .and_then(|f| Frequency::new_hz(f).ok())
}

fn default_offset(tx_hz: Frequency) -> Option<i64> {
    if tx_hz.contained_in(144_000_000..148_000_000) {
        Some(-600_000)
    } else if tx_hz.contained_in(430_000_000..450_000_000) {
        Some(-2_000_000)
    } else {
        None
    }
}

pub async fn generate_users(c: &mut AsyncPgConnection) -> QueryResult<()> {
    service::user::create_user(c, "LA8PV", "la8pv@example.org", "la8pv").await?;
    Ok(())
}

struct CsvFile {
    pub headers: HashMap<String, usize>,
    pub data: Vec<Vec<String>>,
}

impl CsvFile {
    pub(crate) fn get_header(&self, name: &str) -> Result<usize, RepeaterAtlasError> {
        self.headers
            .get(name)
            .map(|sz| sz.clone())
            .ok_or(RepeaterAtlasError::OtherMsg(format!(
                "Missing required column: {name}"
            )))
    }

    pub(crate) fn get_first_header(&self, names: Vec<&str>) -> Result<usize, RepeaterAtlasError> {
        let ns = names.clone().join(",");

        let names = names.iter().map(|s| s.to_string()).collect::<Vec<String>>();

        for n in names {
            if let Some(idx) = self.headers.get(&n) {
                return Ok(*idx);
            }
        }

        Err(RepeaterAtlasError::OtherMsg(format!(
            "Missing required column (one of): {ns}"
        )))
    }

    pub fn get(&self, row: usize, column: usize) -> Option<String> {
        let row = self.data.get(row)?;
        row.get(column)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }

    pub fn get_opt(
        &self,
        row: usize,
        column: &Result<usize, RepeaterAtlasError>,
    ) -> Option<String> {
        match column {
            Ok(column) => self.get(row, *column),
            Err(_) => None,
        }
    }
}
