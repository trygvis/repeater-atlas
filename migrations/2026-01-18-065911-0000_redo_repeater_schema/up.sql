-- Your SQL goes here
-- Redo repeater schema.
-- Data migration is intentionally skipped (DB is expected to be empty).

-- Legacy tables (baseline schema)
DROP TABLE repeater_change_log;
DROP TABLE repeater;
DROP TABLE ham_operator;

-- New schema

CREATE TABLE repeater_site (
    id          BIGSERIAL PRIMARY KEY,
    name        TEXT,
    address     TEXT,
    maidenhead  TEXT,
    latitude    DOUBLE PRECISION,
    longitude   DOUBLE PRECISION,
    elevation_m INTEGER,
    country     TEXT,
    region      TEXT
);

CREATE TABLE repeater_system (
    id          BIGSERIAL PRIMARY KEY,
    ham_club_id BIGINT REFERENCES ham_club (id),
    call_sign   TEXT NOT NULL UNIQUE,
    name        TEXT,
    description TEXT,
    site_id     BIGINT REFERENCES repeater_site (id),
    status      TEXT NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE repeater_port (
    id          BIGSERIAL PRIMARY KEY,
    repeater_id BIGINT NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    label       TEXT NOT NULL,
    rx_hz       BIGINT NOT NULL,
    tx_hz       BIGINT NOT NULL,
    note        TEXT,

    UNIQUE (repeater_id, label)
);

CREATE TYPE repeater_service_kind AS ENUM (
    'fm',
    'am',
    'ssb',
    'dstar',
    'dmr',
    'c4fm',
    'aprs'
);

CREATE TABLE repeater_service (
    id          BIGSERIAL PRIMARY KEY,
    repeater_id BIGINT NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    port_id     BIGINT REFERENCES repeater_port (id) ON DELETE SET NULL,
    kind        repeater_service_kind NOT NULL,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE UNIQUE INDEX repeater_service_unique_port_kind
    ON repeater_service (port_id, kind)
    WHERE port_id IS NOT NULL;

CREATE UNIQUE INDEX repeater_service_unique_system_kind_when_no_port
    ON repeater_service (repeater_id, kind)
    WHERE port_id IS NULL;

CREATE TYPE fm_bandwidth AS ENUM ('narrow', 'wide');
CREATE TYPE tone_kind AS ENUM ('none', 'ctcss', 'dcs');

CREATE TABLE repeater_service_fm (
    service_id       BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    bandwidth        fm_bandwidth NOT NULL,

    access_tone_kind tone_kind NOT NULL DEFAULT 'none',
    access_ctcss_hz  REAL,
    access_dcs_code  INTEGER,

    tx_tone_kind     tone_kind NOT NULL DEFAULT 'none',
    tx_ctcss_hz      REAL,
    tx_dcs_code      INTEGER
);

CREATE TABLE repeater_service_dmr (
    service_id      BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    color_code      SMALLINT,
    dmr_repeater_id BIGINT,
    network         TEXT
);

CREATE TABLE repeater_service_dmr_talkgroup (
    id        BIGSERIAL PRIMARY KEY,
    service_id BIGINT NOT NULL REFERENCES repeater_service_dmr (service_id) ON DELETE CASCADE,
    time_slot SMALLINT NOT NULL,
    talkgroup INTEGER NOT NULL,
    name      TEXT,

    UNIQUE (service_id, time_slot, talkgroup)
);

CREATE TYPE dstar_mode AS ENUM ('dv', 'dd');

CREATE TABLE repeater_service_dstar (
    service_id        BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    mode              dstar_mode NOT NULL DEFAULT 'dv',
    gateway_call_sign TEXT,
    reflector         TEXT
);

CREATE TABLE repeater_service_c4fm (
    service_id      BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    wires_x_node_id INTEGER,
    room            TEXT
);

CREATE TABLE repeater_service_aprs (
    service_id  BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    igate       BOOLEAN NOT NULL DEFAULT FALSE,
    digipeater  BOOLEAN NOT NULL DEFAULT FALSE,
    path        TEXT
);

CREATE TYPE ssb_sideband AS ENUM ('lsb', 'usb');

CREATE TABLE repeater_service_ssb (
    service_id BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    sideband   ssb_sideband
);
