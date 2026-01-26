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

CREATE TYPE REPEATER_SERVICE_KIND AS ENUM (
    'fm',
    'am',
    'ssb',
    'dstar',
    'dmr',
    'c4fm',
    'aprs'
    );

CREATE TYPE FM_BANDWIDTH AS ENUM ('narrow', 'wide');
CREATE TYPE TONE_KIND AS ENUM ('none', 'ctcss', 'dcs');
CREATE TYPE DSTAR_MODE AS ENUM ('dv', 'dd');
CREATE TYPE APRS_MODE AS ENUM ('igate', 'digipeater');
CREATE TYPE SSB_SIDEBAND AS ENUM ('lsb', 'usb');

CREATE TABLE repeater_service
(
    id                      BIGSERIAL PRIMARY KEY,
    repeater_id             BIGINT                NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    kind                    REPEATER_SERVICE_KIND NOT NULL,
    enabled                 BOOLEAN               NOT NULL DEFAULT TRUE,
    label                   TEXT                  NOT NULL,
    note                    TEXT,
    rx_hz                   BIGINT                NOT NULL,
    tx_hz                   BIGINT                NOT NULL,

    fm_bandwidth            FM_BANDWIDTH          NOT NULL DEFAULT 'narrow',
    rx_tone_kind            TONE_KIND             NOT NULL DEFAULT 'none',
    rx_ctcss_hz             REAL,
    rx_dcs_code             INTEGER,
    tx_tone_kind            TONE_KIND             NOT NULL DEFAULT 'none',
    tx_ctcss_hz             REAL,
    tx_dcs_code             INTEGER,

    dmr_color_code          SMALLINT              NOT NULL DEFAULT 0,
    dmr_repeater_id         BIGINT,
    dmr_network             TEXT                  NOT NULL DEFAULT '',

    dstar_mode              DSTAR_MODE            NOT NULL DEFAULT 'dv',
    dstar_gateway_call_sign TEXT,
    dstar_reflector         TEXT,

    c4fm_wires_x_node_id    INTEGER,
    c4fm_room               TEXT,

    aprs_mode               APRS_MODE,
    aprs_path               TEXT,

    ssb_sideband            SSB_SIDEBAND
);

CREATE UNIQUE INDEX repeater_service_unique_label_kind
    ON repeater_service (repeater_id, label, kind);

CREATE TABLE repeater_service_dmr_talkgroup
(
    id         BIGSERIAL PRIMARY KEY,
    service_id BIGINT   NOT NULL REFERENCES repeater_service (id) ON DELETE CASCADE,
    time_slot  SMALLINT NOT NULL,
    talkgroup  INTEGER  NOT NULL,
    name       TEXT,

    UNIQUE (service_id, time_slot, talkgroup)
);
