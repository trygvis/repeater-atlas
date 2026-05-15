Repeater Atlas Requirements
===========================

This document assembles the implemented requirements extracted from the
design documents. Each requirement is stored as a standalone file in
this directory and included below in chapter order.

.. container:: contents

   Chapters

Application Scope
-----------------

These requirements describe the public application scope, deployment
model, and cross-cutting public behavior.

.. container:: requirement

RA-1000: Public Read Access
---------------------------

The application shall allow public users to read repeater pages without
logging in.

Rationale
~~~~~~~~~

The service is a public repeater directory, so browsing must not require
an account.

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- A public user can open public repeater pages without authentication.
- Public pages do not redirect unauthenticated users to the login page.

Links
~~~~~

- Refines: ``docs/design/README.rst``

.. container:: requirement

RA-1001: SEO-Friendly Public Pages
----------------------------------

The application shall render public pages as server-side HTML suitable
for search engine indexing.

.. _rationale-1:

Rationale
~~~~~~~~~

Repeater data should be discoverable through ordinary web search.

.. _acceptance-criteria-1:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Public pages are rendered on the server.
- Public pages include descriptive titles.
- Public pages can be fetched without client-side JavaScript execution.

.. _links-1:

Links
~~~~~

- Refines: ``docs/design/README.rst``

.. container:: requirement

RA-1002: Mobile-Friendly Layout
-------------------------------

The application shall provide layouts that are usable on mobile screens
without horizontal scrolling.

.. _rationale-2:

Rationale
~~~~~~~~~

Repeater directory users commonly access the site from phones while
mobile or portable.

.. _acceptance-criteria-2:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Public pages fit narrow viewports.
- Primary navigation and actions remain usable on mobile screens.
- Content does not require horizontal scrolling on small screens.

.. _links-2:

Links
~~~~~

- Refines: ``docs/design/README.rst``

.. container:: requirement

RA-1003: Single-Instance Deployment
-----------------------------------

The application shall be designed for a single running service instance
in the initial release.

.. _rationale-3:

Rationale
~~~~~~~~~

The MVP does not require horizontal scaling or multi-region operation.

.. _acceptance-criteria-3:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- The deployment model does not require multiple application instances.
- State management assumes one service instance for the MVP.

.. _links-3:

Links
~~~~~

- Refines: ``docs/design/README.rst``

.. container:: requirement

RA-1004: Spatial Data Support
-----------------------------

The application shall use PostgreSQL with PostGIS for spatial repeater
queries.

.. _rationale-4:

Rationale
~~~~~~~~~

Map and nearby-repeater features need reliable geographic range queries.

.. _acceptance-criteria-4:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Database migrations enable the required spatial support.
- Repeater location queries can filter by distance or radius.

.. _links-4:

Links
~~~~~

- Refines: ``docs/design/README.rst``

Club Views and Geocoding
------------------------

These requirements cover club-scoped public views and optional address
geocoding behavior.

.. container:: requirement

RA-1005: Club Read-Only Views
=============================

The application shall support club-scoped read-only views through custom
host names.

.. _rationale-5:

Rationale
---------

Clubs need branded public views without exposing editing capabilities on
those domains.

.. _acceptance-criteria-5:

Acceptance Criteria
-------------------

- A club host resolves to a club-scoped public view.
- Editing features are not available on club-scoped read-only views.

.. _links-5:

Links
-----

- Refines: ``docs/design/README.rst``

.. container:: requirement

RA-1006: Embeddable Club Views
==============================

The application shall provide embeddable club-scoped views with minimal
page chrome.

.. _rationale-6:

Rationale
---------

Clubs should be able to embed repeater information in existing sites
without duplicating the full application layout.

.. _acceptance-criteria-6:

Acceptance Criteria
-------------------

- Embedded views omit global navigation and footer chrome.
- Embedded views render semantic HTML suitable for inclusion in other
  pages.

.. _links-6:

Links
-----

- Refines: ``docs/design/README.rst``

.. container:: requirement

RA-1007: Optional Address Geocoding
-----------------------------------

The application shall be able to derive latitude, longitude, and
Maidenhead locator from an address when geocoding is enabled.

.. _rationale-7:

