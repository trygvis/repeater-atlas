-- This file should undo anything in `up.sql`
-- Roll back repeater schema redo.

DROP TABLE repeater_service_ssb;
DROP TABLE repeater_service_aprs;
DROP TABLE repeater_service_c4fm;
DROP TABLE repeater_service_dstar;
DROP TABLE repeater_service_dmr_talkgroup;
DROP TABLE repeater_service_dmr;
DROP TABLE repeater_service_fm;

DROP INDEX repeater_service_unique_system_kind_when_no_port;
DROP INDEX repeater_service_unique_port_kind;

DROP TABLE repeater_service;
DROP TABLE repeater_port;
DROP TABLE repeater_system;
DROP TABLE repeater_site;

DROP TYPE ssb_sideband;
DROP TYPE dstar_mode;
DROP TYPE tone_kind;
DROP TYPE fm_bandwidth;
DROP TYPE repeater_service_kind;

-- Legacy tables (baseline schema)

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
    modulation         VARCHAR NOT NULL,

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
