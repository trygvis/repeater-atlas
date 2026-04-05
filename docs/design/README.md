# Design: Repeater Atlas

## Overview

Design documents describe what is supposed to be implemented. They capture
intent, constraints, and behavior so that the code and the design stay in sync.
When in doubt about how something should work, the design document is the
authoritative source; when code diverges, the document should be updated.

Repeater Atlas is a public repeater directory and map with club-managed data.
This document covers goals, architecture, tech stack, routes, and key behaviors.

Detailed design topics:

- [authentication.md](authentication.md) — Auth framework options and current
  JWT-cookie implementation
- [export.md](export.md) — Export architecture and CHIRP CSV field mapping
- [repeater.md](repeater.md) — Repeater system data model, services, and linking
- [search.md](search.md) — Call sign search UX, constraints, and result model

Page-level specs live under [pages/](pages/README.md).

## Goals

- Provide a public, mobile-friendly repeater directory with map and detail
  views.
- Enable repeater owners/admins to maintain accurate listings via authenticated
  access.
- Make data discoverable by search engines.
- Support club-branded read-only views via custom domains and embeddable pages.

## Non-Goals

- Full feature set beyond the first release.
- Performance optimization beyond "works reliably for initial use."
- Production-scale operations or multi-region support.

## Problem Statement

Radio amateurs need a reliable, searchable directory of repeaters with accurate
settings and map-based discovery, tailored for Norwegian hams but open to any
region. Repeater owners need a straightforward way to keep entries up to date,
while the public should be able to access the data without logins.

## Scope (MVP)

### In

- Public repeater listing page with search and filters.
- Repeater detail page with full settings and location.
- Map view for repeaters.
- Club-scoped read-only views (via custom domains).
- Embeddable read-only views with minimal chrome.
- Admin login and role-based access (manual role assignment).
- Admin CRUD for repeaters.
- SEO-friendly markup for public pages.
- Mobile-first responsive layout.

### Out (for now)

- Public API for third parties.
- Multi-language UI.
- Native mobile apps.
- Automated imports/exports.
- Advanced analytics or usage tracking.

## Requirements

### Functional

- Public listing and detail pages for repeaters.
- Search and filters (by location, band, mode, etc.).
- Interactive map view with clustering as needed.
- Authenticated admin interface for creating and editing entries.

### Non-Functional

- SEO-friendly markup for public pages.
- Mobile-first responsive UI.
- Public read access without login.
- Standard authentication for admin users; roles assigned manually.

## Assumptions and Constraints

- Web-only UI; no native apps in the initial release.
- English is the primary language; translations can be added later.
- Single global instance; clubs use CNAMEs for branded read-only views.
- PostGIS extension is available for spatial range queries.
- Coordinates are stored and serialized as lat/lon; internal code uses a `Point`
  type to represent locations.

## Tech Stack

- Web framework: Axum (async).
- Templates: Askama (compile-time SSR).
- Frontend: Server-rendered HTML with HTMX for progressive enhancement.
- Database: PostgreSQL + Diesel (async, bb8 pool).
- Spatial: PostGIS extension for range queries.
- Map provider: OpenStreetMap public tiles (Leaflet), swappable later.
- Hosting: Containers (Docker/OCI).
- Auth: JWT in signed cookie, Argon2 password hashing.
- Migrations: Diesel migrations.
- Single-instance service; no horizontal scaling in MVP.

## High-Level Architecture

- Client/UI: Public web app with listings, detail pages, and map view.
- Service: Backend providing SSR pages and admin CRUD.
- Data store: Repeater data, club/user membership, changelog entries, and
  spatial queries via PostGIS.
- External integrations: Optional map tile provider and geocoding.

## Routes

### Public

- `/` map page (primary entry point).
- `/$CALL_SIGN` call sign resolver (repeater → detail page, contact → org page).
- `/-/repeater` repeater list.
- `/-/organization` organization list.
- `/-/embed/club/:id` embeddable club-scoped list/detail (minimal layout).
- `/-/search` call sign search endpoint.

### Authenticated User

- `/-/my` user account page (exports, saved locations, portable logbook).
- `/-/my/location` POST: add a location.
- `/-/my/location/{id}/edit` GET: edit form fragment (loaded into modal).
- `/-/my/location/{id}` PUT: update a location.
- `/-/my/location/{id}` DELETE: remove a location.
- `/-/export/chirp.csv` CHIRP radio export.
- `/-/my/logbook.pdf` POST: generate and download a portable logbook PDF.
- `/-/my/logbook.typ` POST: download the rendered Typst source.

