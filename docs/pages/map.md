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

- `body` is a full-viewport flex row (`height: 100vh`).
- Leaflet map (`#repeater-map-host`) fills the remaining width (`flex: 1`).
- Side pane sits beside the map (not over it), so Leaflet's viewport and
  `getBounds()` reflect only the visible map area.
- Large modal for search results.

## Page sections

- Map: marker cluster for repeaters with coordinates; marker labels show call
  sign.
- Side pane (`#side-pane`): fixed flex container on the right edge. Contains:
  - Pane body (`#pane-body`): collapsible content area. Contains a Pico
    `article` with the site header, nearest repeaters list, and repeater details
    section (see below).
  - Icon column (`#icon-column`): fixed 60 px strip. Contains icon buttons only.
- Search results modal: call sign search field and results list; each result
  links to `/{call_sign}`.

## Icon column

Always visible. 60 px wide. Uses `background: var(--pico-card-background-color)`
and a left border. Buttons use the `.icon-button` utility class (zeroes pico
form spacing) with `class="outline secondary"` for coloring.

Icons are rendered via Lucide. Only the icons in use are imported from
`/static/vendor/lucide/icons/` — not the full icon set.

## Pane body

On desktop the pane body is 300 px wide. On mobile (≤ 576 px, matching pico's
`sm` breakpoint) it spans the full width minus the 60 px icon column. Toggled
via `display: none` — no animation. Starts open on page load.

The toggle button shows a `chevron-right` icon (Lucide) when the pane is open
and `chevron-left` when closed.

## Nearest repeaters list

Lives inside the left pane below the header. Driven by the current zoom level
and map center. Two states:

- **Zoom hint** (`#nearest-zoom-hint`): shown when zoom < 10. Text: "Zoom in to
  see nearby repeaters." (exact wording subject to change).
- **List** (`#nearest-list`): shown when zoom >= 10. Up to 20 repeaters visible
  in the current viewport, sorted ascending by distance from the map center.
  Each entry shows the call sign and distance in metres or kilometres. Clicking
  an entry opens the repeater details section (same as clicking a map marker).
  Recomputed on every `moveend` event (pan or zoom).

The zoom threshold is a JS constant (`NEAREST_ZOOM_THRESHOLD = 10`).

Visible repeaters are determined using `map.getBounds().contains()`. Distance is
computed using `map.distance()` (Haversine). Both operate client-side against
the already loaded `data` array. No backend request is made.

## Repeater details section

Lives inside the left pane, overlaid on top of the nearest-repeaters list when
active. Three states:

- **Hidden:** nearest-repeaters list is shown instead.
- **Populated:** shown on marker click or nearest-list item click. Displays call
  sign, status, and service summary. A "Show details" link navigates to the
  repeater detail page. A close button (X icon) dismisses the section.
- **Replaced:** clicking a new marker replaces the content in place with no
  intermediate empty state.

Escape dismisses the details and returns to the nearest-repeaters list (or zoom
hint). Clicking a marker also opens the pane if it is currently closed.

Panning or zooming while details are visible does not dismiss them; the
nearest-repeaters list updates in the background and becomes visible again once
details are cleared.

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
