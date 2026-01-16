CREATE TABLE ham_club
(
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT,
    description TEXT,
    web_url     TEXT,
    email       TEXT
);

CREATE TABLE repeater
(
    id         BIGSERIAL PRIMARY KEY,
    ham_club   BIGINT REFERENCES ham_club,
    call_sign  VARCHAR NOT NULL UNIQUE,
    frequency  BIGINT  NOT NULL,
    rx_offset  BIGINT  NOT NULL,
    tx_subtone NUMERIC,
    rx_subtone NUMERIC
);

CREATE TABLE repeater_change_log
(
    id         BIGSERIAL PRIMARY KEY,
    repeater   BIGINT REFERENCES repeater,
    created_at TIMESTAMP,
    body       TEXT
);

CREATE TABLE ham_operator
(
    id        BIGSERIAL PRIMARY KEY,
    call_sign VARCHAR NOT NULL UNIQUE
);
