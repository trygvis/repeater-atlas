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
  at the system level for now, and use a PostGIS geography index derived from
  (longitude, latitude) for range queries. Lat/long are the authoritative
  coordinates; the grid locator is optional metadata and should not be used to
  derive coordinates at render time.
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

The schema is available as `schema.tmp.sql` after running
`just db-export-schema`.

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

## Linking

Some repeaters are RF-linked or otherwise bridged as part of a local network. We
model these relationships as an undirected link between two repeater systems.

- Stored in `repeater_link` as `(repeater_a_id, repeater_b_id)` with
  `repeater_a_id < repeater_b_id` to keep the pair unique.
- Optional `note` can describe the link (for example, "RF link", "node ID 1234",
  or a local site label).
- Queries should treat the link as symmetric: a repeater can be linked to many
  others, and the "other" side is whichever ID is not the current repeater.
- Rendering should list linked call signs on the repeater detail page, with the
  note shown only when present.

## Query Patterns (examples)

- Find all systems with DMR:
  - join `repeater_system -> repeater_service` and filter `kind='dmr'`
- List all RF ports for a system (labels + rx/tx):
  - `SELECT * FROM repeater_service WHERE repeater_id = ? ORDER BY label, kind`
- Render a capabilities summary for a system:
  - collect `repeater_service.kind` per system, optionally grouped per label