Rationale
~~~~~~~~~

Repeater and user-location data may be entered with an address instead
of coordinates.

.. _acceptance-criteria-7:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Address lookup can be disabled by configuration.
- Successful address lookup stores latitude, longitude, and Maidenhead
  data.
- Geocoding uses a local CSV cache before calling the external service.

.. _links-7:

Links
~~~~~

- Refines: ``docs/design/README.rst``

Authentication and Authorization
--------------------------------

These requirements describe account identity, JWT-cookie authentication,
signup, logout, validation, and role lookup.

.. container:: requirement

RA-1008: User Account Identity
------------------------------

The application shall store user call sign, email address, and password
hash for authenticated accounts.

.. _rationale-8:

Rationale
~~~~~~~~~

Authenticated editing requires persistent user identity and
password-based login.

.. _acceptance-criteria-8:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- A user account has a call sign.
- A user account has an email address.
- A user account stores a password hash instead of a plaintext password.

.. _links-8:

Links
~~~~~

- Refines: ``docs/design/authentication.md``

.. container:: requirement

RA-1009: Signed JWT Cookie Authentication
=========================================

The application shall authenticate users with a signed JWT stored in a
cookie.

.. _rationale-9:

Rationale
---------

The preferred authentication approach is stateless and avoids a
server-side session store.

.. _acceptance-criteria-9:

Acceptance Criteria
-------------------

- Successful login issues an authentication cookie.
- Authenticated requests validate the JWT cookie.
- Invalid or missing JWT cookies are treated as unauthenticated
  requests.

.. _links-9:

Links
-----

- Refines: ``docs/design/authentication.md``

.. container:: requirement

RA-1010: JWT Claims and Expiration
----------------------------------

Authentication JWTs shall contain subject, issued-at, and expiration
claims, and shall expire after seven days.

.. _rationale-10:

Rationale
~~~~~~~~~

Short, fixed claims keep authentication behavior clear and auditable.

.. _acceptance-criteria-10:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Issued JWTs include ``sub``.
- Issued JWTs include ``iat``.
- Issued JWTs include ``exp``.
- Issued JWTs expire after seven days.

.. _links-10:

Links
~~~~~

- Refines: ``docs/design/authentication.md``

.. container:: requirement

RA-1011: Logout Cookie Expiration
=================================

The application shall log users out by expiring the authentication
cookie on the client.

.. _rationale-11:

Rationale
---------

Stateless JWT authentication does not maintain a server-side session to
revoke.

.. _acceptance-criteria-11:

Acceptance Criteria
-------------------

- A logout request clears or expires the authentication cookie.
- After logout, protected pages require authentication again.

.. _links-11:

Links
-----

- Refines: ``docs/design/authentication.md``

.. container:: requirement

RA-1012: Signup Issues Auth Cookie
----------------------------------

The login page shall be able to create a user and immediately issue the
normal authentication cookie on success.

.. _rationale-12:

Rationale
~~~~~~~~~

Signup and first login should be one continuous flow for the user.

.. _acceptance-criteria-12:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- A valid signup creates a user account.
- A successful signup returns an authentication cookie.
- The created user is authenticated after signup.

.. _links-12:

Links
~~~~~

- Refines: ``docs/design/authentication.md``

.. container:: requirement

RA-1013: Central User Validation
================================

User creation rules shall live in the user service so every entry point
applies the same validation and duplicate-user handling.

.. _rationale-13:

Rationale
---------

Validation in one service avoids inconsistent behavior between web
handlers, tests, and data generation.

.. _acceptance-criteria-13:

Acceptance Criteria
-------------------

- Web signup uses the user service for account creation.
- Duplicate-user handling is implemented by the user service.
- Password validation can be enabled for web entry points.

.. _links-13:

Links
-----

- Refines: ``docs/design/authentication.md``

.. container:: requirement

RA-1014: Role Lookup for Protected Requests
-------------------------------------------

The application shall look up authorization roles when handling
protected requests.

.. _rationale-14:

Rationale
~~~~~~~~~

Roles may change independently of a user's authentication cookie.

.. _acceptance-criteria-14:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Protected handlers can identify the authenticated user.
- Authorization decisions use stored role or membership data.

