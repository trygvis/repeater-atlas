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
    id                 BIGSERIAL PRIMARY KEY,
    ham_club           BIGINT REFERENCES ham_club,
    call_sign          VARCHAR NOT NULL UNIQUE,
    maidenhead_locator TEXT,
    latitude           DOUBLE PRECISION,
    longitude          DOUBLE PRECISION,
    address            TEXT,
    frequency          BIGINT  NOT NULL,
    modulation         VARCHAR,

    rx_offset          BIGINT  NOT NULL,
    subtone_mode       VARCHAR NOT NULL DEFAULT 'None',
    tx_subtone         REAL,
    rx_subtone         REAL,

    has_dmr            BOOLEAN NOT NULL DEFAULT FALSE,
    dmr_id             BIGINT,

    has_aprs           BOOLEAN NOT NULL DEFAULT FALSE
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
