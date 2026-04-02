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
        power: "50W".to_string(),
        comment: service.note.clone(),
        urcall: String::new(),
        rpt1call: String::new(),
        rpt2call: String::new(),
        dvcode: String::new(),
    }
}

fn frequency_fields(tx_hz: Frequency, rx_hz: Frequency) -> (Frequency, String, i64) {
    // CHIRP's Frequency field is what the radio transmits on (= repeater RX).
    // CHIRP's Offset is how to reach the repeater TX from there: tx_hz - rx_hz.
    let offset_hz = tx_hz.hz() - rx_hz.hz();
    if offset_hz == 0 {
        (rx_hz, "".to_string(), 0)
    } else if offset_hz > 0 {
        (rx_hz, "+".to_string(), offset_hz)
    } else {
        (rx_hz, "-".to_string(), -offset_hz)
    }
}

const DEFAULT_TONE_FREQ: &str = "88.5";
const DEFAULT_DTCS_CODE: &str = "023";
const DEFAULT_CROSS_MODE: &str = "Tone->Tone";

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
                DEFAULT_CROSS_MODE.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
            ),
            (Tone::CTCSS(tx_hz), Tone::None) => (
                "Tone".to_string(),
                DEFAULT_CROSS_MODE.to_string(),
                format!("{tx_hz:.1}"),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
            ),
            (Tone::None, Tone::CTCSS(rx_hz)) => (
                "TSQL".to_string(),
                DEFAULT_CROSS_MODE.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                format!("{rx_hz:.1}"),
                DEFAULT_DTCS_CODE.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
            ),
            (Tone::CTCSS(tx_hz), Tone::CTCSS(rx_hz)) if tx_hz == rx_hz => (
                "TSQL".to_string(),
                DEFAULT_CROSS_MODE.to_string(),
                format!("{tx_hz:.1}"),
                format!("{rx_hz:.1}"),
                DEFAULT_DTCS_CODE.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
            ),
            (Tone::CTCSS(tx_hz), Tone::CTCSS(rx_hz)) => (
                "Cross".to_string(),
                "Tone->Tone".to_string(),
                format!("{tx_hz:.1}"),
                format!("{rx_hz:.1}"),
                DEFAULT_DTCS_CODE.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
            ),
            (Tone::DCS(tx_code), Tone::None) => (
                "Cross".to_string(),
                "DTCS->".to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                format_dtcs(tx_code),
                DEFAULT_DTCS_CODE.to_string(),
            ),
            (Tone::None, Tone::DCS(rx_code)) => (
                "Cross".to_string(),
                "->DTCS".to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
                format_dtcs(rx_code),
            ),
            (Tone::DCS(tx_code), Tone::DCS(rx_code)) if tx_code == rx_code => (
                "DTCS".to_string(),
                DEFAULT_CROSS_MODE.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                format_dtcs(tx_code),
                DEFAULT_DTCS_CODE.to_string(),
            ),
            (Tone::DCS(tx_code), Tone::DCS(rx_code)) => (
                "Cross".to_string(),
                "DTCS->DTCS".to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                format_dtcs(tx_code),
                format_dtcs(rx_code),
            ),
            (Tone::CTCSS(tx_hz), Tone::DCS(rx_code)) => (
                "Cross".to_string(),
                "Tone->DTCS".to_string(),
                format!("{tx_hz:.1}"),
                DEFAULT_TONE_FREQ.to_string(),
                DEFAULT_DTCS_CODE.to_string(),
                format_dtcs(rx_code),
            ),
            (Tone::DCS(tx_code), Tone::CTCSS(rx_hz)) => (
                "Cross".to_string(),
                "DTCS->Tone".to_string(),
                DEFAULT_TONE_FREQ.to_string(),
                format!("{rx_hz:.1}"),
                format_dtcs(tx_code),
                DEFAULT_DTCS_CODE.to_string(),
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

fn format_dtcs(code: i32) -> String {
    format!("{code:03}")
}