.. _links-14:

Links
~~~~~

- Refines: ``docs/design/authentication.md``

Call Sign Search
----------------

These requirements describe the public call sign search experience.

.. container:: requirement

RA-1015: Map Search Entry Point
-------------------------------

The map page shall provide a search action near the login action.

.. _rationale-15:

Rationale
~~~~~~~~~

Call sign lookup is a primary navigation tool from the public map page.

.. _acceptance-criteria-15:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- The map page displays a search action.
- The search action is placed with the other primary overlay actions.

.. _links-15:

Links
~~~~~

- Refines: ``docs/design/search.md``

.. container:: requirement

RA-1016: Call Sign Search Modal
===============================

Selecting the search action shall open a large modal containing the call
sign search field and results.

.. _rationale-16:

Rationale
---------

A modal lets users search without leaving the map context.

.. _acceptance-criteria-16:

Acceptance Criteria
-------------------

- The search action opens a modal.
- Typing in the modal search field updates the result list.
- The modal remains open until closed or a result is selected.

.. _links-16:

Links
-----

- Refines: ``docs/design/search.md``

.. container:: requirement

RA-1017: Search Result Content
------------------------------

Call sign search results shall show the call sign value and whether the
result is a repeater or organization.

.. _rationale-17:

Rationale
~~~~~~~~~

Users need to distinguish between repeater systems and organizations
that share the same call sign namespace.

.. _acceptance-criteria-17:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Each result row shows a call sign.
- Each result row shows a type badge.
- Selecting a result navigates to ``/{call_sign}``.

.. _links-17:

Links
~~~~~

- Refines: ``docs/design/search.md``

.. container:: requirement

RA-1018: Call Sign Search Matching
----------------------------------

Call sign search shall trim input, normalize it to uppercase, and match
only call sign values.

.. _rationale-18:

Rationale
~~~~~~~~~

The search surface is intentionally narrow and backed by the call sign
registry.

.. _acceptance-criteria-18:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Leading and trailing whitespace are ignored.
- Matching is case-insensitive.
- Results are sorted by call sign.
- Non-call-sign fields are not searched.

.. _links-18:

Links
~~~~~

- Refines: ``docs/design/search.md``

Repeater Data Model
-------------------

These requirements describe repeater system identity, services,
coordinates, and linking.

.. container:: requirement

RA-1019: Repeater System Identity
---------------------------------

One repeater record shall represent the physical repeater system
identified by a call sign.

.. _rationale-19:

Rationale
~~~~~~~~~

Users identify a repeater as one physical system even when it exposes
multiple services or RF ports.

.. _acceptance-criteria-19:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- A repeater system has one identity record.
- Multiple services can belong to the same repeater system.
- Service-specific capabilities do not require duplicate repeater
  systems.

.. _links-19:

Links
~~~~~

- Refines: ``docs/design/repeater.md``

.. container:: requirement

RA-1020: Authoritative Repeater Coordinates
===========================================

Repeater latitude and longitude shall be stored as the authoritative
coordinates for rendering and spatial queries.

.. _rationale-20:

Rationale
---------

Coordinates should not be repeatedly derived from grid locator data at
render time.

.. _acceptance-criteria-20:

Acceptance Criteria
-------------------

- Repeater records store latitude and longitude.
- Spatial queries use stored coordinates.
- Rendered latitude and longitude come from stored fields.

.. _links-20:

Links
-----

- Refines: ``docs/design/repeater.md``

.. container:: requirement

RA-1021: Multiple Services per Repeater
=======================================

A repeater system shall be able to expose multiple services such as FM,
DMR, D-STAR, C4FM, APRS, SSB, and AM.

.. _rationale-21:

Rationale
---------

Real repeater systems often support several modes or features under one
system identity.

.. _acceptance-criteria-21:

Acceptance Criteria
-------------------

- A repeater system can have more than one service row.
- Services can share a label and frequency pair when appropriate.
- Service kind is represented per service.

.. _links-21:

Links
-----

- Refines: ``docs/design/repeater.md``

.. container:: requirement

RA-1022: Mode-Specific Service Fields
-------------------------------------

Mode-specific data shall live with the relevant service instead of on
the general repeater system identity.

