# Page Spec: My Page (`/-/my`)

## Purpose

Personal account page for logged-in users. Provides access to saved locations
(QTHs), data exports, and tools like the logbook PDF generator.

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

### CHIRP CSV

Download link for `/-/export/chirp.csv`.

## Portable Logbook

A form that generates a printable portable logbook PDF on demand via
`POST /-/my/logbook.pdf`. Nothing is persisted — the user configures options
each time and the PDF is streamed directly as a download.

**Form fields:**

- Page size: A4 (default) or A5.
- Log pages: number of log pages to include (default 10, max 100).
- Optional pages (checkboxes, all enabled by default):
  - Title page: "Portable Logbook for $CALL".
  - Locations page: table of the user's saved QTHs (address, Maidenhead,
    lat/lon); shows "No locations registered." if none are saved.
  - Phonetic alphabet.
  - Frequency bands (IARU Region 1 / Europe).

**PDF layout:**

- Orientation: landscape.
- Font: DejaVu Sans. 12 pt base; log tables use 16 pt.
- Structure: optional title page, then log pages, then a "References" section
  (locations, phonetic alphabet, frequency bands) if any of those are enabled.
- Log pages have a page header (Date and Location write-in fields with equal
  spacing) and a page footer (call sign left, page number right). Reference
  pages use blank margins with no header/footer.
- Log table columns: QSO #, Station, Time, RST Sent, RST Rcvd, Freq, Mode,
  Power. Each entry is two rows: a 1.2 cm data row and a full-width Remarks row.
  The table is vertically centered on the page and fills the full width using
  proportional (`fr`) column units.
- Frequency bands: IARU Region 1 HF (160 m–10 m) and VHF/UHF (6 m, 2 m, 70 cm)
  with CW/Digimodes/Voice/All-modes sub-allocations. Two-column layout.
  Frequencies in MHz or GHz with thousands separator; band-level rows bold.

The form has two submit buttons: "Download PDF" posts to `/-/my/logbook.pdf` and
"Download Typst source" posts to `/-/my/logbook.typ` via `formaction`.

Generated with Typst (`typst compile`). The `typst` binary must be on PATH. The
template is `templates/logbook.typ`, an Askama template (`.typ` extension
registered in `askama.toml` with the `Text` escaper) that also compiles
standalone for development and preview.
