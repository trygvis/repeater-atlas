# Implementation Spec: Repeater Atlas (Web)

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

## Tech Stack

- Web framework: Axum (async).
- Templates: Askama (compile-time SSR).
- Frontend: Server-rendered HTML with HTMX for progressive enhancement.
- Database: PostgreSQL + Diesel (with r2d2 pool).
- Spatial: PostGIS extension for range queries.
- Map provider: OpenStreetMap public tiles (Leaflet), swappable later.
- Hosting: Containers (Docker/OCI).
- Auth: Session cookies (signed/encrypted), Argon2 password hashing.
- Migrations: Diesel migrations.

## Core Attributes

- Single-instance service; no web-scale or horizontal scaling in MVP.
- In-memory session store keyed by cookie ID.

## Data Model

### Repeater

- Identity: callsign, name/label.
- Technical: bands, frequencies, tones, one or more modes.
- Status: active/offline/planned/seasonal.
- Location: coordinates, region, country.
- Metadata: description, contact responsibility.

### User

- Identity: email, callsign, display name.
- Auth: password hash, last login.
- Memberships: contact roles.

### Contact

- Identity: organization/individual, display name, callsign (optional).
- Contact: website, email.
- Metadata: notes.

### Contact Membership

- Link: user, contact.
- Role: admin, editor.

### Repeater Change Log Entry

- Link: repeater, author.
- Content: change note text.
- Metadata: timestamp (can be backdated), public by default.

### User Location

- Owner: user reference.
- Address: optional street address.
- Maidenhead: grid locator (always populated).
- Lat/lon: coordinates (always populated).

## Pages and Routes

### Public

- `/` map page (primary entry point).
- `/$CALL_SIGN` call sign resolver:
  - repeater -> repeater detail page
  - contact -> organization page
- `/-/repeater` repeater list.
- `/-/organization` organization list.
- `/-/embed/club/:id` embeddable club-scoped list/detail (minimal layout).
- `/-/search` call sign search endpoint.

### Authenticated User

- `/-/my` user account page (exports, saved locations).
- `/-/my/location` POST: add a location.
- `/-/my/location/{id}/edit` GET: edit form fragment (loaded into modal).
- `/-/my/location/{id}` PUT: update a location.
- `/-/my/location/{id}` DELETE: remove a location.
- `/-/export/chirp.csv` CHIRP radio export.

### Admin

- `/-/login` login form.
- `/-/logout` log out.
- `/-/admin/repeaters` list/manage.
- `/-/admin/repeaters/new` create form.
- `/-/admin/repeaters/:id/edit` edit form.

## Behaviors

### Map URL Parameters

- The map page (`/`) accepts optional `lat`, `lon`, and `zoom` query parameters
  to center the map on a specific location. These take priority over the
  persisted viewport. Example: `/?lat=63.43&lon=10.40&zoom=12`. Used by "Show on
  map" links from the user locations section.

### Search and Filters

- Filter by location (text or region), band, mode, status.
- Sort by distance (when user provides location), otherwise by callsign.
- Map markers match current filters.

### SEO

- Server-rendered pages for public routes.
- Descriptive titles, meta descriptions, and structured data (if feasible).

### Geocoding (Optional)

- When saving/importing a repeater that has `address` set but no `maidenhead`,
  the service can (optionally) use Nominatim to geocode the address to lat/lon,
  derive Maidenhead from that, and persist all three fields.
- Geocoding uses a local CSV cache at `data/geocoder.csv` with columns:
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

### Admin

- Create/edit repeater entries with validation.
- Role-based access: club admins manage all club repeaters; editors can edit
  assigned entries.
- The login page also exposes a signup flow that creates a user and immediately
  issues the normal auth cookie on success.
- User creation validation lives in the user service, not in the web handler, so
  signup and any future user-creation entry points share the same call sign,
  email, password, and duplicate-user rules.
- The signup handler only maps those user-service results to page errors or a
  successful login redirect.

### Club Views

- CNAME-based virtual host resolves to a club-scoped read-only view.
- Embeddable view uses semantic HTML and omits global header/footer.
- Admin/editing only on the main site domain.

## Validation Rules (MVP)

- callsign: required, uppercase, length <= 10.
- frequencies: numeric, >= 0.
- latitude: -90..90, longitude: -180..180.
- mode/band/status: must be from allowed list.
- region/country: required strings.

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