.. _rationale-22:

Rationale
~~~~~~~~~

Fields such as DMR color code or FM tone do not apply to every service
kind.

.. _acceptance-criteria-22:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- General repeater identity fields are not used for mode-specific
  settings.
- Service records can carry fields relevant to their own kind.

.. _links-22:

Links
~~~~~

- Refines: ``docs/design/repeater.md``

.. container:: requirement

RA-1023: Undirected Repeater Links
----------------------------------

Repeater links shall be stored as unique undirected relationships
between two different repeater systems.

.. _rationale-23:

Rationale
~~~~~~~~~

An RF or network link has the same meaning regardless of which repeater
is viewed first.

.. _acceptance-criteria-23:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- A repeater cannot link to itself.
- A pair of repeaters can only be linked once.
- Query logic treats the relationship symmetrically.

.. _links-23:

Links
~~~~~

- Refines: ``docs/design/repeater.md``

.. container:: requirement

RA-1024: Linked Network Discovery
=================================

The application shall discover a repeater's linked network by traversing
repeater links in either direction.

.. _rationale-24:

Rationale
---------

The detail page needs to show the full connected network, not only
direct links.

.. _acceptance-criteria-24:

Acceptance Criteria
-------------------

- Traversal starts from the requested repeater.
- Traversal follows links in both directions.
- Traversal avoids cycles.
- Results include depth or equivalent ordering information.

.. _links-24:

Links
-----

- Refines: ``docs/design/repeater.md``

Exports
-------

These requirements describe shared export loading and CHIRP CSV
behavior.

.. container:: requirement

RA-1025: Shared Export Data Loading
===================================

Export features shall use shared loading logic that returns fully
populated repeater domain models.

.. _rationale-25:

Rationale
---------

Shared loading keeps export formats focused on format-specific mapping.

.. _acceptance-criteria-25:

Acceptance Criteria
-------------------

- Export code loads repeater systems through shared export service
  logic.
- Format-specific exporters receive fully populated repeater models.
- HTTP or CLI layers do not duplicate export data loading rules.

.. _links-25:

Links
-----

- Refines: ``docs/design/export.md``

.. container:: requirement

RA-1026: CHIRP FM Service Export
================================

The CHIRP CSV exporter shall export FM services and skip non-FM
services.

.. _rationale-26:

Rationale
---------

The implemented CHIRP export scope is limited to FM rows.

.. _acceptance-criteria-26:

Acceptance Criteria
-------------------

- FM services are included in CHIRP CSV output.
- Non-FM services are omitted from CHIRP CSV output.
- Location numbers start at zero.

.. _links-26:

Links
-----

- Refines: ``docs/design/export.md``

.. container:: requirement

RA-1027: CHIRP CSV Field Mapping
================================

The CHIRP CSV exporter shall map repeater service data to
CHIRP-compatible CSV columns.

.. _rationale-27:

Rationale
---------

CHIRP expects specific column names and value formats when importing
memory channels.

.. _acceptance-criteria-27:

Acceptance Criteria
-------------------

- The CSV header matches the expected CHIRP column set.
- Frequency and offset values are formatted in MHz with six decimals.
- Duplex is derived from the repeater TX/RX difference.
- Mode is emitted as ``FM`` or ``NFM``.

.. _links-27:

Links
-----

- Refines: ``docs/design/export.md``

.. container:: requirement

RA-1028: CHIRP Tone Field Mapping
---------------------------------

The CHIRP CSV exporter shall populate tone and cross-mode columns
according to CHIRP import expectations.

.. _rationale-28:

Rationale
~~~~~~~~~

CHIRP rejects rows when required tone-related fields are empty or
inconsistent.

.. _acceptance-criteria-28:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- CTCSS-only services map to CHIRP tone fields correctly.
- DCS-only services map to CHIRP DTCS fields correctly.
- Mixed CTCSS and DCS services use CHIRP cross-mode values.
- Required fallback values are emitted for unused CHIRP tone columns.

.. _links-28:

Links
~~~~~

- Refines: ``docs/design/export.md``

.. container:: requirement

RA-1029: CHIRP RX Tone Option
-----------------------------

