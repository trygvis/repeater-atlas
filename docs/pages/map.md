# Map page

## Purpose

- Provide the primary public map view of repeaters.
- Offer quick access to login and search in a small overlay.
- Let visitors inspect repeaters without leaving the map.

## Routes

- `/` renders this page.

## Data sources

- `dao::repeater_system::select_with_call_sign` loads repeater systems.
- `resolve_site_fields` derives latitude/longitude when stored on the system.
- `dao::repeater_service::select_kinds_by_repeater_ids` provides service kinds.

## Layout

- Full-screen Leaflet map.
  - Small overlay in the top-right for login and a search link.
- Details panel in the bottom-right for the selected repeater.
- Large modal for search results.

## Page sections

- Map:
  - Marker cluster for repeaters with coordinates.
  - Marker labels show call sign.
  - Hero overlay:
    - Login state and link.
    - Search link placed after the login action.
- Repeater details panel:
  - Call sign with link to the detail page.
  - Status and service summary.
  - Search results modal:
    - Call sign search field and results list.
    - List of call sign matches (repeaters and organizations).
    - Each result links to `/{call_sign}`.

## Behavior

- Repeaters without coordinates are excluded from markers.
- Markers are clustered and the map is fit to bounds.
- When no markers exist, the map centers on `[64.4, 11.7]` at zoom 4.
- Clicking a marker populates the details panel.
- Escape or close hides the details panel and returns to the empty state.
  - Search only matches call signs and opens results in the modal.
  - The modal remains full height even when results are short.
