// @generated automatically by Diesel CLI.

diesel::table! {
    ham_club (id) {
        id -> Int8,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    repeater (id) {
        id -> Int8,
        ham_club_id -> Nullable<Int8>,
        callsign -> Nullable<Text>,
    }
}

diesel::table! {
    repeater_change_log (id) {
        id -> Int8,
        repeater_id -> Nullable<Int8>,
        body -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(ham_club, repeater, repeater_change_log,);
