// @generated automatically by Diesel CLI.

diesel::table! {
    ham_club (id) {
        id -> Int8,
        name -> Nullable<Text>,
        description -> Nullable<Text>,
        web_url -> Nullable<Text>,
        email -> Nullable<Text>,
    }
}

diesel::table! {
    ham_operator (id) {
        id -> Int8,
        call_sign -> Varchar,
    }
}

diesel::table! {
    repeater (id) {
        id -> Int8,
        ham_club -> Nullable<Int8>,
        call_sign -> Varchar,
        maidenhead_locator -> Nullable<Text>,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
        address -> Nullable<Text>,
        frequency -> Int8,
        modulation -> Varchar,
        rx_offset -> Int8,
        subtone_mode -> Varchar,
        tx_subtone -> Nullable<Float4>,
        rx_subtone -> Nullable<Float4>,
        has_dmr -> Bool,
        dmr_id -> Nullable<Int8>,
        has_aprs -> Bool,
    }
}

diesel::table! {
    repeater_change_log (id) {
        id -> Int8,
        repeater -> Nullable<Int8>,
        created_at -> Nullable<Timestamp>,
        body -> Nullable<Text>,
    }
}

diesel::joinable!(repeater -> ham_club (ham_club));
diesel::joinable!(repeater_change_log -> repeater (repeater));

diesel::allow_tables_to_appear_in_same_query!(
    ham_club,
    ham_operator,
    repeater,
    repeater_change_log,
);
