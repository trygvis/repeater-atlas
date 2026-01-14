# Design Document: NRRL Relæ

## Overview
NRRL Relæ is a new project with an initial design to be defined. This document captures
the intended purpose, scope, requirements, and a starting architecture to align on
direction before implementation.

## Goals
- Provide a public, mobile-friendly repeater directory with map and detail views.
- Enable repeater owners/admins to maintain accurate listings via authenticated access.
- Make data discoverable by search engines.

## Non-Goals
- Full feature set beyond the first release.
- Performance optimization beyond "works reliably for initial use."
- Production-scale operations or multi-region support.

## Stakeholders and Users
- Primary users: Radio amateurs browsing repeaters and settings.
- Admin users: Repeater owners maintaining listings.
- Operators/maintainers: NRRL maintainers managing roles and system operations.

## Problem Statement
Radio amateurs need a reliable, searchable directory of repeaters with accurate settings
and map-based discovery, tailored for Norwegian hams but open to any region. Repeater
owners need a straightforward way to keep entries up to date, while the public should be
able to access the data without logins.

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

## High-Level Architecture
Describe the components and their interactions. Initial strawman:

- Client/UI: Public web app with listings, detail pages, and map view.
- Service: Backend providing SSR pages and admin CRUD.
- Data store: Repeater data, club/user membership, and changelog entries.
- External integrations: Optional map tile provider and geocoding.

## Data Model (if applicable)
- Entities: Repeater, User, Ham Club, Club Membership, Repeater Change Log Entry.
- Relationships: Repeaters belong to clubs; users manage repeaters via club roles.
- Core fields (MVP draft): callsign, location (lat/lon, region), band, RX/TX freq,
  offset, tone (CTCSS/DCS), modes, power, status, description, changelog, last-updated.

## Interfaces
- External APIs: Not in MVP; consider public read endpoints later.
- Internal interfaces: Admin UI to backend.
- Protocols/transport: HTTPS/JSON.

## Operational Considerations
- Deployment model:
- Configuration:
- Observability (logging, metrics, traces):
- Backup/restore:

## Security and Privacy
- Authentication: Standard username/password (or SSO later).
- Authorization: Admin roles assigned manually.
- Data sensitivity: Public repeater data; admin accounts protected.

## Testing Strategy
- Unit tests:
- Integration tests:
- Hardware-in-the-loop (if applicable):

## Risks and Open Questions
- RISK-1:
- RISK-2:
- QUESTION-1:

## Milestones
- M1: Problem framing and requirements validated.
- M2: Architecture and interfaces agreed.
- M3: MVP implemented and tested.
