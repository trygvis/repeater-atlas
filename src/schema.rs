// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "aprs_mode"))]
    pub struct AprsMode;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "call_sign_kind"))]
    pub struct CallSignKind;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "contact_kind"))]
    pub struct ContactKind;

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
    use diesel::sql_types::*;
    use super::sql_types::CallSignKind;

    call_sign (value) {
        value -> Text,
        kind -> CallSignKind,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ContactKind;

    contact (id) {
        id -> Int8,
        call_sign -> Nullable<Text>,
        kind -> ContactKind,
        display_name -> Text,
        description -> Nullable<Text>,
        web_url -> Nullable<Text>,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        address -> Nullable<Text>,
    }
}

diesel::table! {
    repeater_link (id) {
        id -> Int8,
        repeater_a_id -> Int8,
        repeater_b_id -> Int8,
        note -> Text,
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
        kind -> Nullable<RepeaterServiceKind>,
        enabled -> Bool,
        label -> Text,
        note -> Text,
        rx_hz -> Int8,
        tx_hz -> Int8,
        fm_bandwidth -> Nullable<FmBandwidth>,
        rx_tone_kind -> Nullable<ToneKind>,
        rx_ctcss_hz -> Nullable<Float4>,
        rx_dcs_code -> Nullable<Int4>,
        tx_tone_kind -> Nullable<ToneKind>,
        tx_ctcss_hz -> Nullable<Float4>,
        tx_dcs_code -> Nullable<Int4>,
        dmr_color_code -> Nullable<Int2>,
        dmr_repeater_id -> Nullable<Int8>,
        dmr_network -> Nullable<Text>,
        dstar_mode -> Nullable<DstarMode>,
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
        call_sign -> Text,
        owner -> Nullable<Int8>,
        tech_contact -> Nullable<Int8>,
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
    }
}

diesel::joinable!(contact -> call_sign (call_sign));
diesel::joinable!(repeater_service -> repeater_system (repeater_id));
diesel::joinable!(repeater_service_dmr_talkgroup -> repeater_service (service_id));
diesel::joinable!(repeater_system -> call_sign (call_sign));

diesel::allow_tables_to_appear_in_same_query!(
    app_user,
    call_sign,
    contact,
    repeater_link,
    repeater_service,
    repeater_service_dmr_talkgroup,
    repeater_system,
);
