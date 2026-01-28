# Data Model: Repeater Atlas

## Intent

This document captures the conceptual data model and relationships. It
intentionally avoids exhaustive field listings; the authoritative details live
in the database schema.

## Core Entities

- Repeater: A single repeater listing with technical settings, status, and
  location.
- User: Authenticated user who can manage repeaters via club membership.
- Entity: Global call sign registry (repeaters + contacts).
- Contact: Organization or individual responsible for repeaters.
- Membership/Role: Links users to contacts with permissions.
- Repeater Change Log Entry: Textual audit entry for repeater changes.

## Relationships

- A repeater has an entity (call sign lives on the entity).
- A repeater can have an owner contact and a technical contact (both optional).
- A repeater always has a location (modeled on the repeater entity).
- A user can be a member of multiple contacts.
- A contact can have many members and many repeaters.
- Admin members can edit all repeaters owned by their contact.
- A repeater has many change log entries.
- A change log entry belongs to one repeater and one user (author).

## Repeater

### Field Categories (non-exhaustive)

- Identity: callsign, name/label.
- Technical settings: band, RX/TX frequencies, offset, tones.
- Status: active/offline/planned, last updated.
- Location: coordinates, region, country.
- Metadata: description (freeform), changelog, provenance.

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
- System identifiers: color code, NAC, RAN, talkgroup/network IDs
  (mode-specific).
- Linking: network linkage, reflector/room, local link name/ID.
- Linked networks (examples): BrandMeister, D-STAR reflectors, YSF
  rooms/WIRES-X, EchoLink.

### Status (expanded)

- Active, offline, planned, or seasonal/conditional.

## User

### Field Categories (non-exhaustive)

- Identity: email, callsign, display name.
- Auth: password hash, last login.
- Memberships: contact roles.

## Contact

### Field Categories (non-exhaustive)

- Identity: display name, callsign (optional).
- Contact: website, email.
- Metadata: notes.

## Membership/Role

### Field Categories (non-exhaustive)

- Link: user, contact.
- Role: admin, editor (or similar).

## Repeater Change Log Entry

### Field Categories (non-exhaustive)

- Link: repeater, author.
- Content: text block.
- Metadata: timestamp (can be historical/backdated), public by default.

## Open Decisions
