CREATE TABLE repeater_system
(
    id           BIGSERIAL PRIMARY KEY,
    -- Link to the global call sign registry. One call sign per repeater system.
    call_sign    TEXT NOT NULL UNIQUE REFERENCES call_sign (value) ON DELETE CASCADE,

    -- Responsibility/contacts (optional).
    owner        BIGINT REFERENCES contact (id) ON DELETE SET NULL,
    tech_contact BIGINT REFERENCES contact (id) ON DELETE SET NULL,

    name         TEXT,
    description  TEXT,
    address      TEXT,
    maidenhead   TEXT,
    latitude     DOUBLE PRECISION,
    longitude    DOUBLE PRECISION,
    elevation_m  INTEGER,
    country      TEXT,
    region       TEXT,
    status       TEXT   NOT NULL DEFAULT 'active'
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
    repeater_id             BIGINT NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    kind                    REPEATER_SERVICE_KIND,
    enabled                 BOOLEAN NOT NULL,
    label                   TEXT NOT NULL,
    note                    TEXT NOT NULL,
    rx_hz                   BIGINT NOT NULL,
    tx_hz                   BIGINT NOT NULL,

    fm_bandwidth            FM_BANDWIDTH,
    rx_tone_kind            TONE_KIND,
    rx_ctcss_hz             REAL,
    rx_dcs_code             INTEGER,
    tx_tone_kind            TONE_KIND,
    tx_ctcss_hz             REAL,
    tx_dcs_code             INTEGER,

    dmr_color_code          SMALLINT,
    dmr_repeater_id         BIGINT,
    dmr_network             TEXT,

    dstar_mode              DSTAR_MODE,
    dstar_gateway_call_sign TEXT,
    dstar_reflector         TEXT,

    c4fm_wires_x_node_id    INTEGER,
    c4fm_room               TEXT,

    aprs_mode               APRS_MODE,
    aprs_path               TEXT,

    ssb_sideband            SSB_SIDEBAND,

    CHECK (
        (kind = 'fm' AND fm_bandwidth IS NOT NULL AND rx_tone_kind IS NOT NULL AND tx_tone_kind IS NOT NULL)
            OR (kind IS DISTINCT FROM 'fm' AND fm_bandwidth IS NULL AND rx_tone_kind IS NULL AND tx_tone_kind IS NULL
            AND rx_ctcss_hz IS NULL AND rx_dcs_code IS NULL AND tx_ctcss_hz IS NULL AND tx_dcs_code IS NULL)
        ),
    CHECK (
        rx_tone_kind IS NULL
            OR (rx_tone_kind = 'none' AND rx_ctcss_hz IS NULL AND rx_dcs_code IS NULL)
            OR (rx_tone_kind = 'ctcss' AND rx_ctcss_hz IS NOT NULL AND rx_dcs_code IS NULL)
            OR (rx_tone_kind = 'dcs' AND rx_ctcss_hz IS NULL AND rx_dcs_code IS NOT NULL)
        ),
    CHECK (
        tx_tone_kind IS NULL
            OR (tx_tone_kind = 'none' AND tx_ctcss_hz IS NULL AND tx_dcs_code IS NULL)
            OR (tx_tone_kind = 'ctcss' AND tx_ctcss_hz IS NOT NULL AND tx_dcs_code IS NULL)
            OR (tx_tone_kind = 'dcs' AND tx_ctcss_hz IS NULL AND tx_dcs_code IS NOT NULL)
        ),
    CHECK (
        (kind = 'dmr' AND dmr_color_code IS NOT NULL AND dmr_network IS NOT NULL)
            OR
        (kind IS DISTINCT FROM 'dmr' AND dmr_color_code IS NULL AND dmr_repeater_id IS NULL AND dmr_network IS NULL)
        ),
    CHECK (
        (kind = 'dstar' AND dstar_mode IS NOT NULL)
            OR (kind IS DISTINCT FROM 'dstar' AND dstar_mode IS NULL
            AND dstar_gateway_call_sign IS NULL AND dstar_reflector IS NULL)
        ),
    CHECK (
        kind = 'c4fm'
            OR (kind IS DISTINCT FROM 'c4fm' AND c4fm_wires_x_node_id IS NULL AND c4fm_room IS NULL)
        ),
    CHECK (
        kind = 'aprs'
            OR (kind IS DISTINCT FROM 'aprs' AND aprs_mode IS NULL AND aprs_path IS NULL)
        ),
    CHECK (
        kind = 'ssb'
            OR (kind IS DISTINCT FROM 'ssb' AND ssb_sideband IS NULL)
        )
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

-- Represents an undirected link between two repeater systems (e.g. RF links).
-- Stored as (a,b) where a<b to avoid duplicates.
CREATE TABLE repeater_link
(
    id            BIGSERIAL PRIMARY KEY,
    repeater_a_id BIGINT NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    repeater_b_id BIGINT NOT NULL REFERENCES repeater_system (id) ON DELETE CASCADE,
    note          TEXT   NOT NULL DEFAULT '',

    CHECK (repeater_a_id <> repeater_b_id),
    CHECK (repeater_a_id < repeater_b_id),
    UNIQUE (repeater_a_id, repeater_b_id)
);
