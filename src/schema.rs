// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "aprs_mode"))]
    pub struct AprsMode;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "dstar_mode"))]
    pub struct DstarMode;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "fm_bandwidth"))]
    pub struct FmBandwidth;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "repeater_service_kind"))]
    pub struct RepeaterServiceKind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "ssb_sideband"))]
    pub struct SsbSideband;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tone_kind"))]
    pub struct ToneKind;
}

diesel::table! {
    ham_club (id) {
        id -> Int8,
        name -> Text,
        description -> Nullable<Text>,
        web_url -> Nullable<Text>,
        email -> Nullable<Text>,
    }
}

diesel::table! {
    repeater_port (id) {
        id -> Int8,
        repeater_id -> Int8,
        label -> Text,
        rx_hz -> Int8,
        tx_hz -> Int8,
        note -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RepeaterServiceKind;

    repeater_service (id) {
        id -> Int8,
        repeater_id -> Int8,
        port_id -> Nullable<Int8>,
        kind -> RepeaterServiceKind,
        enabled -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AprsMode;

    repeater_service_aprs (service_id) {
        service_id -> Int8,
        mode -> AprsMode,
        path -> Nullable<Text>,
    }
}

diesel::table! {
    repeater_service_c4fm (service_id) {
        service_id -> Int8,
        wires_x_node_id -> Nullable<Int4>,
        room -> Nullable<Text>,
    }
}

diesel::table! {
    repeater_service_dmr (service_id) {
        service_id -> Int8,
        color_code -> Nullable<Int2>,
        dmr_repeater_id -> Nullable<Int8>,
        network -> Nullable<Text>,
    }
}

diesel::table! {
    repeater_service_dmr_talkgroup (id) {
        id -> Int8,
        service_id -> Int8,
        time_slot -> Int2,
        talkgroup -> Int4,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DstarMode;

    repeater_service_dstar (service_id) {
        service_id -> Int8,
        mode -> DstarMode,
        gateway_call_sign -> Nullable<Text>,
        reflector -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FmBandwidth;
    use super::sql_types::ToneKind;

    repeater_service_fm (service_id) {
        service_id -> Int8,
        bandwidth -> FmBandwidth,
        access_tone_kind -> ToneKind,
        access_ctcss_hz -> Nullable<Float4>,
        access_dcs_code -> Nullable<Int4>,
        tx_tone_kind -> ToneKind,
        tx_ctcss_hz -> Nullable<Float4>,
        tx_dcs_code -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SsbSideband;

    repeater_service_ssb (service_id) {
        service_id -> Int8,
        sideband -> Nullable<SsbSideband>,
    }
}

diesel::table! {
    repeater_site (id) {
        id -> Int8,
        name -> Nullable<Text>,
        address -> Nullable<Text>,
        maidenhead -> Nullable<Text>,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        elevation_m -> Nullable<Int4>,
        country -> Nullable<Text>,
        region -> Nullable<Text>,
    }
}

diesel::table! {
    repeater_system (id) {
        id -> Int8,
        ham_club_id -> Nullable<Int8>,
        call_sign -> Text,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        site_id -> Nullable<Int8>,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(repeater_port -> repeater_system (repeater_id));
diesel::joinable!(repeater_service -> repeater_port (port_id));
diesel::joinable!(repeater_service -> repeater_system (repeater_id));
diesel::joinable!(repeater_service_aprs -> repeater_service (service_id));
diesel::joinable!(repeater_service_c4fm -> repeater_service (service_id));
diesel::joinable!(repeater_service_dmr -> repeater_service (service_id));
diesel::joinable!(repeater_service_dmr_talkgroup -> repeater_service_dmr (service_id));
diesel::joinable!(repeater_service_dstar -> repeater_service (service_id));
diesel::joinable!(repeater_service_fm -> repeater_service (service_id));
diesel::joinable!(repeater_service_ssb -> repeater_service (service_id));
diesel::joinable!(repeater_system -> ham_club (ham_club_id));
diesel::joinable!(repeater_system -> repeater_site (site_id));

diesel::allow_tables_to_appear_in_same_query!(
    ham_club,
    repeater_port,
    repeater_service,
    repeater_service_aprs,
    repeater_service_c4fm,
    repeater_service_dmr,
    repeater_service_dmr_talkgroup,
    repeater_service_dstar,
    repeater_service_fm,
    repeater_service_ssb,
    repeater_site,
    repeater_system,
);
