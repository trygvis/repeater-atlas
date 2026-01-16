CREATE TABLE ham_club (
    id BIGSERIAL PRIMARY KEY,
    name text
);

CREATE TABLE repeater (
    id BIGSERIAL PRIMARY KEY,
    ham_club_id bigint,
    callsign text
);

CREATE TABLE repeater_change_log (
    id BIGSERIAL PRIMARY KEY,
    repeater_id bigint,
    body text,
    created_at timestamp
);
-- Your SQL goes here
