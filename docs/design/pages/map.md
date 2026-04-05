# Map page

## Purpose

- Provide the primary public map view of repeaters.
- Offer quick access to log in, browser-based "my position", and search.
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
- A narrow right icon bar is always visible on the right edge.
- Collapsible left pane to the left of the icon bar.
- Large modal for search results.

## Page sections

- Map: marker cluster for repeaters with coordinates; marker labels show call
  sign.
- Right icon bar (`#map-right-bar`): fixed 60 px strip on the right edge.
  Contains icon buttons only. Currently holds the pane toggle button. TODO: it
  currently holds more than this
- Left pane (`#map-left-pane`): pico `article` element. Contains:
  - Header with the site name.
  - Nav section: login state, "My position" action, search link.
  - Repeater details section (see below).
- Search results modal: call sign search field and the result list; each result
  links to `/{call_sign}`.

## Right icon bar

Always visible. Uses `background: var(--pico-card-background-color)` and a left
border. Buttons use the `.icon-button` utility class (zeroes pico form spacing)
with `class="outline secondary"` for coloring.

Icons are rendered via Lucide. Only the icons in use are imported from
`/static/vendor/lucide/icons/` — not the full icon set.

## Left pane

On desktop the pane is 300 px wide. On mobile (≤ 576 px, matching pico's `sm`
breakpoint) it spans the full width minus the 60 px icon bar. Toggled via
`display: none` — no animation. Starts open on page load.

The toggle button shows a `chevron-right` icon (Lucide) when the pane is open
and `chevron-left` when closed.

## Repeater details section

Lives inside the left pane below the nav. Three states:

- **Empty:** "Select a repeater on the map." prompt.
- **Populated:** Call sign (linked to detail page), status, and service summary.
- **Replaced:** Clicking a new marker replaces the content in place with no
  intermediate empty state.

Escape clears the details back to the empty state. Clicking a marker also opens
the pane if it is currently closed.

## Behavior

- Repeaters without coordinates are excluded from markers.
- Markers are clustered; the map fits to bounds on the first visit.
- After the map is moved or zoomed, the current viewport is stored client-side
  in browser local storage.
- When a saved viewport exists, the page restores that center/zoom instead of
  fitting to bounds.
- "My position" asks the browser for the current location, saves it as the map
  viewport, and recenters the map there.
- If the browser does not allow geolocation, the action does nothing.
- Search only matches call signs and opens results in the modal.
- The modal remains full height even when results are short.
