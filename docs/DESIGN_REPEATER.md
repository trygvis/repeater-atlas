# Repeater Data Model (Design)

**Status:** Design document with a snapshot of the _current_ implementation as
of 2026-01-26.

This document describes a flexible repeater data model beyond a single
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
  repeater ID) out of FM-only rows, and vice versa.
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

## Current SQL Schema (PostgreSQL)

### High-level shape

```
repeater_system (identity + site)
  └─ repeater_service (one per RF service + port label + rx/tx pair)
       └─ repeater_service_dmr_talkgroup (optional per DMR service)
```

### Tables and enums

```sql
-- Physical system identity (what people refer to)
-- Note: the call sign is stored in the global `entity` table.
CREATE TABLE entity (
  id        BIGSERIAL PRIMARY KEY,
  kind      entity_kind NOT NULL, -- 'repeater' | 'contact'
  call_sign TEXT UNIQUE,

  CHECK (call_sign = upper(call_sign)),
  CHECK (kind IS DISTINCT FROM 'repeater' OR call_sign IS NOT NULL)
);

CREATE TABLE contact (
  id           BIGSERIAL PRIMARY KEY,
  entity       BIGINT NOT NULL UNIQUE REFERENCES entity(id) ON DELETE CASCADE,
  kind         contact_kind NOT NULL, -- 'organization' | 'individual'
  display_name TEXT NOT NULL,
  description  TEXT,
  web_url      TEXT,
  email        TEXT,
  phone        TEXT,
  address      TEXT
);

CREATE TABLE repeater_system (
  id            BIGSERIAL PRIMARY KEY,
  entity        BIGINT NOT NULL UNIQUE REFERENCES entity(id) ON DELETE CASCADE,

  -- Optional responsibility/contacts (similar to DNS SOA admin/tech roles).
  owner         BIGINT REFERENCES contact(id) ON DELETE SET NULL,
  tech_contact  BIGINT REFERENCES contact(id) ON DELETE SET NULL,

  name          TEXT,
  description   TEXT,
  address       TEXT,
  maidenhead    TEXT,
  latitude      DOUBLE PRECISION,
  longitude     DOUBLE PRECISION,
  elevation_m   INTEGER,
  country       TEXT,
  region        TEXT,
  status        TEXT NOT NULL DEFAULT 'active'
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

CREATE TYPE fm_bandwidth AS ENUM ('narrow', 'wide');
CREATE TYPE tone_kind AS ENUM ('none', 'ctcss', 'dcs');
CREATE TYPE dstar_mode AS ENUM ('dv', 'dd');
CREATE TYPE aprs_mode AS ENUM ('igate', 'digipeater');
CREATE TYPE ssb_sideband AS ENUM ('lsb', 'usb');

-- Services/features attach to a label + rx/tx pair (label is the RF port name).
CREATE TABLE repeater_service (
  id                      BIGSERIAL PRIMARY KEY,
  repeater_id             BIGINT NOT NULL REFERENCES repeater_system(id) ON DELETE CASCADE,
  kind                    repeater_service_kind NOT NULL,
  enabled                 BOOLEAN NOT NULL DEFAULT TRUE,
  label                   TEXT NOT NULL,
  note                    TEXT,
  rx_hz                   BIGINT NOT NULL,
  tx_hz                   BIGINT NOT NULL,

  fm_bandwidth            fm_bandwidth NOT NULL DEFAULT 'narrow',
  rx_tone_kind            tone_kind NOT NULL DEFAULT 'none',
  rx_ctcss_hz             REAL,
  rx_dcs_code             INTEGER,
  tx_tone_kind            tone_kind NOT NULL DEFAULT 'none',
  tx_ctcss_hz             REAL,
  tx_dcs_code             INTEGER,

  dmr_color_code          SMALLINT NOT NULL DEFAULT 0,
  dmr_repeater_id         BIGINT,
  dmr_network             TEXT NOT NULL DEFAULT '',

  dstar_mode              dstar_mode NOT NULL DEFAULT 'dv',
  dstar_gateway_call_sign TEXT,
  dstar_reflector         TEXT,

  c4fm_wires_x_node_id    INTEGER,
  c4fm_room               TEXT,

  aprs_mode               aprs_mode,
  aprs_path               TEXT,

  ssb_sideband            ssb_sideband
);

-- Prevent duplicates per repeater + label + kind.
CREATE UNIQUE INDEX repeater_service_unique_label_kind
  ON repeater_service(repeater_id, label, kind);

-- Optional: talkgroups/slots as first-class data (DMR only).
CREATE TABLE repeater_service_dmr_talkgroup (
  id                    BIGSERIAL PRIMARY KEY,
  service_id            BIGINT NOT NULL REFERENCES repeater_service(id) ON DELETE CASCADE,
  time_slot             SMALLINT NOT NULL,
  talkgroup             INTEGER NOT NULL,
  name                  TEXT,
  UNIQUE(service_id, time_slot, talkgroup)
);
```

### Suggested constraints (optional, can be added later)

- `CHECK (dmr_color_code BETWEEN 0 AND 15)` for DMR
- Tone-kind/value consistency checks for FM (e.g., if `rx_tone_kind='ctcss'`
  then `rx_ctcss_hz` must be non-null)
- `CHECK (rx_hz > 0 AND tx_hz > 0)` for `repeater_service`

These are useful, but can be postponed until the application layer is stable.

## Examples: Mapping Real Systems

### 1) FM-only repeater with access tone

- `repeater_system`: `call_sign = 'LA5OR'`
- `repeater_service`: kind `fm`, label `VHF`, rx/tx pair, `bandwidth=narrow`,
  `rx_tone_kind=ctcss`, `rx_ctcss_hz=123.0`

### 2) FM + DMR on the same frequencies

Two services with the same label + rx/tx pair:

- `repeater_service(kind='fm', label='VHF', rx_hz=..., tx_hz=...)`
- `repeater_service(kind='dmr', label='VHF', rx_hz=..., tx_hz=...)`

### 3) D-STAR modules A/B/C under one system identity

- `repeater_system`: `call_sign = 'LD1OT'` (system identity)
- `repeater_service` rows:
  - label `C` with 2m frequencies, kind `dstar`
  - label `B` with 70cm frequencies, kind `dstar`
  - label `A` with 23cm frequencies (optional), kind `dstar`

This aligns with how people talk about “LD1OT” as one system, while
radios/directory listings often refer to “LD1OT B/C” as distinct RF modules.

### 4) APRS igate/digipeater

Model APRS as a normal service row with `aprs_mode` set to `igate` or
`digipeater`, plus optional `aprs_path`. Use the usual label + rx/tx pair.

## Query Patterns (examples)

- Find all systems with DMR:
  - join `repeater_system -> repeater_service` and filter `kind='dmr'`
- List all RF ports for a system (labels + rx/tx):
  - `SELECT * FROM repeater_service WHERE repeater_id = ? ORDER BY label, kind`
- Render a capabilities summary for a system:
  - collect `repeater_service.kind` per system, optionally grouped per label