### Admin

- `/-/login` login form and signup flow.
- `/-/logout` log out.
- `/-/admin/repeaters` list/manage.
- `/-/admin/repeaters/new` create form.
- `/-/admin/repeaters/:id/edit` edit form.

## Behaviors

### Map URL Parameters

The map page (`/`) accepts optional `lat`, `lon`, and `zoom` query parameters to
center the map on a specific location. These take priority over the persisted
viewport. Example: `/?lat=63.43&lon=10.40&zoom=12`. Used by "Show on map" links
from the user locations section.

### Search and Filters

- Filter by location (text or region), band, mode, status.
- Sort by distance (when user provides location), otherwise by callsign.
- Map markers match current filters.

### SEO

- Server-rendered pages for public routes.
- Descriptive titles, meta descriptions, and structured data (if feasible).

### Geocoding (Optional)

When saving/importing a repeater that has `address` set but no `maidenhead`, the
service can optionally use Nominatim to geocode the address to lat/lon, derive
Maidenhead from that, and persist all three fields.

- Geocoding uses a local CSV cache at `data/geocoder.csv` with columns
  `query,latitude,longitude`. The cache is loaded at startup, checked before
  hitting Nominatim, and successful lookups are appended to the file.
- Controlled by env:
  - `NOMINATIM_ENABLED` (default: on; set to `0` to disable)
  - `NOMINATIM_BASE_URL` (default: public OSM Nominatim)
  - `NOMINATIM_USER_AGENT` (default: `Repeater Atlas`)

### Database

- The app uses a `bb8` pool on top of `diesel-async` for PostgreSQL connections.
- Pool connection setup is routed through a local wrapper so connection
  lifecycle events can be logged and timed when diagnosing startup or test DB
  failures.
- The PostgreSQL pool uses a custom `diesel-async` setup callback so the app
  establishes TLS connections instead of relying on the default non-TLS
  `AsyncPgConnection::establish` path.
- TLS trust roots are loaded from the platform/native certificate store once at
  process startup/lazy initialization time and reused for later connections.
- For environments that need a custom CA bundle, use `SSL_CERT_FILE` or
  `SSL_CERT_DIR` in the process environment so `rustls-native-certs` picks them
  up when building the PostgreSQL TLS client.

### Club Views

- CNAME-based virtual host resolves to a club-scoped read-only view.
- Embeddable view uses semantic HTML and omits global header/footer.
- Admin/editing only on the main site domain.

## Interfaces

- External APIs: Not in MVP; consider public read endpoints later.
- Internal interfaces: Admin UI to backend.
- Club views: CNAME-based virtual hosts and embeddable read-only pages.
- Protocols/transport: HTTPS/JSON.

## Security and Privacy

- Authentication: JWT cookie (stateless). See
  [authentication.md](authentication.md).
- Authorization: Admin roles assigned manually.
- Data sensitivity: Public repeater data; admin accounts protected.

## Operational Considerations

- Deployment model: single Docker/OCI container.
- Configuration: environment variables.
- Observability: structured logging via `tracing`.
- Backup/restore: PostgreSQL dumps.

## Validation Rules (MVP)

- callsign: required, uppercase, length ≤ 10.
- frequencies: numeric, ≥ 0.
- latitude: -90..90, longitude: -180..180.
- mode/band/status: must be from allowed list.
- region/country: required strings.
- password: minimum 8 characters when `validate_password = true` (bypassed for
  test and data-generation convenience).

## Acceptance Criteria

- Public pages are usable without login.
- Mobile layout works on small screens without horizontal scrolling.
- Search results update consistently with filters.
- Admins can create, edit, and deactivate repeaters.
- SEO checks: pages render with titles and canonical URLs.

## Open Questions

- Enumerations: allowed bands, modes, statuses, regions/countries.
- Admin permissions: editor scope (all club repeaters vs only created).
- SEO specifics: structured data and canonical URL patterns.
- Map behavior: clustering, default center/zoom.
- Club view domain mapping strategy and canonical URL rules.
- Ops basics: backups, secrets management, migration flow.
