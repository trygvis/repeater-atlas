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

The page has exactly two layouts, **desktop** and **mobile**, selected by the
**breakpoint** below. CSS media queries and a JS
`window.matchMedia("(max-width: 576px)")` mirror each other, so styling and
behavior always agree on which layout is active.

### Sizes and terms

Defined once here and referenced by name throughout the rest of this document;
do not restate the literal values elsewhere.

| Term                       | Value                         | Meaning                                                                                                                      |
| -------------------------- | ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **breakpoint**             | pico `sm`, 576 px             | Boundary between the two layouts.                                                                                            |
| **desktop**                | viewport width > breakpoint   | Side-by-side layout (map + side pane).                                                                                       |
| **mobile**                 | viewport width ≤ breakpoint   | Full-screen-map layout with floating controls.                                                                               |
| **pane width**             | 300 px                        | Width of the desktop side pane body.                                                                                         |
| **icon strip width**       | 60 px                         | Width of the desktop icon column.                                                                                            |
| **FAB inset**              | 12 px right, 20 px bottom     | Gap from the viewport edges to the mobile floating controls; the larger bottom value clears the Leaflet attribution control. |
| **nearest-zoom threshold** | `NEAREST_ZOOM_THRESHOLD` = 10 | Minimum map zoom at which the nearby-repeaters list shows.                                                                   |
| **nearest-list cap**       | `NEAREST_MAX` = 20            | Maximum number of entries in the nearby-repeaters list.                                                                      |

### Desktop layout

- `body` is a full-viewport flex row (`height: 100vh`).
- The Leaflet map (`#repeater-map-host`) fills the remaining width (`flex: 1`).
- The side pane sits beside the map (not over it), so Leaflet's viewport and
  `getBounds()` reflect only the visible map area. It is the pane width wide,
  plus the icon strip width.
- Search results open in a large modal.

### Mobile layout

- The map fills the whole viewport.
- The side pane is dropped entirely (`#pane-body` is `display: none`): no site
  header, no nearby-repeaters list, no in-page details panel.
- The icon column (`#icon-column`) floats as a rounded bottom-right FAB stack,
  the FAB inset from the edges (within right-thumb reach), instead of the fixed
  right-edge strip. The pane toggle button is hidden (there is no pane to
  toggle).
- Tapping a map marker navigates straight to the repeater detail page
  (`/{call_sign}`) instead of opening an in-page panel. Marker call-sign
  tooltips and the search modal remain available.

## Page sections

- Map: marker cluster for repeaters with coordinates; marker labels show call
  sign.
- Side pane (`#side-pane`): on desktop, a fixed flex container on the right
  edge; dropped on mobile. Contains:
  - Pane body (`#pane-body`): collapsible content area. Contains a Pico
    `article` with the site header, nearest repeaters list, and repeater details
    section (see below).
  - Icon column (`#icon-column`): icon buttons only — a right-edge strip (icon
    strip width) on desktop, a floating bottom-right FAB stack on mobile.
- Search results modal: call sign search field and results list; each result
  links to `/{call_sign}`.

## Icon column

Always visible. On desktop it is a strip on the right edge (the icon strip
width) using `background: var(--pico-card-background-color)` and a left border.
On mobile it floats as a rounded bottom-right FAB stack, the FAB inset from the
edges (the pane toggle is hidden there). Buttons use the `.icon-button` utility
class (zeroes pico form spacing) with `class="outline secondary"` for coloring.

Icons are rendered via Lucide. Only the icons in use are imported from
`/static/vendor/lucide/icons/` — not the full icon set.

## Pane body

On desktop the pane body is a column beside the map (the pane width). It is
toggled via `display: none` — no animation — and starts open on page load. The
toggle button shows a `chevron-right` icon (Lucide) when the pane is open and
`chevron-left` when closed.

On mobile the pane body is hidden entirely (`display: none`); the nearby list
and details panel below are desktop-only. Crossing the breakpoint recomputes the
map size (`map.invalidateSize()`) and refreshes the list.

## Nearest repeaters list

Desktop only — it lives inside the side pane, which is dropped on mobile
(`updateNearestList()` returns early there). Driven by the current zoom level
and map center. Two states:

- **Zoom hint** (`#nearest-zoom-hint`): shown below the nearest-zoom threshold.
  Text: "Zoom in to see nearby repeaters." (exact wording subject to change).
- **List** (`#nearest-list`): shown at or above the nearest-zoom threshold. Up
  to the nearest-list cap of repeaters visible in the current viewport, sorted
  ascending by distance from the map center. Each entry shows the call sign and
  distance in metres or kilometres. Clicking an entry opens the repeater details
  section (same as clicking a map marker). Recomputed on every `moveend` event
  (pan or zoom).

Visible repeaters are determined using `map.getBounds().contains()`. Distance is
computed using `map.distance()` (Haversine). Both operate client-side against
the already loaded `data` array. No backend request is made.

## Repeater details section

Desktop only. On mobile there is no in-page details panel — tapping a marker
navigates directly to the detail page (see "Mobile layout"). On desktop it lives
inside the left pane, overlaid on top of the nearest-repeaters list when active.
Three states:

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
