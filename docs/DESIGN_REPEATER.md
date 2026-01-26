# Repeater Data Model (Design)

**Status:** Design-only (not fully implemented in the current codebase as of
2026-01-18).

This document proposes a more flexible repeater data model than a single
`modulation` field on a repeater row. The goal is to model a _physical repeater
system_ (what people identify as “the repeater”) that can expose multiple RF
ports (e.g., different bands / D-STAR modules) and multiple services/features
(FM, DMR, D-STAR, C4FM, APRS, etc.), including combinations like FM+DMR on the
same frequencies.

It is intended to be useful both for humans (shared understanding and
future-proofing) and for coding agents (clear entities, constraints, and
migration outline).

## Problem Statement

In practice, “a repeater” often supports multiple communication modes:

- Analog voice: FM (narrow/wide), AM, SSB (LSB/USB)
- Digital voice/data: DMR, D-STAR, C4FM (System Fusion)
- Packet/APRS services (igate/digipeater), sometimes as a standalone “service”
  and sometimes associated with a frequency plan

Modes can be combined:

- FM + DMR on the same RF pair
- FM + D-STAR on the same RF pair (less common, but seen)
- FM + C4FM (common in “auto mode select” / mixed-mode deployments)

And frequencies are not always shared across modes; a “system” may expose
multiple RF ports (different bands, cross-band links, simplex nodes, etc.).

## Design Goals

- **Identity = physical system:** One “repeater” record represents what users
  call “LA1ABC”, not per-module listings.
- **Multiple RF ports:** Represent D-STAR modules (A/B/C), VHF/UHF ports,
  cross-band ports, etc.
- **Multiple services per port:** Attach FM + DMR (etc.) to the same RF port
  without duplicating the port row.
- **Mode-specific fields live with the mode:** Keep DMR fields (color code,
  repeater ID) out of FM rows, and vice versa.
- **Query-friendly:** Support “find all repeaters that have DMR”, “list all RF
  ports”, “show access requirements”.
- **Incremental implementation:** Allow implementing a minimal subset first
  (e.g., FM + DMR) without blocking later features.

## Terminology

- **Repeater system:** The physical installation people refer to (callsign,
  owner/club, site).
- **Site:** The location metadata (lat/lon, address, grid locator). We keep this
  at the system level for now.
- **RF port / channel:** A specific RX/TX frequency plan (may be same band or
  cross-band).
- **Service / feature:** A capability running on a port (FM, DMR, D-STAR, APRS,
  etc.).

## Proposed SQL Schema (PostgreSQL)

### High-level shape

```
repeater_system (identity + site)
  ├─ repeater_port (one per RF port/module/band)
  │    └─ repeater_service (many per port; FM+DMR etc.)
  │         └─ repeater_service_* (mode-specific details)
  └─ (optional) repeater_service with NULL port_id (non-RF services)
```

### Tables and enums

