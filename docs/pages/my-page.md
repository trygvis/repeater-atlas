# Page Spec: My Page (`/-/my`)

## Purpose

Personal account page for logged-in users. Provides access to saved locations
(QTHs) and data exports.

Redirects to `/-/login` if the user is not authenticated.

## Layout

Standard main layout. Single-column content container.

## Sections

### Locations

A list of the user's saved QTH locations followed by an "Add location" button.

Each location is rendered as a Pico CSS `<article>` card containing:

- A `<dl>` with only the populated fields shown: Address, Maidenhead, Lat/Lon.
- A `<footer>` with a `role="group"` button strip:
  - **Map icon** (`map`): link to `/?lat=X&lon=Y&zoom=12`; only shown when
    coordinates are available.
  - **Edit icon** (`pencil`): opens the edit modal pre-populated with the
    location's current values.
  - **Delete icon** (`trash-2`): deletes the location; updates the list in place
    via HTMX.

The list (`#location-list`) is the HTMX swap target for add, edit, and delete
responses — all three replace the full list.

#### Add location modal

Triggered by the "Add location" button below the list. Uses a native `<dialog>`
element. The form has four optional fields: Address, Maidenhead, Latitude,
Longitude. On success the modal closes and the list refreshes. On cancel the
modal closes with no change.

#### Edit location modal

Triggered by the edit button on a card. The edit form fragment is fetched via
HTMX (`GET /-/my/location/{id}/edit`) and injected into the modal body before
the modal opens. On save the modal closes and the list refreshes. On cancel the
modal closes with no change.

#### Resolution logic

When a location is saved or updated the system resolves all three coordinate
representations from whichever inputs were provided:

- Lat/lon given → derive Maidenhead; store as-is.
- Maidenhead given → derive lat/lon from grid center.
- Address given → geocode via Nominatim → derive Maidenhead.

## Exports

- **CHIRP CSV**: download link for `/-/export/chirp.csv`.
