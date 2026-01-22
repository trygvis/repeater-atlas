CREATE TABLE repeater_system
(
    id          BIGSERIAL PRIMARY KEY,
    ham_club_id BIGINT REFERENCES ham_club (id),
    call_sign   TEXT        NOT NULL UNIQUE,
    name        TEXT,
    description TEXT,
    address     TEXT,
    maidenhead  TEXT,
    latitude    DOUBLE PRECISION,
    longitude   DOUBLE PRECISION,
    elevation_m INTEGER,
    country     TEXT,
    region      TEXT,
    status      TEXT        NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE repeater_port
(
    id          BIGSERIAL PRIMARY KEY,
    repeater_id BIGINT NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    label       TEXT   NOT NULL,
    rx_hz       BIGINT NOT NULL,
    tx_hz       BIGINT NOT NULL,
    note        TEXT
);

CREATE TYPE REPEATER_SERVICE_KIND AS ENUM (
    'fm',
    'am',
    'ssb',
    'dstar',
    'dmr',
    'c4fm',
    'aprs'
    );

CREATE TABLE repeater_service
(
    id          BIGSERIAL PRIMARY KEY,
    repeater_id BIGINT                NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    port_id     BIGINT                REFERENCES repeater_port (id) ON DELETE SET NULL,
    kind        REPEATER_SERVICE_KIND NOT NULL,
    enabled     BOOLEAN               NOT NULL DEFAULT TRUE
);

CREATE UNIQUE INDEX repeater_service_unique_port_kind
    ON repeater_service (port_id, kind)
    WHERE port_id IS NOT NULL;

CREATE UNIQUE INDEX repeater_service_unique_system_kind_when_no_port
    ON repeater_service (repeater_id, kind)
    WHERE port_id IS NULL;

CREATE TYPE FM_BANDWIDTH AS ENUM ('narrow', 'wide');
CREATE TYPE TONE_KIND AS ENUM ('none', 'ctcss', 'dcs');

CREATE TABLE repeater_service_fm
(
    service_id       BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    bandwidth        FM_BANDWIDTH NOT NULL,

    access_tone_kind TONE_KIND    NOT NULL DEFAULT 'none',
    access_ctcss_hz  REAL,
    access_dcs_code  INTEGER,

    tx_tone_kind     TONE_KIND    NOT NULL DEFAULT 'none',
    tx_ctcss_hz      REAL,
    tx_dcs_code      INTEGER
);

CREATE TABLE repeater_service_dmr
(
    service_id      BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    color_code      SMALLINT,
    dmr_repeater_id BIGINT,
    network         TEXT
);

CREATE TABLE repeater_service_dmr_talkgroup
(
    id         BIGSERIAL PRIMARY KEY,
    service_id BIGINT   NOT NULL REFERENCES repeater_service_dmr (service_id) ON DELETE CASCADE,
    time_slot  SMALLINT NOT NULL,
    talkgroup  INTEGER  NOT NULL,
    name       TEXT,

    UNIQUE (service_id, time_slot, talkgroup)
);

CREATE TYPE DSTAR_MODE AS ENUM ('dv', 'dd');

CREATE TABLE repeater_service_dstar
(
    service_id        BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    mode              DSTAR_MODE NOT NULL DEFAULT 'dv',
    gateway_call_sign TEXT,
    reflector         TEXT
);

CREATE TABLE repeater_service_c4fm
(
    service_id      BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    wires_x_node_id INTEGER,
    room            TEXT
);

CREATE TYPE APRS_MODE AS ENUM ('igate', 'digipeater');

CREATE TABLE repeater_service_aprs
(
    service_id BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    mode       APRS_MODE NOT NULL,
    path       TEXT
);

CREATE TYPE SSB_SIDEBAND AS ENUM ('lsb', 'usb');

CREATE TABLE repeater_service_ssb
(
    service_id BIGINT PRIMARY KEY REFERENCES repeater_service (id) ON DELETE CASCADE,
    sideband   SSB_SIDEBAND
);
