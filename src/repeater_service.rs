use crate::dao::repeater_service::{
    AprsMode, DstarMode, FmBandwidth, NewRepeaterServiceDao, RepeaterServiceDao,
    RepeaterServiceKind, SsbSideband, ToneKind,
};
use crate::Frequency;

#[derive(Debug, Clone)]
pub enum Tone {
    None,
    CTCSS(f32),
    DCS(i32),
}

#[derive(Debug, Clone)]
pub enum RepeaterService {
    Fm {
        label: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        bandwidth: FmBandwidth,
        rx_tone: Tone,
        tx_tone: Tone,
        note: Option<String>,
    },
    Am {
        label: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        note: Option<String>,
    },
    Ssb {
        label: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        sideband: Option<SsbSideband>,
        note: Option<String>,
    },
    Dstar {
        label: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        mode: DstarMode,
        gateway_call_sign: Option<String>,
        reflector: Option<String>,
        note: Option<String>,
    },
    Dmr {
        label: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        color_code: i16,
        dmr_repeater_id: Option<i64>,
        network: String,
        note: Option<String>,
    },
    C4fm {
        label: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        wires_x_node_id: Option<i32>,
        room: Option<String>,
        note: Option<String>,
    },
    Aprs {
        label: String,
        rx_hz: Frequency,
        tx_hz: Frequency,
        mode: Option<AprsMode>,
        path: Option<String>,
        note: Option<String>,
    },
}

impl RepeaterService {
    pub fn label(&self) -> &str {
        match self {
            RepeaterService::Fm { label, .. }
            | RepeaterService::Am { label, .. }
            | RepeaterService::Ssb { label, .. }
            | RepeaterService::Dstar { label, .. }
            | RepeaterService::Dmr { label, .. }
            | RepeaterService::C4fm { label, .. }
            | RepeaterService::Aprs { label, .. } => label,
        }
    }

    pub fn rx_hz(&self) -> Frequency {
        match self {
            RepeaterService::Fm { rx_hz, .. }
            | RepeaterService::Am { rx_hz, .. }
            | RepeaterService::Ssb { rx_hz, .. }
            | RepeaterService::Dstar { rx_hz, .. }
            | RepeaterService::Dmr { rx_hz, .. }
            | RepeaterService::C4fm { rx_hz, .. }
            | RepeaterService::Aprs { rx_hz, .. } => *rx_hz,
        }
    }

    pub fn tx_hz(&self) -> Frequency {
        match self {
            RepeaterService::Fm { tx_hz, .. }
            | RepeaterService::Am { tx_hz, .. }
            | RepeaterService::Ssb { tx_hz, .. }
            | RepeaterService::Dstar { tx_hz, .. }
            | RepeaterService::Dmr { tx_hz, .. }
            | RepeaterService::C4fm { tx_hz, .. }
            | RepeaterService::Aprs { tx_hz, .. } => *tx_hz,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            RepeaterService::Fm { .. } => "FM",
            RepeaterService::Am { .. } => "AM",
            RepeaterService::Ssb { .. } => "SSB",
            RepeaterService::Dstar { .. } => "D-STAR",
            RepeaterService::Dmr { .. } => "DMR",
            RepeaterService::C4fm { .. } => "C4FM",
            RepeaterService::Aprs { .. } => "APRS",
        }
    }

