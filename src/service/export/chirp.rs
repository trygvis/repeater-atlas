use super::{ExportOptions, load_repeaters_for_export};
use crate::dao::repeater_service::FmBandwidth;
use crate::service::repeater_service::Tone;
use crate::service::repeater_system::FmServiceItem;
use crate::{Frequency, RepeaterAtlasError};
use diesel_async::AsyncPgConnection;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ChripRepeaterRow {
    #[serde(rename = "Location")]
    location: i64,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Frequency")]
    frequency: String,
    #[serde(rename = "Duplex")]
    duplex: String,
    #[serde(rename = "Offset")]
    offset: String,
    #[serde(rename = "Tone")]
    tone: String,
    #[serde(rename = "rToneFreq")]
    r_tone_freq: String,
    #[serde(rename = "cToneFreq")]
    c_tone_freq: String,
    #[serde(rename = "DtcsCode")]
    dtcs_code: String,
    #[serde(rename = "DtcsPolarity")]
    dtcs_polarity: String,
    #[serde(rename = "RxDtcsCode")]
    rx_dtcs_code: String,
    #[serde(rename = "CrossMode")]
    cross_mode: String,
    #[serde(rename = "Mode")]
    mode: String,
    #[serde(rename = "TStep")]
    t_step: String,
    #[serde(rename = "Skip")]
    skip: String,
    #[serde(rename = "Power")]
    power: String,
    #[serde(rename = "Comment")]
    comment: String,
    #[serde(rename = "URCALL")]
    urcall: String,
    #[serde(rename = "RPT1CALL")]
    rpt1call: String,
    #[serde(rename = "RPT2CALL")]
    rpt2call: String,
    #[serde(rename = "DVCODE")]
    dvcode: String,
}

pub async fn chirp_export<W: std::io::Write>(
    c: &mut AsyncPgConnection,
    options: ExportOptions,
    writer: &mut W,
) -> Result<(), RepeaterAtlasError> {
    let repeaters = load_repeaters_for_export(c).await?;
    let mut rows = Vec::new();

    for repeater in repeaters {
        for service in &repeater.services.fm_services {
            rows.push(build_fm_row(
                rows.len() as i64,
                &repeater.call_sign,
                service,
                &options,
            ));
        }
    }

    let mut csv_writer = csv::Writer::from_writer(writer);
    for row in rows {
        csv_writer.serialize(row)?;
    }
    Ok(())
}

fn build_fm_row(
    location: i64,
    call_sign: &str,
    service: &FmServiceItem,
    options: &ExportOptions,
) -> ChripRepeaterRow {
    let (frequency, duplex, offset_hz) = frequency_fields(service.tx_hz, service.rx_hz);
    let (tone, cross_mode, r_tone_freq, c_tone_freq, dtcs_code, rx_dtcs_code, dtcs_polarity) =
        tone_fields(service, options);

    ChripRepeaterRow {
        location,
        name: format!("{call_sign} {}", service.label).trim().to_string(),
        frequency: frequency.to_string_mhz(),
        duplex,
        offset: Frequency::new_hz(offset_hz)
            .expect("offset should be non-negative")
            .to_string_mhz(),
        tone,
        r_tone_freq,
        c_tone_freq,
        dtcs_code,
        dtcs_polarity,
        rx_dtcs_code,
        cross_mode,
        mode: match service.bandwidth {
            FmBandwidth::Narrow => "NFM".to_string(),
            FmBandwidth::Wide => "FM".to_string(),
        },
        t_step: "5.00".to_string(),
        skip: String::new(),
        power: String::new(),
        comment: service.note.clone(),
        urcall: String::new(),
        rpt1call: String::new(),
        rpt2call: String::new(),
        dvcode: String::new(),
    }
}

fn frequency_fields(tx_hz: Frequency, rx_hz: Frequency) -> (Frequency, String, i64) {
    let offset_hz = rx_hz.hz() - tx_hz.hz();
    if offset_hz == 0 {
        (tx_hz, "".to_string(), 0)
    } else if offset_hz > 0 {
        (tx_hz, "+".to_string(), offset_hz)
    } else {
        (tx_hz, "-".to_string(), -offset_hz)
    }
}

fn tone_fields(
    service: &FmServiceItem,
    options: &ExportOptions,
) -> (String, String, String, String, String, String, String) {
    let dtcs_polarity = "NN".to_string();
    let rx_tone = if options.export_rx_tone {
        service.rx_tone.clone()
    } else {
        Tone::None
    };

    let (tone, cross_mode, r_tone_freq, c_tone_freq, dtcs_code, rx_dtcs_code) =
        match (service.tx_tone.clone(), rx_tone) {
            (Tone::None, Tone::None) => (
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ),
            (Tone::CTCSS(tx_hz), Tone::None) => (
                "Tone".to_string(),
                String::new(),
                format!("{tx_hz:.1}"),
                String::new(),
                String::new(),
                String::new(),
            ),
            (Tone::None, Tone::CTCSS(rx_hz)) => (
                "TSQL".to_string(),
                String::new(),
                String::new(),
                format!("{rx_hz:.1}"),
                String::new(),
                String::new(),
            ),
            (Tone::CTCSS(tx_hz), Tone::CTCSS(rx_hz)) if tx_hz == rx_hz => (
                "TSQL".to_string(),
                String::new(),
                format!("{tx_hz:.1}"),
                format!("{rx_hz:.1}"),
                String::new(),
                String::new(),
            ),
            (Tone::CTCSS(tx_hz), Tone::CTCSS(rx_hz)) => (
                "Cross".to_string(),
                "Tone->Tone".to_string(),
                format!("{tx_hz:.1}"),
                format!("{rx_hz:.1}"),
                String::new(),
                String::new(),
            ),
            (Tone::DCS(tx_code), Tone::None) => (
                "Cross".to_string(),
                "DTCS->".to_string(),
                String::new(),
                String::new(),
                format_dtcs(Some(tx_code)),
                String::new(),
            ),
            (Tone::None, Tone::DCS(rx_code)) => (
                "Cross".to_string(),
                "->DTCS".to_string(),
                String::new(),
                String::new(),
                String::new(),
                format_dtcs(Some(rx_code)),
            ),
            (Tone::DCS(tx_code), Tone::DCS(rx_code)) if tx_code == rx_code => (
                "DTCS".to_string(),
                String::new(),
                String::new(),
                String::new(),
                format_dtcs(Some(tx_code)),
                String::new(),
            ),
            (Tone::DCS(tx_code), Tone::DCS(rx_code)) => (
                "Cross".to_string(),
                "DTCS->DTCS".to_string(),
                String::new(),
                String::new(),
                format_dtcs(Some(tx_code)),
                format_dtcs(Some(rx_code)),
            ),
            (Tone::CTCSS(tx_hz), Tone::DCS(rx_code)) => (
                "Cross".to_string(),
                "Tone->DTCS".to_string(),
                format!("{tx_hz:.1}"),
                String::new(),
                String::new(),
                format_dtcs(Some(rx_code)),
            ),
            (Tone::DCS(tx_code), Tone::CTCSS(rx_hz)) => (
                "Cross".to_string(),
                "DTCS->Tone".to_string(),
                String::new(),
                format!("{rx_hz:.1}"),
                format_dtcs(Some(tx_code)),
                String::new(),
            ),
        };

    (
        tone,
        cross_mode,
        r_tone_freq,
        c_tone_freq,
        dtcs_code,
        rx_dtcs_code,
        dtcs_polarity,
    )
}

fn format_dtcs(code: Option<i32>) -> String {
    code.map(|value| format!("{value:03}")).unwrap_or_default()
}
