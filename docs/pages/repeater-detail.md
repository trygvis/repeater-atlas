# Repeater detail

## Purpose

- Provide a public, read-only view of a repeater system and its services.
- Show ownership/contact context, linked repeaters, and nearby repeaters on a
  map.

## Routes

- `/repeater/{call_sign}` renders this page.
- `/call-sign/{call_sign}` resolves to this page when the call sign is a
  repeater.
- Call signs are uppercased before lookup.

## Data sources

- `dao::repeater_system::find_by_call_sign` loads the repeater system.
- `dao::contact::find_with_call_sign` loads owner/technical contact details.
- `dao::repeater_service::select_by_repeater_id` loads service rows.
- `dao::repeater_link::select_with_other_call_sign` loads linked repeaters.
- `dao::repeater_system::select_within_radius` provides map markers within 50
  km.

## Layout

- Two-column layout on wide screens (content + map); single column below 900px.
- A detail sidebar map is only shown when the repeater has coordinates.

## Page sections

- Header: call sign (H1) and optional repeater name.
- Details:
  - Status, owner, technical contact, description.
  - Maidenhead locator and lat/long (shown directly from stored fields).
- Services:
  - Groups by service kind: FM, DMR, D-STAR, C4FM, APRS, SSB, AM.
  - Each service shows TX/RX plus kind-specific fields and a note.
  - Disabled services are labeled "(disabled)".
  - Shows "No services defined." when empty.
- Linked repeaters:
  - List of call signs with optional notes, when any links exist.
- Nearby repeaters:
  - Same set as the map markers, sorted by distance from the current repeater.
  - Shown after linked repeaters.
- Map:
  - Leaflet map with a 50 km radius circle around the repeater.
  - Markers for repeaters within 50 km, labeled by call sign.
- Navigation: "Back to list" link to `/repeater`.

## Behavior

- Description defaults to `-` when missing.
- Maidenhead and lat/long default to `-` when missing.
- Map markers are ordered by call sign (as returned by the radius query).
