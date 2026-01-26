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
    app_user (id) {
        id -> Int8,
        call_sign -> Text,
        email -> Text,
        password_hash -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
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
    use diesel::sql_types::*;
    use super::sql_types::RepeaterServiceKind;
    use super::sql_types::FmBandwidth;
    use super::sql_types::ToneKind;
    use super::sql_types::DstarMode;
    use super::sql_types::AprsMode;
    use super::sql_types::SsbSideband;

    repeater_service (id) {
        id -> Int8,
        repeater_id -> Int8,
        kind -> RepeaterServiceKind,
        enabled -> Bool,
        label -> Text,
        note -> Nullable<Text>,
        rx_hz -> Int8,
        tx_hz -> Int8,
        fm_bandwidth -> FmBandwidth,
        rx_tone_kind -> ToneKind,
        rx_ctcss_hz -> Nullable<Float4>,
        rx_dcs_code -> Nullable<Int4>,
        tx_tone_kind -> ToneKind,
        tx_ctcss_hz -> Nullable<Float4>,
        tx_dcs_code -> Nullable<Int4>,
        dmr_color_code -> Int2,
        dmr_repeater_id -> Nullable<Int8>,
        dmr_network -> Text,
        dstar_mode -> DstarMode,
        dstar_gateway_call_sign -> Nullable<Text>,
        dstar_reflector -> Nullable<Text>,
        c4fm_wires_x_node_id -> Nullable<Int4>,
        c4fm_room -> Nullable<Text>,
        aprs_mode -> Nullable<AprsMode>,
        aprs_path -> Nullable<Text>,
        ssb_sideband -> Nullable<SsbSideband>,
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
    repeater_system (id) {
        id -> Int8,
        ham_club_id -> Nullable<Int8>,
        call_sign -> Text,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        address -> Nullable<Text>,
        maidenhead -> Nullable<Text>,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        elevation_m -> Nullable<Int4>,
        country -> Nullable<Text>,
        region -> Nullable<Text>,
        status -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(repeater_service -> repeater_system (repeater_id));
diesel::joinable!(repeater_service_dmr_talkgroup -> repeater_service (service_id));
diesel::joinable!(repeater_system -> ham_club (ham_club_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_user,
    ham_club,
    repeater_service,
    repeater_service_dmr_talkgroup,
    repeater_system,
);
