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
- Metadata: description, timestamps, club ownership.

### User

- Identity: email, callsign, display name.
- Auth: password hash, last login.
- Memberships: club roles.

### Ham Club

- Identity: name, short name, callsign (if applicable).
- Contact: website, email.
- Metadata: timestamps.

### Club Membership

- Link: user, club.
- Role: admin, editor.

### Repeater Change Log Entry

- Link: repeater, author.
- Content: change note text.
- Metadata: timestamp (can be backdated), public by default.

## Pages and Routes

### Public

- `/` home/overview with search entry point.
- `/repeaters` list with filters and map toggle.
- `/repeaters/:id` detail page.
- `/embed/club/:id` embeddable club-scoped list/detail (minimal layout).

### Admin

- `/admin/login` login form.
- `/admin/repeaters` list/manage.
- `/admin/repeaters/new` create form.
- `/admin/repeaters/:id/edit` edit form.

## Behaviors

### Search and Filters

- Filter by location (text or region), band, mode, status.
- Sort by distance (when user provides location), otherwise by callsign.
- Map markers match current filters.

### SEO

- Server-rendered pages for public routes.
- Descriptive titles, meta descriptions, and structured data (if feasible).

### Admin

- Create/edit repeater entries with validation.
- Role-based access: club admins manage all club repeaters; editors can edit
  assigned entries.

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