    pub fn to_new_dao(self, repeater_id: i64) -> NewRepeaterServiceDao {
        match self {
            RepeaterService::Fm {
                label,
                rx_hz,
                tx_hz,
                bandwidth,
                rx_tone,
                tx_tone,
                note,
            } => {
                let (rx_tone_kind, rx_ctcss_hz, rx_dcs_code) = tone_to_parts(rx_tone);
                let (tx_tone_kind, tx_ctcss_hz, tx_dcs_code) = tone_to_parts(tx_tone);
                NewRepeaterServiceDao {
                    repeater_id,
                    kind: Some(RepeaterServiceKind::Fm),
                    enabled: true,
                    label,
                    rx_hz,
                    tx_hz,
                    note: note.unwrap_or_default(),
                    fm_bandwidth: Some(bandwidth),
                    rx_tone_kind,
                    rx_ctcss_hz,
                    rx_dcs_code,
                    tx_tone_kind,
                    tx_ctcss_hz,
                    tx_dcs_code,
                    dmr_color_code: None,
                    dmr_repeater_id: None,
                    dmr_network: None,
                    dstar_mode: None,
                    dstar_gateway_call_sign: None,
                    dstar_reflector: None,
                    c4fm_wires_x_node_id: None,
                    c4fm_room: None,
                    aprs_mode: None,
                    aprs_path: None,
                    ssb_sideband: None,
                }
            }
            RepeaterService::Am {
                label,
                rx_hz,
                tx_hz,
                note,
            } => NewRepeaterServiceDao {
                repeater_id,
                kind: Some(RepeaterServiceKind::Am),
                enabled: true,
                label,
                rx_hz,
                tx_hz,
                note: note.unwrap_or_default(),
                fm_bandwidth: None,
                rx_tone_kind: None,
                rx_ctcss_hz: None,
                rx_dcs_code: None,
                tx_tone_kind: None,
                tx_ctcss_hz: None,
                tx_dcs_code: None,
                dmr_color_code: None,
                dmr_repeater_id: None,
                dmr_network: None,
                dstar_mode: None,
                dstar_gateway_call_sign: None,
                dstar_reflector: None,
                c4fm_wires_x_node_id: None,
                c4fm_room: None,
                aprs_mode: None,
                aprs_path: None,
                ssb_sideband: None,
            },
            RepeaterService::Ssb {
                label,
                rx_hz,
                tx_hz,
                sideband,
                note,
            } => NewRepeaterServiceDao {
                repeater_id,
                kind: Some(RepeaterServiceKind::Ssb),
                enabled: true,
                label,
                rx_hz,
                tx_hz,
                note: note.unwrap_or_default(),
                fm_bandwidth: None,
                rx_tone_kind: None,
                rx_ctcss_hz: None,
                rx_dcs_code: None,
                tx_tone_kind: None,
                tx_ctcss_hz: None,
                tx_dcs_code: None,
                dmr_color_code: None,
                dmr_repeater_id: None,
                dmr_network: None,
                dstar_mode: None,
                dstar_gateway_call_sign: None,
                dstar_reflector: None,
                c4fm_wires_x_node_id: None,
                c4fm_room: None,
                aprs_mode: None,
                aprs_path: None,
                ssb_sideband: sideband,
            },
            RepeaterService::Dstar {
                label,
                rx_hz,
                tx_hz,
                mode,
                gateway_call_sign,
                reflector,
                note,
            } => NewRepeaterServiceDao {
                repeater_id,
                kind: Some(RepeaterServiceKind::Dstar),
                enabled: true,
                label,
                rx_hz,
                tx_hz,
                note: note.unwrap_or_default(),
                fm_bandwidth: None,
                rx_tone_kind: None,
                rx_ctcss_hz: None,
                rx_dcs_code: None,
                tx_tone_kind: None,
                tx_ctcss_hz: None,
                tx_dcs_code: None,
                dmr_color_code: None,
                dmr_repeater_id: None,
                dmr_network: None,
                dstar_mode: Some(mode),
                dstar_gateway_call_sign: gateway_call_sign,
                dstar_reflector: reflector,
                c4fm_wires_x_node_id: None,
                c4fm_room: None,
                aprs_mode: None,
                aprs_path: None,
                ssb_sideband: None,
            },
            RepeaterService::Dmr {
                label,
                rx_hz,
                tx_hz,
                color_code,
                dmr_repeater_id,
                network,
                note,
            } => NewRepeaterServiceDao {
                repeater_id,
                kind: Some(RepeaterServiceKind::Dmr),
                enabled: true,
                label,
                rx_hz,
                tx_hz,
                note: note.unwrap_or_default(),
                fm_bandwidth: None,
                rx_tone_kind: None,
                rx_ctcss_hz: None,
                rx_dcs_code: None,
                tx_tone_kind: None,
                tx_ctcss_hz: None,
                tx_dcs_code: None,
                dmr_color_code: Some(color_code),
                dmr_repeater_id,
                dmr_network: Some(network),
                dstar_mode: None,
                dstar_gateway_call_sign: None,
                dstar_reflector: None,
                c4fm_wires_x_node_id: None,
                c4fm_room: None,
                aprs_mode: None,
                aprs_path: None,
                ssb_sideband: None,
            },
            RepeaterService::C4fm {
                label,
                rx_hz,
                tx_hz,
                wires_x_node_id,
                room,
                note,
            } => NewRepeaterServiceDao {
                repeater_id,
                kind: Some(RepeaterServiceKind::C4fm),
                enabled: true,
                label,
                rx_hz,
                tx_hz,
                note: note.unwrap_or_default(),
                fm_bandwidth: None,
                rx_tone_kind: None,
                rx_ctcss_hz: None,
                rx_dcs_code: None,
                tx_tone_kind: None,
                tx_ctcss_hz: None,
                tx_dcs_code: None,
                dmr_color_code: None,
                dmr_repeater_id: None,
                dmr_network: None,
                dstar_mode: None,
                dstar_gateway_call_sign: None,
                dstar_reflector: None,
                c4fm_wires_x_node_id: wires_x_node_id,
                c4fm_room: room,
                aprs_mode: None,
                aprs_path: None,
                ssb_sideband: None,
            },
            RepeaterService::Aprs {
                label,
                rx_hz,
                tx_hz,
                mode,
                path,
                note,
            } => NewRepeaterServiceDao {
                repeater_id,
                kind: Some(RepeaterServiceKind::Aprs),
                enabled: true,
                label,
                rx_hz,
                tx_hz,
                note: note.unwrap_or_default(),
                fm_bandwidth: None,
                rx_tone_kind: None,
                rx_ctcss_hz: None,
                rx_dcs_code: None,
                tx_tone_kind: None,
                tx_ctcss_hz: None,
                tx_dcs_code: None,
                dmr_color_code: None,
                dmr_repeater_id: None,
                dmr_network: None,
                dstar_mode: None,
                dstar_gateway_call_sign: None,
                dstar_reflector: None,
                c4fm_wires_x_node_id: None,
                c4fm_room: None,
                aprs_mode: mode,
                aprs_path: path,
                ssb_sideband: None,
            },
        }
    }
}