```sql
-- Physical site (kept at repeater-level, per design choice)
CREATE TABLE repeater_site (
  id            BIGSERIAL PRIMARY KEY,
  name          TEXT,
  address       TEXT,
  maidenhead    TEXT,
  latitude      DOUBLE PRECISION,
  longitude     DOUBLE PRECISION,
  elevation_m   INTEGER,
  country       TEXT,
  region        TEXT
);

-- Physical system identity (what people refer to)
CREATE TABLE repeater_system (
  id            BIGSERIAL PRIMARY KEY,
  ham_club_id   BIGINT REFERENCES ham_club(id),
  call_sign     TEXT NOT NULL,
  name          TEXT,
  description   TEXT,
  site_id       BIGINT REFERENCES repeater_site(id),
  status        TEXT NOT NULL DEFAULT 'active',
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One system can expose multiple RF ports (D-STAR modules, VHF/UHF, cross-band, etc.)
CREATE TABLE repeater_port (
  id            BIGSERIAL PRIMARY KEY,
  repeater_id   BIGINT NOT NULL REFERENCES repeater_system(id) ON DELETE CASCADE,

  -- e.g. "C" (D-STAR 2m module), "B" (70cm), "A" (23cm), "VHF", "UHF", etc.
  label         TEXT NOT NULL,

  rx_hz         BIGINT NOT NULL,
  tx_hz         BIGINT NOT NULL,

  note          TEXT,

  UNIQUE (repeater_id, label)
);

-- Services/features attach to a port (or NULL for future non-RF services if needed)
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
  id            BIGSERIAL PRIMARY KEY,
  repeater_id   BIGINT NOT NULL REFERENCES repeater_system(id) ON DELETE CASCADE,
  port_id       BIGINT REFERENCES repeater_port(id) ON DELETE SET NULL,
  kind          repeater_service_kind NOT NULL,
  enabled       BOOLEAN NOT NULL DEFAULT TRUE
);

-- Prevent duplicates:
CREATE UNIQUE INDEX repeater_service_unique_port_kind
  ON repeater_service(port_id, kind)
  WHERE port_id IS NOT NULL;

CREATE UNIQUE INDEX repeater_service_unique_system_kind_when_no_port
  ON repeater_service(repeater_id, kind)
  WHERE port_id IS NULL;

-- FM details
CREATE TYPE fm_bandwidth AS ENUM ('narrow', 'wide');
CREATE TYPE tone_kind AS ENUM ('none', 'ctcss', 'dcs');

CREATE TABLE repeater_service_fm (
  service_id            BIGINT PRIMARY KEY REFERENCES repeater_service(id) ON DELETE CASCADE,
  bandwidth             fm_bandwidth NOT NULL,

  -- "access" = what users must transmit; "tx" = what repeater transmits (if any)
  rx_tone_kind          tone_kind NOT NULL DEFAULT 'none',
  rx_ctcss_hz           REAL,
  rx_dcs_code           INTEGER,

  tx_tone_kind          tone_kind NOT NULL DEFAULT 'none',
  tx_ctcss_hz           REAL,
  tx_dcs_code           INTEGER
);

-- DMR details
CREATE TABLE repeater_service_dmr (
  service_id            BIGINT PRIMARY KEY REFERENCES repeater_service(id) ON DELETE CASCADE,
  color_code            SMALLINT,
  dmr_repeater_id       BIGINT,
  network               TEXT
);

-- Optional: talkgroups/slots as first-class data
CREATE TABLE repeater_service_dmr_talkgroup (
  id                    BIGSERIAL PRIMARY KEY,
  service_id            BIGINT NOT NULL REFERENCES repeater_service_dmr(service_id) ON DELETE CASCADE,
  time_slot             SMALLINT NOT NULL,
  talkgroup             INTEGER NOT NULL,
  name                  TEXT,
  UNIQUE(service_id, time_slot, talkgroup)
);

-- D-STAR details
CREATE TYPE dstar_mode AS ENUM ('dv', 'dd');

CREATE TABLE repeater_service_dstar (
  service_id            BIGINT PRIMARY KEY REFERENCES repeater_service(id) ON DELETE CASCADE,
  mode                  dstar_mode NOT NULL DEFAULT 'dv',
  gateway_call_sign     TEXT,
  reflector             TEXT
);

-- C4FM / System Fusion details
CREATE TABLE repeater_service_c4fm (
  service_id            BIGINT PRIMARY KEY REFERENCES repeater_service(id) ON DELETE CASCADE,
  wires_x_node_id       INTEGER,
  room                  TEXT
);

-- APRS details
CREATE TABLE repeater_service_aprs (
  service_id            BIGINT PRIMARY KEY REFERENCES repeater_service(id) ON DELETE CASCADE,
  igate                 BOOLEAN NOT NULL DEFAULT FALSE,
  digipeater            BOOLEAN NOT NULL DEFAULT FALSE,
  path                  TEXT
);

-- SSB details (instead of splitting LSB/USB into separate "kinds")
CREATE TYPE ssb_sideband AS ENUM ('lsb', 'usb');

CREATE TABLE repeater_service_ssb (
  service_id            BIGINT PRIMARY KEY REFERENCES repeater_service(id) ON DELETE CASCADE,
  sideband              ssb_sideband
);
```

### Suggested constraints (optional, can be added later)

- `CHECK (color_code BETWEEN 0 AND 15)` for DMR
- Tone-kind/value consistency checks for FM (e.g., if `rx_tone_kind='ctcss'`
  then `rx_ctcss_hz` must be non-null)
- `CHECK (rx_hz > 0 AND tx_hz > 0)` for `repeater_port`

These are useful, but can be postponed until the application layer is stable.

## Examples: Mapping Real Systems

### 1) FM-only repeater with access tone

- `repeater_system`: `call_sign = 'LA5OR'`
- `repeater_port`: label `VHF`, rx/tx pair
- `repeater_service`: kind `fm` -> port
- `repeater_service_fm`: `bandwidth=narrow`, `rx_tone_kind=ctcss`,
  `rx_ctcss_hz=123.0`

### 2) FM + DMR on the same frequencies

Single `repeater_port` row, two services attached:

- `repeater_service(kind='fm', port_id=...)`
- `repeater_service(kind='dmr', port_id=...)`

### 3) D-STAR modules A/B/C under one system identity

- `repeater_system`: `call_sign = 'LD1OT'` (system identity)
- `repeater_port` rows:
  - label `C` with 2m frequencies
  - label `B` with 70cm frequencies
  - label `A` with 23cm frequencies (optional)
- Attach `repeater_service(kind='dstar')` to each port.

This aligns with how people talk about “LD1OT” as one system, while
radios/directory listings often refer to “LD1OT B/C” as distinct RF modules.

### 4) APRS igate/digipeater

Two common approaches:

1. Treat APRS as an RF-backed service and attach it to a port (e.g.,
   144.800/144.800).
2. Treat APRS as a non-RF service by allowing `repeater_service.port_id` to be
   NULL.

Start with (1) for simplicity and consistency with “frequency-plan” UI.

## Query Patterns (examples)

- Find all systems with DMR:
  - join `repeater_system -> repeater_service` and filter `kind='dmr'`
- List all RF ports for a system:
  - `SELECT * FROM repeater_port WHERE repeater_id = ? ORDER BY label`
- Render a capabilities summary for a system:
  - collect `repeater_service.kind` per system, optionally grouped per port
    label
