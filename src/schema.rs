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
        frequency -> Int8,
        rx_offset -> Int8,
        tx_subtone -> Nullable<Numeric>,
        rx_subtone -> Nullable<Numeric>,
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