The CHIRP CSV exporter shall support an option that controls whether RX
tone data is included.

.. _rationale-29:

Rationale
~~~~~~~~~

Some radio programming workflows only need TX tone data.

.. _acceptance-criteria-29:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- When RX tone export is enabled, RX tone data can be emitted.
- When RX tone export is disabled, only TX tone data or no tone is
  emitted.

.. _links-29:

Links
~~~~~

- Refines: ``docs/design/export.md``

Map Page
--------

These requirements describe the primary public map page.

.. container:: requirement

RA-1030: Map Page Route
-----------------------

The application shall render the primary public map page at ``/``.

.. _rationale-30:

Rationale
~~~~~~~~~

The map is the primary public entry point for browsing repeaters.

.. _acceptance-criteria-30:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- A public user can request ``/`` without authentication.
- The response renders the map page.

.. _links-30:

Links
~~~~~

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1031: Full-Viewport Map Layout
=================================

The map page shall use a full-viewport layout where the map fills the
available space beside the side pane.

.. _rationale-31:

Rationale
---------

The map is the primary visual surface and should use the available
screen area.

.. _acceptance-criteria-31:

Acceptance Criteria
-------------------

- The page body uses full viewport height.
- The Leaflet map fills the remaining width beside the pane.
- Leaflet bounds reflect the visible map area.

.. _links-31:

Links
-----

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1032: Map Markers
--------------------

The map page shall render clustered markers for repeaters that have
coordinates.

.. _rationale-32:

Rationale
~~~~~~~~~

Map browsing depends on visible repeater locations, while repeaters
without coordinates cannot be placed on the map.

.. _acceptance-criteria-32:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Repeaters with coordinates are represented as markers.
- Repeaters without coordinates are excluded from markers.
- Marker labels show call signs.
- Markers are clustered when appropriate.

.. _links-32:

Links
~~~~~

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1033: Map Side Pane
----------------------

The map page shall provide a side pane beside the map for header
content, nearest repeaters, and repeater details.

.. _rationale-33:

Rationale
~~~~~~~~~

Users should be able to inspect map-related information without leaving
the map.

.. _acceptance-criteria-33:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- The side pane is placed beside the map.
- The pane body can contain the site header.
- The pane body can show nearest repeaters and selected repeater
  details.

.. _links-33:

Links
~~~~~

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1034: Always-Visible Icon Column
===================================

The map side pane shall include an always-visible icon column for
primary map actions.

.. _rationale-34:

Rationale
---------

Core map actions should remain available when the pane body is
collapsed.

.. _acceptance-criteria-34:

Acceptance Criteria
-------------------

- The icon column remains visible when the pane body is hidden.
- The icon column contains icon buttons only.
- The pane toggle icon reflects the pane state.

.. _links-34:

Links
-----

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1035: Nearest Repeaters List
===============================

The map page shall show a nearest repeaters list when the map is zoomed
in far enough.

.. _rationale-35:

Rationale
---------

Nearby repeaters help users discover relevant systems in the current map
area.

.. _acceptance-criteria-35:

Acceptance Criteria
-------------------

- A zoom hint is shown below the nearest-repeater zoom threshold.
- Up to 20 visible repeaters are listed at or above the threshold.
- Listed repeaters are sorted by distance from the map center.
- The list updates after map movement.

.. _links-35:

Links
-----

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1036: In-Pane Repeater Details
---------------------------------

The map page shall show selected repeater details inside the side pane.

.. _rationale-36:

Rationale
~~~~~~~~~

Visitors should be able to inspect a repeater without leaving the map
page.

.. _acceptance-criteria-36:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Selecting a marker opens repeater details.
- Selecting a nearest-list item opens repeater details.
- Details include call sign, status, and service summary.
- Details include a link to the full repeater detail page.
- Details can be dismissed.

.. _links-36:

Links
~~~~~

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1037: Map Viewport Persistence
---------------------------------

The map page shall store the current viewport after movement and restore
it on a later visit.

.. _rationale-37:

Rationale
~~~~~~~~~

Users should return to the same map area they were viewing previously.

.. _acceptance-criteria-37:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Map movement stores center and zoom client-side.
- A saved viewport is restored on page load.
- Query parameters for ``lat``, ``lon``, and ``zoom`` take priority when
  present.

