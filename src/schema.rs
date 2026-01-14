diesel::table! {
    repeaters (id) {
        id -> Uuid,
        callsign -> Text,
    }
}
