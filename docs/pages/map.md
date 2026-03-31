# Map page

## Purpose

- Provide the primary public map view of repeaters.
- Offer quick access to login, browser-based "my position", and search.
- Let visitors inspect repeaters without leaving the map.

## Routes

- `/` renders this page.

## Data sources

- `dao::repeater_system::select_with_call_sign` loads repeater systems.
- `repeater.location()` derives a `Point` when the repeater has stored
  latitude/longitude.
- `dao::repeater_service::select_kinds_by_repeater_ids` provides service kinds.

## Layout

- Full-screen Leaflet map.
- A toggle button anchored to the right edge of the screen opens/closes the side
  pane.
- Side pane on the right containing nav/auth and repeater details.
- Large modal for search results.

## Page sections

- Map:
  - Marker cluster for repeaters with coordinates.
  - Marker labels show call sign.
- Side pane (`#map-side-pane`):
  - Nav section: login state, "My position" action, search link.
  - Repeater details section (see below).
- Toggle button (`#pane-toggle`): opens and closes the side pane. Moves left by
  the pane width when the pane is open so it stays visible.
- Search results modal:
  - Call sign search field and results list.
  - List of call sign matches (repeaters and organizations).
  - Each result links to `/{call_sign}`.

## Side pane

On desktop the pane is 300 px wide and slides in from the right edge. On mobile
(≤ 600 px) it is full-screen. The pane starts closed on page load.

The toggle button shows `❯` when closed and `❮` when open.

## Repeater details section

Lives inside the side pane below the nav. Three states:

- **Empty:** "Select a repeater on the map." prompt; no close button.
- **Populated:** Call sign (linked to detail page), status, and service summary.
  A close button is shown.
- **Replaced:** Clicking a new marker while details are shown replaces the
  content in place with no intermediate empty state.

Closing the details section (close button or Escape) returns to empty state
without closing the pane. Clicking a marker also opens the pane if it is
currently closed.

## Behavior

- Repeaters without coordinates are excluded from markers.
- Markers are clustered; the map fits to bounds on first visit.
- After the map is moved or zoomed, the current viewport is stored client-side
  in browser local storage.
- When a saved viewport exists, the page restores that center/zoom instead of
  fitting to bounds.
- "My position" asks the browser for the current location, saves it as the map
  viewport, and recenters the map there.
- If the browser does not allow geolocation, the action does nothing.
- Search only matches call signs and opens results in the modal.
- The modal remains full height even when results are short.