.. _links-37:

Links
~~~~~

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1038: Browser Position Action
--------------------------------

The map page shall provide an action that asks the browser for the
user's current location and centers the map there when permitted.

.. _rationale-38:

Rationale
~~~~~~~~~

Location-based centering helps users find repeaters near their current
position.

.. _acceptance-criteria-38:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- The action requests browser geolocation.
- When geolocation succeeds, the map recenters to that position.
- When geolocation is denied, the action does not break the map page.

.. _links-38:

Links
~~~~~

- Refines: ``docs/design/pages/map.md``

.. container:: requirement

RA-1039: Map Login Access
-------------------------

The map page shall provide quick access to the login flow.

.. _rationale-39:

Rationale
~~~~~~~~~

Editors and administrators may start from the public map page before
logging in.

.. _acceptance-criteria-39:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- The map page contains a login action.
- The login action is available without leaving the map page first.
- The login action navigates to the login flow.

.. _links-39:

Links
~~~~~

- Refines: ``docs/design/pages/map.md``

User Account Page
-----------------

These requirements describe authenticated user account features.

.. container:: requirement

RA-1040: Authenticated User Account Page
----------------------------------------

The application shall provide a personal account page at ``/-/my`` for
logged-in users and redirect unauthenticated users to login.

.. _rationale-40:

Rationale
~~~~~~~~~

Saved locations and personal exports belong to authenticated users.

.. _acceptance-criteria-40:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Authenticated users can access ``/-/my``.
- Unauthenticated users are redirected to ``/-/login``.

.. _links-40:

Links
~~~~~

- Refines: ``docs/design/pages/my-page.md``

.. container:: requirement

RA-1041: Saved Location Management
----------------------------------

The user account page shall let users list, add, edit, and delete saved
QTH locations.

.. _rationale-41:

Rationale
~~~~~~~~~

Saved QTHs support map navigation, exports, and portable logbook output.

.. _acceptance-criteria-41:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- The account page lists the user's saved locations.
- Users can add a saved location.
- Users can edit a saved location.
- Users can delete a saved location.
- The location list updates after each change.

.. _links-41:

Links
~~~~~

- Refines: ``docs/design/pages/my-page.md``

.. container:: requirement

RA-1042: Saved Location Resolution
----------------------------------

When a saved location is created or updated, the application shall
resolve address, Maidenhead, and latitude/longitude from the provided
inputs.

.. _rationale-42:

Rationale
~~~~~~~~~

Users may know a location as coordinates, a grid locator, or an address.

.. _acceptance-criteria-42:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Given latitude and longitude, the application derives Maidenhead.
- Given Maidenhead, the application derives latitude and longitude.
- Given address, the application can geocode coordinates and derive
  Maidenhead.

.. _links-42:

Links
~~~~~

- Refines: ``docs/design/pages/my-page.md``

.. container:: requirement

RA-1043: CHIRP Export Download
==============================

The user account page shall provide a download link for the CHIRP CSV
export.

.. _rationale-43:

Rationale
---------

Authenticated users need a convenient path to radio programming exports.

.. _acceptance-criteria-43:

Acceptance Criteria
-------------------

- The account page links to ``/-/export/chirp.csv``.
- The link downloads CHIRP-compatible CSV output.

.. _links-43:

Links
-----

- Refines: ``docs/design/pages/my-page.md``

.. container:: requirement

RA-1044: Portable Logbook Generation
------------------------------------

The user account page shall let users generate a portable logbook PDF
and the corresponding Typst source.

.. _rationale-44:

Rationale
~~~~~~~~~

Portable operation benefits from printable log sheets and reference
material.

.. _acceptance-criteria-44:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Users can submit logbook options to download a PDF.
- Users can submit the same form to download Typst source.
- Generated output is streamed directly without persisting a logbook
  record.

.. _links-44:

Links
~~~~~

- Refines: ``docs/design/pages/my-page.md``

Repeater Detail Page
--------------------

These requirements describe the public repeater detail page.

.. container:: requirement

RA-1045: Repeater Detail Route
------------------------------