impl From<RepeaterServiceDao> for RepeaterService {
    fn from(value: RepeaterServiceDao) -> Self {
        let kind = require_field(value.kind, "kind");
        let label = value.label;
        let rx_hz = value.rx_hz;
        let tx_hz = value.tx_hz;
        let rx_tone = tone_from_parts(value.rx_tone_kind, value.rx_ctcss_hz, value.rx_dcs_code);
        let tx_tone = tone_from_parts(value.tx_tone_kind, value.tx_ctcss_hz, value.tx_dcs_code);

        match kind {
            RepeaterServiceKind::Fm => RepeaterService::Fm {
                label,
                rx_hz,
                tx_hz,
                bandwidth: require_field(value.fm_bandwidth, "fm_bandwidth"),
                rx_tone,
                tx_tone,
                note: (!value.note.is_empty()).then_some(value.note),
            },
            RepeaterServiceKind::Am => RepeaterService::Am {
                label,
                rx_hz,
                tx_hz,
                note: (!value.note.is_empty()).then_some(value.note),
            },
            RepeaterServiceKind::Ssb => RepeaterService::Ssb {
                label,
                rx_hz,
                tx_hz,
                sideband: value.ssb_sideband,
                note: (!value.note.is_empty()).then_some(value.note),
            },
            RepeaterServiceKind::Dstar => RepeaterService::Dstar {
                label,
                rx_hz,
                tx_hz,
                mode: require_field(value.dstar_mode, "dstar_mode"),
                gateway_call_sign: value.dstar_gateway_call_sign,
                reflector: value.dstar_reflector,
                note: (!value.note.is_empty()).then_some(value.note),
            },
            RepeaterServiceKind::Dmr => RepeaterService::Dmr {
                label,
                rx_hz,
                tx_hz,
                color_code: require_field(value.dmr_color_code, "dmr_color_code"),
                dmr_repeater_id: value.dmr_repeater_id,
                network: require_field(value.dmr_network, "dmr_network"),
                note: (!value.note.is_empty()).then_some(value.note),
            },
            RepeaterServiceKind::C4fm => RepeaterService::C4fm {
                label,
                rx_hz,
                tx_hz,
                wires_x_node_id: value.c4fm_wires_x_node_id,
                room: value.c4fm_room,
                note: (!value.note.is_empty()).then_some(value.note),
            },
            RepeaterServiceKind::Aprs => RepeaterService::Aprs {
                label,
                rx_hz,
                tx_hz,
                mode: value.aprs_mode,
                path: value.aprs_path,
                note: (!value.note.is_empty()).then_some(value.note),
            },
        }
    }
}

fn tone_to_parts(tone: Tone) -> (Option<ToneKind>, Option<f32>, Option<i32>) {
    match tone {
        Tone::None => (Some(ToneKind::None), None, None),
        Tone::CTCSS(freq) => (Some(ToneKind::CTCSS), Some(freq), None),
        Tone::DCS(code) => (Some(ToneKind::DCS), None, Some(code)),
    }
}

fn tone_from_parts(kind: Option<ToneKind>, ctcss_hz: Option<f32>, dcs_code: Option<i32>) -> Tone {
    match kind {
        Some(ToneKind::CTCSS) => ctcss_hz.map(Tone::CTCSS).unwrap_or(Tone::None),
        Some(ToneKind::DCS) => dcs_code.map(Tone::DCS).unwrap_or(Tone::None),
        Some(ToneKind::None) | None => Tone::None,
    }
}

fn require_field<T>(value: Option<T>, field: &'static str) -> T {
    value.unwrap_or_else(|| panic!("repeater_service missing required field {field}"))
}
