# Data Model: NRRL Relæ

## Intent
This document captures the conceptual data model and relationships. It intentionally
avoids exhaustive field listings; the authoritative details live in the database schema.

## Core Entities
- Repeater: A single repeater listing with technical settings, status, and location.
- User: Authenticated user who can manage repeaters via club membership.
- Ham Club: Organization that owns repeaters.
- Membership/Role: Links users to clubs with permissions.
- Repeater Change Log Entry: Textual audit entry for repeater changes.

## Relationships
- A repeater belongs to exactly one ham club.
- A repeater always has a location (modeled on the repeater entity).
- A user can be a member of multiple ham clubs.
- A ham club has many members and many repeaters.
- Admin members can edit all repeaters owned by their club.
- A repeater has many change log entries.
- A change log entry belongs to one repeater and one user (author).

## Repeater
### Field Categories (non-exhaustive)
- Identity: callsign, name/label.
- Technical settings: band, RX/TX frequencies, offset, tones.
- Status: active/offline/planned, last updated.
- Location: coordinates, region, country, coverage.
- Metadata: description (freeform), changelog, timestamps, provenance.

### Modes (one or more)
- Analog FM
- DMR
- D-STAR
- C4FM/YSF
- P25
- NXDN
- Analog + Digital mixed

### Technical (optional details)
- Power/ERP: transmitter power, antenna gain/height.
- Access control: access tone vs transmit tone, carrier access.
- System identifiers: color code, NAC, RAN, talkgroup/network IDs (mode-specific).
- Linking: network linkage, reflector/room, local link name/ID.
- Linked networks (examples): BrandMeister, D-STAR reflectors, YSF rooms/WIRES-X, EchoLink.

### Status (expanded)
- Active, offline, planned, or seasonal/conditional.

## User
### Field Categories (non-exhaustive)
- Identity: email, callsign, display name.
- Auth: password hash, last login.
- Memberships: club roles.

## Ham Club
### Field Categories (non-exhaustive)
- Identity: name, short name, callsign (if applicable).
- Contact: website, email.
- Metadata: notes, timestamps.

## Membership/Role
### Field Categories (non-exhaustive)
- Link: user, club.
- Role: admin, editor (or similar).

## Repeater Change Log Entry
### Field Categories (non-exhaustive)
- Link: repeater, author.
- Content: text block.
- Metadata: timestamp (can be historical/backdated), public by default.

## Open Decisions
- Coverage representation: radius (km) vs polygon.