The application shall render a public repeater detail page for a
repeater call sign at ``/{call_sign}``.

.. _rationale-45:

Rationale
~~~~~~~~~

Each repeater needs a stable public page with complete details.

.. _acceptance-criteria-45:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- A repeater call sign resolves to the repeater detail page.
- Call signs are uppercased before lookup.
- The page is readable without authentication.

.. _links-45:

Links
~~~~~

- Refines: ``docs/design/pages/repeater-detail.md``

.. container:: requirement

RA-1046: Repeater Detail Responsive Layout
------------------------------------------

The repeater detail page shall use a two-column content and map layout
on wide screens and a single-column layout on narrow screens.

.. _rationale-46:

Rationale
~~~~~~~~~

Repeater details must remain usable on both desktop and mobile devices.

.. _acceptance-criteria-46:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Wide screens show content and map in two columns.
- Narrow screens show a single-column layout.
- The detail map is shown only when the repeater has coordinates.

.. _links-46:

Links
~~~~~

- Refines: ``docs/design/pages/repeater-detail.md``

.. container:: requirement

RA-1047: Repeater Detail Sections
=================================

The repeater detail page shall show identity, status, contacts,
description, location, and service information for the repeater.

.. _rationale-47:

Rationale
---------

Users need the repeater's operational and contact details in one public
view.

.. _acceptance-criteria-47:

Acceptance Criteria
-------------------

- The page header shows the call sign.
- Details include status and contacts when available.
- Location fields are shown from stored data.
- Services are grouped by service kind.

.. _links-47:

Links
-----

- Refines: ``docs/design/pages/repeater-detail.md``

.. container:: requirement

RA-1048: Repeater Detail Related Repeaters
------------------------------------------

The repeater detail page shall show linked repeaters, a linked network
map, and nearby repeaters when data is available.

.. _rationale-48:

Rationale
~~~~~~~~~

Related repeaters help users understand network topology and nearby
options.

.. _acceptance-criteria-48:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Linked repeaters are listed when links exist.
- The linked network map shows connected repeaters when appropriate.
- Nearby repeaters within 50 km are shown and sorted by distance.
- Nearby markers are labeled by call sign.

.. _links-48:

Links
~~~~~

- Refines: ``docs/design/pages/repeater-detail.md``

.. container:: requirement

RA-1049: Missing Detail Fallback Values
=======================================

The repeater detail page shall show safe fallback values when optional
description or location data is missing.

.. _rationale-49:

Rationale
---------

Missing optional data should not produce blank or broken detail pages.

.. _acceptance-criteria-49:

Acceptance Criteria
-------------------

- Missing description displays ``-``.
- Missing Maidenhead displays ``-``.
- Missing latitude and longitude display ``-``.

.. _links-49:

Links
-----

- Refines: ``docs/design/pages/repeater-detail.md``

Admin and Validation
--------------------

These requirements describe admin editing and core validation behavior.

.. container:: requirement

RA-1050: Admin Repeater Editing
-------------------------------

The application shall provide authenticated admin pages for listing,
creating, and editing repeater entries.

.. _rationale-50:

Rationale
~~~~~~~~~

Repeater owners and administrators need a protected interface for
maintaining directory data.

.. _acceptance-criteria-50:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Authenticated admins can access the repeater management list.
- Authenticated admins can open a new repeater form.
- Authenticated admins can open an edit form for an existing repeater.
- Public users cannot access admin editing pages.

.. _links-50:

Links
~~~~~

- Refines: ``docs/design/README.rst``

.. container:: requirement

RA-1051: Core Validation Rules
------------------------------

The application shall validate core repeater and user input fields
before persistence.

.. _rationale-51:

Rationale
~~~~~~~~~

Basic validation keeps stored repeater and account data consistent.

.. _acceptance-criteria-51:

Acceptance Criteria
~~~~~~~~~~~~~~~~~~~

- Call signs are required, uppercase, and at most 10 characters.
- Frequencies are numeric and non-negative.
- Latitude is within -90..90 and longitude is within -180..180.
- Required region and country strings are present.
- Password validation enforces the minimum length when enabled.

.. _links-51:

Links
~~~~~

- Refines: ``docs/design/README.rst``
